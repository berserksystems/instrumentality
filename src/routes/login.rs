//! Routes for fetching user information.
//!
//! The /login route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/login/>.

use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::database::DBHandle;
use crate::response::LoginResponse;
use crate::user::User;

pub async fn login(user: User, mut db: DBHandle) -> impl IntoResponse {
    let subjects = user.subjects(&mut db).await.unwrap_or_default();
    let groups = user.groups(&mut db).await.unwrap_or_default();
    let resp = LoginResponse::new(user.clone(), subjects, groups);

    db.session.commit_transaction().await.unwrap();
    (StatusCode::OK, Json(resp))
}
