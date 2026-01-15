use async_trait::async_trait;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{mock, models::*};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn Store>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Arc::new(DataStore::from_mock()),
        }
    }
}

#[async_trait]
pub trait Store: Send + Sync {
    async fn node_info(&self) -> NodeInfo;
    async fn communities(&self) -> Vec<CommunitySummary>;
    async fn community_nodes(&self) -> Vec<CommunityNode>;
    async fn community_services(&self) -> Vec<CommunityService>;
    async fn discovered_services(&self) -> Vec<ServiceDescriptor>;
    async fn subscribed_services(&self) -> Vec<SubscribedService>;
    async fn published_services(&self) -> Vec<PublishedService>;
    async fn proxy_status(&self) -> ProxyStatus;

    async fn subscribe_service(&self, req: SubscribeRequest) -> SubscribedService;
    async fn update_subscription_status(&self, id: &str, status: &str) -> bool;
    async fn publish_service(&self, req: PublishRequest) -> PublishedService;
    async fn unpublish_service(&self, id: &str) -> bool;
    async fn remove_published(&self, id: &str) -> bool;
    async fn set_service_announced(&self, id: &str, announced: bool) -> bool;
    async fn set_node_ban(&self, id: &str, banned: bool) -> bool;
    async fn set_proxy_enabled(&self, enabled: bool);
}

pub struct DataStore {
    node_info: RwLock<NodeInfo>,
    communities: RwLock<Vec<CommunitySummary>>,
    community_nodes: RwLock<Vec<CommunityNode>>,
    community_services: RwLock<Vec<CommunityService>>,
    discovered: RwLock<Vec<ServiceDescriptor>>,
    subscribed: RwLock<Vec<SubscribedService>>,
    published: RwLock<Vec<PublishedService>>,
    proxy_status: RwLock<ProxyStatus>,
}

impl DataStore {
    fn from_mock() -> Self {
        Self {
            node_info: RwLock::new(mock::node_info()),
            communities: RwLock::new(mock::communities()),
            community_nodes: RwLock::new(mock::community_nodes()),
            community_services: RwLock::new(mock::community_services()),
            discovered: RwLock::new(mock::discovered_services()),
            subscribed: RwLock::new(mock::subscribed_services()),
            published: RwLock::new(mock::published_services()),
            proxy_status: RwLock::new(mock::proxy_status()),
        }
    }
}

#[async_trait]
impl Store for DataStore {
    async fn node_info(&self) -> NodeInfo {
        self.node_info.read().await.clone()
    }

    async fn communities(&self) -> Vec<CommunitySummary> {
        self.communities.read().await.clone()
    }

    async fn community_nodes(&self) -> Vec<CommunityNode> {
        self.community_nodes.read().await.clone()
    }

    async fn community_services(&self) -> Vec<CommunityService> {
        self.community_services.read().await.clone()
    }

    async fn discovered_services(&self) -> Vec<ServiceDescriptor> {
        self.discovered.read().await.clone()
    }

    async fn subscribed_services(&self) -> Vec<SubscribedService> {
        self.subscribed.read().await.clone()
    }

    async fn published_services(&self) -> Vec<PublishedService> {
        self.published.read().await.clone()
    }

    async fn proxy_status(&self) -> ProxyStatus {
        self.proxy_status.read().await.clone()
    }

    async fn subscribe_service(&self, req: SubscribeRequest) -> SubscribedService {
        let mut list = self.subscribed.write().await;
        let id = req
            .id
            .clone()
            .unwrap_or_else(|| format!("sub-{}", list.len() + 1));
        if let Some(existing) = list.iter_mut().find(|s| s.id == id) {
            existing.name = req.name;
            existing.r#type = req.r#type;
            existing.community = req.community;
            existing.remote_addr = req.remote_addr;
            existing.local_mapping = req.local_mapping;
            return existing.clone();
        }
        let new_item = SubscribedService {
            id: id.clone(),
            name: req.name,
            r#type: req.r#type,
            community: req.community,
            remote_addr: req.remote_addr,
            local_mapping: req.local_mapping,
            status: "畅通".into(),
        };
        list.push(new_item.clone());
        new_item
    }

    async fn update_subscription_status(&self, id: &str, status: &str) -> bool {
        let mut list = self.subscribed.write().await;
        if let Some(item) = list.iter_mut().find(|s| s.id == id) {
            item.status = status.to_string();
            true
        } else {
            false
        }
    }

    async fn publish_service(&self, req: PublishRequest) -> PublishedService {
        let mut list = self.published.write().await;
        let id = req
            .id
            .clone()
            .unwrap_or_else(|| format!("pub-{}", list.len() + 1));
        if let Some(existing) = list.iter_mut().find(|s| s.id == id) {
            existing.name = req.name;
            existing.r#type = req.r#type;
            existing.port = req.port;
            existing.summary = req.summary;
            existing.status = "在线".into();
            return existing.clone();
        }
        let new_item = PublishedService {
            id: id.clone(),
            name: req.name,
            r#type: req.r#type,
            port: req.port,
            summary: req.summary,
            subscriptions: 0,
            status: "在线".into(),
            publish_date: "2026-01-10".into(),
        };
        list.push(new_item.clone());
        new_item
    }

    async fn unpublish_service(&self, id: &str) -> bool {
        let mut list = self.published.write().await;
        if let Some(item) = list.iter_mut().find(|s| s.id == id) {
            item.status = "已下架".into();
            true
        } else {
            false
        }
    }

    async fn remove_published(&self, id: &str) -> bool {
        let mut list = self.published.write().await;
        let len_before = list.len();
        list.retain(|s| s.id != id);
        len_before != list.len()
    }

    async fn set_service_announced(&self, id: &str, announced: bool) -> bool {
        let mut list = self.community_services.write().await;
        if let Some(item) = list.iter_mut().find(|s| s.id == id) {
            item.announced = announced;
            true
        } else {
            false
        }
    }

    async fn set_node_ban(&self, id: &str, banned: bool) -> bool {
        let mut list = self.community_nodes.write().await;
        if let Some(item) = list.iter_mut().find(|n| n.id == id) {
            item.banned = banned;
            true
        } else {
            false
        }
    }

    async fn set_proxy_enabled(&self, enabled: bool) {
        let mut status = self.proxy_status.write().await;
        status.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_load_mock_data() {
        let state = AppState::default();
        let communities = state.store.communities().await;
        assert_eq!(communities.len(), 3);
        let proxy = state.store.proxy_status().await;
        assert!(proxy.enabled);
    }
}
