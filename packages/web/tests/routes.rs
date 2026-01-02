use axum::{
    body::Body,
    body::to_bytes,
    http::{Method, Request, StatusCode},
};
use serde_json::{Value, json};
use std::{env, ffi::OsString, sync::Mutex};
use tower::ServiceExt;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct TempHomeGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    _temp: tempfile::TempDir,
    prev_home: Option<OsString>,
}

impl TempHomeGuard {
    fn new() -> Self {
        let lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("tempdir");
        let prev_home = env::var_os("HOME");
        unsafe {
            env::set_var("HOME", temp.path());
        }
        Self {
            _lock: lock,
            _temp: temp,
            prev_home,
        }
    }
}

impl Drop for TempHomeGuard {
    fn drop(&mut self) {
        match self.prev_home.take() {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
    }
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body bytes");
    String::from_utf8_lossy(&bytes).to_string()
}

async fn json_response(response: axum::response::Response) -> Value {
    let text = response_text(response).await;
    serde_json::from_str(&text).expect("valid json")
}

#[tokio::test]
async fn root_route_returns_hello_world() {
    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_text(response).await, "Hello World!");
}

#[tokio::test]
async fn health_route_returns_ok() {
    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert_eq!(json.get("status").and_then(Value::as_str), Some("ok"));
}

#[tokio::test(flavor = "current_thread")]
async fn get_config_returns_defaults_in_fresh_home() {
    let _home = TempHomeGuard::new();
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_config")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");

    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert_eq!(json.get("serverUrl").and_then(Value::as_str), Some(""));
    assert_eq!(
        json.get("capabilities")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn add_and_remove_capability_roundtrip() {
    let _home = TempHomeGuard::new();

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/add_capability/ethereum")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::OK);

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_capabilities")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert_eq!(
        json.as_array()
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["ethereum"])
    );

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/remove_capability/ethereum")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::OK);

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_capabilities")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    let json = json_response(response).await;
    assert_eq!(json.as_array().map(Vec::len), Some(0));
}

#[tokio::test(flavor = "current_thread")]
async fn get_package_catalog_includes_ethereum() {
    let _home = TempHomeGuard::new();
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_package_catalog")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");

    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert!(json.get("ethereum").is_some());
}

#[tokio::test(flavor = "current_thread")]
async fn get_package_config_returns_empty_for_missing_file() {
    let _home = TempHomeGuard::new();
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_package_config/ethereum")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");

    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert_eq!(
        json.get("values")
            .and_then(Value::as_object)
            .map(|obj| obj.len()),
        Some(0)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn get_packages_skips_unknown_names() {
    let _home = TempHomeGuard::new();
    let app = kittynode_web::app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/get_packages")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "names": ["does-not-exist"] }))
                        .expect("serialize body"),
                ))
                .expect("build request"),
        )
        .await
        .expect("service call");

    assert_eq!(response.status(), StatusCode::OK);
    let json = json_response(response).await;
    assert_eq!(json.as_object().map(|obj| obj.len()), Some(0));
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_package_routes_map_to_404() {
    let _home = TempHomeGuard::new();

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/get_package/does-not-exist")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let app = kittynode_web::app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/delete_package/does-not-exist")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("service call");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
