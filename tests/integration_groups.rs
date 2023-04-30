mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// group_creation tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /user/login as provided with no changes.
#[tokio::test]
async fn group_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid);

    env.cleanup().await;
}

/// group_bad_key_creation tests:
/// - /create requires authentication to create group.
#[tokio::test]
async fn group_bad_key_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", INVALID_API_KEY)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

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

/// group_deletion tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /user/login as provided with no changes.
/// - Group is removed upon deletion.
#[tokio::test]
async fn group_deletion() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::groups::delete::DeleteGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let lr: LoginResponse = env.login().await;

    let group_uuid = lr.groups[0].uuid.clone();

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::DELETE)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/delete")
                .body(Body::from(
                    serde_json::to_vec(&DeleteGroupRequest {
                        uuid: group_uuid.clone(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

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

/// group_subject_deletion tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /user/login as provided with no changes.
/// - When a subject that is a member of the group is deleted, it is removed
///   from the group.
#[tokio::test]
async fn group_subject_deletion() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::DELETE)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/subjects/delete")
                .body(Body::from(
                    serde_json::to_vec(&DeleteSubjectRequest {
                        uuid: subject_uuid.clone(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert_eq!(lr.groups[0].subjects.len(), 0);
    assert!(lr.subjects.is_empty());

    env.cleanup().await;
}

/// group_update tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /user/login as provided with no changes.
/// - Group is updated correctly.
#[tokio::test]
async fn group_update() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::groups::update::UpdateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "testers".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let group_uuid = lr.groups[0].uuid.clone();

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_2".to_string(),
        vec!["user2".to_string(), "user2_priv".to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test2".to_string(),
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

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert_eq!(lr.subjects.len(), 2);
    assert_eq!(lr.groups.len(), 1);
    assert_eq!(lr.groups[0].subjects.len(), 1);

    let subject2_uuid = lr.subjects[1].uuid.clone();

    let subjects = vec![subject_uuid.clone(), subject2_uuid.clone()];

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(
                    serde_json::to_vec(&UpdateGroupRequest {
                        uuid: group_uuid.clone(),
                        name: "testers".to_string(),
                        subjects,
                        description: Some("The testers.".to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert_eq!(lr.subjects.len(), 2);
    assert_eq!(lr.groups.len(), 1);
    assert_eq!(lr.groups[0].subjects.len(), 2);

    env.cleanup().await;
}

/// group_bad_uuid_creation tests:
/// - Authentication of the test user works as expected.
/// - Group fails create if subject UUID isn't valid.
#[tokio::test]
async fn group_bad_uuid_creation() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = "SOMETHINGELSE".to_string();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

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

/// group_bad_uuid_update tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /user/login as provided with no changes.
/// - Update of group is rejected upon attempting to add non-existant subject
///   UUID.
#[tokio::test]
async fn group_bad_uuid_update() {
    use std::collections::HashMap;

    use instrumentality::routes::groups::create::CreateGroupRequest;
    use instrumentality::routes::groups::update::UpdateGroupRequest;
    use instrumentality::routes::response::CreateResponse;
    use instrumentality::routes::response::LoginResponse;
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
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateGroupRequest {
        name: "testers".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get(PLATFORM_NAME).unwrap()
            == &vec![USERNAME.to_string(), USERNAME_PRIV.to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let group_uuid = lr.groups[0].uuid.clone();

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_2".to_string(),
        vec!["user2".to_string(), "user2_priv".to_string()],
    );
    let new_subject = CreateSubjectRequest {
        name: "test2".to_string(),
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

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert_eq!(lr.subjects.len(), 2);
    assert_eq!(lr.groups.len(), 1);
    assert_eq!(lr.groups[0].subjects.len(), 1);

    let subject2_uuid = "SOMETHINGELSE".to_string();

    let subjects = vec![subject_uuid.clone(), subject2_uuid.clone()];

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user_key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/groups/create")
                .body(Body::from(
                    serde_json::to_vec(&UpdateGroupRequest {
                        uuid: group_uuid.clone(),
                        name: "testers".to_string(),
                        subjects,
                        description: Some("The testers.".to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert_eq!(lr.subjects.len(), 2);
    assert_eq!(lr.groups.len(), 1);
    assert_eq!(lr.groups[0].subjects.len(), 1);

    env.cleanup().await;
}
