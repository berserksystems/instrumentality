//! Tests for creating, updating, deleting subjects.

mod common;
use common::Environment;
use common::TEST_ENVIRONMENT_CONFIG;

use axum::http::StatusCode;
use hyper::Body;
use hyper::Request;
use tower::Service;

/// test_group_creation tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /login as provided with no changes.
#[tokio::test]
async fn test_group_creation() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid);

    env.cleanup().await;
}

/// test_group_bad_key_creation tests:
/// - /create requires authentication to create group.
#[tokio::test]
async fn test_group_bad_key_creation() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", "MADEITUP")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
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
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// test_group_deletion tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /login as provided with no changes.
/// - Group is removed upon deletion.
#[tokio::test]
async fn test_group_deletion() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::delete::DeleteData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    let group_uuid = lr.groups[0].uuid.clone();
    println!("{}", group_uuid);

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let res = env
        .app
        .call(
            Request::builder()
                .method("DELETE")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/delete")
                .body(Body::from(
                    serde_json::to_vec(&DeleteData {
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
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// test_group_subject_deletion tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /login as provided with no changes.
/// - When a subject that is a member of the group is deleted, it is removed
///   from the group.
#[tokio::test]
async fn test_group_subject_deletion() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::delete::DeleteData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    let group_uuid = lr.groups[0].uuid.clone();
    println!("{}", group_uuid);

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let res = env
        .app
        .call(
            Request::builder()
                .method("DELETE")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/delete")
                .body(Body::from(
                    serde_json::to_vec(&DeleteData {
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

/// test_group_update tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /login as provided with no changes.
/// - Group is updated correctly.
#[tokio::test]
async fn test_group_update() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::update::UpdateData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "testers".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let group_uuid = lr.groups[0].uuid.clone();

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_2".to_string(),
        vec!["user2".to_string(), "user2_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test2".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
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
    assert_eq!(lr.groups[0].subjects.len(), 1);

    let subject2_uuid = lr.subjects[1].uuid.clone();

    let subjects = vec![subject_uuid.clone(), subject2_uuid.clone()];

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/update")
                .body(Body::from(
                    serde_json::to_vec(&UpdateData::UpdateGroup {
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

/// test_group_bad_uuid_creation tests:
/// - Authentication of the test user works as expected.
/// - Group fails create if subject UUID isn't valid.
#[tokio::test]
async fn test_group_bad_uuid_creation() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = "SOMETHINGELSE".to_string();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "test".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
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
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    env.cleanup().await;
}

/// test_group_bad_uuid_update tests:
/// - Authentication of the test user works as expected.
/// - Group is created upon post request.
/// - Group can be seen via /login as provided with no changes.
/// - Update of group is rejected upon attempting to add non-existant subject
///   UUID.
#[tokio::test]
async fn test_group_bad_uuid_update() {
    use instrumentality::response::CreateResponse;
    use instrumentality::response::LoginResponse;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::update::UpdateData;
    use std::collections::HashMap;

    let mut env: Environment = Environment::new(TEST_ENVIRONMENT_CONFIG).await;
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_1".to_string(),
        vec!["user1".to_string(), "user1_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };
    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let cr: CreateResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(cr.response, "OK".to_string());

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert!(lr.groups.is_empty());

    let subject_uuid = lr.subjects[0].uuid.clone();

    let subjects = vec![subject_uuid.clone()];

    let new_group = CreateData::CreateGroup {
        name: "testers".to_string(),
        subjects,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/create")
                .body(Body::from(serde_json::to_vec(&new_group).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let lr: LoginResponse = env.login().await;

    assert_eq!(lr.response, "OK".to_string());
    assert_eq!(lr.user, env.user.clone());
    assert!(
        lr.subjects[0].profiles.get("PLATFORM_1").unwrap()
            == &vec!["user1".to_string(), "user1_priv".to_string()]
    );
    assert_eq!(lr.groups[0].subjects[0], subject_uuid.clone());

    let group_uuid = lr.groups[0].uuid.clone();

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(
        "PLATFORM_2".to_string(),
        vec!["user2".to_string(), "user2_priv".to_string()],
    );
    let new_subject = CreateData::CreateSubject {
        name: "test2".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .uri("/create")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(Body::from(serde_json::to_vec(&new_subject).unwrap()))
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
    assert_eq!(lr.groups[0].subjects.len(), 1);

    let subject2_uuid = "SOMETHINGELSE".to_string();

    let subjects = vec![subject_uuid.clone(), subject2_uuid.clone()];

    let res = env
        .app
        .call(
            Request::builder()
                .method("POST")
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/update")
                .body(Body::from(
                    serde_json::to_vec(&UpdateData::UpdateGroup {
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
