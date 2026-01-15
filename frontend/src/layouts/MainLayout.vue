<template>
  <el-container class="layout">
    <el-aside width="220px" class="sidebar">
      <div class="brand">
        <div class="brand-logo" />
        <div>
          <div class="brand-title">LibP2P 隧道</div>
          <div class="brand-subtitle">安全管理面板</div>
        </div>
      </div>
      <el-menu :default-active="activeMenu" class="menu" router>
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <span>设置</span>
        </el-menu-item>
        <el-menu-item index="/communities">
          <el-icon><User /></el-icon>
          <span>社区</span>
        </el-menu-item>
        <el-menu-item index="/services">
          <el-icon><Link /></el-icon>
          <span>服务</span>
        </el-menu-item>
        <el-menu-item index="/node-management">
          <el-icon><Connection /></el-icon>
          <span>节点管理</span>
        </el-menu-item>
        <el-menu-item index="/community-services">
          <el-icon><Management /></el-icon>
          <span>服务管理</span>
        </el-menu-item>
        <el-menu-item index="/publish">
          <el-icon><Promotion /></el-icon>
          <span>服务发布</span>
        </el-menu-item>
      </el-menu>
      <div class="sidebar-footer">
        <el-tag type="success">在线</el-tag>
        <span>3 个连接</span>
      </div>
    </el-aside>
    <el-container>
      <el-header class="header">
        <el-tabs
          v-model="activeTab"
          class="tabs"
          type="card"
          closable
          @tab-remove="removeTab"
          @tab-change="switchTab"
        >
          <el-tab-pane
            v-for="tab in openTabs"
            :key="tab.path"
            :label="tab.title"
            :name="tab.path"
            :closable="tab.closable"
          />
        </el-tabs>
      </el-header>
      <el-main class="content">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  Connection,
  Link,
  Management,
  Promotion,
  Setting,
  User
} from "@element-plus/icons-vue";

const route = useRoute();
const router = useRouter();
const activeMenu = computed(() => route.path);
const activeTab = ref(route.path);

type TabInfo = { title: string; path: string; closable: boolean };
const openTabs = reactive<TabInfo[]>([
  { title: "设置", path: "/settings", closable: false }
]);

const addTab = (path: string, title?: string) => {
  const exists = openTabs.find((t) => t.path === path);
  if (exists) return;
  openTabs.push({
    title: title || path,
    path,
    closable: path !== "/settings"
  });
};

const removeTab = (path: string) => {
  const idx = openTabs.findIndex((t) => t.path === path);
  if (idx === -1) return;
  const isActive = activeTab.value === path;
  openTabs.splice(idx, 1);
  if (isActive) {
    const fallback = openTabs[idx - 1] || openTabs[idx] || openTabs[0];
    if (fallback) {
      activeTab.value = fallback.path;
      router.push(fallback.path);
    }
  }
};

const switchTab = (path: string) => {
  router.push(path);
};

watch(
  () => route.fullPath,
  (path) => {
    activeTab.value = path;
    const title = (route.meta.title as string) || "未命名";
    addTab(path, title);
  },
  { immediate: true }
);
</script>

<style scoped>
.layout {
  height: 100%;
}

.sidebar {
  background: #ffffff;
  border-right: 1px solid #e5e7eb;
  display: flex;
  flex-direction: column;
}

.brand {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 20px 16px;
}

.brand-logo {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #111827;
}

.brand-title {
  font-weight: 600;
}

.brand-subtitle {
  font-size: 12px;
  color: #6b7280;
}

.menu {
  border-right: none;
  flex: 1;
}

.header {
  background: #ffffff;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  padding: 0 24px;
}

.tabs {
  width: 100%;
}

.content {
  padding: 24px;
}

.sidebar-footer {
  padding: 16px;
  font-size: 12px;
  color: #6b7280;
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>
