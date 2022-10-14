//! Route for the front page.
//!
//! The / route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/frontpage/>.

use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::response::Ok;

pub async fn frontpage() -> impl IntoResponse {
    (StatusCode::OK, Json(Ok::new()))
}
