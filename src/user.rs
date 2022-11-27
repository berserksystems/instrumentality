//! Basic user concepts for Instrumentality.

use std::fmt::Write;

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, RequestPartsExt};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::SessionCursor;
use mongodb::{bson::doc, Collection, Cursor};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::database::DBHandle;
use crate::database::DBPool;
use crate::group::Group;
use crate::response::ErrorResponse;
use crate::subject::Subject;

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub uuid: String,
    pub name: String,
    pub key: String,
    pub admin: bool,
    pub banned: bool,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            name: name.to_string(),
            key: Self::new_key(),
            admin: false,
            banned: false,
            created_at: Utc::now(),
        }
    }

    pub fn new_key() -> String {
        let key_bytes: &mut [u8] = &mut [0; 32];
        getrandom::getrandom(key_bytes).unwrap();
        let mut key = String::new();
        for b in key_bytes {
            write!(&mut key, "{b:0>2X}").unwrap();
        }
        key
    }

    pub async fn subjects(&self, db: &mut DBHandle) -> Option<Vec<Subject>> {
        let subj_coll: Collection<Subject> = db.collection("subjects");
        let mut cursor: SessionCursor<Subject> = subj_coll
            .find_with_session(
                doc! {"created_by": &self.uuid},
                None,
                &mut db.session,
            )
            .await
            .unwrap();

        let subjects = cursor
            .stream(&mut db.session)
            .try_collect::<Vec<Subject>>()
            .await
            .unwrap();
        if subjects.is_empty() {
            None
        } else {
            Some(subjects)
        }
    }

    pub async fn groups(&self, db: &mut DBHandle) -> Option<Vec<Group>> {
        let group_coll: Collection<Group> = db.collection("groups");
        let cursor: Cursor<Group> = group_coll
            .find(doc! {"created_by": &self.uuid}, None)
            .await
            .unwrap();

        let results: Vec<Result<Group, mongodb::error::Error>> =
            cursor.collect().await;
        let groups: Vec<Group> =
            results.into_iter().map(|d| d.unwrap()).collect();
        if groups.is_empty() {
            None
        } else {
            Some(groups)
        }
    }

    pub async fn with_key(key: &str, db: &mut DBHandle) -> Option<Self> {
        let users_coll: Collection<User> = db.collection("users");
        users_coll
            .find_one_with_session(doc! {"key": key}, None, &mut db.session)
            .await
            .unwrap()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let db = parts.extract::<Extension<DBPool>>().await.unwrap();
        let key = parts.headers.get("x-api-key");
        match key {
            Some(key) => {
                let key = key.to_str().unwrap();
                let user = User::with_key(key, &mut db.handle().await).await;

                match user {
                    Some(user) => Ok(user),
                    _ => Err(response!(
                        UNAUTHORIZED,
                        ErrorResponse::from_text("Unauthorised.")
                    )
                    .into_response()),
                }
            }
            None => Err(response!(
                UNAUTHORIZED,
                ErrorResponse::from_text("Unauthorised.")
            )
            .into_response()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_user() {
        let user = User::new("test");

        assert!(!user.banned);
        assert!(!user.admin);
        assert_eq!(user.name, "test");
    }

    #[test]
    fn test_key() {
        let user = User::new("test");
        let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();

        assert_eq!(user.key.len(), 64);
        assert!(re.is_match(&user.key));
    }
}
