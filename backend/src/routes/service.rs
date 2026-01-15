use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde_json::json;

use crate::{
    models::{PublishRequest, SubscribeRequest, UpdateSessionRequest},
    resp,
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/porta/service/discover", get(get_discovered_services))
        .route("/porta/service/subscribe", post(subscribe))
        .route("/porta/service/subscriptions", get(get_subscribed_services))
        .route("/porta/service/connect", post(connect))
        .route("/porta/service/disconnect", post(disconnect))
        .route("/porta/service/sessions", get(get_sessions))
        .route("/porta/service/access", post(get_access_url))
        .route("/porta/service/publish", post(publish))
        .route("/porta/service/unpublish", post(unpublish))
        .route("/porta/service/remove", post(remove_publish))
        .route("/porta/service/published", get(get_published_services))
        .with_state(state)
}

async fn get_discovered_services(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.discovered_services().await))
}

async fn get_subscribed_services(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.subscribed_services().await))
}

async fn subscribe(
    State(state): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> impl axum::response::IntoResponse {
    if req.name.is_empty() || req.r#type.is_empty() || req.community.is_empty() {
        return resp::err("缺少必填字段 name/type/community");
    }
    let saved = state.store.subscribe_service(req).await;
    resp::ok(Some(saved))
}

async fn connect(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    let updated = state
        .store
        .update_subscription_status(&req.id, "畅通")
        .await;
    if updated {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到订阅")
    }
}

async fn disconnect(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    let updated = state
        .store
        .update_subscription_status(&req.id, "断开")
        .await;
    if updated {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到订阅")
    }
}

async fn get_sessions() -> impl axum::response::IntoResponse {
    resp::ok(Some(json!([
        { "session_id": "sess-001", "state": "connected" },
        { "session_id": "sess-002", "state": "connecting" }
    ])))
}

async fn get_access_url() -> impl axum::response::IntoResponse {
    resp::ok(Some(json!({ "local_url": "http://localhost:8080" })))
}

async fn get_published_services(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    resp::ok(Some(state.store.published_services().await))
}

async fn publish(
    State(state): State<AppState>,
    Json(req): Json<PublishRequest>,
) -> impl axum::response::IntoResponse {
    if req.name.is_empty() || req.r#type.is_empty() {
        return resp::err("缺少必填字段 name/type");
    }
    let saved = state.store.publish_service(req).await;
    resp::ok(Some(saved))
}

async fn unpublish(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    let updated = state.store.unpublish_service(&req.id).await;
    if updated {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到发布服务")
    }
}

async fn remove_publish(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    let removed = state.store.remove_published(&req.id).await;
    if removed {
        resp::ok::<()> (None)
    } else {
        resp::err("未找到发布服务")
    }
}
