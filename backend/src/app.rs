use std::sync::Arc;

use anyhow::{anyhow, Result};
use libp2p::Multiaddr;

use crate::{
    models::{
        DiscoveredService, PublishedService, PublishRequest, ServiceDescriptor, SessionInfo,
        SubscribeRequest, SubscribedService,
    },
    p2p::{P2pRequest, P2pResponse},
    state::Store,
    tunnel,
};

#[derive(Clone)]
pub struct AppService {
    store: Arc<dyn Store>,
    p2p: crate::p2p::NodeHandle,
}

impl AppService {
    pub fn new(store: Arc<dyn Store>, p2p: crate::p2p::NodeHandle) -> Self {
        Self { store, p2p }
    }

    pub async fn connect_community(&self, id: &str) -> Result<()> {
        let Some(addr) = self.store.community_multiaddr(id).await? else {
            return Err(anyhow!("社区缺少 multiaddr"));
        };
        let addr: Multiaddr = addr.parse()?;
        self.p2p.dial(addr).await?;
        self.store.connect_community(id).await?;
        Ok(())
    }

    pub async fn discover_services(
        &self,
        community_id: Option<String>,
    ) -> Result<Vec<DiscoveredService>> {
        let Some(community_id) = community_id else {
            return Ok(self.store.discovered_services(None).await?);
        };
        let addr = self
            .store
            .community_multiaddr(&community_id)
            .await?
            .ok_or_else(|| anyhow!("社区缺少 multiaddr"))?;
        let addr: Multiaddr = addr.parse()?;
        let peer_id = self.p2p.dial(addr).await?;
        let response = self
            .p2p
            .request(peer_id, P2pRequest::DiscoverServices { community_id: community_id.clone() })
            .await?;
        let services = match response {
            P2pResponse::ServiceList { services } => services,
            _ => Vec::new(),
        };
        self.store
            .upsert_discovered_services(&community_id, services)
            .await?;
        let mut list = self
            .store
            .discovered_services(Some(community_id))
            .await?;
        let subscribed = self.store.subscribed_services().await?;
        for item in &mut list {
            let hit = subscribed
                .iter()
                .any(|sub| sub.service_uuid.as_deref() == Some(item.uuid.as_str()));
            if hit {
                item.subscribed = Some(true);
            }
        }
        Ok(list)
    }

    pub async fn subscribe_service(&self, req: SubscribeRequest) -> Result<SubscribedService> {
        let saved = self.store.subscribe_service(req.clone()).await?;
        if let Some(service_uuid) = req.service_uuid {
            if let Some(community_id) = self.find_community_for_service(&service_uuid).await? {
                let addr = self
                    .store
                    .community_multiaddr(&community_id)
                    .await?
                    .ok_or_else(|| anyhow!("社区缺少 multiaddr"))?;
                let addr: Multiaddr = addr.parse()?;
                let peer_id = self.p2p.dial(addr).await?;
                let _ = self
                    .p2p
                    .request(
                        peer_id,
                        P2pRequest::SubscribeService {
                            community_id,
                            service_uuid,
                        },
                    )
                    .await?;
            }
        }
        Ok(saved)
    }

    pub async fn connect_service(&self, id: &str) -> Result<()> {
        let Some(subscription) = self.store.find_subscription(id).await? else {
            return Err(anyhow!("未找到订阅"));
        };
        self.store.update_subscription_status(id, "畅通").await?;
        let local_port = parse_local_port(&subscription.local_mapping)?;
        let session = SessionInfo {
            session_id: format!("sess-{}", id),
            service_id: id.to_string(),
            local_port,
            remote_peer: subscription.remote_addr.clone(),
            state: "connected".into(),
        };
        self.store.upsert_session(session).await?;
        tunnel::ensure_mapping(local_port, subscription.remote_addr).await?;
        Ok(())
    }

    pub async fn disconnect_service(&self, id: &str) -> Result<()> {
        self.store.update_subscription_status(id, "断开").await?;
        let session = SessionInfo {
            session_id: format!("sess-{}", id),
            service_id: id.to_string(),
            local_port: 0,
            remote_peer: "".into(),
            state: "closed".into(),
        };
        self.store.upsert_session(session).await?;
        Ok(())
    }

    pub async fn publish_service(&self, req: PublishRequest) -> Result<PublishedService> {
        let published = self.store.publish_service(req.clone()).await?;
        let communities = self.store.communities().await?;
        for community in communities.into_iter().filter(|c| c.joined) {
            if let Some(addr) = community.multiaddr.clone() {
                if let Ok(peer_id) = self.p2p.dial(addr.parse()?).await {
                    let _ = self
                        .p2p
                        .request(
                            peer_id,
                            P2pRequest::PublishService {
                                service: ServiceDescriptor {
                                    uuid: published.id.clone(),
                                    name: published.name.clone(),
                                    r#type: published.r#type.clone(),
                                    remote_port: published.port,
                                    provider: self.p2p.peer_id(),
                                    description: published.summary.clone(),
                                },
                            },
                        )
                        .await;
                }
            }
        }
        Ok(published)
    }

    pub async fn unpublish_service(&self, id: &str) -> Result<()> {
        let updated = self.store.unpublish_service(id).await?;
        if !updated {
            return Err(anyhow!("未找到发布服务"));
        }
        let communities = self.store.communities().await?;
        for community in communities.into_iter().filter(|c| c.joined) {
            if let Some(addr) = community.multiaddr.clone() {
                if let Ok(peer_id) = self.p2p.dial(addr.parse()?).await {
                    let _ = self
                        .p2p
                        .request(peer_id, P2pRequest::UnpublishService { service_uuid: id.into() })
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn find_community_for_service(&self, service_uuid: &str) -> Result<Option<String>> {
        let list = self.store.discovered_services(None).await?;
        Ok(list
            .into_iter()
            .find(|item| item.uuid == service_uuid)
            .and_then(|item| item.community_id))
    }
}

fn parse_local_port(mapping: &str) -> Result<u16> {
    let port_str = mapping
        .split(':')
        .last()
        .ok_or_else(|| anyhow!("无效本地映射"))?;
    Ok(port_str.parse::<u16>()?)
}
