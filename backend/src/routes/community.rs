use axum::{routing::get, routing::post, Json, Router};

use crate::mock;

pub fn router() -> Router {
    Router::new()
        .route("/porta/community/list", get(get_communities))
        .route("/porta/community/add", post(ok))
        .route("/porta/community/remove", post(ok))
        .route("/porta/community/connect", post(ok))
        .route("/porta/community/node/list", get(get_nodes))
        .route("/porta/community/node/ban", post(ok))
        .route("/porta/community/node/unban", post(ok))
        .route("/porta/community/service/list", get(get_services))
        .route("/porta/community/service/announce", post(ok))
        .route("/porta/community/service/disable", post(ok))
}

async fn get_communities() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::communities()).unwrap())
}

async fn get_nodes() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::community_nodes()).unwrap())
}

async fn get_services() -> Json<serde_json::Value> {
    Json(serde_json::to_value(mock::community_services()).unwrap())
}

async fn ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}
