//! Route for deleting subjects and groups.
//!
//! The /delete route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/delete/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::group::Group;
use crate::response::Error;
use crate::response::Ok;
use crate::routes::queue;
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteData {
    pub uuid: String,
}

// This is ugly. Can probably do better than an if-else.
pub async fn delete(
    user: User,
    mut db: DBHandle,
    Json(data): Json<DeleteData>,
) -> impl IntoResponse {
    // UUID of the requester.
    let req_uuid = user.uuid;
    let subj_coll: Collection<Subject> = db.collection("subjects");
    if let Ok(Some(subject)) = subj_coll
        .find_one_with_session(
            doc! {"uuid": &data.uuid, "created_by": &req_uuid},
            None,
            &mut db.session,
        )
        .await
    {
        let group_coll: Collection<Group> = db.collection("groups");
        let result = group_coll
            .update_many_with_session(
                doc! {"subjects": &data.uuid},
                doc! {"$pull": {"subjects": &data.uuid}},
                None,
                &mut db.session,
            )
            .await;

        if result.is_ok() {
            subj_coll
                .delete_one_with_session(
                    doc! {"uuid": &data.uuid, "created_by": &req_uuid},
                    None,
                    &mut db.session,
                )
                .await
                .unwrap();

            for platform in subject.profiles.keys() {
                for id in subject.profiles.get(platform).unwrap() {
                    queue::remove_queue_item(id, platform, &mut db).await;
                }
            }

            db.session.commit_transaction().await.unwrap();
            Ok((StatusCode::OK, Json(Ok::new())))
        } else {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Error::new("Internal server error.")),
            ))
        }
    } else {
        let group_coll: Collection<Group> = db.collection("groups");
        if let Ok(Some(_)) = group_coll
            .find_one_with_session(
                doc! {"uuid": &data.uuid, "created_by": &req_uuid},
                None,
                &mut db.session,
            )
            .await
        {
            group_coll
                .delete_one_with_session(
                    doc! {"uuid": &data.uuid, "created_by": &req_uuid},
                    None,
                    &mut db.session,
                )
                .await
                .unwrap();
            db.session.commit_transaction().await.unwrap();
            Ok((StatusCode::OK, Json(Ok::new())))
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                Json(Error::new(
                    "No such group or subject exists or it was
                 not created by the user with the given key.",
                )),
            ))
        }
    }
}
