use axum::{routing::get, routing::post, Json, Router};

use crate::mock;

pub fn router() -> Router {
    Router::new()
        .route("/porta/proxy/enable", post(ok))
        .route("/porta/proxy/disable", post(ok))
        .route("/porta/proxy/status", get(get_proxy_status))
}

async fn get_proxy_status() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::proxy_status()).unwrap())
}

async fn ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}
