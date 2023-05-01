mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// invite tests:
/// - Authentication of the test user works as expected.
/// - Invite route returns the correct information:
///     - an OK,
///     - an invite code with a length of 128 characters containing only numbers
///       0 through 9 and letters A through F.
///
/// N.B.: The integration test for register tests the whole flow of
/// invite -> register.
#[tokio::test]
async fn invite() {
    use instrumentality::routes::response::InviteResponse;

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user_key)
                .uri("/users/invite")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let ir: InviteResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(ir.response, "OK".to_string());

    let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();
    assert!(ir.code.len() == 128);
    assert!(re.is_match(&ir.code));

    env.cleanup().await;
}

/// invite_bad_key tests:
/// - A bad key fails with UNAUTHORIZED status code.
/// - Returns JSON explaining error.
#[tokio::test]
async fn invite_bad_key() {
    use instrumentality::routes::response::ErrorResponse;

    const INVALID_API_KEY: &str = "INVALID_API_KEY";

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", INVALID_API_KEY)
                .uri("/users/invite")
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
