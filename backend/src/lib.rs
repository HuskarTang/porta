pub mod app;
pub mod models;
pub mod p2p;
pub mod proxy;
pub mod resp;
pub mod response;
pub mod routes;
pub mod state;
pub mod tunnel;

use axum::Router;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

pub async fn create_app() -> Router {
    let state = AppState::new().await.expect("init state");
    Router::new()
        .merge(routes::node::router(state.clone()))
        .merge(routes::community::router(state.clone()))
        .merge(routes::service::router(state.clone()))
        .merge(routes::proxy::router(state))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
}
