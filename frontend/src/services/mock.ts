import type {
  CommunityNode,
  CommunityService,
  CommunitySummary,
  NodeInfo,
  PublishedService,
  ServiceDescriptor,
  SubscribedService
} from "../types";

export const mockNodeInfo: NodeInfo = {
  name: "我的节点",
  node_id: "12D3KooWQy2J8K7X9nW3bPf5Y8kF7H9N2Q5R8S4T6U7V9W1A2B3C",
  uuid: "a3f2e1d4-5c6b-7a8e-9f0d-1c2b3a4e5f6g",
  key_path: "/home/user/.porta/node.key",
  tcp_listen_enable: true,
  tcp_listen_port: 0,
  quci_listen_enable: true,
  quci_listen_port: 0,
  external_addr: ["example.com:4001"],
  mdns_enable: true,
  dht_enable: true
};

export const mockCommunities: CommunitySummary[] = [
  {
    id: "dev-community",
    name: "开发者社区",
    description: "共享开发环境和工具服务",
    peers: 24,
    joined: true
  },
  {
    id: "data-team",
    name: "数据科学团队",
    description: "机器学习模型和数据分析服务",
    peers: 12,
    joined: true
  },
  {
    id: "test-env",
    name: "测试环境",
    description: "测试和预发布环境服务",
    peers: 8,
    joined: false
  }
];

export const mockDiscoveredServices: ServiceDescriptor[] = [
  {
    uuid: "svc-jupyter",
    name: "Jupyter Hub",
    type: "HTTP",
    remote_port: 8888,
    provider: "节点-D999",
    description: "JupyterLab 环境"
  },
  {
    uuid: "svc-mlflow",
    name: "MLflow Server",
    type: "HTTP",
    remote_port: 5000,
    provider: "节点-E777",
    description: "ML 模型管理平台"
  }
];

export const mockSubscribedServices: SubscribedService[] = [
  {
    id: "sub-web",
    name: "Web Dashboard",
    type: "HTTP",
    community: "开发者社区",
    remote_addr: "节点-A789:8080",
    local_mapping: "localhost:8080",
    status: "畅通"
  },
  {
    id: "sub-db",
    name: "Database Admin",
    type: "Database",
    community: "开发者社区",
    remote_addr: "节点-B456:5432",
    local_mapping: "localhost:5432",
    status: "畅通"
  },
  {
    id: "sub-ssh",
    name: "SSH Server",
    type: "SSH",
    community: "测试环境",
    remote_addr: "节点-C123:22",
    local_mapping: "localhost:2222",
    status: "畅通"
  },
  {
    id: "sub-jupyter",
    name: "Jupyter Notebook",
    type: "HTTP",
    community: "数据科学团队",
    remote_addr: "节点-D999:8888",
    local_mapping: "localhost:8888",
    status: "连接中"
  },
  {
    id: "sub-api",
    name: "Development API",
    type: "HTTP",
    community: "开发者社区",
    remote_addr: "节点-E777:3000",
    local_mapping: "localhost:3000",
    status: "断开"
  }
];

export const mockPublishedServices: PublishedService[] = [
  {
    id: "pub-dashboard",
    name: "Team Dashboard",
    type: "HTTP",
    port: 8080,
    summary: "团队协作面板，提供实时数据可视化",
    subscriptions: 12,
    status: "在线",
    publish_date: "2026-01-10"
  },
  {
    id: "pub-db",
    name: "Development Database",
    type: "Database",
    port: 5432,
    summary: "PostgreSQL 开发数据库实例",
    subscriptions: 8,
    status: "在线",
    publish_date: "2026-01-08"
  },
  {
    id: "pub-storage",
    name: "File Storage",
    type: "HTTP",
    port: 9000,
    summary: "MinIO 对象存储服务",
    subscriptions: 5,
    status: "已下架",
    publish_date: "2026-01-05"
  }
];

export const mockCommunityNodes: CommunityNode[] = [
  { id: "node-001", uuid: "a3f5e8d2-4b1c-4e7f-9a2d-1c8e5f7b3d9a", status: "在线", banned: false },
  { id: "node-002", uuid: "b7c9d4e1-5a2f-4b8c-8d3e-2f9a6c8d4e1b", status: "在线", banned: false },
  { id: "node-003", uuid: "c8d1e5f2-6b3c-4d9e-7f4a-3b1d7e8f5c2a", status: "离线", banned: false },
  { id: "node-004", uuid: "d9e2f6a3-7c4d-4e1b-8a5f-4c2e8f9a6d3b", status: "在线", banned: false },
  { id: "node-005", uuid: "e1f3a7b4-8d5e-4f2c-9b6a-5d3f9a1b7e4c", status: "离线", banned: true },
  { id: "node-006", uuid: "f2a4b8c5-9e6f-4a3d-1c7b-6e4a1c8f2b5d", status: "在线", banned: false }
];

export const mockCommunityServices: CommunityService[] = [
  {
    id: "svc-web",
    name: "Web 服务器",
    uuid: "a1b2c3d4...",
    protocol: "HTTP",
    port: 8080,
    online: true,
    announced: true
  },
  {
    id: "svc-api",
    name: "API 网关",
    uuid: "b2c3d4e5...",
    protocol: "HTTPS",
    port: 443,
    online: true,
    announced: true
  },
  {
    id: "svc-db",
    name: "数据库服务",
    uuid: "c3d4e5f6...",
    protocol: "TCP",
    port: 5432,
    online: false,
    announced: false
  },
  {
    id: "svc-ssh",
    name: "SSH 隧道",
    uuid: "d4e5f6a7...",
    protocol: "SSH",
    port: 22,
    online: true,
    announced: false
  }
];
