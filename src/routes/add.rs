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
    user: User,
    Json(datas): Json<Datas>,
    db: DBHandle,
    config: IConfig,
) -> impl IntoResponse {
    if datas.data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new("No data was submitted.")),
        ));
    }

    if let Some(queue_id) = datas.queue_id.clone() {
        if !valid_queue_id(&queue_id, &db).await {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(Error::new("Invalid queue ID.")),
            ));
        }
    }

    let datas = datas.tag(&user.uuid).verify_for_config(&config);

    if !datas.data.is_empty() {
        let processed_datas = process_queue(datas, &db).await;
        if let Some(datas) = processed_datas {
            let data_coll: Collection<Data> = db.collection("data");
            data_coll.insert_many(datas.data, None).await.unwrap();
            return Ok((StatusCode::CREATED, Json(Ok::new())));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(Error::new(
            "No valid data was submitted. Ensure the given platforms and 
            content/presence types are supported by this server. Ensure all data
             was correctly labeled for queue jobs.",
        )),
    ))
}

async fn valid_queue_id(queue_id: &str, db: &DBHandle) -> bool {
    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let q_item = q_coll
        .find_one(doc! {"queue_id": &queue_id }, None)
        .await
        .unwrap();

    q_item.is_some()
}

// The logic for this function needs to be simplified significantly.
// There are several sources of uncertainty that this function resolves:
// - Is there data to be processed and is there an attached queue_id?
// - Does the given queue_id reference an actual job?
// - Does the queue item have a username attached or a platform id?
// - Does all the data in self.data pertain to the queue job? If not filter it
//   out.
// Then get relevant data and pass it to the queue for processing.
async fn process_queue(datas: Datas, db: &DBHandle) -> Option<Datas> {
    if datas.queue_id.is_none() {
        return Some(datas);
    }

    let q_coll: Collection<InternalQueueItem> = db.collection("queue");
    let q_item = q_coll
        .find_one(doc! {"queue_id": &datas.queue_id.as_ref().unwrap() }, None)
        .await
        .unwrap();

    if q_item.is_none() {
        return Some(datas);
    }

    let q_item = q_item.unwrap();
    let verified_datas = datas.verify_for_queue(q_item);

    if verified_datas.data.is_empty() {
        None
    } else {
        let (platform_id, platform, added_by, username) = verified_datas.info();

        let process_success = queue::process(
            verified_datas.queue_id.as_ref().unwrap(),
            &platform_id,
            &platform,
            &added_by,
            username,
            db,
        )
        .await;

        if process_success {
            Some(verified_datas)
        } else {
            None
        }
    }
}
