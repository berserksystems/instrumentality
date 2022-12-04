mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// reset tests:
/// - Authentication of the test user works as expected.
/// - Reset route returns the correct information:
///     - an OK,
///     - a new key with a length of 64 characters containing only numbers 0
///       through 9 and letters A through F.
/// - Then a test login to determine that the key was actually reset.
#[tokio::test]
async fn reset() {
    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::response::ResetResponse;

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/reset")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let rr: ResetResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(rr.response, "OK".to_string());

    let re = regex::Regex::new(r"^([A-F0-9])*$").unwrap();
    assert!(rr.key.len() == 64);
    assert!(re.is_match(&rr.key));

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &rr.key)
                .uri("/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let lr: LoginResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user.uuid, env.user.uuid);
    assert!(lr.subjects.is_empty());
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}
