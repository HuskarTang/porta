use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app::AppService,
    models::{
        CommunityAddRequest, CommunityNode, CommunityService, CommunitySummary, DiscoveredService,
        KeyImportRequest, NodeConfigUpdate, NodeInfo, ProxyStatus, PublishRequest, PublishedService,
        ServiceDescriptor, SessionInfo, SubscribeRequest, SubscribedService,
    },
    p2p,
};

pub type StoreResult<T> = anyhow::Result<T>;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn Store>,
    pub p2p: p2p::NodeHandle,
    pub app: AppService,
}

impl AppState {
    pub async fn new() -> StoreResult<Self> {
        let db_path = std::env::var("PORTA_DB").unwrap_or_else(|_| "porta.db".into());
        let store = SqliteStore::new(&db_path).await?;
        let p2p = p2p::NodeHandle::spawn(store.clone()).await?;
        let peer_id = p2p.peer_id();
        store.ensure_node_identity(&peer_id).await?;
        let app = AppService::new(store.clone(), p2p.clone());
        Ok(Self { store, p2p, app })
    }
}

#[async_trait]
pub trait Store: Send + Sync {
    async fn node_info(&self) -> StoreResult<NodeInfo>;
    async fn update_node_config(&self, req: NodeConfigUpdate) -> StoreResult<NodeInfo>;
    async fn import_key(&self, req: KeyImportRequest) -> StoreResult<NodeInfo>;
    async fn generate_key(&self) -> StoreResult<NodeInfo>;
    async fn ensure_node_identity(&self, peer_id: &str) -> StoreResult<()>;

    async fn communities(&self) -> StoreResult<Vec<CommunitySummary>>;
    async fn add_community(&self, req: CommunityAddRequest) -> StoreResult<CommunitySummary>;
    async fn remove_community(&self, id: &str) -> StoreResult<bool>;
    async fn connect_community(&self, id: &str) -> StoreResult<bool>;
    async fn community_multiaddr(&self, id: &str) -> StoreResult<Option<String>>;

    async fn community_nodes(&self) -> StoreResult<Vec<CommunityNode>>;
    async fn community_services(&self) -> StoreResult<Vec<CommunityService>>;
    async fn discovered_services(
        &self,
        community_id: Option<String>,
    ) -> StoreResult<Vec<DiscoveredService>>;
    async fn upsert_discovered_services(
        &self,
        community_id: &str,
        services: Vec<ServiceDescriptor>,
    ) -> StoreResult<()>;
    async fn subscribed_services(&self) -> StoreResult<Vec<SubscribedService>>;
    async fn find_subscription(&self, id: &str) -> StoreResult<Option<SubscribedService>>;
    async fn published_services(&self) -> StoreResult<Vec<PublishedService>>;
    async fn proxy_status(&self) -> StoreResult<ProxyStatus>;
    async fn sessions(&self) -> StoreResult<Vec<SessionInfo>>;
    async fn upsert_session(&self, session: SessionInfo) -> StoreResult<()>;

    async fn subscribe_service(&self, req: SubscribeRequest) -> StoreResult<SubscribedService>;
    async fn update_subscription_status(&self, id: &str, status: &str) -> StoreResult<bool>;
    async fn publish_service(&self, req: PublishRequest) -> StoreResult<PublishedService>;
    async fn unpublish_service(&self, id: &str) -> StoreResult<bool>;
    async fn remove_published(&self, id: &str) -> StoreResult<bool>;
    async fn set_service_announced(&self, id: &str, announced: bool) -> StoreResult<bool>;
    async fn set_node_ban(&self, id: &str, banned: bool) -> StoreResult<bool>;
    async fn set_proxy_enabled(&self, enabled: bool) -> StoreResult<()>;
}

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(path: &str) -> StoreResult<Arc<Self>> {
        let url = format!("sqlite://{}", path);
        let pool = SqlitePool::connect(&url).await?;
        let store = Self { pool };
        store.init_schema().await?;
        store.seed_if_empty().await?;
        Ok(Arc::new(store))
    }

    #[cfg(test)]
    pub async fn new_in_memory() -> StoreResult<Arc<Self>> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let store = Self { pool };
        store.init_schema().await?;
        store.seed_if_empty().await?;
        Ok(Arc::new(store))
    }

    async fn init_schema(&self) -> StoreResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS node_config (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                node_id TEXT NOT NULL,
                uuid TEXT NOT NULL,
                key_path TEXT NOT NULL,
                tcp_listen_enable INTEGER NOT NULL,
                tcp_listen_port INTEGER NOT NULL,
                quci_listen_enable INTEGER NOT NULL,
                quci_listen_port INTEGER NOT NULL,
                external_addr TEXT NOT NULL,
                mdns_enable INTEGER NOT NULL,
                dht_enable INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS communities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                peers INTEGER NOT NULL,
                joined INTEGER NOT NULL,
                multiaddr TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS community_nodes (
                id TEXT PRIMARY KEY,
                uuid TEXT NOT NULL,
                status TEXT NOT NULL,
                banned INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS community_services (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                uuid TEXT NOT NULL,
                protocol TEXT NOT NULL,
                port INTEGER NOT NULL,
                online INTEGER NOT NULL,
                announced INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS discovered_services (
                uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                provider TEXT NOT NULL,
                description TEXT NOT NULL,
                community_id TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS subscribed_services (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                community TEXT NOT NULL,
                remote_addr TEXT NOT NULL,
                local_mapping TEXT NOT NULL,
                status TEXT NOT NULL,
                service_uuid TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS published_services (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                port INTEGER NOT NULL,
                summary TEXT NOT NULL,
                subscriptions INTEGER NOT NULL,
                status TEXT NOT NULL,
                publish_date TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proxy_status (
                id INTEGER PRIMARY KEY,
                enabled INTEGER NOT NULL,
                listen_port INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                service_id TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                remote_peer TEXT NOT NULL,
                state TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn seed_if_empty(&self) -> StoreResult<()> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM node_config")
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("count");
        if count == 0 {
            let uuid = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO node_config
                (id, name, node_id, uuid, key_path, tcp_listen_enable, tcp_listen_port,
                 quci_listen_enable, quci_listen_port, external_addr, mdns_enable, dht_enable)
                VALUES (1, ?, ?, ?, ?, 1, 0, 1, 0, ?, 1, 1);
                "#,
            )
            .bind("我的节点")
            .bind("")
            .bind(uuid)
            .bind("porta.node.key")
            .bind("[]")
            .execute(&self.pool)
            .await?;
        }

        let comm_row = sqlx::query("SELECT COUNT(*) as count FROM communities")
            .fetch_one(&self.pool)
            .await?;
        let comm_count: i64 = comm_row.get("count");
        if comm_count == 0 {
            let entries = vec![
                (
                    "dev-community",
                    "开发者社区",
                    "共享开发环境和工具服务",
                    24,
                    0,
                    Some("/ip4/127.0.0.1/tcp/4001".to_string()),
                ),
                (
                    "data-team",
                    "数据科学团队",
                    "机器学习模型和数据分析服务",
                    12,
                    0,
                    Some("/ip4/127.0.0.1/tcp/4002".to_string()),
                ),
                (
                    "test-env",
                    "测试环境",
                    "测试和预发布环境服务",
                    8,
                    0,
                    Some("/ip4/127.0.0.1/tcp/4003".to_string()),
                ),
            ];
            for (id, name, desc, peers, joined, multiaddr) in entries {
                sqlx::query(
                    "INSERT INTO communities (id, name, description, peers, joined, multiaddr) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(id)
                .bind(name)
                .bind(desc)
                .bind(peers)
                .bind(joined)
                .bind(multiaddr)
                .execute(&self.pool)
                .await?;
            }
        }

        let proxy_row = sqlx::query("SELECT COUNT(*) as count FROM proxy_status")
            .fetch_one(&self.pool)
            .await?;
        let proxy_count: i64 = proxy_row.get("count");
        if proxy_count == 0 {
            sqlx::query("INSERT INTO proxy_status (id, enabled, listen_port) VALUES (1, 1, 1080)")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn load_node_info(&self) -> StoreResult<NodeInfo> {
        let row = sqlx::query(
            r#"
            SELECT name, node_id, uuid, key_path, tcp_listen_enable, tcp_listen_port,
                   quci_listen_enable, quci_listen_port, external_addr, mdns_enable, dht_enable
            FROM node_config WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let external_addr: String = row.get("external_addr");
        let parsed: Vec<String> = serde_json::from_str(&external_addr).unwrap_or_default();
        Ok(NodeInfo {
            name: row.get("name"),
            node_id: row.get("node_id"),
            uuid: row.get("uuid"),
            key_path: row.get("key_path"),
            tcp_listen_enable: row.get::<i64, _>("tcp_listen_enable") == 1,
            tcp_listen_port: row.get::<i64, _>("tcp_listen_port") as u16,
            quci_listen_enable: row.get::<i64, _>("quci_listen_enable") == 1,
            quci_listen_port: row.get::<i64, _>("quci_listen_port") as u16,
            external_addr: parsed,
            mdns_enable: row.get::<i64, _>("mdns_enable") == 1,
            dht_enable: row.get::<i64, _>("dht_enable") == 1,
        })
    }

    async fn save_node_info(&self, info: &NodeInfo) -> StoreResult<()> {
        let external_addr = serde_json::to_string(&info.external_addr)?;
        sqlx::query(
            r#"
            UPDATE node_config
            SET name = ?, node_id = ?, uuid = ?, key_path = ?,
                tcp_listen_enable = ?, tcp_listen_port = ?,
                quci_listen_enable = ?, quci_listen_port = ?,
                external_addr = ?, mdns_enable = ?, dht_enable = ?
            WHERE id = 1
            "#,
        )
        .bind(&info.name)
        .bind(&info.node_id)
        .bind(&info.uuid)
        .bind(&info.key_path)
        .bind(if info.tcp_listen_enable { 1 } else { 0 })
        .bind(info.tcp_listen_port as i64)
        .bind(if info.quci_listen_enable { 1 } else { 0 })
        .bind(info.quci_listen_port as i64)
        .bind(external_addr)
        .bind(if info.mdns_enable { 1 } else { 0 })
        .bind(if info.dht_enable { 1 } else { 0 })
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_seed_node_config() {
        let store = SqliteStore::new_in_memory().await.unwrap();
        let info = store.node_info().await.unwrap();
        assert!(!info.name.is_empty());
        assert!(!info.key_path.is_empty());
    }

    #[tokio::test]
    async fn should_save_subscription() {
        let store = SqliteStore::new_in_memory().await.unwrap();
        let req = SubscribeRequest {
            id: None,
            service_uuid: Some("svc-1".into()),
            name: "Service A".into(),
            r#type: "HTTP".into(),
            community: "dev".into(),
            remote_addr: "127.0.0.1:8080".into(),
            local_mapping: "127.0.0.1:18080".into(),
        };
        let saved = store.subscribe_service(req).await.unwrap();
        assert_eq!(saved.status, "畅通");
        let list = store.subscribed_services().await.unwrap();
        assert_eq!(list.len(), 1);
    }
}

#[async_trait]
impl Store for SqliteStore {
    async fn node_info(&self) -> StoreResult<NodeInfo> {
        self.load_node_info().await
    }

    async fn update_node_config(&self, req: NodeConfigUpdate) -> StoreResult<NodeInfo> {
        let mut info = self.load_node_info().await?;
        if let Some(name) = req.name {
            info.name = name;
        }
        if let Some(tcp_listen_enable) = req.tcp_listen_enable {
            info.tcp_listen_enable = tcp_listen_enable;
        }
        if let Some(tcp_listen_port) = req.tcp_listen_port {
            info.tcp_listen_port = tcp_listen_port;
        }
        if let Some(quci_listen_enable) = req.quci_listen_enable {
            info.quci_listen_enable = quci_listen_enable;
        }
        if let Some(quci_listen_port) = req.quci_listen_port {
            info.quci_listen_port = quci_listen_port;
        }
        if let Some(external_addr) = req.external_addr {
            info.external_addr = external_addr;
        }
        if let Some(mdns_enable) = req.mdns_enable {
            info.mdns_enable = mdns_enable;
        }
        if let Some(dht_enable) = req.dht_enable {
            info.dht_enable = dht_enable;
        }
        self.save_node_info(&info).await?;
        Ok(info)
    }

    async fn import_key(&self, req: KeyImportRequest) -> StoreResult<NodeInfo> {
        let mut info = self.load_node_info().await?;
        info.key_path = req.key_path;
        self.save_node_info(&info).await?;
        Ok(info)
    }

    async fn generate_key(&self) -> StoreResult<NodeInfo> {
        let mut info = self.load_node_info().await?;
        let new_path = format!("porta-{}.key", Uuid::new_v4());
        info.key_path = new_path;
        self.save_node_info(&info).await?;
        Ok(info)
    }

    async fn ensure_node_identity(&self, peer_id: &str) -> StoreResult<()> {
        let mut info = self.load_node_info().await?;
        if info.node_id.is_empty() {
            info.node_id = peer_id.to_string();
            self.save_node_info(&info).await?;
        }
        Ok(())
    }

    async fn communities(&self) -> StoreResult<Vec<CommunitySummary>> {
        let rows = sqlx::query(
            "SELECT id, name, description, peers, joined, multiaddr FROM communities",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| CommunitySummary {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                peers: row.get::<i64, _>("peers") as u32,
                joined: row.get::<i64, _>("joined") == 1,
                multiaddr: row.get::<Option<String>, _>("multiaddr"),
            })
            .collect())
    }

    async fn add_community(&self, req: CommunityAddRequest) -> StoreResult<CommunitySummary> {
        let id = req.id.unwrap_or_else(|| format!("community-{}", Uuid::new_v4()));
        sqlx::query(
            "INSERT INTO communities (id, name, description, peers, joined, multiaddr) VALUES (?, ?, ?, 0, 0, ?)",
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.multiaddr)
        .execute(&self.pool)
        .await?;
        Ok(CommunitySummary {
            id,
            name: req.name,
            description: req.description,
            peers: 0,
            joined: false,
            multiaddr: req.multiaddr,
        })
    }

    async fn remove_community(&self, id: &str) -> StoreResult<bool> {
        let result = sqlx::query("DELETE FROM communities WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn connect_community(&self, id: &str) -> StoreResult<bool> {
        let result = sqlx::query("UPDATE communities SET joined = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn community_multiaddr(&self, id: &str) -> StoreResult<Option<String>> {
        let row = sqlx::query("SELECT multiaddr FROM communities WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.get::<Option<String>, _>("multiaddr")))
    }

    async fn community_nodes(&self) -> StoreResult<Vec<CommunityNode>> {
        let rows = sqlx::query("SELECT id, uuid, status, banned FROM community_nodes")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| CommunityNode {
                id: row.get("id"),
                uuid: row.get("uuid"),
                status: row.get("status"),
                banned: row.get::<i64, _>("banned") == 1,
            })
            .collect())
    }

    async fn community_services(&self) -> StoreResult<Vec<CommunityService>> {
        let rows = sqlx::query(
            "SELECT id, name, uuid, protocol, port, online, announced FROM community_services",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| CommunityService {
                id: row.get("id"),
                name: row.get("name"),
                uuid: row.get("uuid"),
                protocol: row.get("protocol"),
                port: row.get::<i64, _>("port") as u16,
                online: row.get::<i64, _>("online") == 1,
                announced: row.get::<i64, _>("announced") == 1,
            })
            .collect())
    }

    async fn discovered_services(
        &self,
        community_id: Option<String>,
    ) -> StoreResult<Vec<DiscoveredService>> {
        let rows = if let Some(id) = community_id {
            sqlx::query(
                "SELECT uuid, name, type, remote_port, provider, description, community_id FROM discovered_services WHERE community_id = ?",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT uuid, name, type, remote_port, provider, description, community_id FROM discovered_services",
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows
            .into_iter()
            .map(|row| DiscoveredService {
                uuid: row.get("uuid"),
                name: row.get("name"),
                r#type: row.get("type"),
                remote_port: row.get::<i64, _>("remote_port") as u16,
                provider: row.get("provider"),
                description: row.get("description"),
                subscribed: None,
                community_id: row.get("community_id"),
            })
            .collect())
    }

    async fn upsert_discovered_services(
        &self,
        community_id: &str,
        services: Vec<ServiceDescriptor>,
    ) -> StoreResult<()> {
        for svc in services {
            sqlx::query(
                r#"
                INSERT INTO discovered_services (uuid, name, type, remote_port, provider, description, community_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(uuid) DO UPDATE SET
                    name = excluded.name,
                    type = excluded.type,
                    remote_port = excluded.remote_port,
                    provider = excluded.provider,
                    description = excluded.description,
                    community_id = excluded.community_id
                "#,
            )
            .bind(svc.uuid)
            .bind(svc.name)
            .bind(svc.r#type)
            .bind(svc.remote_port as i64)
            .bind(svc.provider)
            .bind(svc.description)
            .bind(community_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn subscribed_services(&self) -> StoreResult<Vec<SubscribedService>> {
        let rows = sqlx::query(
            "SELECT id, name, type, community, remote_addr, local_mapping, status, service_uuid FROM subscribed_services",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| SubscribedService {
                id: row.get("id"),
                name: row.get("name"),
                r#type: row.get("type"),
                community: row.get("community"),
                remote_addr: row.get("remote_addr"),
                local_mapping: row.get("local_mapping"),
                status: row.get("status"),
                service_uuid: row.get("service_uuid"),
            })
            .collect())
    }

    async fn find_subscription(&self, id: &str) -> StoreResult<Option<SubscribedService>> {
        let row = sqlx::query(
            "SELECT id, name, type, community, remote_addr, local_mapping, status, service_uuid FROM subscribed_services WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| SubscribedService {
            id: row.get("id"),
            name: row.get("name"),
            r#type: row.get("type"),
            community: row.get("community"),
            remote_addr: row.get("remote_addr"),
            local_mapping: row.get("local_mapping"),
            status: row.get("status"),
            service_uuid: row.get("service_uuid"),
        }))
    }

    async fn published_services(&self) -> StoreResult<Vec<PublishedService>> {
        let rows = sqlx::query(
            "SELECT id, name, type, port, summary, subscriptions, status, publish_date FROM published_services",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| PublishedService {
                id: row.get("id"),
                name: row.get("name"),
                r#type: row.get("type"),
                port: row.get::<i64, _>("port") as u16,
                summary: row.get("summary"),
                subscriptions: row.get::<i64, _>("subscriptions") as u32,
                status: row.get("status"),
                publish_date: row.get("publish_date"),
            })
            .collect())
    }

    async fn proxy_status(&self) -> StoreResult<ProxyStatus> {
        let row = sqlx::query("SELECT enabled, listen_port FROM proxy_status WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(ProxyStatus {
            enabled: row.get::<i64, _>("enabled") == 1,
            listen_port: row.get::<i64, _>("listen_port") as u16,
        })
    }

    async fn sessions(&self) -> StoreResult<Vec<SessionInfo>> {
        let rows =
            sqlx::query("SELECT session_id, service_id, local_port, remote_peer, state FROM sessions")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|row| SessionInfo {
                session_id: row.get("session_id"),
                service_id: row.get("service_id"),
                local_port: row.get::<i64, _>("local_port") as u16,
                remote_peer: row.get("remote_peer"),
                state: row.get("state"),
            })
            .collect())
    }

    async fn upsert_session(&self, session: SessionInfo) -> StoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sessions (session_id, service_id, local_port, remote_peer, state)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(session_id) DO UPDATE SET
                service_id = excluded.service_id,
                local_port = excluded.local_port,
                remote_peer = excluded.remote_peer,
                state = excluded.state
            "#,
        )
        .bind(session.session_id)
        .bind(session.service_id)
        .bind(session.local_port as i64)
        .bind(session.remote_peer)
        .bind(session.state)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn subscribe_service(&self, req: SubscribeRequest) -> StoreResult<SubscribedService> {
        let id = req.id.unwrap_or_else(|| format!("sub-{}", Uuid::new_v4()));
        sqlx::query(
            r#"
            INSERT INTO subscribed_services (id, name, type, community, remote_addr, local_mapping, status, service_uuid)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                type = excluded.type,
                community = excluded.community,
                remote_addr = excluded.remote_addr,
                local_mapping = excluded.local_mapping,
                status = excluded.status,
                service_uuid = excluded.service_uuid
            "#,
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.r#type)
        .bind(&req.community)
        .bind(&req.remote_addr)
        .bind(&req.local_mapping)
        .bind("畅通")
        .bind(&req.service_uuid)
        .execute(&self.pool)
        .await?;
        Ok(SubscribedService {
            id,
            name: req.name,
            r#type: req.r#type,
            community: req.community,
            remote_addr: req.remote_addr,
            local_mapping: req.local_mapping,
            status: "畅通".into(),
            service_uuid: req.service_uuid,
        })
    }

    async fn update_subscription_status(&self, id: &str, status: &str) -> StoreResult<bool> {
        let result = sqlx::query("UPDATE subscribed_services SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn publish_service(&self, req: PublishRequest) -> StoreResult<PublishedService> {
        let id = req.id.unwrap_or_else(|| format!("pub-{}", Uuid::new_v4()));
        sqlx::query(
            r#"
            INSERT INTO published_services (id, name, type, port, summary, subscriptions, status, publish_date)
            VALUES (?, ?, ?, ?, ?, 0, ?, date('now'))
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                type = excluded.type,
                port = excluded.port,
                summary = excluded.summary,
                status = excluded.status
            "#,
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.r#type)
        .bind(req.port as i64)
        .bind(&req.summary)
        .bind("在线")
        .execute(&self.pool)
        .await?;
        let row = sqlx::query("SELECT publish_date FROM published_services WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;
        let publish_date: String = row.get("publish_date");
        Ok(PublishedService {
            id,
            name: req.name,
            r#type: req.r#type,
            port: req.port,
            summary: req.summary,
            subscriptions: 0,
            status: "在线".into(),
            publish_date,
        })
    }

    async fn unpublish_service(&self, id: &str) -> StoreResult<bool> {
        let result = sqlx::query("UPDATE published_services SET status = '已下架' WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn remove_published(&self, id: &str) -> StoreResult<bool> {
        let result = sqlx::query("DELETE FROM published_services WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_service_announced(&self, id: &str, announced: bool) -> StoreResult<bool> {
        let result = sqlx::query("UPDATE community_services SET announced = ? WHERE id = ?")
            .bind(if announced { 1 } else { 0 })
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_node_ban(&self, id: &str, banned: bool) -> StoreResult<bool> {
        let result = sqlx::query("UPDATE community_nodes SET banned = ? WHERE id = ?")
            .bind(if banned { 1 } else { 0 })
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_proxy_enabled(&self, enabled: bool) -> StoreResult<()> {
        sqlx::query("UPDATE proxy_status SET enabled = ? WHERE id = 1")
            .bind(if enabled { 1 } else { 0 })
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
