//! Route for creating subjects.
//!
//! The /subjects/create route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/subjects/create/>.

use std::collections::HashMap;

use axum::Extension;
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::IConfig;
use crate::database::DBHandle;
use crate::routes::queue;
use crate::routes::response::{CreateResponse, ErrorResponse};
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSubjectRequest {
    pub name: String,
    pub profiles: HashMap<String, Vec<String>>,
    pub description: Option<String>,
}

pub async fn create(
    user: User,
    mut db: DBHandle,
    Extension(config): Extension<IConfig>,
    Json(data): Json<CreateSubjectRequest>,
) -> impl IntoResponse {
    let subj_coll: Collection<Subject> = db.collection("subjects");
    let subject = subject_from_create(data, user).await;
    for platform in subject.profiles.keys() {
        if !config.valid_platform(platform) {
            return error!(
                BAD_REQUEST,
                "Profiles contains unsupported platform(s)."
            );
        }
    }
    subj_coll
        .insert_one_with_session(&subject, None, &mut db.session)
        .await
        .unwrap();
    if db.session.commit_transaction().await.is_ok() {
        for platform in subject.profiles.keys() {
            for id in subject.profiles.get(platform).unwrap() {
                queue::add_queue_item(id, platform, &mut db, false).await;
            }
        }
        ok!(CREATED, CreateResponse::from_uuid(&subject.uuid))
    } else {
        error!(CONFLICT, "Subject by that name already exists.")
    }
}

pub async fn subject_from_create(
    cs: CreateSubjectRequest,
    user: User,
) -> Subject {
    Subject {
        uuid: Uuid::new_v4().to_string(),
        created_at: Utc::now(),
        created_by: user.uuid,
        name: cs.name,
        profiles: cs.profiles,
        description: cs.description,
    }
}
