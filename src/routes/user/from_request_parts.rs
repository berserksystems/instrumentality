use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, RequestPartsExt};
use axum::{Extension, Json};

use crate::concepts::user::User;
use crate::database::DBPool;
use crate::routes::response::{response, ErrorResponse};
use crate::utils::random;

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let db = parts.extract::<Extension<DBPool>>().await.unwrap();
        let key = parts.headers.get("x-api-key");
        match key {
            Some(key) => {
                let key = key.to_str().unwrap();
                let hashed_key = random::hash_string(key);
                let user =
                    User::with_key(&hashed_key, &mut db.handle().await).await;

                match user {
                    Some(user) => Ok(user),
                    _ => Err(response!(
                        UNAUTHORIZED,
                        ErrorResponse::from_text("Unauthorised.")
                    )
                    .into_response()),
                }
            }
            None => Err(response!(
                UNAUTHORIZED,
                ErrorResponse::from_text("Unauthorised.")
            )
            .into_response()),
        }
    }
}
