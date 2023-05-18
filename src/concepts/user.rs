//! Basic user concepts for Instrumentality.

use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::SessionCursor;
use mongodb::{bson::doc, Collection, Cursor};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::concepts::group::Group;
use crate::concepts::subject::Subject;
use crate::database::DBHandle;
use crate::utils::random;

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub uuid: String,
    pub name: String,
    pub hashed_key: String,
    pub admin: bool,
    pub banned: bool,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: &str) -> (Self, String) {
        let (key, hashed_key) = random::new_key();
        (
            Self {
                uuid: Uuid::new_v4().to_string(),
                name: name.to_string(),
                hashed_key,
                admin: false,
                banned: false,
                created_at: Utc::now(),
            },
            key,
        )
    }

    pub fn new_admin(name: &str) -> (Self, String) {
        let (mut admin, key) = Self::new(name);
        admin.admin = true;
        (admin, key)
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
            .find_one_with_session(
                doc! {"hashed_key": key},
                None,
                &mut db.session,
            )
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_user() {
        let (user, _) = User::new("test");

        assert!(!user.banned);
        assert!(!user.admin);
        assert_eq!(user.name, "test");
    }

    #[test]
    fn test_key() {
        let (_, key) = User::new("test");
        let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();

        assert_eq!(key.len(), 64);
        assert!(re.is_match(&key));
    }
}
