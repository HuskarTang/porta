use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use libp2p::{Multiaddr, PeerId};
use tokio::sync::RwLock;

use crate::{
    models::{
        CommunityAddRequest, CommunitySummary, DiscoveredService, PublishedService, PublishRequest,
        SecureConnectRequest, SecureRoute, ServiceRegistryItem, SessionInfo, SubscribeRequest,
        SubscribedService,
    },
    p2p::{P2pRequest, P2pResponse},
    state::Store,
    tunnel,
};

#[derive(Clone)]
pub struct AppService {
    store: Arc<dyn Store>,
    p2p: crate::p2p::NodeHandle,
    peer_cache: Arc<RwLock<HashMap<String, PeerId>>>,
}

impl AppService {
    pub fn new(store: Arc<dyn Store>, p2p: crate::p2p::NodeHandle) -> Self {
        Self {
            store,
            p2p,
            peer_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_community(&self, mut req: CommunityAddRequest) -> Result<CommunitySummary> {
        if req.name.trim().is_empty() {
            return Err(anyhow!("社区名称不能为空"));
        }
        if req.description.trim().is_empty() {
            return Err(anyhow!("社区描述不能为空"));
        }
        if req.multiaddr.as_ref().is_none() {
            return Err(anyhow!("缺少 multiaddr"));
        }
        let existing = self.store.communities().await?;
        if existing.iter().any(|item| item.name == req.name) {
            return Err(anyhow!("社区名称已存在"));
        }
        if let Some(id) = req.id.as_deref() {
            if existing.iter().any(|item| item.id == id) {
                return Err(anyhow!("社区 ID 已存在"));
            }
        }
        if let Some(addr) = req.multiaddr.as_ref() {
            let addr: Multiaddr = addr.parse()?;
            let peer_id = extract_peer_id(&addr)
                .ok_or_else(|| anyhow!("multiaddr 缺少 /p2p/peerId"))?;
            if self.store.community_exists_by_peer(&peer_id.to_string()).await? {
                return Err(anyhow!("该社区已存在"));
            }
            req.peer_id = Some(peer_id.to_string());
        }
        let saved = self.store.add_community(req.clone()).await?;
        tracing::info!("新增社区: {} ({})", saved.name, saved.id);
        Ok(saved)
    }

    pub async fn remove_community(&self, id: &str) -> Result<()> {
        let removed = self.store.remove_community(id).await?;
        if removed {
            Ok(())
        } else {
            Err(anyhow!("未找到社区"))
        }
    }

    pub async fn connect_community(&self, id: &str) -> Result<()> {
        tracing::info!("正在连接社区: {}", id);
        let peer_id = self.ensure_community_peer(id).await?;
        self.store.connect_community(id).await?;
        tracing::info!("社区 {} 连接成功 (peer: {})", id, peer_id);
        Ok(())
    }

    pub async fn discover_services(
        &self,
        community_id: Option<String>,
    ) -> Result<Vec<DiscoveredService>> {
        let Some(community_id) = community_id else {
            return Ok(self.store.discovered_services(None).await?);
        };
        tracing::info!("发现服务: 社区 {}", community_id);
        let peer_id = self.ensure_community_peer(&community_id).await?;
        let response = self
            .p2p
            .request(
                peer_id,
                P2pRequest::DiscoverServices {
                    community_id: community_id.clone(),
                },
            )
            .await?;
        let services = match response {
            P2pResponse::ServiceList { services } => services,
            P2pResponse::Error { message } => {
                tracing::error!("服务发现失败: {}", message);
                return Err(anyhow!(message));
            }
            _ => Vec::new(),
        };
        tracing::info!("从社区 {} 发现 {} 个服务", community_id, services.len());
        let registry = services
            .into_iter()
            .map(|svc| ServiceRegistryItem {
                uuid: svc.uuid,
                name: svc.name,
                r#type: svc.r#type,
                port: svc.port,
                description: svc.description,
                provider_peer: svc.provider_peer,
                provider_addr: svc.provider_addr,
                online: true,
            })
            .collect::<Vec<_>>();
        self.store
            .upsert_discovered_services(&community_id, registry)
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
        if req.service_uuid.is_none() {
            return Err(anyhow!("缺少 service_uuid"));
        }
        tracing::info!("订阅服务: {}", req.name);
        let saved = self.store.subscribe_service(req.clone()).await?;
        if let Some(service_uuid) = req.service_uuid {
            if let Some(community_id) = self.find_community_for_service(&service_uuid).await? {
                let peer_id = self.ensure_community_peer(&community_id).await?;
                match self
                    .p2p
                    .request(
                        peer_id,
                        P2pRequest::SubscribeService {
                            service_uuid: service_uuid.clone(),
                            subscriber_peer: self.p2p.peer_id(),
                        },
                    )
                    .await
                {
                    Ok(_) => tracing::info!("服务 {} 订阅成功", service_uuid),
                    Err(err) => tracing::warn!("服务 {} 订阅通知失败: {}", service_uuid, err),
                }
            }
        }
        Ok(saved)
    }

    pub async fn connect_service(&self, id: &str) -> Result<()> {
        tracing::info!("正在连接服务: {}", id);
        let Some(subscription) = self.store.find_subscription(id).await? else {
            return Err(anyhow!("未找到订阅"));
        };
        let local_port = parse_local_port(&subscription.local_mapping)?;
        let Some(service_uuid) = subscription.service_uuid.clone() else {
            return Err(anyhow!("订阅缺少 service_uuid"));
        };
        let Some(community_id) = self.find_community_for_service(&service_uuid).await? else {
            return Err(anyhow!("未找到社区"));
        };
        let peer_id = self.ensure_community_peer(&community_id).await?;
        let service_for_stream = service_uuid.clone();
        let response = self
            .p2p
            .request(
                peer_id,
                P2pRequest::ConnectService {
                    service_uuid: service_uuid.clone(),
                    subscriber_peer: self.p2p.peer_id(),
                },
            )
            .await?;
        let (provider_peer, provider_addr, port) = match response {
            P2pResponse::ConnectInfo {
                provider_peer,
                provider_addr,
                port,
                ..
            } => (provider_peer, provider_addr, port),
            P2pResponse::Error { message } => {
                tracing::error!("服务连接被拒绝: {}", message);
                return Err(anyhow!(message));
            }
            _ => return Err(anyhow!("连接失败")),
        };
        let remote_addr = compose_remote_addr(&provider_addr, port);
        let updated = self
            .store
            .update_subscription_endpoint(id, &remote_addr, "畅通")
            .await?;
        if !updated {
            return Err(anyhow!("更新订阅失败"));
        }
        let session = SessionInfo {
            session_id: format!("sess-{}", id),
            service_id: id.to_string(),
            local_port,
            remote_peer: remote_addr.clone(),
            state: "connected".into(),
            created_at: None,
            last_active: None,
        };
        self.store.upsert_session(session).await?;
        let peer_id: PeerId = provider_peer.parse()?;
        tunnel::ensure_stream_mapping(
            local_port,
            peer_id,
            service_for_stream,
            self.p2p.clone(),
        )
        .await?;
        tracing::info!("服务 {} 连接成功，本地端口: {}", id, local_port);
        Ok(())
    }

    pub async fn disconnect_service(&self, id: &str) -> Result<()> {
        tracing::info!("正在断开服务: {}", id);
        self.store.update_subscription_status(id, "断开").await?;
        let session = SessionInfo {
            session_id: format!("sess-{}", id),
            service_id: id.to_string(),
            local_port: 0,
            remote_peer: "".into(),
            state: "closed".into(),
            created_at: None,
            last_active: None,
        };
        self.store.upsert_session(session).await?;
        tracing::info!("服务 {} 已断开", id);
        Ok(())
    }

    pub async fn publish_service(&self, req: PublishRequest) -> Result<PublishedService> {
        tracing::info!("发布服务: {} ({}:{})", req.name, req.r#type, req.port);
        let published = self.store.publish_service(req.clone()).await?;
        let node = self.store.node_info().await?;
        let provider_addr = node
            .external_addr
            .first()
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".into());
        let communities = self.store.communities().await?;
        let mut publish_count = 0;
        for community in communities.into_iter().filter(|c| c.joined) {
            if let Ok(peer_id) = self.ensure_community_peer(&community.id).await {
                match self
                    .p2p
                    .request(
                        peer_id,
                        P2pRequest::PublishService {
                            service: crate::p2p::protocol::ServiceAnnouncement {
                                uuid: published.id.clone(),
                                name: published.name.clone(),
                                r#type: published.r#type.clone(),
                                port: published.port,
                                provider_peer: self.p2p.peer_id(),
                                provider_addr: provider_addr.clone(),
                                description: published.summary.clone(),
                            },
                        },
                    )
                    .await
                {
                    Ok(_) => {
                        publish_count += 1;
                        tracing::debug!("服务已发布到社区: {}", community.id);
                    }
                    Err(err) => tracing::warn!("向社区 {} 发布失败: {}", community.id, err),
                }
            }
        }
        tracing::info!("服务 {} 发布完成，已同步到 {} 个社区", published.name, publish_count);
        Ok(published)
    }

    pub async fn unpublish_service(&self, id: &str) -> Result<()> {
        let updated = self.store.unpublish_service(id).await?;
        if !updated {
            return Err(anyhow!("未找到发布服务"));
        }
        let communities = self.store.communities().await?;
        for community in communities.into_iter().filter(|c| c.joined) {
            if let Ok(peer_id) = self.ensure_community_peer(&community.id).await {
                let _ = self
                    .p2p
                    .request(peer_id, P2pRequest::UnpublishService { service_uuid: id.into() })
                    .await;
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

    async fn build_hello(&self) -> Result<crate::p2p::protocol::NodeHello> {
        let info = self.store.node_info().await?;
        let role = current_role();
        Ok(crate::p2p::protocol::NodeHello {
            node_id: info.node_id,
            role,
        })
    }

    async fn ensure_community_peer(&self, community_id: &str) -> Result<PeerId> {
        {
            let cache = self.peer_cache.read().await;
            if let Some(cached_peer) = cache.get(community_id) {
                tracing::debug!("使用缓存的 peer: {} -> {}", community_id, cached_peer);
                return Ok(*cached_peer);
            }
        }

        let Some(community) = self.store.community_by_id(community_id).await? else {
            return Err(anyhow!("未找到社区"));
        };
        let Some(addr) = community.multiaddr.clone() else {
            return Err(anyhow!("社区缺少 multiaddr"));
        };
        let addr: Multiaddr = addr.parse()?;
        let expected_peer = extract_peer_id(&addr)
            .ok_or_else(|| anyhow!("multiaddr 缺少 /p2p/peerId"))?;
        let peer_id = self.p2p.dial(addr).await?;
        if peer_id != expected_peer {
            return Err(anyhow!("peerId 校验失败"));
        }
        if self.store.peer_is_banned(&peer_id.to_string()).await? {
            return Err(anyhow!("对端 peer 已被封禁"));
        }
        let response = self
            .p2p
            .request(peer_id, P2pRequest::Hello { hello: self.build_hello().await? })
            .await?;
        match response {
            P2pResponse::HelloAck { hello } => {
                if hello.role != "community" {
                    return Err(anyhow!("对端角色不匹配"));
                }
                self.store
                    .upsert_peer(&peer_id.to_string(), &hello.node_id, &hello.role, "online")
                    .await?;
            }
            P2pResponse::Error { message } => return Err(anyhow!(message)),
            _ => return Err(anyhow!("握手失败")),
        }

        let mut cache = self.peer_cache.write().await;
        cache.insert(community_id.to_string(), peer_id);
        tracing::debug!("缓存 peer: {} -> {}", community_id, peer_id);

        Ok(peer_id)
    }

    pub async fn secure_connect_service(&self, req: SecureConnectRequest) -> Result<SecureRoute> {
        if req.relay_peers.len() < 2 {
            return Err(anyhow!("至少需要两个中继节点"));
        }
        tracing::info!("建立安全连接: 订阅 {} 经由 {} 个中继", req.subscription_id, req.relay_peers.len());
        let Some(subscription) = self.store.find_subscription(&req.subscription_id).await? else {
            return Err(anyhow!("未找到订阅"));
        };
        let Some(service_uuid) = subscription.service_uuid.clone() else {
            return Err(anyhow!("订阅缺少 service_uuid"));
        };
        let local_port = req.local_port.unwrap_or_else(|| {
            parse_local_port(&subscription.local_mapping).unwrap_or(0)
        });
        if local_port == 0 {
            return Err(anyhow!("无效本地端口"));
        }
        let first_relay = req.relay_peers.first().ok_or_else(|| anyhow!("中继链为空"))?;
        let first_peer: PeerId = first_relay.parse()?;
        let response = self
            .p2p
            .request(
                first_peer,
                P2pRequest::BuildRelayRoute {
                    service_uuid: service_uuid.clone(),
                    relay_chain: req.relay_peers.clone(),
                    initiator_peer: self.p2p.peer_id(),
                },
            )
            .await?;
        match response {
            P2pResponse::RelayRouteReady { .. } => {
                tracing::debug!("中继链路已就绪");
            }
            P2pResponse::ConnectInfo { .. } => {
                tracing::debug!("获取到服务连接信息");
            }
            P2pResponse::Error { message } => {
                tracing::error!("建立中继链路失败: {}", message);
                return Err(anyhow!(message));
            }
            _ => return Err(anyhow!("建立中继链路失败")),
        }
        let route_id = format!("secure-{}", uuid::Uuid::new_v4());
        let route = SecureRoute {
            id: route_id.clone(),
            subscription_id: req.subscription_id.clone(),
            relay_peers: req.relay_peers.clone(),
            local_port,
            status: "connected".into(),
        };
        self.store.add_secure_route(route.clone()).await?;
        tunnel::ensure_secure_mapping(
            local_port,
            first_peer,
            service_uuid,
            req.relay_peers.clone(),
            self.p2p.clone(),
        )
        .await?;
        tracing::info!("安全路由 {} 建立成功，本地端口: {}", route_id, local_port);
        Ok(route)
    }

    pub async fn disconnect_secure_route(&self, id: &str) -> Result<()> {
        let updated = self.store.update_secure_route_status(id, "断开").await?;
        if !updated {
            return Err(anyhow!("未找到安全路由"));
        }
        Ok(())
    }

    pub async fn publish_proxy_service(&self) -> Result<()> {
        let proxy_status = self.store.proxy_status().await?;
        let req = PublishRequest {
            id: Some("proxy-omega".into()),
            name: "Omega 代理".into(),
            r#type: "omega".into(),
            port: proxy_status.listen_port,
            summary: "HTTP/HTTPS/SOCKS5 代理服务".into(),
        };
        let _ = self.publish_service(req).await?;
        Ok(())
    }

    pub async fn unpublish_proxy_service(&self) -> Result<()> {
        let _ = self.unpublish_service("proxy-omega").await;
        Ok(())
    }

    pub async fn reconnect_communities(&self) -> Result<()> {
        let communities = self.store.communities().await?;
        for community in communities.into_iter().filter(|c| c.joined) {
            if let Some(addr) = community.multiaddr.clone() {
                let addr: Multiaddr = match addr.parse() {
                    Ok(a) => a,
                    Err(_) => {
                        tracing::warn!("无效的社区 multiaddr: {}", community.id);
                        continue;
                    }
                };
                let expected_peer = match extract_peer_id(&addr) {
                    Some(p) => p,
                    None => {
                        tracing::warn!("社区缺少 peerId: {}", community.id);
                        continue;
                    }
                };
                match self.p2p.dial(addr.clone()).await {
                    Ok(peer_id) if peer_id == expected_peer => {
                        tracing::debug!("社区 {} 连接保活成功", community.id);
                    }
                    Ok(peer_id) => {
                        tracing::warn!("社区 {} peerId 不匹配: 期望 {}, 实际 {}", community.id, expected_peer, peer_id);
                    }
                    Err(err) => {
                        tracing::warn!("社区 {} 连接失败: {}", community.id, err);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<()> {
        let timeout_minutes = 30;
        let removed = self.store.cleanup_expired_sessions(timeout_minutes).await?;
        if removed > 0 {
            tracing::info!("清理过期会话 {} 个", removed);
        }
        Ok(())
    }
}

fn parse_local_port(mapping: &str) -> Result<u16> {
    let port_str = mapping
        .split(':')
        .last()
        .ok_or_else(|| anyhow!("无效本地映射"))?;
    Ok(port_str.parse::<u16>()?)
}

fn current_role() -> String {
    std::env::var("PORTA_ROLE").unwrap_or_else(|_| "edge".into())
}

fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|protocol| {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
            Some(peer_id)
        } else {
            None
        }
    })
}

fn compose_remote_addr(provider_addr: &str, port: u16) -> String {
    if provider_addr.contains(':') {
        provider_addr.to_string()
    } else {
        format!("{}:{}", provider_addr, port)
    }
}

#[cfg(test)]
mod tests {
    use super::compose_remote_addr;

    #[test]
    fn should_compose_remote_addr() {
        assert_eq!(compose_remote_addr("127.0.0.1", 8080), "127.0.0.1:8080");
        assert_eq!(compose_remote_addr("127.0.0.1:9000", 8080), "127.0.0.1:9000");
    }
}
