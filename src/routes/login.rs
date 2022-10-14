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

pub async fn login(user: User, db: DBHandle) -> impl IntoResponse {
    let subjects = user.subjects(&db).await.unwrap_or_default();
    let groups = user.groups(&db).await.unwrap_or_default();
    let resp = LoginResponse::new(user.clone(), subjects, groups);

    (StatusCode::OK, Json(resp))
}
