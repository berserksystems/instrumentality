mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// login tests:
/// - Authentication of the test user works as expected.
/// - Login route returns the correct information:
///     - an OK,
///     - the user info,
///     - empty subjects and groups array
#[tokio::test]
async fn login() {
    use instrumentality::routes::response::LoginResponse;

    let mut env: Environment = Environment::default().await;

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(lr.subjects.is_empty());
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// login_no_key tests:
/// - Authentication without a X-API-KEY header returns not authorised.
#[tokio::test]
async fn login_no_key() {
    use instrumentality::routes::response::ErrorResponse;

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/user/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());

    env.cleanup().await;
}

/// login_bad_key tests:
/// - Authentication without a X-API-KEY header returns not authorised.
#[tokio::test]
async fn login_bad_key() {
    use instrumentality::routes::response::ErrorResponse;

    const INVALID_API_KEY: &str = "INVALID_API_KEY";

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", INVALID_API_KEY)
                .uri("/user/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());

    env.cleanup().await;
}
