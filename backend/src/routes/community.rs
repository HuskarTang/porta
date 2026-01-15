use axum::{extract::State, routing::get, routing::post, Json, Router};
use crate::{models::ToggleRequest, resp, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/community/list", get(get_communities))
        .route("/porta/community/add", post(ok))
        .route("/porta/community/remove", post(ok))
        .route("/porta/community/connect", post(ok))
        .route("/porta/community/node/list", get(get_nodes))
        .route("/porta/community/node/ban", post(ban_node))
        .route("/porta/community/node/unban", post(unban_node))
        .route("/porta/community/service/list", get(get_services))
        .route("/porta/community/service/announce", post(announce_service))
        .route("/porta/community/service/disable", post(disable_service))
        .with_state(state)
}

async fn get_communities(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.communities().await))
}

async fn get_nodes(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.community_nodes().await))
}

async fn get_services(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.community_services().await))
}

async fn ban_node(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    let ok = state.store.set_node_ban(&req.id, true).await;
    if ok {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到节点")
    }
}

async fn unban_node(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    let ok = state.store.set_node_ban(&req.id, false).await;
    if ok {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到节点")
    }
}

async fn announce_service(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    let ok = state.store.set_service_announced(&req.id, true).await;
    if ok {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到服务")
    }
}

async fn disable_service(
    State(state): State<AppState>,
    Json(req): Json<ToggleRequest>,
) -> impl axum::response::IntoResponse {
    let ok = state.store.set_service_announced(&req.id, false).await;
    if ok {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到服务")
    }
}

async fn ok() -> impl axum::response::IntoResponse {
    resp::ok::<()> (None)
}
