//! Route for stopping the server.
//!
//! The /halt route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/halt/>.

use std::time::Duration;

use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_server::Handle;

use crate::response::{Error, Ok};
use crate::user::User;

pub async fn halt(
    user: User,
    server_handle: ServerHandle,
) -> impl IntoResponse {
    if user.admin {
        server_handle
            .handle
            .graceful_shutdown(Duration::from_secs(5).into());
        Ok((StatusCode::OK, Json(Ok::new())))
    } else {
        Err((StatusCode::UNAUTHORIZED, Json(Error::new("Unauthorized."))))
    }
}

pub struct ServerHandle {
    pub handle: Handle,
}

#[async_trait]
impl<B: Send> FromRequest<B> for ServerHandle {
    type Rejection = Response;

    async fn from_request(
        request: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let handle = request.extensions().get::<Handle>().unwrap();
        Ok(ServerHandle {
            handle: handle.clone(),
        })
    }
}
