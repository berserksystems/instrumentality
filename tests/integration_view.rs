mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

use crate::common::create_mock_content;

/// view tests:
/// - Instrumentality serves a view response to requests to /view with valid
///   subject arguments.
#[tokio::test]
async fn view() {
    use std::collections::HashMap;

    use instrumentality::data::Datas;
    use instrumentality::routes::create::CreateData;
    use instrumentality::routes::response::LoginResponse;
    use instrumentality::routes::response::OkResponse;
    use instrumentality::routes::response::ViewResponse;

    const PLATFORM_NAME: &str = "PLATFORM_1";
    const USERNAME: &str = "TEST_USER_1";

    let mut env = Environment::default().await;

    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    profiles.insert(PLATFORM_NAME.to_string(), vec![USERNAME.to_string()]);
    let new_subject = CreateData::CreateSubject {
        name: USERNAME.to_string(),
        profiles,
        description: None,
    };

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::POST)
                .uri("/create")
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

    let lr: LoginResponse = env.login().await;
    let uuid = lr.subjects[0].uuid.clone();

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user_key)
                .uri(format!("/view?subjects={}", uuid))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let vr: ViewResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(vr.response, "OK");
    assert_eq!(vr.view_data.subject_data[0].subject.name, USERNAME);

    env.cleanup().await;
}

/// view_no_subjects tests:
/// - Instrumentality serves an error response requests to /view with no subject
///   arguments.
#[tokio::test]
async fn view_no_subjects() {
    use instrumentality::routes::response::ErrorResponse;

    let mut env = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .header("X-API-KEY", &env.user_key)
                .uri("/view")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let er: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(er.response, "ERROR");

    env.cleanup().await;
}
