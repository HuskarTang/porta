import { createRouter, createWebHistory } from "vue-router";
import SettingsPage from "../pages/SettingsPage.vue";
import CommunitiesPage from "../pages/CommunitiesPage.vue";
import CommunityDetailPage from "../pages/CommunityDetailPage.vue";
import ServicesPage from "../pages/ServicesPage.vue";
import ServiceAccessPage from "../pages/ServiceAccessPage.vue";
import PublishPage from "../pages/PublishPage.vue";
import NodeManagementPage from "../pages/NodeManagementPage.vue";
import CommunityServiceManagementPage from "../pages/CommunityServiceManagementPage.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/settings" },
    { path: "/settings", component: SettingsPage, meta: { title: "设置" } },
    { path: "/communities", component: CommunitiesPage, meta: { title: "社区管理" } },
    { path: "/communities/:id", component: CommunityDetailPage, meta: { title: "社区详情" } },
    { path: "/services", component: ServicesPage, meta: { title: "服务管理" } },
    { path: "/service-access", component: ServiceAccessPage, meta: { title: "服务访问" } },
    { path: "/publish", component: PublishPage, meta: { title: "服务发布" } },
    { path: "/node-management", component: NodeManagementPage, meta: { title: "节点管理" } },
    {
      path: "/community-services",
      component: CommunityServiceManagementPage,
      meta: { title: "社区服务管理" }
    }
  ]
});

export default router;
