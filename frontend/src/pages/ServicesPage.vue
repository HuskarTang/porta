<template>
  <div>
    <h2 class="page-title">服务管理</h2>
    <p class="page-subtitle">管理所有已订阅并建立隧道的服务</p>

    <div class="stat-cards">
      <el-card shadow="never">
        <div class="stat-label">总服务数</div>
        <div class="stat-value">{{ stats.total }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">已连接</div>
        <div class="stat-value success">{{ stats.connected }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">已断开</div>
        <div class="stat-value danger">{{ stats.disconnected }}</div>
      </el-card>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="section-title">服务列表</div>
      <div class="section-subtitle">所有已订阅的服务和隧道状态</div>
      <el-table :data="services" style="width: 100%; margin-top: 12px">
        <el-table-column label="服务别名" prop="name" />
        <el-table-column label="类型" prop="type" width="120" />
        <el-table-column label="社区/来源" prop="community" width="140" />
        <el-table-column label="远端地址" prop="remote_addr" width="160" />
        <el-table-column label="本地映射" prop="local_mapping" width="140" />
        <el-table-column label="隧道状态" width="120">
          <template #default="{ row }">
            <el-tag :type="statusTag(row.status)">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="140">
          <template #default="{ row }">
            <el-button size="small" type="primary" :disabled="row.status !== '畅通'" @click="openAccess(row)">
              访问
            </el-button>
            <el-button size="small" text>⋮</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { fetchSubscribedServices } from "../services/api";
import type { SubscribedService } from "../types";

const router = useRouter();
const services = ref<SubscribedService[]>([]);

const stats = computed(() => {
  const total = services.value.length;
  const connected = services.value.filter((item) => item.status === "畅通").length;
  const disconnected = services.value.filter((item) => item.status === "断开").length;
  return { total, connected, disconnected };
});

const statusTag = (status: SubscribedService["status"]) => {
  if (status === "畅通") return "success";
  if (status === "连接中") return "warning";
  return "danger";
};

const openAccess = (row: SubscribedService) => {
  router.push({
    path: "/service-access",
    query: { name: row.name, url: `http://${row.local_mapping}` }
  });
};

onMounted(async () => {
  services.value = await fetchSubscribedServices();
});
</script>

<style scoped>
.section-title {
  font-weight: 600;
}

.section-subtitle {
  color: #6b7280;
  font-size: 12px;
  margin-top: 4px;
}

.stat-label {
  color: #6b7280;
  font-size: 12px;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  margin-top: 8px;
}

.stat-value.success {
  color: #16a34a;
}

.stat-value.danger {
  color: #dc2626;
}
</style>
