<template>
  <div>
    <h2 class="page-title">服务管理</h2>
    <p class="page-subtitle">管理注册到本社区节点的服务</p>

    <div class="stat-cards">
      <el-card shadow="never">
        <div class="stat-label">服务总数</div>
        <div class="stat-value">{{ stats.total }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">在线服务</div>
        <div class="stat-value success">{{ stats.online }}</div>
      </el-card>
    </div>

    <el-card class="table-card" shadow="never" style="margin-top: 16px">
      <div class="section-title">服务列表</div>
      <div class="section-subtitle">管理和监控注册的服务</div>
      <el-table
        :data="services"
        style="width: 100%; margin-top: 12px"
        table-layout="fixed"
      >
        <el-table-column
          label="服务名称"
          prop="name"
          min-width="220"
          show-overflow-tooltip
        />
        <el-table-column
          label="UUID"
          prop="uuid"
          min-width="260"
          show-overflow-tooltip
        />
        <el-table-column label="协议" prop="protocol" width="120" />
        <el-table-column label="端口" prop="port" width="100" />
        <el-table-column label="在线状态" width="120" align="center" header-align="center">
          <template #default="{ row }">
            <el-tag :type="row.online ? 'success' : 'info'">
              {{ row.online ? "在线" : "离线" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="220" align="center" header-align="center">
          <template #default="{ row }">
            <el-switch
              v-model="row.announced"
              active-text="公告中"
              inactive-text="已禁止"
              style="min-width: 160px"
              @change="(val) => onToggleAnnounce(row.id, val)"
            />
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import { announceService, fetchCommunityServiceList } from "../services/api";
import type { CommunityService } from "../types";

const services = ref<CommunityService[]>([]);

const stats = computed(() => {
  const total = services.value.length;
  const online = services.value.filter((item) => item.online).length;
  return { total, online };
});

onMounted(async () => {
  services.value = await fetchCommunityServiceList();
});

const onToggleAnnounce = async (id: string, val: boolean) => {
  await announceService(id, val);
  ElMessage.success(val ? "已公告" : "已禁止");
};
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
</style>
