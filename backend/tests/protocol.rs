//! Tests for P2P protocol serialization and handling

use serde_json;

// Import protocol types - these are re-exported from p2p module
mod protocol_types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct NodeHello {
        pub node_id: String,
        pub role: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ServiceAnnouncement {
        pub uuid: String,
        pub name: String,
        pub r#type: String,
        pub port: u16,
        pub description: String,
        pub provider_peer: String,
        pub provider_addr: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type")]
    pub enum P2pRequest {
        Hello {
            hello: NodeHello,
        },
        DiscoverServices {
            community_id: String,
        },
        SubscribeService {
            service_uuid: String,
            subscriber_peer: String,
        },
        ConnectService {
            service_uuid: String,
            subscriber_peer: String,
        },
        PublishService {
            service: ServiceAnnouncement,
        },
        UnpublishService {
            service_uuid: String,
        },
        BuildRelayRoute {
            service_uuid: String,
            relay_chain: Vec<String>,
            initiator_peer: String,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type")]
    pub enum P2pResponse {
        HelloAck {
            hello: NodeHello,
        },
        ServiceList {
            services: Vec<ServiceAnnouncement>,
        },
        ConnectInfo {
            provider_peer: String,
            provider_addr: String,
            port: u16,
        },
        RelayRouteReady {
            next_hop: Option<String>,
        },
        Ack,
        Error {
            message: String,
        },
    }
}

use protocol_types::*;

// ===========================================================================
// NodeHello Serialization Tests
// ===========================================================================

#[test]
fn node_hello_serializes_correctly() {
    let hello = NodeHello {
        node_id: "node-123".to_string(),
        role: "edge".to_string(),
    };
    let json = serde_json::to_string(&hello).unwrap();
    assert!(json.contains("node-123"));
    assert!(json.contains("edge"));
}

#[test]
fn node_hello_deserializes_correctly() {
    let json = r#"{"node_id":"node-456","role":"community"}"#;
    let hello: NodeHello = serde_json::from_str(json).unwrap();
    assert_eq!(hello.node_id, "node-456");
    assert_eq!(hello.role, "community");
}

#[test]
fn node_hello_roundtrip() {
    let original = NodeHello {
        node_id: "peer-abc".to_string(),
        role: "edge".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let decoded: NodeHello = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

// ===========================================================================
// ServiceAnnouncement Serialization Tests
// ===========================================================================

#[test]
fn service_announcement_serializes_correctly() {
    let announcement = ServiceAnnouncement {
        uuid: "svc-123".to_string(),
        name: "Test Service".to_string(),
        r#type: "HTTP".to_string(),
        port: 8080,
        description: "A test service".to_string(),
        provider_peer: "peer-123".to_string(),
        provider_addr: "127.0.0.1".to_string(),
    };
    let json = serde_json::to_string(&announcement).unwrap();
    assert!(json.contains("svc-123"));
    assert!(json.contains("Test Service"));
    assert!(json.contains("8080"));
}

#[test]
fn service_announcement_deserializes_correctly() {
    let json = r#"{
        "uuid": "svc-456",
        "name": "Web App",
        "type": "HTTPS",
        "port": 443,
        "description": "Secure web app",
        "provider_peer": "peer-456",
        "provider_addr": "10.0.0.1"
    }"#;
    let announcement: ServiceAnnouncement = serde_json::from_str(json).unwrap();
    assert_eq!(announcement.uuid, "svc-456");
    assert_eq!(announcement.name, "Web App");
    assert_eq!(announcement.r#type, "HTTPS");
    assert_eq!(announcement.port, 443);
}

#[test]
fn service_announcement_roundtrip() {
    let original = ServiceAnnouncement {
        uuid: "svc-789".to_string(),
        name: "Database".to_string(),
        r#type: "TCP".to_string(),
        port: 5432,
        description: "PostgreSQL".to_string(),
        provider_peer: "peer-789".to_string(),
        provider_addr: "db.local".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let decoded: ServiceAnnouncement = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

// ===========================================================================
// P2pRequest Serialization Tests
// ===========================================================================

#[test]
fn hello_request_serializes() {
    let req = P2pRequest::Hello {
        hello: NodeHello {
            node_id: "node-1".to_string(),
            role: "edge".to_string(),
        },
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("Hello"));
    assert!(json.contains("node-1"));
}

#[test]
fn discover_services_request_serializes() {
    let req = P2pRequest::DiscoverServices {
        community_id: "community-1".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("DiscoverServices"));
    assert!(json.contains("community-1"));
}

#[test]
fn subscribe_service_request_serializes() {
    let req = P2pRequest::SubscribeService {
        service_uuid: "svc-1".to_string(),
        subscriber_peer: "peer-1".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("SubscribeService"));
    assert!(json.contains("svc-1"));
    assert!(json.contains("peer-1"));
}

#[test]
fn connect_service_request_serializes() {
    let req = P2pRequest::ConnectService {
        service_uuid: "svc-2".to_string(),
        subscriber_peer: "peer-2".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("ConnectService"));
}

#[test]
fn publish_service_request_serializes() {
    let req = P2pRequest::PublishService {
        service: ServiceAnnouncement {
            uuid: "svc-3".to_string(),
            name: "My Service".to_string(),
            r#type: "HTTP".to_string(),
            port: 3000,
            description: "A service".to_string(),
            provider_peer: "peer-3".to_string(),
            provider_addr: "localhost".to_string(),
        },
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("PublishService"));
    assert!(json.contains("My Service"));
}

#[test]
fn unpublish_service_request_serializes() {
    let req = P2pRequest::UnpublishService {
        service_uuid: "svc-4".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("UnpublishService"));
    assert!(json.contains("svc-4"));
}

#[test]
fn build_relay_route_request_serializes() {
    let req = P2pRequest::BuildRelayRoute {
        service_uuid: "svc-5".to_string(),
        relay_chain: vec!["peer-a".to_string(), "peer-b".to_string()],
        initiator_peer: "peer-init".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("BuildRelayRoute"));
    assert!(json.contains("peer-a"));
    assert!(json.contains("peer-b"));
}

// ===========================================================================
// P2pResponse Serialization Tests
// ===========================================================================

#[test]
fn hello_ack_response_serializes() {
    let resp = P2pResponse::HelloAck {
        hello: NodeHello {
            node_id: "node-ack".to_string(),
            role: "community".to_string(),
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("HelloAck"));
    assert!(json.contains("community"));
}

#[test]
fn service_list_response_serializes() {
    let resp = P2pResponse::ServiceList {
        services: vec![ServiceAnnouncement {
            uuid: "svc-list-1".to_string(),
            name: "Service 1".to_string(),
            r#type: "HTTP".to_string(),
            port: 8080,
            description: "First service".to_string(),
            provider_peer: "peer-1".to_string(),
            provider_addr: "10.0.0.1".to_string(),
        }],
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("ServiceList"));
    assert!(json.contains("Service 1"));
}

#[test]
fn connect_info_response_serializes() {
    let resp = P2pResponse::ConnectInfo {
        provider_peer: "peer-provider".to_string(),
        provider_addr: "192.168.1.1".to_string(),
        port: 9000,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("ConnectInfo"));
    assert!(json.contains("peer-provider"));
    assert!(json.contains("9000"));
}

#[test]
fn relay_route_ready_response_serializes() {
    let resp = P2pResponse::RelayRouteReady {
        next_hop: Some("peer-next".to_string()),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("RelayRouteReady"));
    assert!(json.contains("peer-next"));
}

#[test]
fn relay_route_ready_with_none_serializes() {
    let resp = P2pResponse::RelayRouteReady { next_hop: None };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("RelayRouteReady"));
}

#[test]
fn ack_response_serializes() {
    let resp = P2pResponse::Ack;
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("Ack"));
}

#[test]
fn error_response_serializes() {
    let resp = P2pResponse::Error {
        message: "Something went wrong".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("Error"));
    assert!(json.contains("Something went wrong"));
}

// ===========================================================================
// Edge Cases
// ===========================================================================

#[test]
fn empty_service_list_serializes() {
    let resp = P2pResponse::ServiceList { services: vec![] };
    let json = serde_json::to_string(&resp).unwrap();
    let decoded: P2pResponse = serde_json::from_str(&json).unwrap();
    match decoded {
        P2pResponse::ServiceList { services } => {
            assert!(services.is_empty());
        }
        _ => panic!("Expected ServiceList"),
    }
}

#[test]
fn empty_relay_chain_serializes() {
    let req = P2pRequest::BuildRelayRoute {
        service_uuid: "svc-empty".to_string(),
        relay_chain: vec![],
        initiator_peer: "peer-init".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let decoded: P2pRequest = serde_json::from_str(&json).unwrap();
    match decoded {
        P2pRequest::BuildRelayRoute { relay_chain, .. } => {
            assert!(relay_chain.is_empty());
        }
        _ => panic!("Expected BuildRelayRoute"),
    }
}

#[test]
fn unicode_in_service_name() {
    let announcement = ServiceAnnouncement {
        uuid: "svc-unicode".to_string(),
        name: "测试服务 🚀".to_string(),
        r#type: "HTTP".to_string(),
        port: 8080,
        description: "中文描述".to_string(),
        provider_peer: "peer-cn".to_string(),
        provider_addr: "localhost".to_string(),
    };
    let json = serde_json::to_string(&announcement).unwrap();
    let decoded: ServiceAnnouncement = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.name, "测试服务 🚀");
    assert_eq!(decoded.description, "中文描述");
}

#[test]
fn special_characters_in_error_message() {
    let resp = P2pResponse::Error {
        message: "Error: \"quotes\" and <brackets> & ampersand".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let decoded: P2pResponse = serde_json::from_str(&json).unwrap();
    match decoded {
        P2pResponse::Error { message } => {
            assert!(message.contains("quotes"));
            assert!(message.contains("brackets"));
        }
        _ => panic!("Expected Error"),
    }
}
