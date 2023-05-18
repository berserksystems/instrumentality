//! Route for creating invites for registering users.
//!
//! The /users/invite route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/users/invite/>.

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::concepts::user::User;
use crate::database::DBHandle;
use crate::routes::response::{ErrorResponse, InviteResponse};
use crate::utils::random;

pub struct InviteError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Referral {
    created_by: String,
    created_at: DateTime<Utc>,
    hashed_code: String,
    used: bool,
    used_by: Option<String>,
}

impl Referral {
    pub fn new(created_by: String) -> (Self, String) {
        let (invite_code, hashed_invite_code) = random::new_invite_code();
        (
            Self {
                created_by,
                created_at: Utc::now(),
                hashed_code: hashed_invite_code,
                used: false,
                used_by: None,
            },
            invite_code,
        )
    }
}

pub async fn invite(
    user: User,
    mut db: DBHandle,
) -> Result<(StatusCode, Json<InviteResponse>), (StatusCode, Json<ErrorResponse>)>
{
    let (referral, invite_code) = Referral::new(user.uuid);
    let refer_coll: Collection<Referral> = db.collection("referrals");
    refer_coll
        .insert_one_with_session(&referral, None, &mut db.session)
        .await
        .unwrap();

    db.session.commit_transaction().await.unwrap();
    ok!(CREATED, InviteResponse::new(invite_code))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_invite() {
        let (referral, _) = Referral::new("test".to_string());

        assert!(!referral.used);
        assert_eq!(referral.created_by, "test");
        assert_eq!(referral.used_by, None);
    }

    #[test]
    fn test_code() {
        let (_, invite_code) = Referral::new("test".to_string());

        let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();
        assert_eq!(invite_code.len(), 128);
        assert!(re.is_match(&invite_code));
    }
}
