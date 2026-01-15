use axum::{body, body::Body, http::Request};
use porta_backend::create_app;
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn node_info_should_return_name() {
    let app = create_app();
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
    let app = create_app();
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
    let app = create_app();
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
    let first_protocol = json
        .get("data")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("protocol"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(!first_protocol.is_empty());
}
