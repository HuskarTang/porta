<template>
  <div>
    <h2 class="page-title">社区管理</h2>
    <p class="page-subtitle">发现并加入社区，订阅共享服务</p>

    <div class="toolbar">
      <el-input
        v-model="keyword"
        placeholder="输入社区 MultiAddr 或邀请码"
        class="toolbar-grow"
      />
      <el-button @click="load">
        <el-icon><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="section-title">社区列表</div>
      <div class="section-subtitle">浏览和管理你的社区</div>
      <el-table :data="filteredCommunities" style="width: 100%; margin-top: 12px">
        <el-table-column label="社区名称" prop="name" />
        <el-table-column label="描述" prop="description" />
        <el-table-column label="对等节点数" prop="peers" width="120" />
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="row.joined ? 'success' : 'info'">
              {{ row.joined ? "已加入" : "未加入" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button size="small" @click="viewDetail(row)">查看详情</el-button>
            <el-button
              size="small"
              type="primary"
              :disabled="row.joined"
              @click="joinCommunity(row)"
            >
              加入
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Refresh } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { connectCommunity, fetchCommunities } from "../services/api";
import type { CommunitySummary } from "../types";

const router = useRouter();
const keyword = ref("");
const communities = ref<CommunitySummary[]>([]);

const filteredCommunities = computed(() => {
  if (!keyword.value) return communities.value;
  return communities.value.filter((item) =>
    item.name.includes(keyword.value)
  );
});

const viewDetail = (row: CommunitySummary) => {
  router.push({
    path: `/communities/${row.id}`,
    query: { name: row.name, desc: row.description }
  });
};

onMounted(async () => {
  await load();
});

const load = async () => {
  communities.value = await fetchCommunities();
};

const joinCommunity = async (row: CommunitySummary) => {
  await connectCommunity(row.id);
  ElMessage.success(`已加入 ${row.name}`);
  await load();
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
