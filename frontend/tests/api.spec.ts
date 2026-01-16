import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock fetch globally
const mockFetch = vi.fn()
global.fetch = mockFetch

// Mock ElMessage
vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    warning: vi.fn()
  }
}))

// API response helper
function mockApiResponse<T>(data: T, code = 0, message = 'success') {
  return {
    ok: code === 0,
    json: () => Promise.resolve({ code, message, data })
  }
}

describe('API Service', () => {
  beforeEach(() => {
    mockFetch.mockClear()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('fetchNodeInfo', () => {
    it('should call the correct endpoint', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse({
        name: 'Test Node',
        node_id: 'peer-123',
        uuid: 'uuid-456',
        key_path: '/path/key',
        tcp_listen_enable: true,
        tcp_listen_port: 4001,
        quci_listen_enable: false,
        quci_listen_port: 0,
        external_addr: [],
        mdns_enable: true,
        dht_enable: true
      }))

      const { fetchNodeInfo } = await import('../src/services/api')
      const result = await fetchNodeInfo()

      expect(mockFetch).toHaveBeenCalledWith('/porta/node/info', expect.objectContaining({
        headers: { 'Content-Type': 'application/json' }
      }))
      expect(result.name).toBe('Test Node')
    })
  })

  describe('fetchCommunities', () => {
    it('should return array of communities', async () => {
      const communities = [
        { id: 'comm-1', name: 'Dev', description: 'Devs', peers: 10, joined: true },
        { id: 'comm-2', name: 'Test', description: 'Testers', peers: 5, joined: false }
      ]
      mockFetch.mockResolvedValueOnce(mockApiResponse(communities))

      const { fetchCommunities } = await import('../src/services/api')
      const result = await fetchCommunities()

      expect(result).toHaveLength(2)
      expect(result[0].name).toBe('Dev')
    })
  })

  describe('connectCommunity', () => {
    it('should send POST request with id', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { connectCommunity } = await import('../src/services/api')
      await connectCommunity('comm-123')

      expect(mockFetch).toHaveBeenCalledWith('/porta/community/connect', expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ id: 'comm-123' })
      }))
    })
  })

  describe('fetchCommunityServices', () => {
    it('should include communityId in query', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse([]))

      const { fetchCommunityServices } = await import('../src/services/api')
      await fetchCommunityServices('comm-456')

      expect(mockFetch).toHaveBeenCalledWith(
        '/porta/service/discover?communityId=comm-456',
        expect.any(Object)
      )
    })
  })

  describe('subscribeService', () => {
    it('should POST subscription payload', async () => {
      const payload = {
        name: 'Web Service',
        type: 'HTTP',
        community: 'dev',
        remote_addr: '192.168.1.1:8080',
        local_mapping: '127.0.0.1:18080'
      }
      mockFetch.mockResolvedValueOnce(mockApiResponse({ id: 'sub-1', ...payload, status: '畅通' }))

      const { subscribeService } = await import('../src/services/api')
      await subscribeService(payload)

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/subscribe', expect.objectContaining({
        method: 'POST',
        body: JSON.stringify(payload)
      }))
    })
  })

  describe('connectService', () => {
    it('should call connect endpoint when connect=true', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { connectService } = await import('../src/services/api')
      await connectService('sub-1', true)

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/connect', expect.any(Object))
    })

    it('should call disconnect endpoint when connect=false', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { connectService } = await import('../src/services/api')
      await connectService('sub-1', false)

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/disconnect', expect.any(Object))
    })
  })

  describe('publishService', () => {
    it('should POST publish payload', async () => {
      const payload = {
        name: 'My API',
        type: 'HTTP',
        port: 3000,
        summary: 'REST API'
      }
      mockFetch.mockResolvedValueOnce(mockApiResponse({
        id: 'pub-1',
        ...payload,
        subscriptions: 0,
        status: '在线',
        publish_date: '2026-01-15'
      }))

      const { publishService } = await import('../src/services/api')
      await publishService(payload)

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/publish', expect.objectContaining({
        method: 'POST'
      }))
    })
  })

  describe('unpublishService', () => {
    it('should POST with service id', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { unpublishService } = await import('../src/services/api')
      await unpublishService('pub-1')

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/unpublish', expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ id: 'pub-1' })
      }))
    })
  })

  describe('toggleProxy', () => {
    it('should call enable endpoint when enabled=true', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { toggleProxy } = await import('../src/services/api')
      await toggleProxy(true)

      expect(mockFetch).toHaveBeenCalledWith('/porta/proxy/enable', expect.any(Object))
    })

    it('should call disable endpoint when enabled=false', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { toggleProxy } = await import('../src/services/api')
      await toggleProxy(false)

      expect(mockFetch).toHaveBeenCalledWith('/porta/proxy/disable', expect.any(Object))
    })
  })

  describe('secureConnect', () => {
    it('should POST secure connect payload', async () => {
      const payload = {
        subscription_id: 'sub-1',
        relay_peers: ['peer-a', 'peer-b'],
        local_port: 19000
      }
      mockFetch.mockResolvedValueOnce(mockApiResponse({
        id: 'route-1',
        ...payload,
        status: 'connected'
      }))

      const { secureConnect } = await import('../src/services/api')
      await secureConnect(payload)

      expect(mockFetch).toHaveBeenCalledWith('/porta/service/secure-connect', expect.objectContaining({
        method: 'POST'
      }))
    })
  })

  describe('banNode', () => {
    it('should call ban endpoint when banned=true', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { banNode } = await import('../src/services/api')
      await banNode('peer-1', true)

      expect(mockFetch).toHaveBeenCalledWith('/porta/community/node/ban', expect.any(Object))
    })

    it('should call unban endpoint when banned=false', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { banNode } = await import('../src/services/api')
      await banNode('peer-1', false)

      expect(mockFetch).toHaveBeenCalledWith('/porta/community/node/unban', expect.any(Object))
    })
  })

  describe('announceService', () => {
    it('should call announce endpoint when enabled=true', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { announceService } = await import('../src/services/api')
      await announceService('svc-1', true)

      expect(mockFetch).toHaveBeenCalledWith('/porta/community/service/announce', expect.any(Object))
    })

    it('should call disable endpoint when enabled=false', async () => {
      mockFetch.mockResolvedValueOnce(mockApiResponse(null))

      const { announceService } = await import('../src/services/api')
      await announceService('svc-1', false)

      expect(mockFetch).toHaveBeenCalledWith('/porta/community/service/disable', expect.any(Object))
    })
  })

  describe('Error handling', () => {
    it('should throw on non-success response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: () => Promise.resolve({ code: 1, message: 'Error occurred', data: null })
      })

      const { fetchNodeInfo } = await import('../src/services/api')
      await expect(fetchNodeInfo()).rejects.toThrow('Error occurred')
    })
  })
})
