use axum::{body, body::Body, http::Request};
use porta_backend::create_app;
use serde_json::Value;
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
