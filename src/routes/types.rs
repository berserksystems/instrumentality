//! Route to get supported content and presence types.
//!
//! The /types route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/types/>.

use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::config::IConfig;
use crate::response::TypesResponse;

pub async fn types(config: IConfig) -> impl IntoResponse {
    let resp = TypesResponse::new(config.content_types, config.presence_types);

    (StatusCode::OK, Json(resp))
}
