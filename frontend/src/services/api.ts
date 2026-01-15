import type {
  CommunityNode,
  CommunityService,
  CommunitySummary,
  NodeInfo,
  PublishedService,
  ServiceDescriptor,
  SubscribedService
} from "../types";
import {
  mockCommunities,
  mockCommunityNodes,
  mockCommunityServices,
  mockDiscoveredServices,
  mockNodeInfo,
  mockPublishedServices,
  mockSubscribedServices
} from "./mock";

const baseUrl = "";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options
  });
  if (!response.ok) throw new Error(`Request failed: ${response.status}`);
  return (await response.json()) as T;
}

export async function fetchNodeInfo(): Promise<NodeInfo> {
  try {
    return await request<NodeInfo>("/porta/node/info");
  } catch {
    return mockNodeInfo;
  }
}

export async function fetchCommunities(): Promise<CommunitySummary[]> {
  try {
    return await request<CommunitySummary[]>("/porta/community/list");
  } catch {
    return mockCommunities;
  }
}

export async function fetchCommunityServices(
  communityId: string
): Promise<ServiceDescriptor[]> {
  try {
    return await request<ServiceDescriptor[]>(
      `/porta/service/discover?communityId=${communityId}`
    );
  } catch {
    return mockDiscoveredServices;
  }
}

export async function fetchSubscribedServices(): Promise<SubscribedService[]> {
  try {
    return await request<SubscribedService[]>(
      "/porta/service/subscriptions"
    );
  } catch {
    return mockSubscribedServices;
  }
}

export async function fetchPublishedServices(): Promise<PublishedService[]> {
  try {
    return await request<PublishedService[]>("/porta/service/published");
  } catch {
    return mockPublishedServices;
  }
}

export async function fetchCommunityNodes(): Promise<CommunityNode[]> {
  try {
    return await request<CommunityNode[]>("/porta/community/node/list");
  } catch {
    return mockCommunityNodes;
  }
}

export async function fetchCommunityServiceList(): Promise<CommunityService[]> {
  try {
    return await request<CommunityService[]>(
      "/porta/community/service/list"
    );
  } catch {
    return mockCommunityServices;
  }
}
