use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub name: String,
    pub node_id: String,
    pub uuid: String,
    pub key_path: String,
    pub tcp_listen_enable: bool,
    pub tcp_listen_port: u16,
    pub quci_listen_enable: bool,
    pub quci_listen_port: u16,
    pub external_addr: Vec<String>,
    pub mdns_enable: bool,
    pub dht_enable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommunitySummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub peers: u32,
    pub joined: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDescriptor {
    pub uuid: String,
    pub name: String,
    pub r#type: String,
    pub remote_port: u16,
    pub provider: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribedService {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub community: String,
    pub remote_addr: String,
    pub local_mapping: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublishedService {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub port: u16,
    pub summary: String,
    pub subscriptions: u32,
    pub status: String,
    pub publish_date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommunityNode {
    pub id: String,
    pub uuid: String,
    pub status: String,
    pub banned: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommunityService {
    pub id: String,
    pub name: String,
    pub uuid: String,
    pub protocol: String,
    pub port: u16,
    pub online: bool,
    pub announced: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProxyStatus {
    pub enabled: bool,
    pub listen_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeRequest {
    pub id: Option<String>,
    pub name: String,
    pub r#type: String,
    pub community: String,
    pub remote_addr: String,
    pub local_mapping: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSessionRequest {
    pub id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublishRequest {
    pub id: Option<String>,
    pub name: String,
    pub r#type: String,
    pub port: u16,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToggleRequest {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProxyToggle {
    pub enabled: bool,
}
