//! Route for updating subjects and groups.
//!
//! The /update route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/register/>.

use std::collections::HashMap;

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::{bson, Collection};
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::group::Group;
use crate::routes::queue;
use crate::routes::response::{ErrorResponse, OkResponse};
use crate::subject::*;
use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum UpdateData {
    UpdateSubject {
        uuid: String,
        name: String,
        profiles: HashMap<String, Vec<String>>,
        description: Option<String>,
    },
    UpdateGroup {
        uuid: String,
        name: String,
        subjects: Vec<String>,
        description: Option<String>,
    },
}

pub async fn update(
    user: User,
    mut db: DBHandle,
    Json(data): Json<UpdateData>,
) -> impl IntoResponse {
    match data {
        UpdateData::UpdateSubject { .. } => {
            update_subject(&data, &mut db, &user).await
        }
        UpdateData::UpdateGroup { .. } => {
            update_group(&data, &mut db, &user).await
        }
    }
}

async fn update_subject(
    data: &UpdateData,
    db: &mut DBHandle,
    user: &User,
) -> Result<(StatusCode, Json<OkResponse>), (StatusCode, Json<ErrorResponse>)> {
    let (uuid, name, profiles, description) = match data {
        UpdateData::UpdateSubject {
            uuid,
            name,
            profiles,
            description,
        } => (uuid, name, profiles, description),
        _ => panic!("Expected UpdateSubject."),
    };
    let req_uuid = &user.uuid;
    let subj_coll: Collection<Subject> = db.collection("subjects");
    if let Ok(Some(subject)) = subj_coll
        .find_one_with_session(
            doc! {"uuid": &uuid, "created_by": &req_uuid},
            None,
            &mut db.session,
        )
        .await
    {
        let mut old_profiles: Vec<(&String, &String)> = Vec::new();
        for platform in subject.profiles.keys() {
            for id in subject.profiles.get(platform).unwrap() {
                old_profiles.push((platform, id));
            }
        }
        let mut new_profiles: Vec<(&String, &String)> = Vec::new();
        for platform in profiles.keys() {
            for id in profiles.get(platform).unwrap() {
                new_profiles.push((platform, id));
            }
        }

        let removed_profiles: Vec<_> = old_profiles
            .iter()
            .filter(|x| new_profiles.contains(x))
            .collect();
        let added_profiles: Vec<_> = new_profiles
            .iter()
            .filter(|x| old_profiles.contains(x))
            .collect();

        for (platform, id) in added_profiles {
            queue::add_queue_item(id, platform, db, false).await;
        }
        for (platform, id) in removed_profiles {
            queue::remove_queue_item(id, platform, db).await;
        }

        subj_coll
            .update_one_with_session(
                doc! {"uuid": &uuid, "created_by": &req_uuid},
                doc! {"$set":
                    {"name": name,
                    "profiles": bson::to_bson(&profiles).unwrap(),
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
            "Subject does not exist or was not created by you."
        )
    }
}

async fn update_group(
    data: &UpdateData,
    db: &mut DBHandle,
    user: &User,
) -> Result<(StatusCode, Json<OkResponse>), (StatusCode, Json<ErrorResponse>)> {
    let (uuid, name, subjects, description) = match data {
        UpdateData::UpdateGroup {
            uuid,
            name,
            subjects,
            description,
        } => (uuid, name, subjects, description),
        _ => panic!("Expected UpdateGroup."),
    };
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
        for s in subjects {
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
