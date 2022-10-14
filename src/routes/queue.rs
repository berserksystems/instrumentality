//! Route for the queue.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/queue/>.
//!
//! The queue is a looping structure containing all the profiles currently
//! being tracked by Instrumentality. Profiles are only being tracked if they
//! exist under a subject.
//!
//! # Locking
//! An efficient queue structure that has jobs that takes time needs a system
//! of locks in order to ensure that data providers are not working over one
//! another. This massively increases the system throughput given that each
//! fetch has an opportunity cost.
//!
//! # Incentives
//! Doing jobs in the queue should be preferable to simply posting whatever
//! data the provider cares to. Ideally there is a leaderboard that awards
//! points based on work done. This would have to reset monthly to allow for
//! fair competition to new providers.
//!
//! # Username changes
//! A fundamental problem with a queue is that we store all our data in terms
//! of platform-unique user IDs rather than by username. This is because
//! platforms generally allow for usernames to be changed, and we must allow
//! for this locally. So when we want to refresh the data we have on a subject,
//! we need to be able to turn an ID into a username that may have changed.
//! This is assuming that the platform doesn't allow outside lookups by ID.
//!
//! The most simple way of doing this is taking the ID from the subject, going
//! to metadata under our data collection in MongoDB and searching for that
//! user's most recent 'username' for that platform. Then we trust that a data
//! provider is going to be able to turn a (platform, username) pair into fresh
//! content/presence/meta data. This will succeed _most_ of the time.
//!
//! In the event that the username has changed, the above method will fail upon
//! finding that the profile is empty or has been replaced by a user with a
//! different ID. Given that there is no way to turn IDs to usernames, we can
//! only advise data providers to use older data from that platform user such
//! as searching previous content posts for users with the same ID. However,
//! this will be heavily platform specific and falls outside the scope of
//! Instrumentality.
//!
//! # Round robin vs. Alternatives
//! A naive queue implementation would be to take every platform user and cycle
//! them, putting most recently fetched data at the bottom of the queue.
//!
//! However, this might not be ideal. Intuitively, a user that posts data is
//! more likely to post data again soon than one that hasn't posted recently.
//! If we know that a user has posted data recently, we want to prioritise
//! fetching them again soon in order to catch more data about them in case
//! of deletion. This is assuming some level of opportunity cost with each
//! fetch.
//!
//! We still want to guarantee some level of coverage to all profiles and don't
//! wish to tune this to be so aggressive that profiles that happen to not
//! have any recent activity become stuck at the back of the queue.
//!
//! One method of implementing this is a hot and cold queue. Naturally, the
//! queue will still be presented to data providers as a single queue, but
//! Instrumentality will interleave the hot queue in at the top of the global
//! queue in order to ensure new hot profiles are still being detected.
//!
//! Additionally, profiles under a single subject become hot by association.

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use chrono::offset::TimeZone;
use chrono::{DateTime, Duration, Utc};
use mongodb::bson::doc;
use mongodb::bson::Bson;
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::IConfig;
use crate::data::Data;
use crate::database::DBHandle;
use crate::response::{Error, QueueResponse};
use crate::subject::Subject;
use crate::user::User;
use crate::utils::deserialise_array::deserialise_array;

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalQueueItem {
    pub queue_id: String, // Queue ID.
    pub platform_id: String,
    pub platform: String,
    pub last_processed: DateTime<Utc>,
    pub lock_holder: Option<String>, // None means not locked.
    pub lock_acquired_at: Option<DateTime<Utc>>,
    pub references: u64,
    pub confirmed_id: bool,
}

impl InternalQueueItem {
    fn new(platform_id: String, platform: String) -> Self {
        Self {
            queue_id: Uuid::new_v4().to_string(),
            platform_id,
            platform,
            last_processed: Utc.ymd(1970, 1, 1).and_hms(0, 0, 1),
            lock_holder: None,
            lock_acquired_at: None,
            references: 1,
            confirmed_id: false,
        }
    }
}

#[derive(Deserialize)]
pub struct QueueQuery {
    #[serde(deserialize_with = "deserialise_array")]
    platforms: Vec<String>,
}

pub async fn queue(
    // We use an Option so we can return a useful error when /queue is called
    // with no arguments.
    queue_query: Option<Query<QueueQuery>>,
    db: DBHandle,
    user: User,
    config: IConfig,
) -> impl IntoResponse {
    if queue_query.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new(
                "You must provide a list of supported platforms.",
            )),
        ));
    }

    let platforms = &queue_query.unwrap().platforms;
    if platforms.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new(
                "You must specify which platforms you are performing jobs for.",
            )),
        ));
    }
    if platforms.iter().any(|p| !config.valid_platform(p)) {
        Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new(concat!(
                "One or more of your given platforms is not valid. ",
                "See /types for supported platforms.",
            ))),
        ))
    } else {
        let filter_builder = FindOneAndUpdateOptions::builder()
            .sort(doc! {"last_processed": -1_i32});

        let filter = filter_builder.build();

        let q_coll: Collection<InternalQueueItem> = db.collection("queue");
        let result = q_coll
            .find_one_and_update(
                doc! {"lock_holder": Bson::Null, "platform": {"$in": &platforms}},
                doc! {"$set": 
                        {
                        "lock_holder": user.uuid,
                        "lock_acquired_at": Utc::now().to_string()
                        }
                    },
                filter,
            )
            .await
            .unwrap();
        if let Some(q_item) = result {
            let username_hint: String =
                get_username_hint(&q_item.platform_id, &q_item.platform, &db)
                    .await;

            Ok((
                StatusCode::OK,
                Json(QueueResponse::new(
                    q_item.queue_id,
                    q_item.platform,
                    q_item.platform_id,
                    username_hint,
                )),
            ))
        } else {
            Err((
                StatusCode::OK,
                Json(Error::new(
                    "There are no jobs available. Please try again later.",
                )),
            ))
        }
    }
}

// This is a really bad function. The logic should be simplified significantly.
// There are several sources of uncertainty that this function resolves:
// - Does the supplied queue_id actually exist?
// - Is the queue item's lock held by the user submitting the data?
// - Does the queue item contain a username instead of a platform id?
pub async fn process(
    queue_id: &str,
    id: &str,
    platform: &str,
    added_by: &str,
    username: Option<String>,
    db: &DBHandle,
) -> bool {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    // If this is a metadata update...
    if let Some(username) = username {
        let find_result = q_coll
            .find_one(
                // It's possible we haven't found an ID for this user yet.
                doc! {
                    "queue_id" : queue_id,
                    "platform": platform,
                    "platform_id": &username,
                    "lock_holder": added_by,
                    "confirmed_id": false
                },
                None,
            )
            .await
            .unwrap();
        // and if so...
        if find_result.is_some() {
            // Remove the temporary username queue item...
            remove_queue_item(&username, platform, db).await;
            // and either merge the username with the already existing queue
            // item with that name or create a new one with the
            // platform id.
            add_queue_item(id, platform, db, true).await;

            // and update the subject to use the platform ID instead of a
            // username.
            let subj_coll: Collection<Subject> = db.collection("subjects");
            subj_coll
                .update_one(
                    doc! {&format!("profiles.{}", platform): &username},
                    doc! {"$set": {&format!("profiles.{}.$", platform): id}},
                    None,
                )
                .await
                .unwrap();
            return true;
        }
    }

    let q_update_result = q_coll
        .update_one(
            doc! {"queue_id" : queue_id, "lock_holder": added_by},
            doc! {"$set":
                {
                    "lock_holder": Bson::Null,
                    "lock_acquired_at": Bson::Null,
                    "last_processed": Utc::now().to_string()
                }
            },
            None,
        )
        .await
        .unwrap();

    q_update_result.modified_count == 1
}

pub async fn add_queue_item(
    platform_id: &str,
    platform: &str,
    db: &DBHandle,
    confirmed_id: bool,
) {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let q_item = q_coll
        .find_one(
            doc! {"platform_id": platform_id, "platform": platform},
            None,
        )
        .await
        .unwrap();
    if q_item.is_some() {
        q_coll
            .update_one(
                doc! {"platform_id": platform_id, "platform": platform},
                doc! {"$inc": {"references": 1_u32}},
                None,
            )
            .await
            .unwrap();
        if confirmed_id {
            q_coll
                .update_one(
                    doc! {"platform_id": platform_id, "platform": platform},
                    doc! {"$set": {"confirmed_id": true}},
                    None,
                )
                .await
                .unwrap();
        }
    } else {
        let q_item: InternalQueueItem = InternalQueueItem::new(
            platform_id.to_string(),
            platform.to_string(),
        );
        q_coll.insert_one(q_item, None).await.unwrap();
    }
}

pub async fn remove_queue_item(
    platform_id: &str,
    platform: &str,
    db: &DBHandle,
) {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let result = q_coll
        .delete_one(
            doc! {
                "platform_id": platform_id,
                "platform": platform,
                "references": 1
            },
            None,
        )
        .await
        .unwrap();
    if result.deleted_count == 0 {
        q_coll
            .update_one(
                doc! {"platform_id": platform_id, "platform": platform},
                doc! {"$inc": {"references": -1_i32}},
                None,
            )
            .await
            .unwrap();
    }
}

pub async fn get_username_hint(
    platform_id: &str,
    platform: &str,
    db: &DBHandle,
) -> String {
    async fn from_meta(
        platform_id: &str,
        platform: &str,
        db: &DBHandle,
    ) -> Option<String> {
        let filter = FindOneOptions::builder()
            .projection(doc! {"username": 1_u32})
            .build();

        #[derive(Debug, Serialize, Deserialize)]
        struct Username {
            username: String,
        }

        let data_coll: Collection<Data> = db.collection("data");
        let username = data_coll
            .clone_with_type::<Username>()
            .find_one(doc! {"id": &platform_id, "platform": &platform}, filter)
            .await;

        if let Ok(Some(username)) = username {
            return Some(username.username);
        }
        None
    }

    from_meta(platform_id, platform, db)
        .await
        .unwrap_or_else(|| platform_id.to_string())
}

pub async fn clear_old_locks(db: &DBHandle, timeout: Duration) {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let thirty_seconds_ago = Utc::now() - timeout;
    q_coll
        .update_many(
            doc! {"lock_acquired_at": {"$lt": thirty_seconds_ago.to_string()}},
            doc! {"$set":
                {"lock_acquired_at": Bson::Null,
                "lock_holder": Bson::Null}
            },
            None,
        )
        .await
        .unwrap();
}
