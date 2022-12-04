//! Route for stopping the server.
//!
//! The /halt route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/halt/>.

use std::time::Duration;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_server::Handle;

use crate::routes::response::{ErrorResponse, OkResponse};
use crate::user::User;

pub async fn halt(
    user: User,
    server_handle: ServerHandle,
) -> impl IntoResponse {
    if user.admin {
        server_handle
            .handle
            .graceful_shutdown(Duration::from_secs(5).into());
        ok!()
    } else {
        error!(UNAUTHORIZED, "Unauthorised.")
    }
}

pub struct ServerHandle {
    pub handle: Handle,
}

#[async_trait]
impl<S> FromRequestParts<S> for ServerHandle
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let handle = parts.extensions.get::<Handle>().unwrap();
        Ok(ServerHandle {
            handle: handle.clone(),
        })
    }
}
