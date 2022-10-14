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
use crate::response::{Error, RegisterResponse};
use crate::routes::invite::Referral;
use crate::user::User;

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
    Json(req): Json<RegisterRequest>,
    db: DBHandle,
) -> impl IntoResponse {
    if !username_available(&req, &db).await {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(Error::new("This username is taken.")),
        ));
    }
    let result = register_user(&req, &db).await;
    match result {
        Ok(user) => Ok((StatusCode::OK, Json(RegisterResponse::new(user)))),
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(Error::new("Invalid invite code.")),
        )),
    }
}

async fn username_available(req: &RegisterRequest, db: &DBHandle) -> bool {
    let users_coll: Collection<User> = db.collection("users");
    let result = users_coll
        .find_one(doc! {"user": req.name.as_str()}, None)
        .await;
    matches!(result, Ok(None))
}

async fn register_user(
    req: &RegisterRequest,
    db: &DBHandle,
) -> Result<User, RegisterError> {
    let user = User::new(&req.name);
    let result = use_invite(&user, req, db).await;
    if result.is_ok() {
        let users_coll: Collection<User> = db.collection("users");

        let result = users_coll.insert_one(&user, None).await;
        match result {
            Ok(_) => Ok(user),
            _ => Err(RegisterError),
        }
    } else {
        Err(RegisterError)
    }
}

async fn use_invite(
    user: &User,
    req: &RegisterRequest,
    db: &DBHandle,
) -> Result<Referral, RegisterError> {
    let refs_coll: Collection<Referral> = db.collection("referrals");
    let result = refs_coll
        .find_one_and_update(
            doc! {"code": req.code.as_str(), "used": false},
            doc! {"$set": {"used": true, "used_by": &user.uuid}},
            None,
        )
        .await
        .unwrap();
    match result {
        Some(entry) => Ok(entry),
        _ => Err(RegisterError),
    }
}
