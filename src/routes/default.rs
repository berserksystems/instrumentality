use crate::response::Error;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};

pub async fn default() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(Error::new("Not found.")))
}

pub async fn error_transformer<B>(
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err((
            StatusCode::METHOD_NOT_ALLOWED,
            Json(Error::new("Method not allowed.")),
        )
            .into_response()),
        StatusCode::UNPROCESSABLE_ENTITY => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(Error::new("Missing field.")),
        )
            .into_response()),
        _ => Ok(resp),
    }
}
