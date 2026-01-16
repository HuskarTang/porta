import { describe, it, expect } from 'vitest'
import type {
  NodeInfo,
  CommunitySummary,
  ServiceDescriptor,
  SubscribedService,
  PublishedService,
  CommunityNode,
  CommunityService,
  SecureRoute
} from '../src/types'

describe('Type Definitions', () => {
  describe('NodeInfo', () => {
    it('should have all required properties', () => {
      const nodeInfo: NodeInfo = {
        name: 'Test Node',
        node_id: 'peer-123',
        uuid: 'uuid-456',
        key_path: '/path/to/key',
        tcp_listen_enable: true,
        tcp_listen_port: 4001,
        quci_listen_enable: false,
        quci_listen_port: 0,
        external_addr: ['192.168.1.1:4001'],
        mdns_enable: true,
        dht_enable: true
      }

      expect(nodeInfo.name).toBe('Test Node')
      expect(nodeInfo.node_id).toBe('peer-123')
      expect(nodeInfo.tcp_listen_enable).toBe(true)
      expect(nodeInfo.external_addr).toHaveLength(1)
    })

    it('should allow empty external_addr array', () => {
      const nodeInfo: NodeInfo = {
        name: 'Minimal',
        node_id: '',
        uuid: '',
        key_path: '',
        tcp_listen_enable: false,
        tcp_listen_port: 0,
        quci_listen_enable: false,
        quci_listen_port: 0,
        external_addr: [],
        mdns_enable: false,
        dht_enable: false
      }

      expect(nodeInfo.external_addr).toEqual([])
    })
  })

  describe('CommunitySummary', () => {
    it('should have all required properties', () => {
      const community: CommunitySummary = {
        id: 'comm-1',
        name: 'Dev Community',
        description: 'For developers',
        peers: 42,
        joined: true
      }

      expect(community.id).toBe('comm-1')
      expect(community.name).toBe('Dev Community')
      expect(community.peers).toBe(42)
      expect(community.joined).toBe(true)
    })
  })

  describe('ServiceDescriptor', () => {
    it('should describe a service', () => {
      const service: ServiceDescriptor = {
        uuid: 'svc-123',
        name: 'Web API',
        type: 'HTTP',
        remote_port: 8080,
        provider: 'peer-provider',
        description: 'REST API service'
      }

      expect(service.uuid).toBe('svc-123')
      expect(service.type).toBe('HTTP')
      expect(service.remote_port).toBe(8080)
    })
  })

  describe('SubscribedService', () => {
    it('should have correct status values', () => {
      const connected: SubscribedService = {
        id: 'sub-1',
        name: 'Service A',
        type: 'HTTP',
        community: 'dev',
        remote_addr: '192.168.1.1:8080',
        local_mapping: '127.0.0.1:18080',
        status: '畅通'
      }

      const connecting: SubscribedService = {
        ...connected,
        id: 'sub-2',
        status: '连接中'
      }

      const disconnected: SubscribedService = {
        ...connected,
        id: 'sub-3',
        status: '断开'
      }

      expect(connected.status).toBe('畅通')
      expect(connecting.status).toBe('连接中')
      expect(disconnected.status).toBe('断开')
    })
  })

  describe('PublishedService', () => {
    it('should have correct status values', () => {
      const online: PublishedService = {
        id: 'pub-1',
        name: 'My API',
        type: 'HTTP',
        port: 3000,
        summary: 'REST API',
        subscriptions: 5,
        status: '在线',
        publish_date: '2026-01-15'
      }

      const offline: PublishedService = {
        ...online,
        id: 'pub-2',
        status: '已下架'
      }

      expect(online.status).toBe('在线')
      expect(offline.status).toBe('已下架')
    })

    it('should track subscription count', () => {
      const service: PublishedService = {
        id: 'pub-1',
        name: 'Popular Service',
        type: 'HTTP',
        port: 8080,
        summary: 'A popular service',
        subscriptions: 100,
        status: '在线',
        publish_date: '2026-01-01'
      }

      expect(service.subscriptions).toBe(100)
    })
  })

  describe('CommunityNode', () => {
    it('should track ban status', () => {
      const normalNode: CommunityNode = {
        id: 'peer-1',
        uuid: 'node-uuid-1',
        status: '在线',
        banned: false
      }

      const bannedNode: CommunityNode = {
        id: 'peer-2',
        uuid: 'node-uuid-2',
        status: '离线',
        banned: true
      }

      expect(normalNode.banned).toBe(false)
      expect(bannedNode.banned).toBe(true)
    })
  })

  describe('CommunityService', () => {
    it('should track online and announced status', () => {
      const service: CommunityService = {
        id: 'svc-1',
        name: 'Test Service',
        uuid: 'svc-uuid-1',
        protocol: 'HTTP',
        port: 8080,
        online: true,
        announced: true
      }

      expect(service.online).toBe(true)
      expect(service.announced).toBe(true)
    })

    it('should allow various protocols', () => {
      const protocols = ['http', 'https', 'tcp', 'udp', 'omega', 'ws']
      
      protocols.forEach(protocol => {
        const service: CommunityService = {
          id: `svc-${protocol}`,
          name: `${protocol} Service`,
          uuid: `uuid-${protocol}`,
          protocol: protocol,
          port: 8080,
          online: true,
          announced: true
        }
        expect(service.protocol).toBe(protocol)
      })
    })
  })

  describe('SecureRoute', () => {
    it('should have relay peers array', () => {
      const route: SecureRoute = {
        id: 'route-1',
        subscription_id: 'sub-1',
        relay_peers: ['peer-a', 'peer-b', 'peer-c'],
        local_port: 19000,
        status: 'connected'
      }

      expect(route.relay_peers).toHaveLength(3)
      expect(route.relay_peers[0]).toBe('peer-a')
    })

    it('should require at least 2 relay peers conceptually', () => {
      const route: SecureRoute = {
        id: 'route-min',
        subscription_id: 'sub-min',
        relay_peers: ['peer-1', 'peer-2'],
        local_port: 10000,
        status: 'connecting'
      }

      expect(route.relay_peers.length).toBeGreaterThanOrEqual(2)
    })
  })
})
