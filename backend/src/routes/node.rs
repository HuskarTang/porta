use axum::{extract::State, routing::get, routing::post, Json, Router};

use crate::{
    models::{KeyImportRequest, NodeConfigUpdate},
    resp,
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/node/info", get(get_node_info))
        .route("/porta/node/config", post(update_config))
        .route("/porta/node/key/import", post(import_key))
        .route("/porta/node/key/generate", post(generate_key))
        .with_state(state)
}

async fn get_node_info(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.node_info().await {
        Ok(data) => resp::ok(Some(data)),
        Err(err) => resp::err(&format!("读取节点信息失败: {}", err)),
    }
}

async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<NodeConfigUpdate>,
) -> impl axum::response::IntoResponse {
    match state.store.update_node_config(req).await {
        Ok(updated) => resp::ok(Some(updated)),
        Err(err) => resp::err(&format!("更新配置失败: {}", err)),
    }
}

async fn import_key(
    State(state): State<AppState>,
    Json(req): Json<KeyImportRequest>,
) -> impl axum::response::IntoResponse {
    if req.key_path.is_empty() {
        return resp::err("缺少 key_path");
    }
    match state.store.import_key(req).await {
        Ok(updated) => resp::ok(Some(updated)),
        Err(err) => resp::err(&format!("导入密钥失败: {}", err)),
    }
}

async fn generate_key(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.generate_key().await {
        Ok(updated) => resp::ok(Some(updated)),
        Err(err) => resp::err(&format!("生成密钥失败: {}", err)),
    }
}
