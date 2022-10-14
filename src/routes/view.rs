//! Route for viewing data about a subject or group.
//!
//! The /view route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/view/>.

use axum::{extract::Query, http::StatusCode, Json};
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOptions;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::data::Data;
use crate::database::DBHandle;
use crate::response::{Error, ViewResponse};
use crate::subject::Subject;
use crate::user::User;
use crate::utils::deserialise_array::deserialise_array;

#[derive(Serialize, Deserialize)]
pub struct ViewData {
    pub subject_data: Vec<SubjectData>,
}

impl ViewData {
    fn new() -> Self {
        Self {
            subject_data: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SubjectData {
    pub subject: Subject,
    pub platforms: Vec<PlatformData>,
}

impl SubjectData {
    fn new(subject: Subject) -> Self {
        Self {
            subject,
            platforms: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlatformData {
    pub platform: String,
    pub profiles: Vec<ProfileData>,
}

impl PlatformData {
    fn new(platform: String) -> Self {
        Self {
            platform,
            profiles: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProfileData {
    pub meta: Option<Data>,
    pub content: Vec<Data>,
    pub presence: Vec<Data>,
}

impl ProfileData {
    fn new(meta: Option<Data>) -> Self {
        Self {
            meta,
            content: Vec::new(),
            presence: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct ViewQuery {
    #[serde(deserialize_with = "deserialise_array")]
    subjects: Vec<String>,
}

pub async fn view(
    view_query: Option<Query<ViewQuery>>,
    db: DBHandle,
    _user: User,
) -> Result<(StatusCode, Json<ViewResponse>), (StatusCode, Json<Error>)> {
    if view_query.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new("You must provide a list of subjects.")),
        ));
    }

    let subjects = &view_query.unwrap().subjects;

    let data_coll: Collection<Data> = db.collection("data");
    let filter_builder = FindOptions::builder()
        .limit(100)
        .sort(doc! {"retrieved_at": -1_i32})
        .batch_size(100);
    let filter = filter_builder.build();

    let subj_coll: Collection<Subject> = db.collection("subjects");
    let doc: Document = doc! {"uuid": {"$in": &subjects}};
    let subj_cursor = subj_coll.find(doc, None).await.unwrap();
    let results: Vec<Result<Subject, mongodb::error::Error>> =
        subj_cursor.collect().await;
    let subjects: Vec<Subject> =
        results.into_iter().map(|d| d.unwrap()).collect();

    let mut view_data = ViewData::new();

    for s in subjects {
        let mut subject_data: SubjectData = SubjectData::new(s.clone());
        for platform_name in s.profiles.keys() {
            let mut platform_data =
                PlatformData::new(platform_name.to_string());
            for platform_id in s.profiles.get(platform_name).unwrap() {
                let f = filter.clone();
                let meta_data = data_coll
                    .find_one(
                        doc! {"id": &platform_id,
                            "platform": &platform_name,
                            "profile_picture": {"$exists": true}
                        },
                        None,
                    )
                    .await
                    .unwrap();
                let mut profile_data: ProfileData = ProfileData::new(meta_data);

                let presence_cursor = data_coll
                    .find(
                        doc! {"id": &platform_id,
                            "platform": &platform_name,
                            "presence_type": {"$exists": true}
                        },
                        f.clone(),
                    )
                    .await
                    .unwrap();
                let presence_data: Vec<Result<Data, mongodb::error::Error>> =
                    presence_cursor.collect().await;
                profile_data.presence =
                    presence_data.into_iter().map(|d| d.unwrap()).collect();

                let content_cursor = data_coll
                    .find(
                        doc! {"id": &platform_id,
                            "platform": &platform_name,
                            "content_type": {"$exists": true}
                        },
                        f.clone(),
                    )
                    .await
                    .unwrap();
                let content_data: Vec<Result<Data, mongodb::error::Error>> =
                    content_cursor.collect().await;
                profile_data.content =
                    content_data.into_iter().map(|d| d.unwrap()).collect();

                platform_data.profiles.push(profile_data);
            }
            subject_data.platforms.push(platform_data);
        }
        view_data.subject_data.push(subject_data);
    }

    Ok((StatusCode::OK, Json(ViewResponse::new(view_data))))
}
