import type {
  CommunityNode,
  CommunityService,
  CommunitySummary,
  NodeInfo,
  PublishedService,
  ServiceDescriptor,
  SubscribedService
} from "../types";
import { ElMessage } from "element-plus";

const baseUrl = "";

interface ApiResp<T> {
  code: number;
  message: string;
  data?: T;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options
  });
  const json = (await response.json()) as ApiResp<T>;
  if (!response.ok || json.code !== 0) {
    const msg = json.message || `Request failed: ${response.status}`;
    ElMessage.error(msg);
    throw new Error(msg);
  }
  return json.data as T;
}

export async function fetchNodeInfo(): Promise<NodeInfo> {
  return await request<NodeInfo>("/porta/node/info");
}

export async function updateNodeConfig(payload: Record<string, unknown>): Promise<NodeInfo> {
  return await request<NodeInfo>("/porta/node/config", {
    method: "POST",
    body: JSON.stringify(payload)
  });
}

export async function importNodeKey(payload: Record<string, unknown>): Promise<NodeInfo> {
  return await request<NodeInfo>("/porta/node/key/import", {
    method: "POST",
    body: JSON.stringify(payload)
  });
}

export async function generateNodeKey(): Promise<NodeInfo> {
  return await request<NodeInfo>("/porta/node/key/generate", {
    method: "POST",
    body: JSON.stringify({})
  });
}

export async function fetchCommunities(): Promise<CommunitySummary[]> {
  return await request<CommunitySummary[]>("/porta/community/list");
}

export async function connectCommunity(id: string) {
  return await request("/porta/community/connect", {
    method: "POST",
    body: JSON.stringify({ id })
  });
}

export async function fetchCommunityServices(
  communityId: string
): Promise<ServiceDescriptor[]> {
  return await request<ServiceDescriptor[]>(
    `/porta/service/discover?communityId=${communityId}`
  );
}

export async function fetchSubscribedServices(): Promise<SubscribedService[]> {
  return await request<SubscribedService[]>("/porta/service/subscriptions");
}

export async function fetchPublishedServices(): Promise<PublishedService[]> {
  return await request<PublishedService[]>("/porta/service/published");
}

export async function fetchCommunityNodes(): Promise<CommunityNode[]> {
  return await request<CommunityNode[]>("/porta/community/node/list");
}

export async function fetchCommunityServiceList(): Promise<CommunityService[]> {
  return await request<CommunityService[]>("/porta/community/service/list");
}

export async function subscribeService(payload: Record<string, unknown>) {
  return await request("/porta/service/subscribe", {
    method: "POST",
    body: JSON.stringify(payload)
  });
}

export async function connectService(id: string, connect = true) {
  return await request(
    connect ? "/porta/service/connect" : "/porta/service/disconnect",
    {
      method: "POST",
      body: JSON.stringify({ id })
    }
  );
}

export async function publishService(payload: Record<string, unknown>) {
  return await request("/porta/service/publish", {
    method: "POST",
    body: JSON.stringify(payload)
  });
}

export async function unpublishService(id: string) {
  return await request("/porta/service/unpublish", {
    method: "POST",
    body: JSON.stringify({ id })
  });
}

export async function removePublished(id: string) {
  return await request("/porta/service/remove", {
    method: "POST",
    body: JSON.stringify({ id })
  });
}

export async function announceService(id: string, enabled: boolean) {
  return await request(
    enabled
      ? "/porta/community/service/announce"
      : "/porta/community/service/disable",
    {
      method: "POST",
      body: JSON.stringify({ id })
    }
  );
}

export async function banNode(id: string, banned: boolean) {
  return await request(
    banned ? "/porta/community/node/ban" : "/porta/community/node/unban",
    {
      method: "POST",
      body: JSON.stringify({ id })
    }
  );
}

export async function toggleProxy(enabled: boolean) {
  return await request(enabled ? "/porta/proxy/enable" : "/porta/proxy/disable", {
    method: "POST",
    body: JSON.stringify({ enabled })
  });
}
