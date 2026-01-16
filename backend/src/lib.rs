pub mod models;
pub mod app;
pub mod p2p;
pub mod routes;
pub mod state;
pub mod response;
pub mod resp;
pub mod tunnel;
pub mod proxy;

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
