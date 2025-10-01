use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn get_package_config_route_is_registered() {
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/get_package_config/test")
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("service call failed");

    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn root_returns_hello_world_payload() {
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("service call failed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), 1024)
        .await
        .expect("body read should succeed");
    let body = std::str::from_utf8(&bytes).expect("response should be utf8");
    assert_eq!(body, "Hello World!");
}
