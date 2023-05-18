//! Route for updating subjects.
//!
//! The /subjects/update route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/subjects/update/>.

use std::collections::HashMap;

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::{bson, Collection};
use serde::{Deserialize, Serialize};

use crate::concepts::subject::*;
use crate::concepts::user::User;
use crate::database::DBHandle;
use crate::routes::queue;
use crate::routes::response::{ErrorResponse, OkResponse};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubjectRequest {
    pub uuid: String,
    pub name: String,
    pub profiles: HashMap<String, Vec<String>>,
    pub description: Option<String>,
}

pub async fn update(
    user: User,
    mut db: DBHandle,
    Json(data): Json<UpdateSubjectRequest>,
) -> impl IntoResponse {
    let UpdateSubjectRequest {
        uuid,
        name,
        profiles,
        description,
    } = data;
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
            queue::add_queue_item(id, platform, &mut db, false).await;
        }
        for (platform, id) in removed_profiles {
            queue::remove_queue_item(id, platform, &mut db).await;
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
