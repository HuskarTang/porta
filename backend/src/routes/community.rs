use axum::{extract::State, routing::get, routing::post, Json, Router};
use crate::{
    models::{CommunityAddRequest, ToggleRequest},
    resp,
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/community/list", get(get_communities))
        .route("/porta/community/add", post(add_community))
        .route("/porta/community/remove", post(remove_community))
        .route("/porta/community/connect", post(connect_community))
        .route("/porta/community/node/list", get(get_nodes))
        .route("/porta/community/node/ban", post(ban_node))
        .route("/porta/community/node/unban", post(unban_node))
        .route("/porta/community/service/list", get(get_services))
        .route("/porta/community/service/announce", post(announce_service))
        .route("/porta/community/service/disable", post(disable_service))
        .with_state(state)
}

async fn get_communities(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.communities().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("读取社区列表失败: {}", err)),
    }
}

async fn get_nodes(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.community_nodes().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("读取节点列表失败: {}", err)),
    }
}

async fn get_services(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.community_services().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("读取服务列表失败: {}", err)),
    }
}

async fn add_community(
    State(state): State<AppState>,
    Json(req): Json<CommunityAddRequest>,
) -> impl axum::response::IntoResponse {
    match state.app.add_community(req).await {
        Ok(item) => resp::ok(Some(item)),
        Err(err) => resp::err(&format!("新增社区失败: {}", err)),
    }
}

async fn remove_community(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    match state.app.remove_community(&req.id).await {
        Ok(()) => resp::ok::<()> (None),
        Err(err) => resp::err(&format!("移除社区失败: {}", err)),
    }
}

async fn connect_community(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    tracing::info!("[API] 收到连接社区请求: id={}", req.id);
    match state.app.connect_community(&req.id).await {
        Ok(()) => {
            tracing::info!("[API] 社区连接成功: id={}", req.id);
            resp::ok::<()> (None)
        }
        Err(err) => {
            let error_msg = format!("连接社区失败: {}", err);
            tracing::error!("[API] 社区连接失败: id={}, error={}", req.id, error_msg);
            resp::err(&error_msg)
        }
    }
}

async fn ban_node(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    match state.store.set_node_ban(&req.id, true).await {
        Ok(true) => resp::ok::<()> (None),
        Ok(false) => resp::err("未找到节点"),
        Err(err) => resp::err(&format!("封禁节点失败: {}", err)),
    }
}

async fn unban_node(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    match state.store.set_node_ban(&req.id, false).await {
        Ok(true) => resp::ok::<()> (None),
        Ok(false) => resp::err("未找到节点"),
        Err(err) => resp::err(&format!("解封节点失败: {}", err)),
    }
}

async fn announce_service(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    match state.store.set_service_announced(&req.id, true).await {
        Ok(true) => resp::ok::<()> (None),
        Ok(false) => resp::err("未找到服务"),
        Err(err) => resp::err(&format!("公告服务失败: {}", err)),
    }
}

async fn disable_service(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    match state.store.set_service_announced(&req.id, false).await {
        Ok(true) => resp::ok::<()> (None),
        Ok(false) => resp::err("未找到服务"),
        Err(err) => resp::err(&format!("禁用服务失败: {}", err)),
    }
}

