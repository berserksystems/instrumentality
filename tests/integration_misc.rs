mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// catcher_404 tests:
/// - Instrumentality serves a NOT FOUND error to a request to an invalid route
///   i.e. (/404)
#[tokio::test]
async fn catcher_404() {
    use instrumentality::routes::response::ErrorResponse;

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/404")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());

    env.cleanup().await;
}
