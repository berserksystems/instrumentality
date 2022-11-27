use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};

use crate::response::ErrorResponse;

pub async fn default() -> impl IntoResponse {
    response!(NOT_FOUND, ErrorResponse::from_text("Not found."))
}

pub async fn error_transformer<B>(
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED => {
            error!(METHOD_NOT_ALLOWED, "Method not allowed.")
        }
        StatusCode::UNPROCESSABLE_ENTITY => error!(
            UNPROCESSABLE_ENTITY,
            "The given data is missing a field or is otherwise unprocessable."
        ),
        _ => Ok(resp),
    }
}
