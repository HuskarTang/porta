use axum::{extract::Query, extract::State, routing::get, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::{
    models::{
        AccessRequest, PublishRequest, SecureConnectRequest, SubscribeRequest, UpdateSessionRequest,
    },
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
        .route("/porta/service/secure-connect", post(secure_connect))
        .route("/porta/service/secure-disconnect", post(secure_disconnect))
        .route("/porta/service/secure-routes", get(get_secure_routes))
        .with_state(state)
}

#[derive(Deserialize)]
struct DiscoverQuery {
    #[serde(rename = "communityId")]
    community_id: Option<String>,
}

async fn get_discovered_services(
    State(state): State<AppState>,
    Query(query): Query<DiscoverQuery>,
) -> impl axum::response::IntoResponse {
    match state.app.discover_services(query.community_id).await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("获取服务发现失败: {}", err)),
    }
}

async fn get_subscribed_services(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    match state.store.subscribed_services().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("获取订阅列表失败: {}", err)),
    }
}

async fn subscribe(
    State(state): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> impl axum::response::IntoResponse {
    if req.name.is_empty() || req.r#type.is_empty() || req.community.is_empty() {
        return resp::err("缺少必填字段 name/type/community");
    }
    match state.app.subscribe_service(req).await {
        Ok(saved) => resp::ok(Some(saved)),
        Err(err) => resp::err(&format!("订阅失败: {}", err)),
    }
}

async fn connect(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.app.connect_service(&req.id).await {
        Ok(()) => resp::ok::<()>(None),
        Err(err) => resp::err(&format!("连接订阅失败: {}", err)),
    }
}

async fn disconnect(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.app.disconnect_service(&req.id).await {
        Ok(()) => resp::ok::<()>(None),
        Err(err) => resp::err(&format!("断开订阅失败: {}", err)),
    }
}

async fn get_sessions(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.sessions().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("获取会话失败: {}", err)),
    }
}

async fn get_access_url(
    State(state): State<AppState>,
    Json(req): Json<AccessRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.store.find_subscription(&req.id).await {
        Ok(Some(subscription)) => {
            let url = format!("http://{}", subscription.local_mapping);
            resp::ok(Some(json!({ "local_url": url })))
        }
        Ok(None) => resp::err("未找到订阅"),
        Err(err) => resp::err(&format!("获取访问地址失败: {}", err)),
    }
}

async fn get_published_services(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    match state.store.published_services().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("获取发布列表失败: {}", err)),
    }
}

async fn publish(
    State(state): State<AppState>,
    Json(req): Json<PublishRequest>,
) -> impl axum::response::IntoResponse {
    if req.name.is_empty() || req.r#type.is_empty() {
        return resp::err("缺少必填字段 name/type");
    }
    match state.app.publish_service(req).await {
        Ok(saved) => resp::ok(Some(saved)),
        Err(err) => resp::err(&format!("发布失败: {}", err)),
    }
}

async fn unpublish(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.app.unpublish_service(&req.id).await {
        Ok(()) => resp::ok::<()>(None),
        Err(err) => resp::err(&format!("下架失败: {}", err)),
    }
}

async fn remove_publish(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.store.remove_published(&req.id).await {
        Ok(true) => resp::ok::<()>(None),
        Ok(false) => resp::err("未找到发布服务"),
        Err(err) => resp::err(&format!("删除失败: {}", err)),
    }
}

async fn secure_connect(
    State(state): State<AppState>,
    Json(req): Json<SecureConnectRequest>,
) -> impl axum::response::IntoResponse {
    if req.subscription_id.is_empty() {
        return resp::err("缺少 subscription_id");
    }
    if req.relay_peers.len() < 2 {
        return resp::err("至少需要两个中继节点");
    }
    match state.app.secure_connect_service(req).await {
        Ok(route) => resp::ok(Some(route)),
        Err(err) => resp::err(&format!("建立安全连接失败: {}", err)),
    }
}

async fn secure_disconnect(
    State(state): State<AppState>,
    Json(req): Json<UpdateSessionRequest>,
) -> impl axum::response::IntoResponse {
    if req.id.is_empty() {
        return resp::err("缺少 id");
    }
    match state.app.disconnect_secure_route(&req.id).await {
        Ok(()) => resp::ok::<()>(None),
        Err(err) => resp::err(&format!("断开安全路由失败: {}", err)),
    }
}

async fn get_secure_routes(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match state.store.secure_routes().await {
        Ok(list) => resp::ok(Some(list)),
        Err(err) => resp::err(&format!("获取安全路由失败: {}", err)),
    }
}
