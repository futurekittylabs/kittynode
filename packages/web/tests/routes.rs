use std::collections::HashMap;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    response::Response,
};
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{
    OperationalMode, OperationalState, PackageConfig, PackageRuntimeState,
};
use kittynode_web::{app, read_state, run_with_port, with_harness};
use serde::de::DeserializeOwned;
use serde_json::json;
use tower::ServiceExt;

async fn call(router: &Router, request: Request<Body>) -> Response {
    router
        .clone()
        .oneshot(request)
        .await
        .expect("service call failed")
}

async fn response_body_string(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    String::from_utf8(bytes.to_vec()).expect("response body was not utf-8")
}

async fn response_json<T: DeserializeOwned>(response: Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read json body");
    serde_json::from_slice(&bytes).expect("failed to deserialize json body")
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .expect("failed to build GET request")
}

fn post(path: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("content-type", "application/json")
        .body(body)
        .expect("failed to build POST request")
}

#[tokio::test(flavor = "current_thread")]
async fn add_capability_route_reports_statuses() {
    let _guard = with_harness(|h| {
        h.add_capability.push_ok(());
        h.add_capability.push_err("capability operation failed");
    });

    let router = app();

    let ok_response = call(&router, post("/add_capability/demo", Body::empty())).await;
    assert_eq!(ok_response.status(), StatusCode::OK);

    let err_response = call(&router, post("/add_capability/demo", Body::empty())).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "capability operation failed"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn get_capabilities_route_serializes_core_response() {
    let capabilities = vec!["alpha".to_string(), "beta".to_string()];

    let _guard = with_harness(|h| {
        h.get_capabilities.push_ok(capabilities.clone());
        h.get_capabilities.push_err("capabilities unavailable");
    });

    let router = app();

    let ok_response = call(&router, get("/get_capabilities")).await;
    assert_eq!(ok_response.status(), StatusCode::OK);
    let body: Vec<String> = response_json(ok_response).await;
    assert_eq!(body, capabilities);

    let err_response = call(&router, get("/get_capabilities")).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "capabilities unavailable"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn update_package_config_forwards_payload_and_errors() {
    let values = HashMap::from([
        ("network".to_string(), "hoodi".to_string()),
        ("pruning".to_string(), "archive".to_string()),
    ]);

    let _guard = with_harness(|h| {
        h.update_package_config.push_ok(());
        h.update_package_config.push_err("write failed");
    });

    let router = app();
    let body = Body::from(
        json!({
            "values": {
                "network": "hoodi",
                "pruning": "archive"
            }
        })
        .to_string(),
    );

    let ok_response = call(&router, post("/update_package_config/demo", body)).await;
    assert_eq!(ok_response.status(), StatusCode::OK);

    let recorded = read_state(|state| state.update_package_config_calls.clone());
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].0, "demo");
    assert_eq!(recorded[0].1.values, values);

    let err_body = Body::from(json!({ "values": {} }).to_string());
    let err_response = call(&router, post("/update_package_config/demo", err_body)).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_body_string(err_response).await, "write failed");
}

#[tokio::test(flavor = "current_thread")]
async fn delete_package_includes_query_flag() {
    let _guard = with_harness(|h| {
        h.delete_package.push_ok(());
        h.delete_package.push_err("delete blocked");
    });

    let router = app();

    let ok_response = call(&router, post("/delete_package/demo", Body::empty())).await;
    assert_eq!(ok_response.status(), StatusCode::OK);

    let err_response = call(
        &router,
        post("/delete_package/demo?include_images=true", Body::empty()),
    )
    .await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response_body_string(err_response).await, "delete blocked");

    let calls = read_state(|state| state.delete_package_calls.clone());
    assert_eq!(
        calls,
        vec![("demo".to_string(), false), ("demo".to_string(), true),]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn docker_status_route_uses_boolean_outcome() {
    let _guard = with_harness(|h| {
        h.is_docker_running.push_ok(true);
        h.is_docker_running.push_ok(false);
    });

    let router = app();

    let ok_response = call(&router, get("/is_docker_running")).await;
    assert_eq!(ok_response.status(), StatusCode::OK);

    let err_response = call(&router, get("/is_docker_running")).await;
    assert_eq!(err_response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response_body_string(err_response).await,
        "Docker is not running"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn operational_state_route_maps_results() {
    let state = OperationalState {
        mode: OperationalMode::Remote,
        docker_running: true,
        can_install: false,
        can_manage: true,
        diagnostics: vec!["looks good".to_string()],
    };

    let _guard = with_harness(|h| {
        h.get_operational_state.push_ok(state.clone());
        h.get_operational_state.push_err("cannot read state");
    });

    let router = app();

    let ok_response = call(&router, get("/get_operational_state")).await;
    assert_eq!(ok_response.status(), StatusCode::OK);
    let body: OperationalState = response_json(ok_response).await;
    assert_eq!(body.diagnostics, state.diagnostics);
    assert_eq!(body.mode, state.mode);

    let err_response = call(&router, get("/get_operational_state")).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "cannot read state"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn package_runtime_route_returns_payload() {
    let runtime = PackageRuntimeState { running: true };

    let _guard = with_harness(|h| {
        h.get_package_runtime_state.push_ok(runtime.clone());
        h.get_package_runtime_state.push_err("runtime unavailable");
    });

    let router = app();

    let ok_response = call(&router, get("/package_runtime/demo")).await;
    assert_eq!(ok_response.status(), StatusCode::OK);
    let body: PackageRuntimeState = response_json(ok_response).await;
    assert_eq!(body.running, true);

    let err_response = call(&router, get("/package_runtime/demo")).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "runtime unavailable"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn package_config_route_returns_json_and_errors() {
    let config = PackageConfig {
        values: HashMap::from([
            ("api".to_string(), "enabled".to_string()),
            ("slots".to_string(), "1024".to_string()),
        ]),
    };

    let _guard = with_harness(|h| {
        h.get_package_config.push_ok(config.clone());
        h.get_package_config.push_err("config unavailable");
    });

    let router = app();

    let ok_response = call(&router, get("/get_package_config/demo")).await;
    assert_eq!(ok_response.status(), StatusCode::OK);
    let body: PackageConfig = response_json(ok_response).await;
    assert_eq!(body.values, config.values);

    let err_response = call(&router, get("/get_package_config/demo")).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "config unavailable"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn start_docker_route_relays_core_status() {
    let _guard = with_harness(|h| {
        h.start_docker_if_needed
            .push_ok(DockerStartStatus::Starting);
        h.start_docker_if_needed.push_err("auto-start disabled");
    });

    let router = app();

    let ok_response = call(&router, post("/start_docker_if_needed", Body::empty())).await;
    assert_eq!(ok_response.status(), StatusCode::OK);
    let body: DockerStartStatus = response_json(ok_response).await;
    assert_eq!(body, DockerStartStatus::Starting);

    let err_response = call(&router, post("/start_docker_if_needed", Body::empty())).await;
    assert_eq!(err_response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body_string(err_response).await,
        "auto-start disabled"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn run_with_port_respects_validate_web_port_failures() {
    let _guard = with_harness(|h| {
        h.validate_web_port.push_err("port rejected");
    });

    let error = run_with_port(4242)
        .await
        .expect_err("run_with_port should fail");
    assert_eq!(error.to_string(), "port rejected");

    let calls = read_state(|state| state.validate_web_port_calls.clone());
    assert_eq!(calls, vec![4242]);
}

#[tokio::test(flavor = "current_thread")]
async fn run_with_port_skips_server_when_configured() {
    let _guard = with_harness(|h| {
        h.validate_web_port.push_ok(5151);
        h.skip_server_start = true;
    });

    run_with_port(5151)
        .await
        .expect("run_with_port should succeed when server start skipped");

    let calls = read_state(|state| state.validate_web_port_calls.clone());
    assert_eq!(calls, vec![5151]);
}
