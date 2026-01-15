use crate::models::*;

pub fn node_info() -> NodeInfo {
    NodeInfo {
        name: "我的节点".into(),
        node_id: "12D3KooWQy2J8K7X9nW3bPf5Y8kF7H9N2Q5R8S4T6U7V9W1A2B3C".into(),
        uuid: "a3f2e1d4-5c6b-7a8e-9f0d-1c2b3a4e5f6g".into(),
        key_path: "/home/user/.porta/node.key".into(),
        tcp_listen_enable: true,
        tcp_listen_port: 0,
        quci_listen_enable: true,
        quci_listen_port: 0,
        external_addr: vec!["example.com:4001".into()],
        mdns_enable: true,
        dht_enable: true,
    }
}

pub fn communities() -> Vec<CommunitySummary> {
    vec![
        CommunitySummary {
            id: "dev-community".into(),
            name: "开发者社区".into(),
            description: "共享开发环境和工具服务".into(),
            peers: 24,
            joined: true,
        },
        CommunitySummary {
            id: "data-team".into(),
            name: "数据科学团队".into(),
            description: "机器学习模型和数据分析服务".into(),
            peers: 12,
            joined: true,
        },
        CommunitySummary {
            id: "test-env".into(),
            name: "测试环境".into(),
            description: "测试和预发布环境服务".into(),
            peers: 8,
            joined: false,
        },
    ]
}

pub fn discovered_services() -> Vec<ServiceDescriptor> {
    vec![
        ServiceDescriptor {
            uuid: "svc-jupyter".into(),
            name: "Jupyter Hub".into(),
            r#type: "HTTP".into(),
            remote_port: 8888,
            provider: "节点-D999".into(),
            description: "JupyterLab 环境".into(),
        },
        ServiceDescriptor {
            uuid: "svc-mlflow".into(),
            name: "MLflow Server".into(),
            r#type: "HTTP".into(),
            remote_port: 5000,
            provider: "节点-E777".into(),
            description: "ML 模型管理平台".into(),
        },
    ]
}

pub fn subscribed_services() -> Vec<SubscribedService> {
    vec![
        SubscribedService {
            id: "sub-web".into(),
            name: "Web Dashboard".into(),
            r#type: "HTTP".into(),
            community: "开发者社区".into(),
            remote_addr: "节点-A789:8080".into(),
            local_mapping: "localhost:8080".into(),
            status: "畅通".into(),
        },
        SubscribedService {
            id: "sub-db".into(),
            name: "Database Admin".into(),
            r#type: "Database".into(),
            community: "开发者社区".into(),
            remote_addr: "节点-B456:5432".into(),
            local_mapping: "localhost:5432".into(),
            status: "畅通".into(),
        },
        SubscribedService {
            id: "sub-ssh".into(),
            name: "SSH Server".into(),
            r#type: "SSH".into(),
            community: "测试环境".into(),
            remote_addr: "节点-C123:22".into(),
            local_mapping: "localhost:2222".into(),
            status: "畅通".into(),
        },
        SubscribedService {
            id: "sub-jupyter".into(),
            name: "Jupyter Notebook".into(),
            r#type: "HTTP".into(),
            community: "数据科学团队".into(),
            remote_addr: "节点-D999:8888".into(),
            local_mapping: "localhost:8888".into(),
            status: "连接中".into(),
        },
        SubscribedService {
            id: "sub-api".into(),
            name: "Development API".into(),
            r#type: "HTTP".into(),
            community: "开发者社区".into(),
            remote_addr: "节点-E777:3000".into(),
            local_mapping: "localhost:3000".into(),
            status: "断开".into(),
        },
    ]
}

pub fn published_services() -> Vec<PublishedService> {
    vec![
        PublishedService {
            id: "pub-dashboard".into(),
            name: "Team Dashboard".into(),
            r#type: "HTTP".into(),
            port: 8080,
            summary: "团队协作面板，提供实时数据可视化".into(),
            subscriptions: 12,
            status: "在线".into(),
            publish_date: "2026-01-10".into(),
        },
        PublishedService {
            id: "pub-db".into(),
            name: "Development Database".into(),
            r#type: "Database".into(),
            port: 5432,
            summary: "PostgreSQL 开发数据库实例".into(),
            subscriptions: 8,
            status: "在线".into(),
            publish_date: "2026-01-08".into(),
        },
        PublishedService {
            id: "pub-storage".into(),
            name: "File Storage".into(),
            r#type: "HTTP".into(),
            port: 9000,
            summary: "MinIO 对象存储服务".into(),
            subscriptions: 5,
            status: "已下架".into(),
            publish_date: "2026-01-05".into(),
        },
    ]
}

pub fn community_nodes() -> Vec<CommunityNode> {
    vec![
        CommunityNode {
            id: "node-001".into(),
            uuid: "a3f5e8d2-4b1c-4e7f-9a2d-1c8e5f7b3d9a".into(),
            status: "在线".into(),
            banned: false,
        },
        CommunityNode {
            id: "node-002".into(),
            uuid: "b7c9d4e1-5a2f-4b8c-8d3e-2f9a6c8d4e1b".into(),
            status: "在线".into(),
            banned: false,
        },
        CommunityNode {
            id: "node-003".into(),
            uuid: "c8d1e5f2-6b3c-4d9e-7f4a-3b1d7e8f5c2a".into(),
            status: "离线".into(),
            banned: false,
        },
        CommunityNode {
            id: "node-004".into(),
            uuid: "d9e2f6a3-7c4d-4e1b-8a5f-4c2e8f9a6d3b".into(),
            status: "在线".into(),
            banned: false,
        },
        CommunityNode {
            id: "node-005".into(),
            uuid: "e1f3a7b4-8d5e-4f2c-9b6a-5d3f9a1b7e4c".into(),
            status: "离线".into(),
            banned: true,
        },
        CommunityNode {
            id: "node-006".into(),
            uuid: "f2a4b8c5-9e6f-4a3d-1c7b-6e4a1c8f2b5d".into(),
            status: "在线".into(),
            banned: false,
        },
    ]
}

pub fn community_services() -> Vec<CommunityService> {
    vec![
        CommunityService {
            id: "svc-web".into(),
            name: "Web 服务器".into(),
            uuid: "a1b2c3d4...".into(),
            protocol: "HTTP".into(),
            port: 8080,
            online: true,
            announced: true,
        },
        CommunityService {
            id: "svc-api".into(),
            name: "API 网关".into(),
            uuid: "b2c3d4e5...".into(),
            protocol: "HTTPS".into(),
            port: 443,
            online: true,
            announced: true,
        },
        CommunityService {
            id: "svc-db".into(),
            name: "数据库服务".into(),
            uuid: "c3d4e5f6...".into(),
            protocol: "TCP".into(),
            port: 5432,
            online: false,
            announced: false,
        },
        CommunityService {
            id: "svc-ssh".into(),
            name: "SSH 隧道".into(),
            uuid: "d4e5f6a7...".into(),
            protocol: "SSH".into(),
            port: 22,
            online: true,
            announced: false,
        },
    ]
}

pub fn proxy_status() -> ProxyStatus {
    ProxyStatus {
        enabled: true,
        listen_port: 1080,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_three_communities() {
        let data = communities();
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn proxy_default_enabled() {
        let p = proxy_status();
        assert!(p.enabled);
        assert_eq!(p.listen_port, 1080);
    }
}
