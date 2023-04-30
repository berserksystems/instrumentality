//! Route for deleting groups.
//!
//! The /delete route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/groups/delete/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::group::Group;
use crate::routes::response::ErrorResponse;
use crate::routes::response::OkResponse;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteGroupRequest {
    pub uuid: String,
}

// This is ugly. Can probably do better than an if-else.
pub async fn delete(
    user: User,
    mut db: DBHandle,
    Json(data): Json<DeleteGroupRequest>,
) -> impl IntoResponse {
    // UUID of the requester.
    let req_uuid = user.uuid;
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
        ok!()
    } else {
        error!(
            BAD_REQUEST,
            "No such group exists or it was not created by the user with the given key."
        )
    }
}
