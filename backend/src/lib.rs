pub mod models;
pub mod mock;
pub mod routes;
pub mod state;
pub mod response;
pub mod resp;

use axum::Router;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

pub fn create_app() -> Router {
    let state = AppState::default();
    Router::new()
        .merge(routes::node::router(state.clone()))
        .merge(routes::community::router(state.clone()))
        .merge(routes::service::router(state.clone()))
        .merge(routes::proxy::router(state))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
}
