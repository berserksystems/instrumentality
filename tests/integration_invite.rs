//! Ensure you have read the doc comments in common.rs if you are having
//! difficulty getting tests to work.

mod common;
use common::Environment;
use common::TEST_ENVIRONMENT_CONFIG;

use axum::http::StatusCode;
use hyper::Body;
use hyper::Request;
use tower::Service;

/// test_invite tests:
/// - Authentication of the test user works as expected.
/// - Invite route returns the correct information:
///     - an OK,
///     - an invite code with a length of 128 characters containing only numbers
///       0 through 9 and letters A through F.
///
/// N.B.: The integration test for register tests the whole flow of
/// invite -> register.
#[tokio::test]
async fn test_invite() {
    use instrumentality::response::InviteResponse;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;

    let res = env
        .app
        .call(
            Request::builder()
                .method("GET")
                .header("X-API-KEY", &env.user.key)
                .uri("/invite")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let ir: InviteResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(ir.response, "OK".to_string());

    let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();
    assert!(ir.code.len() == 128);
    assert!(re.is_match(&ir.code));

    env.cleanup().await;
}

/// test_invite_bad_key tests:
/// - A bad key fails with UNAUTHORIZED status code.
/// - Returns JSON explaining error.
#[tokio::test]
async fn test_invite_bad_key() {
    use instrumentality::response::Error;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;

    let res = env
        .app
        .call(
            Request::builder()
                .method("GET")
                .header("X-API-KEY", "MAKINGITUP")
                .uri("/invite")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let e: Error = serde_json::from_slice(&body).unwrap();

    assert_eq!(e.response, "ERROR".to_string());

    env.cleanup().await;
}
