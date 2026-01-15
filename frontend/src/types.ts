export interface NodeInfo {
  name: string;
  node_id: string;
  uuid: string;
  key_path: string;
  tcp_listen_enable: boolean;
  tcp_listen_port: number;
  quci_listen_enable: boolean;
  quci_listen_port: number;
  external_addr: string[];
  mdns_enable: boolean;
  dht_enable: boolean;
}

export interface CommunitySummary {
  id: string;
  name: string;
  description: string;
  peers: number;
  joined: boolean;
}

export interface ServiceDescriptor {
  uuid: string;
  name: string;
  type: string;
  remote_port: number;
  provider: string;
  description: string;
}

export interface SubscribedService {
  id: string;
  name: string;
  type: string;
  community: string;
  remote_addr: string;
  local_mapping: string;
  status: "畅通" | "连接中" | "断开";
}

export interface PublishedService {
  id: string;
  name: string;
  type: string;
  port: number;
  summary: string;
  subscriptions: number;
  status: "在线" | "已下架";
  publish_date: string;
}

export interface CommunityNode {
  id: string;
  uuid: string;
  status: "在线" | "离线";
  banned: boolean;
}

export interface CommunityService {
  id: string;
  name: string;
  uuid: string;
  protocol: string;
  port: number;
  online: boolean;
  announced: boolean;
}
