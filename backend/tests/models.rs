//! Tests for data models and serialization

use serde_json::json;

// Model types matching the backend models
mod model_types {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct CommunitySummary {
        pub id: String,
        pub name: String,
        pub description: String,
        pub peers: u32,
        pub joined: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub multiaddr: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub peer_id: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct SubscribedService {
        pub id: String,
        pub name: String,
        pub r#type: String,
        pub community: String,
        pub remote_addr: String,
        pub local_mapping: String,
        pub status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub service_uuid: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct SecureRoute {
        pub id: String,
        pub subscription_id: String,
        pub relay_peers: Vec<String>,
        pub local_port: u16,
        pub status: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct SessionInfo {
        pub session_id: String,
        pub service_id: String,
        pub local_port: u16,
        pub remote_peer: String,
        pub state: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub created_at: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_active: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ProxyStatus {
        pub enabled: bool,
        pub listen_port: u16,
    }
}

use model_types::*;

// ===========================================================================
// NodeInfo Tests
// ===========================================================================

#[test]
fn node_info_serializes_all_fields() {
    let info = NodeInfo {
        name: "My Node".to_string(),
        node_id: "12D3KooW...".to_string(),
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        key_path: "porta.node.key".to_string(),
        tcp_listen_enable: true,
        tcp_listen_port: 4001,
        quci_listen_enable: false,
        quci_listen_port: 0,
        external_addr: vec!["192.168.1.1:4001".to_string()],
        mdns_enable: true,
        dht_enable: true,
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("My Node"));
    assert!(json.contains("tcp_listen_enable"));
    assert!(json.contains("external_addr"));
}

#[test]
fn node_info_deserializes_from_json() {
    let json = json!({
        "name": "Test Node",
        "node_id": "peer-123",
        "uuid": "uuid-456",
        "key_path": "/path/to/key",
        "tcp_listen_enable": true,
        "tcp_listen_port": 9000,
        "quci_listen_enable": false,
        "quci_listen_port": 0,
        "external_addr": ["10.0.0.1:9000"],
        "mdns_enable": false,
        "dht_enable": true
    });
    let info: NodeInfo = serde_json::from_value(json).unwrap();
    assert_eq!(info.name, "Test Node");
    assert_eq!(info.tcp_listen_port, 9000);
    assert_eq!(info.external_addr.len(), 1);
}

#[test]
fn node_info_empty_external_addr() {
    let info = NodeInfo {
        name: "Minimal".to_string(),
        node_id: "".to_string(),
        uuid: "".to_string(),
        key_path: "".to_string(),
        tcp_listen_enable: false,
        tcp_listen_port: 0,
        quci_listen_enable: false,
        quci_listen_port: 0,
        external_addr: vec![],
        mdns_enable: false,
        dht_enable: false,
    };
    let json = serde_json::to_string(&info).unwrap();
    let decoded: NodeInfo = serde_json::from_str(&json).unwrap();
    assert!(decoded.external_addr.is_empty());
}

// ===========================================================================
// CommunitySummary Tests
// ===========================================================================

#[test]
fn community_summary_serializes() {
    let summary = CommunitySummary {
        id: "comm-1".to_string(),
        name: "Dev Community".to_string(),
        description: "For developers".to_string(),
        peers: 42,
        joined: true,
        multiaddr: Some("/ip4/127.0.0.1/tcp/4001".to_string()),
        peer_id: Some("12D3KooW...".to_string()),
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("Dev Community"));
    assert!(json.contains("42"));
    assert!(json.contains("multiaddr"));
}

#[test]
fn community_summary_optional_fields_skip() {
    let summary = CommunitySummary {
        id: "comm-2".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        peers: 0,
        joined: false,
        multiaddr: None,
        peer_id: None,
    };
    let json = serde_json::to_string(&summary).unwrap();
    // Optional fields with None should be skipped
    assert!(!json.contains("multiaddr"));
    assert!(!json.contains("peer_id"));
}

#[test]
fn community_summary_roundtrip() {
    let original = CommunitySummary {
        id: "comm-3".to_string(),
        name: "科技社区".to_string(),
        description: "中文描述".to_string(),
        peers: 100,
        joined: true,
        multiaddr: Some("/ip4/10.0.0.1/tcp/4001/p2p/12D3...".to_string()),
        peer_id: Some("12D3...".to_string()),
    };
    let json = serde_json::to_string(&original).unwrap();
    let decoded: CommunitySummary = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

// ===========================================================================
// SubscribedService Tests
// ===========================================================================

#[test]
fn subscribed_service_serializes() {
    let service = SubscribedService {
        id: "sub-1".to_string(),
        name: "Web App".to_string(),
        r#type: "HTTP".to_string(),
        community: "dev".to_string(),
        remote_addr: "192.168.1.100:8080".to_string(),
        local_mapping: "127.0.0.1:18080".to_string(),
        status: "畅通".to_string(),
        service_uuid: Some("svc-uuid-123".to_string()),
    };
    let json = serde_json::to_string(&service).unwrap();
    assert!(json.contains("Web App"));
    assert!(json.contains("畅通"));
}

#[test]
fn subscribed_service_status_values() {
    let statuses = vec!["畅通", "连接中", "断开"];
    for status in statuses {
        let service = SubscribedService {
            id: "sub-test".to_string(),
            name: "Test".to_string(),
            r#type: "TCP".to_string(),
            community: "test".to_string(),
            remote_addr: "".to_string(),
            local_mapping: "".to_string(),
            status: status.to_string(),
            service_uuid: None,
        };
        let json = serde_json::to_string(&service).unwrap();
        let decoded: SubscribedService = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, status);
    }
}

// ===========================================================================
// PublishedService Tests
// ===========================================================================

#[test]
fn published_service_serializes() {
    let service = PublishedService {
        id: "pub-1".to_string(),
        name: "My API".to_string(),
        r#type: "HTTP".to_string(),
        port: 3000,
        summary: "REST API service".to_string(),
        subscriptions: 5,
        status: "在线".to_string(),
        publish_date: "2026-01-15".to_string(),
    };
    let json = serde_json::to_string(&service).unwrap();
    assert!(json.contains("My API"));
    assert!(json.contains("3000"));
    assert!(json.contains("在线"));
}

#[test]
fn published_service_status_values() {
    let statuses = vec!["在线", "已下架"];
    for status in statuses {
        let service = PublishedService {
            id: "pub-test".to_string(),
            name: "Test".to_string(),
            r#type: "TCP".to_string(),
            port: 8080,
            summary: "".to_string(),
            subscriptions: 0,
            status: status.to_string(),
            publish_date: "".to_string(),
        };
        let json = serde_json::to_string(&service).unwrap();
        let decoded: PublishedService = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.status, status);
    }
}

// ===========================================================================
// SecureRoute Tests
// ===========================================================================

#[test]
fn secure_route_serializes() {
    let route = SecureRoute {
        id: "route-1".to_string(),
        subscription_id: "sub-1".to_string(),
        relay_peers: vec!["peer-a".to_string(), "peer-b".to_string(), "peer-c".to_string()],
        local_port: 19000,
        status: "connected".to_string(),
    };
    let json = serde_json::to_string(&route).unwrap();
    assert!(json.contains("route-1"));
    assert!(json.contains("peer-a"));
    assert!(json.contains("19000"));
}

#[test]
fn secure_route_minimum_relays() {
    let route = SecureRoute {
        id: "route-min".to_string(),
        subscription_id: "sub-min".to_string(),
        relay_peers: vec!["peer-1".to_string(), "peer-2".to_string()],
        local_port: 10000,
        status: "connecting".to_string(),
    };
    let json = serde_json::to_string(&route).unwrap();
    let decoded: SecureRoute = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.relay_peers.len(), 2);
}

// ===========================================================================
// SessionInfo Tests
// ===========================================================================

#[test]
fn session_info_serializes() {
    let session = SessionInfo {
        session_id: "sess-1".to_string(),
        service_id: "svc-1".to_string(),
        local_port: 8080,
        remote_peer: "peer-remote".to_string(),
        state: "connected".to_string(),
        created_at: Some("2026-01-15T10:00:00Z".to_string()),
        last_active: Some("2026-01-15T10:30:00Z".to_string()),
    };
    let json = serde_json::to_string(&session).unwrap();
    assert!(json.contains("sess-1"));
    assert!(json.contains("connected"));
}

#[test]
fn session_info_optional_timestamps() {
    let session = SessionInfo {
        session_id: "sess-2".to_string(),
        service_id: "svc-2".to_string(),
        local_port: 9000,
        remote_peer: "peer-2".to_string(),
        state: "connecting".to_string(),
        created_at: None,
        last_active: None,
    };
    let json = serde_json::to_string(&session).unwrap();
    // Optional fields should be skipped
    assert!(!json.contains("created_at"));
    assert!(!json.contains("last_active"));
}

#[test]
fn session_info_state_values() {
    let states = vec!["connecting", "connected", "closed", "error"];
    for state in states {
        let session = SessionInfo {
            session_id: "sess-test".to_string(),
            service_id: "svc-test".to_string(),
            local_port: 0,
            remote_peer: "".to_string(),
            state: state.to_string(),
            created_at: None,
            last_active: None,
        };
        let json = serde_json::to_string(&session).unwrap();
        let decoded: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.state, state);
    }
}

// ===========================================================================
// ProxyStatus Tests
// ===========================================================================

#[test]
fn proxy_status_serializes() {
    let status = ProxyStatus {
        enabled: true,
        listen_port: 1080,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("true"));
    assert!(json.contains("1080"));
}

#[test]
fn proxy_status_disabled() {
    let status = ProxyStatus {
        enabled: false,
        listen_port: 0,
    };
    let json = serde_json::to_string(&status).unwrap();
    let decoded: ProxyStatus = serde_json::from_str(&json).unwrap();
    assert!(!decoded.enabled);
    assert_eq!(decoded.listen_port, 0);
}

// ===========================================================================
// API Response Format Tests
// ===========================================================================

#[test]
fn api_response_success_format() {
    let response = json!({
        "code": 0,
        "message": "success",
        "data": {
            "name": "Test"
        }
    });
    let code = response["code"].as_i64().unwrap();
    assert_eq!(code, 0);
    assert!(response.get("data").is_some());
}

#[test]
fn api_response_error_format() {
    let response = json!({
        "code": 1,
        "message": "Error message",
        "data": null
    });
    let code = response["code"].as_i64().unwrap();
    assert_ne!(code, 0);
    assert!(response["data"].is_null());
}
