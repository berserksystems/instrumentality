mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use chrono::Utc;
use common::create_mock_content;
use common::create_mock_presence;
use common::Environment;
use instrumentality::concepts::data::Datas;
use tower::Service;
use uuid::Uuid;

/// add tests:
/// - Authentication of the test user works as expected.
/// - Upon receiving a valid single piece of data the add route returns:
///     - an OK.
#[tokio::test]
async fn add() {
    use instrumentality::routes::response::OkResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![create_mock_content(USERNAME, PLATFORM_NAME)],
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
    env.cleanup().await;
}

/// add_multiple tests:
/// - Authentication of the test user works as expected.
/// - Upon receiving multiple valid pieces of data the add route returns:
///     - an OK.
#[tokio::test]
async fn add_multiple() {
    use instrumentality::routes::response::OkResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![
            create_mock_content(USERNAME, PLATFORM_NAME),
            create_mock_presence(USERNAME, PLATFORM_NAME),
        ],
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
    env.cleanup().await;
}

/// add_bad_key tests:
/// - Authentication of the test user fails with an invalid key.
#[tokio::test]
async fn add_bad_key() {
    use instrumentality::routes::response::ErrorResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_API_KEY: &str = "INVALID_API_KEY";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![create_mock_content(USERNAME, PLATFORM_NAME)],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
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

/// add_bad_queue_id tests:
/// - Adding data fails when an invalid queue ID is provided.
#[tokio::test]
async fn add_bad_queue_id() {
    use instrumentality::routes::response::ErrorResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_QUEUE_ID: &str = "INVALID_QUEUE_ID";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: Some(INVALID_QUEUE_ID.to_string()),
        data: vec![create_mock_content(USERNAME, PLATFORM_NAME)],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());
    env.cleanup().await;
}

/// add_empty_data tests:
/// - Adding data fails when no data is provided.
#[tokio::test]
async fn add_empty_data() {
    use instrumentality::routes::response::ErrorResponse;

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR".to_string());
    env.cleanup().await;
}

/// add_invalid_content_type tests:
/// - Authentication of the test user works as expected.
/// - Adding data fails when a content with an invalid content type is provided.
#[tokio::test]
async fn add_invalid_content_type() {
    use instrumentality::concepts::data::Data;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_CONTENT_TYPE: &str = "INVALID_CONTENT_TYPE";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![Data::Content {
            id: USERNAME.to_string(),
            platform: PLATFORM_NAME.to_string(),
            content_type: INVALID_CONTENT_TYPE.to_string(),
            retrieved_at: Utc::now(),
            content_id: Uuid::new_v4().to_string(),
            deleted: Some(false),
            retrieved_from: None,
            created_at: None,
            body: None,
            media: None,
            references: None,
            added_by: None,
            added_at: None,
        }],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    env.cleanup().await;
}

/// add_invalid_presence_type tests:
/// - Authentication of the test user works as expected.
/// - Adding data fails when a presence with an invalid presence type is
///   provided.
#[tokio::test]
async fn add_invalid_presence_type() {
    use instrumentality::concepts::data::Data;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_PRESENCE_TYPE: &str = "INVALID_PRESENCE_TYPE";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![Data::Presence {
            id: USERNAME.to_string(),
            platform: PLATFORM_NAME.to_string(),
            presence_type: INVALID_PRESENCE_TYPE.to_string(),
            retrieved_at: Utc::now(),
            added_by: None,
            added_at: None,
        }],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    env.cleanup().await;
}

/// add_invalid_presence_type_for_platform tests:
/// - Authentication of the test user works as expected.
/// - Adding data fails when a presence with an invalid presence type is
///   provided, but the presence type would be valid for another platform.
#[tokio::test]
async fn add_invalid_presence_type_for_platform() {
    use instrumentality::concepts::data::Data;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_PRESENCE_TYPE_FOR_PLATFORM: &str = "streaming_now";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![Data::Presence {
            id: USERNAME.to_string(),
            platform: PLATFORM_NAME.to_string(),
            presence_type: INVALID_PRESENCE_TYPE_FOR_PLATFORM.to_string(),
            retrieved_at: Utc::now(),
            added_by: None,
            added_at: None,
        }],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    env.cleanup().await;
}

/// add_invalid_content_type_for_platform tests:
/// - Authentication of the test user works as expected.
/// - Adding data fails when a content with an invalid content type is provided,
///   but the content type would be valid for another platform.
#[tokio::test]
async fn add_invalid_content_type_for_platform() {
    use instrumentality::concepts::data::Data;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";
    const INVALID_CONTENT_TYPE_FOR_PLATFORM: &str = "scrobble";

    let mut env = Environment::default().await;

    let datas = Datas {
        queue_id: None,
        data: vec![Data::Content {
            id: USERNAME.to_string(),
            platform: PLATFORM_NAME.to_string(),
            content_type: INVALID_CONTENT_TYPE_FOR_PLATFORM.to_string(),
            retrieved_at: Utc::now(),
            content_id: Uuid::new_v4().to_string(),
            deleted: Some(false),
            retrieved_from: None,
            created_at: None,
            body: None,
            media: None,
            references: None,
            added_by: None,
            added_at: None,
        }],
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
                .uri("/add")
                .body(Body::from(serde_json::to_vec(&datas).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    env.cleanup().await;
}
