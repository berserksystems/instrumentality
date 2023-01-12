//! Route for registering as a new user for Instrumentality.
//!
//! The /register route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/register/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::database::DBHandle;
use crate::routes::invite::Referral;
use crate::routes::response::{ErrorResponse, RegisterResponse};
use crate::user::User;
use crate::utils::random;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub code: String,
    pub name: String,
}

#[derive(Debug)]
pub struct RegisterError;

// Invites can't be double used but we are double requesting with every attempt
// /register wrt invite_valid and use_invite.
pub async fn register(
    mut db: DBHandle,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if !username_available(&req, &mut db).await {
        return error!(BAD_REQUEST, "This username is taken.");
    }
    let result = register_user(&req, &mut db).await;
    match result {
        Ok((user, code)) => {
            ok!(CREATED, RegisterResponse::from_user_with_code(user, code))
        }
        Err(_) => error!(UNAUTHORIZED, "Invalid invite code."),
    }
}

async fn username_available(req: &RegisterRequest, db: &mut DBHandle) -> bool {
    let users_coll: Collection<User> = db.collection("users");
    let result = users_coll
        .find_one_with_session(
            doc! {"user": req.name.as_str()},
            None,
            &mut db.session,
        )
        .await;
    matches!(result, Ok(None))
}

async fn register_user(
    req: &RegisterRequest,
    db: &mut DBHandle,
) -> Result<(User, String), RegisterError> {
    let (user, key) = User::new(&req.name);
    let result = use_invite(&user, req, db).await;
    if result.is_ok() {
        let users_coll: Collection<User> = db.collection("users");

        users_coll
            .insert_one_with_session(&user, None, &mut db.session)
            .await
            .unwrap();

        let result = db.session.commit_transaction().await;
        match result {
            Ok(_) => Ok((user, key)),
            _ => Err(RegisterError),
        }
    } else {
        Err(RegisterError)
    }
}

async fn use_invite(
    user: &User,
    req: &RegisterRequest,
    db: &mut DBHandle,
) -> Result<Referral, RegisterError> {
    let refs_coll: Collection<Referral> = db.collection("referrals");
    let hashed_code = random::hash_string(&req.code);
    let result = refs_coll
        .find_one_and_update_with_session(
            doc! {"hashed_code": hashed_code, "used": false},
            doc! {"$set": {"used": true, "used_by": &user.uuid}},
            None,
            &mut db.session,
        )
        .await
        .unwrap();
    match result {
        Some(entry) => Ok(entry),
        _ => Err(RegisterError),
    }
}
