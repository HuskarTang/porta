use axum::{routing::get, Json, Router};

use crate::mock;

pub fn router() -> Router {
    Router::new()
        .route("/porta/node/info", get(get_node_info))
        .route("/porta/node/config", get(ok))
        .route("/porta/node/key/import", get(ok))
        .route("/porta/node/key/generate", get(ok))
}

async fn get_node_info() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::node_info()).unwrap())
}

async fn ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}
