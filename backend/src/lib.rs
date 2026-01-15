pub mod models;
pub mod mock;
pub mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub fn create_app() -> Router {
    Router::new()
        .merge(routes::node::router())
        .merge(routes::community::router())
        .merge(routes::service::router())
        .merge(routes::proxy::router())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
}
