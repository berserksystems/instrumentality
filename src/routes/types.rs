//! Route to get supported content and presence types.
//!
//! The /types route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/types/>.

use axum::Extension;
use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::config::IConfig;
use crate::response::TypesResponse;

pub async fn types(Extension(config): Extension<IConfig>) -> impl IntoResponse {
    let tr =
        TypesResponse::from_types(config.content_types, config.presence_types);

    response!(OK, tr)
}
