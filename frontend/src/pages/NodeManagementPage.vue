<template>
  <div>
    <h2 class="page-title">节点管理</h2>
    <p class="page-subtitle">管理接入本节点的所有普通节点</p>

    <div class="stat-cards">
      <el-card shadow="never">
        <div class="stat-label">注册节点数</div>
        <div class="stat-value">{{ stats.total }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">在线节点数</div>
        <div class="stat-value success">{{ stats.online }}</div>
      </el-card>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="section-title">节点列表</div>
      <div class="section-subtitle">管理所有接入的普通节点</div>
      <el-table :data="nodes" style="width: 100%; margin-top: 12px">
        <el-table-column label="节点 ID" prop="id" width="140" />
        <el-table-column label="UUID" prop="uuid" />
        <el-table-column label="在线状态" width="120">
          <template #default="{ row }">
            <el-tag :type="row.status === '在线' ? 'success' : 'info'">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="140">
          <template #default="{ row }">
            <el-button size="small" type="danger" v-if="!row.banned">
              封禁
            </el-button>
            <el-button size="small" type="success" v-else>
              解除封禁
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { fetchCommunityNodes } from "../services/api";
import type { CommunityNode } from "../types";

const nodes = ref<CommunityNode[]>([]);

const stats = computed(() => {
  const total = nodes.value.length;
  const online = nodes.value.filter((item) => item.status === "在线").length;
  return { total, online };
});

onMounted(async () => {
  nodes.value = await fetchCommunityNodes();
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
</style>
