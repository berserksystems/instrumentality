//! Route for deleting subjects.
//!
//! The /subjects/delete route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/subjects/delete/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::group::Group;
use crate::routes::queue;
use crate::routes::response::ErrorResponse;
use crate::routes::response::OkResponse;
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteSubjectRequest {
    pub uuid: String,
}

// This is ugly. Can probably do better than an if-else.
pub async fn delete(
    user: User,
    mut db: DBHandle,
    Json(data): Json<DeleteSubjectRequest>,
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
            ok!()
        } else {
            error!(INTERNAL_SERVER_ERROR, "Internal server error.")
        }
    } else {
        error!(
            BAD_REQUEST,
            "No such subject exists, or it was not created by the user with the 
            given key."
        )
    }
}
