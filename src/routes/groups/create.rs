//! Route for creating groups.
//!
//! The /groups/create route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/groups/create/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::DBHandle;
use crate::group::Group;
use crate::routes::response::{CreateResponse, ErrorResponse};
use crate::subject::Subject;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub subjects: Vec<String>,
    pub description: Option<String>,
}

pub async fn create(
    user: User,
    mut db: DBHandle,
    Json(data): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    let group_coll: Collection<Group> = db.collection("groups");
    let group = group_from_create(data, user).await;
    let subj_coll: Collection<Subject> = db.collection("subjects");
    for s in &group.subjects {
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
        .insert_one_with_session(&group, None, &mut db.session)
        .await
        .unwrap();

    if db.session.commit_transaction().await.is_ok() {
        ok!(CREATED, CreateResponse::from_uuid(&group.uuid))
    } else {
        error!(CONFLICT, "Group by that name already exists.")
    }
}

pub async fn group_from_create(cg: CreateGroupRequest, user: User) -> Group {
    Group {
        uuid: Uuid::new_v4().to_string(),
        created_at: Utc::now(),
        created_by: user.uuid,
        name: cg.name,
        subjects: cg.subjects,
        description: cg.description,
    }
}
