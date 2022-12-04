mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use chrono::Utc;
use common::Environment;
use tower::Service;

use crate::common::create_mock_content;
use crate::common::create_mock_presence;

/// queue_entry_created tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=[PLATFORM_1]) yields queue item.
#[tokio::test]
async fn queue_entry_created() {
    use std::collections::HashMap;

    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=[PLATFORM_1]")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);
    env.cleanup().await;
}

/// queue_entry_created_alternate_syntax tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=PLATFORM_1) yields queue item.
#[tokio::test]
async fn queue_entry_created_alternate_syntax() {
    use std::collections::HashMap;

    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);
    env.cleanup().await;
}

/// queue_locks_entry tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=[PLATFORM_1]) yields queue item.
/// - Fetching another queue item (with only one queue item in the system) does
///   not yield the same queue item for at least 30 seconds.
#[tokio::test]
async fn queue_locks_entry() {
    use std::collections::HashMap;

    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::ErrorResponse;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());
    assert_eq!(
        er.text,
        "There are no jobs available. Please try again later."
    );

    env.cleanup().await;
}

/// queue_add_unlocks_entry tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=[PLATFORM_1]) yields queue item.
/// - Adding the item by the given queue item's ID succeeds.
/// - The queue item is immediately available in the queue again after being
///   added.
#[tokio::test]
async fn queue_add_unlocks_entry() {
    use std::collections::HashMap;

    use instrumentality::data::Datas;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri(format!("/queue?platforms={}", PLATFORM_NAME))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    let queue_id: String = qr.queue_id;

    let datas = Datas {
        queue_id: Some(queue_id),
        data: vec![create_mock_content(USERNAME, PLATFORM_NAME)],
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    env.cleanup().await;
}

/// queue_add_meta_data_modifies_platform_id tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=[PLATFORM_1]) yields queue item.
/// - Adding the item by the given queue item's ID succeeds.
/// - The queue item is immediately available in the queue again after being
///   added.
/// - Providing meta data for a user updates their queue entry to include a
///   unique platform ID separate from a username.
#[tokio::test]
async fn queue_add_meta_data_modifies_platform_id() {
    use std::collections::HashMap;

    use instrumentality::data::Data;
    use instrumentality::data::Datas;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const USER_PLATFORM_ID: &str = "123456789";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri(format!("/queue?platforms={}", PLATFORM_NAME))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    let queue_id: String = qr.queue_id;

    let mock_meta = Data::Meta {
        id: USER_PLATFORM_ID.to_string(),
        platform: PLATFORM_NAME.to_string(),
        username: USERNAME.to_string(),
        private: false,
        suspended_or_banned: false,
        retrieved_at: Utc::now(),
        display_name: None,
        profile_picture: None,
        bio: None,
        verified: None,
        references: None,
        link: None,
        added_by: None,
        added_at: None,
    };

    let datas = Datas {
        queue_id: Some(queue_id),
        data: vec![
            mock_meta,
            create_mock_content(USER_PLATFORM_ID, PLATFORM_NAME),
            create_mock_presence(USER_PLATFORM_ID, PLATFORM_NAME),
        ],
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .header("X-API-KEY", &env.user.key)
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USER_PLATFORM_ID);
    assert_eq!(qr.platform_username_hint, USERNAME);

    env.cleanup().await;
}

/// queue_lock_times_out tests:
/// - Subject is created upon post request.
/// - Queue entry is created upon subject creation with profiles.
/// - Fetching from the queue with square bracket syntax
///   (/queue?platforms=[PLATFORM_1]) yields queue item.
/// - The queue item is available after the lock timeout period, even if no data
///   is added.
#[tokio::test]
async fn queue_lock_times_out() {
    use std::collections::HashMap;

    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::QueueResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: "test".to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
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

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let okr: OkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(okr.response, "OK".to_string());

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    tokio::time::sleep(
        std::time::Duration::from_secs(
            env.config.settings.queue_timeout_secs.try_into().unwrap(),
        ) * 2,
    )
    .await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user.key)
                .uri("/queue?platforms=PLATFORM_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let qr: QueueResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(qr.response, "OK");
    assert_eq!(qr.platform, PLATFORM_NAME);
    assert_eq!(qr.platform_id, USERNAME);
    assert_eq!(qr.platform_username_hint, USERNAME);

    env.cleanup().await;
}
