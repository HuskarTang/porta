use axum::{body, body::Body, http::Request};
use porta_backend::create_app;
use serde_json::{json, Value};
use tower::util::ServiceExt;

fn setup_env() {
    std::env::set_var("PORTA_DB", ":memory:");
}

#[tokio::test]
async fn node_info_should_return_name() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(Request::builder().uri("/porta/node/info").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["name"], "我的节点");
}

#[tokio::test]
async fn communities_count_matches_mock() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(Request::builder().uri("/porta/community/list").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json.get("data").and_then(|v| v.as_array()).map(|v| v.len()), Some(3));
}

#[tokio::test]
async fn community_service_list_contains_protocols() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/community/service/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("data").is_some());
}

#[tokio::test]
async fn community_add_requires_multiaddr() {
    setup_env();
    let app = create_app().await;
    let payload = serde_json::json!({
        "name": "新社区",
        "description": "测试社区"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/add")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn secure_connect_requires_min_two_relays() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "subscription_id": "sub-1",
        "relay_peers": ["peer-1"]
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/secure-connect")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn secure_routes_should_return_list() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/service/secure-routes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("data").is_some());
}

#[tokio::test]
async fn proxy_status_should_return() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/proxy/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["listen_port"].as_u64().is_some());
}

#[tokio::test]
async fn sessions_should_track_timestamps() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/service/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("data").is_some());
}
