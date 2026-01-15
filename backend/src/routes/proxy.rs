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
    resp::ok(Some(state.store.proxy_status().await))
}

async fn ok() -> impl axum::response::IntoResponse {
    resp::ok::<()> (None)
}

async fn enable_proxy(
    State(state): State<AppState>,
    Json(_): Json<ProxyToggle>,
) -> impl axum::response::IntoResponse {
    state.store.set_proxy_enabled(true).await;
    ok().await
}

async fn disable_proxy(
    State(state): State<AppState>,
    Json(_): Json<ProxyToggle>,
) -> impl axum::response::IntoResponse {
    state.store.set_proxy_enabled(false).await;
    ok().await
}
