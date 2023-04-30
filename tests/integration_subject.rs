mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// subject_creation tests:
/// - Authentication of the test user works as expected.
/// - Subject is created upon post request.
/// - Subject can be seen via /user/login as provided with no changes.
#[tokio::test]
async fn subject_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::subjects::create::CreateSubjectRequest;

    const USERNAME: &str = "TEST_USER_1";
    const USERNAME_PRIV: &str = "TEST_USER_1_PRIV";
    const PLATFORM_NAME: &str = "PLATFORM_1";

    let mut env: Environment = Environment::default().await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        PLATFORM_NAME.to_string(),
        vec![USERNAME.to_string(), USERNAME_PRIV.to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// subject_bad_key_creation tests:
/// - /create requires authentication to create subject (or group).
#[tokio::test]
async fn subject_bad_key_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::response::ErrorResponse;
    use instrumentality::routes::subjects::create::CreateSubjectRequest;

    const USERNAME: &str = "TEST_USER_1";
    const USERNAME_PRIV: &str = "TEST_USER_1_PRIV";
    const PLATFORM_NAME: &str = "PLATFORM_1";
    const INVALID_API_KEY: &str = "INVALID_API_KEY";

    let mut env: Environment = Environment::default().await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        PLATFORM_NAME.to_string(),
        vec![USERNAME.to_string(), USERNAME_PRIV.to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", INVALID_API_KEY)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
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

/// subject_deletion tests:
/// - Authentication of the test user works as expected.
/// - Subject is created upon post request.
/// - Subject can be seen via /user/login as provided with no changes.
/// - Subject is removed upon deletion.
#[tokio::test]
async fn subject_deletion() {
    use std::collections::HashMap;

    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::subjects::create::CreateSubjectRequest;
    use instrumentality::routes::subjects::delete::DeleteSubjectRequest;

    const USERNAME: &str = "TEST_USER_1";
    const USERNAME_PRIV: &str = "TEST_USER_1_PRIV";
    const PLATFORM_NAME: &str = "PLATFORM_1";

    let mut env: Environment = Environment::default().await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        PLATFORM_NAME.to_string(),
        vec![USERNAME.to_string(), USERNAME_PRIV.to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;
    let uuid = lr.subjects[0].uuid.clone();
    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::DELETE)
                .uri("/subjects/delete")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(
                    serde_json::to_vec(&DeleteSubjectRequest { uuid }).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(lr.subjects.is_empty());
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// subject_update tests:
/// - Authentication of the test user works as expected.
/// - Subject is created upon post request.
/// - Subject can be seen via /user/login as provided with no changes.
/// - Subject is updated correctly.
#[tokio::test]
async fn subject_update() {
    use std::collections::HashMap;

    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::subjects::create::CreateSubjectRequest;
    use instrumentality::routes::subjects::update::UpdateSubjectRequest;

    const USERNAME_PLATFORM_1: &str = "TEST_USER_1";
    const USERNAME_PLATFORM_1_PRIV: &str = "TEST_USER_1_PRIV";
    const USERNAME_PLATFORM_2: &str = "TEST_USER_1_ON_PLATFORM_2";
    const PLATFORM_1_NAME: &str = "PLATFORM_1";
    const PLATFORM_2_NAME: &str = "PLATFORM_2";

    let mut env: Environment = Environment::default().await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        PLATFORM_1_NAME.to_string(),
        vec![
            USERNAME_PLATFORM_1.to_string(),
            USERNAME_PLATFORM_1_PRIV.to_string(),
        ],
    );
    let new_subject = CreateSubjectRequest {
        name: USERNAME_PLATFORM_1.to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;
    let uuid = lr.subjects[0].uuid.clone();
    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_1_NAME).unwrap()
            == &vec![
                USERNAME_PLATFORM_1.to_string(),
                USERNAME_PLATFORM_1_PRIV.to_string()
            ]
    );
    assert!(lr.groups.is_empty());

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        PLATFORM_1_NAME.to_string(),
        vec![USERNAME_PLATFORM_1.to_string()],
    );
    profiles.insert(
        PLATFORM_2_NAME.to_string(),
        vec![USERNAME_PLATFORM_2.to_string()],
    );
    let updated_subject = UpdateSubjectRequest {
        uuid,
        name: USERNAME_PLATFORM_1.to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&updated_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_1_NAME).unwrap()
            == &vec![USERNAME_PLATFORM_1.to_string()]
    );
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_2_NAME).unwrap()
            == &vec![USERNAME_PLATFORM_2.to_string()]
    );
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// subject_bad_platform_creation tests:
/// - Authentication of the test user works as expected.
/// - Subject is created upon post request.
/// - Subject can be seen via /user/login as provided with no changes.
#[tokio::test]
async fn subject_bad_platform_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::response::ErrorResponse;
    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::subjects::create::CreateSubjectRequest;

    const USERNAME: &str = "TEST_USER_1";
    const USERNAME_PRIV: &str = "TEST_USER_1_PRIV";

    let mut env: Environment = Environment::default().await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "BADPLATFORM".to_string(),
        vec![USERNAME.to_string(), USERNAME_PRIV.to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/subjects/create")
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(lr.subjects.is_empty());
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}
