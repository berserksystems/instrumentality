//! Route for adding data to Instrumentality.
//!
//! The /add route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/add/>.
//!
//! See [`Data`] for examples of valid data objects.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::Collection;

use crate::config::IConfig;
use crate::data::{Data, Datas};
use crate::database::DBHandle;
use crate::response::{Error, Ok};
use crate::routes::queue;
use crate::routes::queue::InternalQueueItem;
use crate::user::User;

pub async fn add(
    Json(datas): Json<Datas>,
    config: IConfig,
    user: User,
    mut db: DBHandle,
) -> impl IntoResponse {
    if datas.data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new("No data was submitted.")),
        ));
    }

    if let Some(queue_id) = datas.queue_id.as_ref() {
        if get_queue_item(queue_id, &user, &mut db).await.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(Error::new("Invalid queue ID.")),
            ));
        }
    }

    match process(datas, &config, &user, &mut db).await {
        true => Ok((StatusCode::CREATED, Json(Ok::new()))),
        false => Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new(
                "No valid data was submitted. Ensure the given platforms and 
                content/presence types are supported by this server. Ensure all data
                 was correctly labeled for queue jobs.",
            )),
        ))
    }
}

async fn get_queue_item(
    queue_id: &str,
    user: &User,
    db: &mut DBHandle,
) -> Option<InternalQueueItem> {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let queue_item = q_coll
        .find_one_with_session(
            doc! {"queue_id": &queue_id, "lock_holder": &user.uuid },
            None,
            &mut db.session,
        )
        .await
        .unwrap();

    queue_item
}

// The logic for this function needs to be simplified.
// This function resolves a number of uncertainties:
// - Is there any data remaining after verifying against the config?
// - If there is a valid queue ID, if there is a valid queue item for that queue
//   ID, is there any data remaining after verifying against the queue item?
// - Does the data succeed in verifying against the queue?
// And then it adds the data if all the answers are yes.
async fn process(
    datas: Datas,
    config: &IConfig,
    user: &User,
    db: &mut DBHandle,
) -> bool {
    let data_coll: Collection<Data> = db.collection("data");

    let datas = datas.tag(&user.uuid).verify_for_config(config);

    if datas.data.is_empty() {
        return false;
    }

    if let Some(queue_id) = &datas.queue_id.clone() {
        let queue_item = get_queue_item(queue_id, user, db).await;

        if queue_item.is_none() {
            return false;
        }

        let queue_item = queue_item.unwrap();
        let datas = datas.verify_for_queue(queue_item);

        if datas.data.is_empty() {
            return false;
        }

        let (platform_id, platform, added_by, username) = datas.info();

        let process_success = queue::process(
            queue_id,
            &platform_id,
            &platform,
            &added_by,
            username,
            db,
        )
        .await;

        if !process_success {
            false
        } else {
            data_coll
                .insert_many_with_session(datas.data, None, &mut db.session)
                .await
                .unwrap();
            db.session.commit_transaction().await.is_ok()
        }
    } else {
        data_coll
            .insert_many_with_session(datas.data, None, &mut db.session)
            .await
            .unwrap();
        db.session.commit_transaction().await.is_ok()
    }
}
