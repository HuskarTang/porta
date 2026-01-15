<template>
  <div>
    <el-button link @click="goBack">← 返回社区列表</el-button>
    <h2 class="page-title">{{ communityName }}</h2>
    <p class="page-subtitle">机器学习模型和数据分析服务</p>

    <el-card class="table-card" shadow="never">
      <div class="section-title">可订阅服务</div>
      <div class="section-subtitle">选择要订阅的服务并配置本地端口映射</div>
      <el-table :data="services" style="width: 100%; margin-top: 12px">
        <el-table-column label="服务名称" prop="name" />
        <el-table-column label="类型" prop="type" width="120" />
        <el-table-column label="远端端口" prop="remote_port" width="120" />
        <el-table-column label="提供者" prop="provider" />
        <el-table-column label="描述" prop="description" />
        <el-table-column label="操作" width="140">
          <template #default="{ row }">
            <el-button
              size="small"
              type="primary"
              :disabled="row.subscribed"
              @click="onSubscribe(row)"
            >
              {{ row.subscribed ? "已订阅" : "订阅" }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { fetchCommunityServices, subscribeService } from "../services/api";
import type { ServiceDescriptor } from "../types";

const route = useRoute();
const router = useRouter();
const communityName = ref("数据科学团队");
const services = ref<(ServiceDescriptor & { subscribed?: boolean })[]>([]);

const goBack = () => {
  router.push("/communities");
};

onMounted(async () => {
  const communityId = route.params.id as string;
  services.value = await fetchCommunityServices(communityId);
});

const onSubscribe = async (row: ServiceDescriptor) => {
  await subscribeService({
    name: row.name,
    type: row.type,
    community: communityName.value,
    remote_addr: `${row.provider}:${row.remote_port}`,
    local_mapping: `localhost:${row.remote_port}`
  });
  ElMessage.success("订阅成功");
  const communityId = route.params.id as string;
  services.value = await fetchCommunityServices(communityId);
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
</style>
