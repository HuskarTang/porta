use axum::{extract::State, routing::get, routing::post, Json, Router};

use crate::{models::ProxyToggle, resp, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/proxy/enable", post(enable_proxy))
        .route("/porta/proxy/disable", post(disable_proxy))
        .route("/porta/proxy/status", get(get_proxy_status))
        .with_state(state)
}

async fn get_proxy_status(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.proxy_status().await {
        Ok(status) => resp::ok(Some(status)),
        Err(err) => resp::err(&format!("获取代理状态失败: {}", err)),
    }
}

async fn ok() -> impl axum::response::IntoResponse {
    resp::ok::<()> (None)
}

async fn enable_proxy(
    State(state): State<AppState>,
    Json(_): Json<ProxyToggle>,
) -> impl axum::response::IntoResponse {
    match state.store.set_proxy_enabled(true).await {
        Ok(()) => ok().await,
        Err(err) => resp::err(&format!("启用代理失败: {}", err)),
    }
}

async fn disable_proxy(
    State(state): State<AppState>,
    Json(_): Json<ProxyToggle>,
) -> impl axum::response::IntoResponse {
    match state.store.set_proxy_enabled(false).await {
        Ok(()) => ok().await,
        Err(err) => resp::err(&format!("禁用代理失败: {}", err)),
    }
}
