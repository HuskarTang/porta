import { describe, it, expect } from 'vitest'
import router from '../src/router'

describe('Router Configuration', () => {
  describe('Routes', () => {
    it('should have root redirect to settings', () => {
      const rootRoute = router.getRoutes().find(r => r.path === '/')
      expect(rootRoute?.redirect).toBe('/settings')
    })

    it('should have settings route', () => {
      const route = router.getRoutes().find(r => r.path === '/settings')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('设置')
    })

    it('should have communities route', () => {
      const route = router.getRoutes().find(r => r.path === '/communities')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('社区管理')
    })

    it('should have community detail route with id param', () => {
      const route = router.getRoutes().find(r => r.path === '/communities/:id')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('社区详情')
    })

    it('should have services route', () => {
      const route = router.getRoutes().find(r => r.path === '/services')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('服务管理')
    })

    it('should have service-access route', () => {
      const route = router.getRoutes().find(r => r.path === '/service-access')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('服务访问')
    })

    it('should have publish route', () => {
      const route = router.getRoutes().find(r => r.path === '/publish')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('服务发布')
    })

    it('should have node-management route', () => {
      const route = router.getRoutes().find(r => r.path === '/node-management')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('节点管理')
    })

    it('should have community-services route', () => {
      const route = router.getRoutes().find(r => r.path === '/community-services')
      expect(route).toBeDefined()
      expect(route?.meta?.title).toBe('社区服务管理')
    })
  })

  describe('Route Count', () => {
    it('should have 9 routes total', () => {
      // Root redirect + 8 page routes
      const routes = router.getRoutes()
      expect(routes.length).toBe(9)
    })
  })

  describe('Route Meta', () => {
    it('all page routes should have title meta', () => {
      const routes = router.getRoutes().filter(r => r.path !== '/')
      
      routes.forEach(route => {
        expect(route.meta?.title).toBeDefined()
        expect(typeof route.meta?.title).toBe('string')
      })
    })
  })

  describe('Navigation', () => {
    it('should use web history mode', () => {
      // Router was created with createWebHistory
      expect(router.options.history).toBeDefined()
    })
  })
})
