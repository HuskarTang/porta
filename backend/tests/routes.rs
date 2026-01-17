use axum::{body, body::Body, http::Request};
use porta_backend::create_app;
use serde_json::{json, Value};
use tower::util::ServiceExt;

fn setup_env() {
    std::env::set_var("PORTA_DB", ":memory:");
}

// ===========================================================================
// Node Configuration Tests
// ===========================================================================

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
async fn node_info_should_contain_all_fields() {
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
    let data = &json["data"];
    assert!(data.get("name").is_some());
    assert!(data.get("node_id").is_some());
    assert!(data.get("uuid").is_some());
    assert!(data.get("key_path").is_some());
    assert!(data.get("tcp_listen_enable").is_some());
    assert!(data.get("tcp_listen_port").is_some());
    assert!(data.get("quci_listen_enable").is_some());
    assert!(data.get("quci_listen_port").is_some());
    assert!(data.get("external_addr").is_some());
    assert!(data.get("mdns_enable").is_some());
    assert!(data.get("dht_enable").is_some());
}

#[tokio::test]
async fn node_config_update_should_change_name() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "name": "新节点名称" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/node/config")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["name"], "新节点名称");
}

#[tokio::test]
async fn node_config_update_tcp_settings() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "tcp_listen_enable": true,
        "tcp_listen_port": 9000
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/node/config")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["tcp_listen_port"], 9000);
    assert_eq!(json["data"]["tcp_listen_enable"], true);
}

#[tokio::test]
async fn node_key_generate_should_update_path() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/node/key/generate")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    let key_path = json["data"]["key_path"].as_str().unwrap();
    assert!(key_path.starts_with("porta-"));
    assert!(key_path.ends_with(".key"));
}

#[tokio::test]
async fn node_key_import_requires_path() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "key_path": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/node/key/import")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

// ===========================================================================
// Community Management Tests
// ===========================================================================

#[tokio::test]
async fn communities_list_returns_array() {
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
    assert!(json.get("data").and_then(|v| v.as_array()).is_some());
}

#[tokio::test]
async fn communities_contain_required_fields() {
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
    let communities = json["data"].as_array().unwrap();
    for community in communities {
        assert!(community.get("id").is_some());
        assert!(community.get("name").is_some());
        assert!(community.get("description").is_some());
        assert!(community.get("peers").is_some());
        assert!(community.get("joined").is_some());
    }
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
async fn community_add_requires_name() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "",
        "description": "测试社区",
        "multiaddr": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
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
async fn community_add_requires_description() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "测试社区",
        "description": "",
        "multiaddr": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
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
async fn community_remove_nonexistent() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "nonexistent-id" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/remove")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn community_node_list_returns_array() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/community/node/list")
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
    assert!(json["data"].is_array());
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

// ===========================================================================
// Service Discovery and Subscription Tests
// ===========================================================================

#[tokio::test]
async fn service_discover_returns_list() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/service/discover")
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
async fn service_subscriptions_returns_list() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/service/subscriptions")
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
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn service_subscribe_requires_name() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "",
        "type": "HTTP",
        "community": "dev",
        "remote_addr": "127.0.0.1:8080",
        "local_mapping": "127.0.0.1:18080"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/subscribe")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_subscribe_requires_type() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "Test Service",
        "type": "",
        "community": "dev",
        "remote_addr": "127.0.0.1:8080",
        "local_mapping": "127.0.0.1:18080"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/subscribe")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_connect_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/connect")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_disconnect_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/disconnect")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

// ===========================================================================
// Service Publishing Tests
// ===========================================================================

#[tokio::test]
async fn service_published_returns_list() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/porta/service/published")
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
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn service_publish_requires_name() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "",
        "type": "HTTP",
        "port": 8080,
        "summary": "Test service"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/publish")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_publish_requires_type() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "name": "Test Service",
        "type": "",
        "port": 8080,
        "summary": "Test service"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/publish")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_unpublish_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/unpublish")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn service_remove_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/remove")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

// ===========================================================================
// Secure Routing Tests
// ===========================================================================

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
async fn secure_connect_requires_subscription_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({
        "subscription_id": "",
        "relay_peers": ["peer-1", "peer-2"]
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
async fn secure_disconnect_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/secure-disconnect")
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

// ===========================================================================
// Session Tests
// ===========================================================================

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

// ===========================================================================
// Proxy Tests
// ===========================================================================

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
async fn proxy_status_contains_enabled_field() {
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
    assert!(json["data"].get("enabled").is_some());
}

// ===========================================================================
// Access URL Tests
// ===========================================================================

#[tokio::test]
async fn service_access_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/service/access")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_client_error());
}

// ===========================================================================
// Community Node Management Tests
// ===========================================================================

#[tokio::test]
async fn community_node_ban_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "nonexistent" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/node/ban")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should fail because node doesn't exist
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn community_node_unban_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "nonexistent" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/node/unban")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should fail because node doesn't exist
    assert!(response.status().is_client_error());
}

// ===========================================================================
// Community Service Management Tests
// ===========================================================================

#[tokio::test]
async fn community_service_announce_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "nonexistent" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/service/announce")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should fail because service doesn't exist
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn community_service_disable_requires_id() {
    setup_env();
    let app = create_app().await;
    let payload = json!({ "id": "nonexistent" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/porta/community/service/disable")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should fail because service doesn't exist
    assert!(response.status().is_client_error());
}

// ===========================================================================
// API Response Format Tests
// ===========================================================================

#[tokio::test]
async fn api_response_has_correct_format() {
    setup_env();
    let app = create_app().await;
    let response = app
        .oneshot(Request::builder().uri("/porta/node/info").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("code").is_some());
    assert!(json.get("message").is_some());
    assert!(json.get("data").is_some());
    assert_eq!(json["code"], 0);
}
