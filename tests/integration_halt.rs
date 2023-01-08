mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use instrumentality::user::User;
use tower::Service;

/// halt tests:
/// - Instrumentality serves an OK response to a request to halt with an admin
///   account key.
#[tokio::test]
async fn halt() {
    let mut env: Environment = Environment::default().await;

    let (test_admin_user, key) = User::new_admin("test_admin");

    Environment::inject_account(&env.config, &test_admin_user).await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/halt")
                .header("X-API-KEY", &key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    env.cleanup().await;
}

/// halt_bad_key tests:
/// - Instrumentality serves an UNAUTHORIZED response to a request to halt with
///   an invalid key.
#[tokio::test]
async fn halt_bad_key() {
    const INVALID_API_KEY: &str = "INVALID_API_KEY";

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/halt")
                .header("X-API-KEY", INVALID_API_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    env.cleanup().await;
}

/// halt_bad_permissions tests:
/// - Instrumentality serves an UNAUTHORIZED response to a request to halt with
///   an account that does not have permission to do so.
#[tokio::test]
async fn halt_bad_permissions() {
    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/halt")
                .header("X-API-KEY", &env.user_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    env.cleanup().await;
}
