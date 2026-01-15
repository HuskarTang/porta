<template>
  <div>
    <div class="page-header">
      <div>
        <h2 class="page-title">服务发布</h2>
        <p class="page-subtitle">发布本地服务供其他节点访问</p>
      </div>
      <el-button type="primary" @click="dialogVisible = true">
        + 新增发布
      </el-button>
    </div>

    <div class="stat-cards">
      <el-card shadow="never">
        <div class="stat-label">已发布服务</div>
        <div class="stat-value">{{ stats.total }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">在线服务</div>
        <div class="stat-value success">{{ stats.online }}</div>
      </el-card>
      <el-card shadow="never">
        <div class="stat-label">总订阅数</div>
        <div class="stat-value">{{ stats.subscriptions }}</div>
      </el-card>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="section-title">已发布服务</div>
      <div class="section-subtitle">管理你发布的所有服务</div>
      <el-table :data="services" style="width: 100%; margin-top: 12px">
        <el-table-column label="服务名称" prop="name" />
        <el-table-column label="类型" prop="type" width="120" />
        <el-table-column label="监听端口" prop="port" width="120" />
        <el-table-column label="概述" prop="summary" />
        <el-table-column label="订阅数" prop="subscriptions" width="120" />
        <el-table-column label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="row.status === '在线' ? 'success' : 'info'">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="发布日期" prop="publish_date" width="120" />
        <el-table-column label="操作" width="180">
          <template #default="{ row }">
            <el-button size="small">
              {{ row.status === "在线" ? "下架" : "上架" }}
            </el-button>
            <el-button size="small" type="danger">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="dialogVisible" title="新增服务发布" width="480px">
      <el-form :model="newService" label-width="100px">
        <el-form-item label="服务名称" required>
          <el-input v-model="newService.name" placeholder="例如：Team Dashboard" />
        </el-form-item>
        <el-form-item label="服务类型" required>
          <el-select v-model="newService.type" placeholder="选择类型">
            <el-option label="HTTP" value="HTTP" />
            <el-option label="Database" value="Database" />
            <el-option label="TCP" value="TCP" />
          </el-select>
        </el-form-item>
        <el-form-item label="监听端口" required>
          <el-input v-model="newService.port" placeholder="例如：8080" />
        </el-form-item>
        <el-form-item label="服务概述">
          <el-input
            v-model="newService.summary"
            type="textarea"
            placeholder="简要描述服务的功能和用途"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitPublish">确认发布</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { fetchPublishedServices } from "../services/api";
import type { PublishedService } from "../types";

const services = ref<PublishedService[]>([]);
const dialogVisible = ref(false);
const newService = reactive({
  name: "",
  type: "HTTP",
  port: "",
  summary: ""
});

const stats = computed(() => {
  const total = services.value.length;
  const online = services.value.filter((item) => item.status === "在线").length;
  const subscriptions = services.value.reduce(
    (sum, item) => sum + item.subscriptions,
    0
  );
  return { total, online, subscriptions };
});

const submitPublish = () => {
  dialogVisible.value = false;
};

onMounted(async () => {
  services.value = await fetchPublishedServices();
});
</script>

<style scoped>
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

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
