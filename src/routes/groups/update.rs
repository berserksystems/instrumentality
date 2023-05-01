//! Route for updating groups.
//!
//! The /groups/update route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/groups/update/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::{bson, Collection};
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::group::Group;
use crate::routes::response::{ErrorResponse, OkResponse};
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateGroupRequest {
    pub uuid: String,
    pub name: String,
    pub subjects: Vec<String>,
    pub description: Option<String>,
}

pub async fn update(
    user: User,
    mut db: DBHandle,
    Json(data): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    let UpdateGroupRequest {
        uuid,
        name,
        subjects,
        description,
    } = data;

    let req_uuid = &user.uuid;
    let group_coll: Collection<Group> = db.collection("groups");
    if let Ok(Some(_)) = group_coll
        .find_one_with_session(
            doc! {"uuid": &uuid, "created_by": &req_uuid},
            None,
            &mut db.session,
        )
        .await
    {
        let subj_coll: Collection<Subject> = db.collection("subjects");
        for s in &subjects {
            let subject = subj_coll
                .find_one_with_session(doc! {"uuid": s}, None, &mut db.session)
                .await
                .unwrap();
            if subject.is_none() {
                return error!(
                    BAD_REQUEST,
                    "One or more of the subjects does not exist."
                );
            }
        }
        group_coll
            .update_one_with_session(
                doc! {"uuid": &uuid, "created_by": &req_uuid},
                doc! {"$set":
                    {"name": name,
                    "subjects": bson::to_bson(&subjects).unwrap(),
                    "description": description}
                },
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
            "Group does not exist or was not created by you."
        )
    }
}
