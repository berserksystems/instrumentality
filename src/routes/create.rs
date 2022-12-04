//! Route for creating subjects and groups.
//!
//! The /create route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/create/>.

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
use crate::group::Group;
use crate::routes::queue;
use crate::routes::response::{CreateResponse, ErrorResponse};
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CreateData {
    CreateSubject {
        name: String,
        profiles: HashMap<String, Vec<String>>,
        description: Option<String>,
    },
    CreateGroup {
        name: String,
        subjects: Vec<String>,
        description: Option<String>,
    },
}

pub async fn create(
    user: User,
    mut db: DBHandle,
    Extension(config): Extension<IConfig>,
    Json(data): Json<CreateData>,
) -> impl IntoResponse {
    match data {
        CreateData::CreateSubject { .. } => {
            let subj_coll: Collection<Subject> = db.collection("subjects");
            if let Some(subject) = subject_from_create(data, user).await {
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
                            queue::add_queue_item(id, platform, &mut db, false)
                                .await;
                        }
                    }
                    ok!(CREATED, CreateResponse::from_uuid(&subject.uuid))
                } else {
                    error!(CONFLICT, "Subject by that name already exists.")
                }
            } else {
                error!(
                    BAD_REQUEST,
                    "Subject couldn't be created from the given data."
                )
            }
        }

        CreateData::CreateGroup { .. } => {
            let group_coll: Collection<Group> = db.collection("groups");
            if let Some(group) = group_from_create(data, user).await {
                let subj_coll: Collection<Subject> = db.collection("subjects");
                for s in &group.subjects {
                    let subject = subj_coll
                        .find_one_with_session(
                            doc! {"uuid": s},
                            None,
                            &mut db.session,
                        )
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
            } else {
                error!(BAD_REQUEST, "Group couldn't be created from data.")
            }
        }
    }
}

pub async fn group_from_create(cs: CreateData, user: User) -> Option<Group> {
    match cs {
        CreateData::CreateGroup {
            name,
            subjects,
            description,
        } => Some(Group {
            uuid: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: user.uuid,
            name,
            subjects,
            description,
        }),
        _ => None,
    }
}

pub async fn subject_from_create(
    cs: CreateData,
    user: User,
) -> Option<Subject> {
    match cs {
        CreateData::CreateSubject {
            name,
            profiles,
            description,
        } => Some(Subject {
            uuid: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: user.uuid,
            name,
            profiles,
            description,
        }),
        _ => None,
    }
}
