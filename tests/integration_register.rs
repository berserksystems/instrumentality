mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// register_with_invite tests:
/// - Authentication of the test user works as expected.
/// - Invite route returns the correct information:
///     - an OK,
///     - an invite code with a length of 128 characters containing only numbers
///       0 through 9 and letters A through F.
/// - Register route called with returned invite code allows creation of new
///   user.
/// - Login route called with created user's key is OK and has correct name.
#[tokio::test]
async fn register_with_invite() {
    use instrumentality::response::InviteResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::response::UserResponse;
    use instrumentality::routes::register::RegisterRequest;

    const USERNAME: &str = "TEST_USER_1";

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/invite")
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

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/register")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(
                    serde_json::to_vec(&RegisterRequest {
                        code: ir.code.clone(),
                        name: USERNAME.to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let rr: UserResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(rr.response, "OK".to_string());
    assert_eq!(rr.user.name, USERNAME.to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &rr.user.key)
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
    assert_eq!(lr.user.name, USERNAME);
    assert!(lr.subjects.is_empty());
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// register_bad_invite_code tests:
/// - Authentication of the test user works as expected.
/// - Invite route returns the correct information:
///     - an OK,
///     - an invite code with a length of 128 characters containing only numbers
///       0 through 9 and letters A through F.
/// - Register route called with returned invite code allows creation of new
///   user.
/// - Login route called with created user's key is OK and has correct name.
#[tokio::test]
async fn register_bad_invite_code() {
    use instrumentality::response::ErrorResponse;
    use instrumentality::routes::register::RegisterRequest;

    const USERNAME: &str = "TEST_USER_1";
    const INVALID_INVITE_CODE: &str = "INVALID_INVITE_CODE";

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/register")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(
                    serde_json::to_vec(&RegisterRequest {
                        code: INVALID_INVITE_CODE.to_string(),
                        name: USERNAME.to_string(),
                    })
                    .unwrap(),
                ))
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
