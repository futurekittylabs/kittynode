use axum::{
    body::Body,
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
