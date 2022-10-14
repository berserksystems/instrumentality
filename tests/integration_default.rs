mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// frontpage_bad_method tests:
/// - Instrumentality serves a method not allowed error to bad methods.
///   This test is general across all routes.
#[tokio::test]
async fn frontpage_bad_method() {
    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);

    env.cleanup().await;
}