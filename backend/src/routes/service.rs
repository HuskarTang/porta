use axum::{routing::get, routing::post, Json, Router};

use crate::mock;

pub fn router() -> Router {
    Router::new()
        .route("/porta/service/discover", get(get_discovered_services))
        .route("/porta/service/subscribe", post(ok))
        .route("/porta/service/subscriptions", get(get_subscribed_services))
        .route("/porta/service/connect", post(ok))
        .route("/porta/service/disconnect", post(ok))
        .route("/porta/service/sessions", get(get_sessions))
        .route("/porta/service/access", post(get_access_url))
        .route("/porta/service/publish", post(ok))
        .route("/porta/service/unpublish", post(ok))
        .route("/porta/service/remove", post(ok))
        .route("/porta/service/published", get(get_published_services))
}

async fn get_discovered_services() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::discovered_services()).unwrap())
}

async fn get_subscribed_services() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::subscribed_services()).unwrap())
}

async fn get_sessions() -> Json<serde_json::Value> {
    Json(serde_json::json!([
        { "session_id": "sess-001", "state": "connected" },
        { "session_id": "sess-002", "state": "connecting" }
    ]))
}

async fn get_access_url() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "local_url": "http://localhost:8080" }))
}

async fn get_published_services() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::published_services()).unwrap())
}

async fn ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}
