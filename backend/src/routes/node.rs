use axum::{extract::State, routing::get, Router};

use crate::{resp, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/node/info", get(get_node_info))
        .route("/porta/node/config", get(ok))
        .route("/porta/node/key/import", get(ok))
        .route("/porta/node/key/generate", get(ok))
        .with_state(state)
}

async fn get_node_info(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let data = state.store.node_info().await;
    resp::ok(Some(data))
}

async fn ok() -> impl axum::response::IntoResponse {
    resp::ok::<()> (None)
}
