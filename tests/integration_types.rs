mod common;
use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use common::Environment;
use tower::Service;

/// types tests:
/// - /types serves an OK response.
/// - the response corresponds with the given configuration.
#[tokio::test]
async fn types() {
    use instrumentality::response::TypesResponse;

    let mut env: Environment = Environment::default().await;

    let res = env
        .app
        .call(
            Request::builder()
                .method(Method::GET)
                .uri("/types")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let tr: TypesResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(tr.response, "OK");
    assert_eq!(tr.content_types, env.config.content_types);
    assert_eq!(tr.presence_types, env.config.presence_types);

    env.cleanup().await;
}
