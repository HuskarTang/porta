<template>
  <div>
    <h2 class="page-title">社区管理</h2>
    <p class="page-subtitle">发现并加入社区，订阅共享服务</p>

    <div class="toolbar">
      <el-button type="primary" @click="showAddDialog = true">
        <el-icon><Plus /></el-icon>
        添加社区
      </el-button>
      <el-button @click="load">
        <el-icon><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="section-title">社区列表</div>
      <div class="section-subtitle">浏览和管理你的社区</div>
      <el-table :data="communities" style="width: 100%; margin-top: 12px">
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

    <!-- 添加社区对话框 -->
    <el-dialog v-model="showAddDialog" title="添加社区" width="500px">
      <el-form :model="addForm" label-width="100px">
        <el-form-item label="社区名称" required>
          <el-input v-model="addForm.name" placeholder="请输入社区名称" />
        </el-form-item>
        <el-form-item label="社区描述" required>
          <el-input
            v-model="addForm.description"
            type="textarea"
            :rows="3"
            placeholder="请输入社区描述"
          />
        </el-form-item>
        <el-form-item label="MultiAddr" required>
          <el-input
            v-model="addForm.multiaddr"
            placeholder="/ip4/127.0.0.1/tcp/9010/p2p/..."
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="handleAddCommunity" :loading="adding">
          确定
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Refresh, Plus } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { addCommunity, connectCommunity, fetchCommunities } from "../services/api";
import type { CommunitySummary } from "../types";

const router = useRouter();
const communities = ref<CommunitySummary[]>([]);
const showAddDialog = ref(false);
const adding = ref(false);
const addForm = ref({
  name: "",
  description: "",
  multiaddr: ""
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
  try {
    await connectCommunity(row.id);
    ElMessage.success(`已加入 ${row.name}`);
    await load();
  } catch (error) {
    // Error already handled by API service
    console.error("加入社区失败:", error);
  }
};

const handleAddCommunity = async () => {
  if (!addForm.value.name.trim()) {
    ElMessage.warning("请输入社区名称");
    return;
  }
  if (!addForm.value.description.trim()) {
    ElMessage.warning("请输入社区描述");
    return;
  }
  if (!addForm.value.multiaddr.trim()) {
    ElMessage.warning("请输入 MultiAddr");
    return;
  }
  adding.value = true;
  try {
    await addCommunity({
      name: addForm.value.name.trim(),
      description: addForm.value.description.trim(),
      multiaddr: addForm.value.multiaddr.trim()
    });
    ElMessage.success("社区添加成功");
    showAddDialog.value = false;
    addForm.value = { name: "", description: "", multiaddr: "" };
    await load();
  } catch (error) {
    // Error already handled by API service
  } finally {
    adding.value = false;
  }
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
