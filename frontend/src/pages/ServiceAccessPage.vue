<template>
  <div>
    <div class="toolbar">
      <el-input v-model="address" class="toolbar-grow" />
      <el-button @click="copyAddress">复制地址</el-button>
    </div>

    <el-card class="table-card" shadow="never">
      <div class="access-placeholder">
        <el-icon size="32"><Compass /></el-icon>
        <div class="access-title">正在加载服务</div>
        <div class="access-desc">
          实际环境中，这里将显示 iframe 或 webview 内容
        </div>
        <div class="access-url">{{ address }}</div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useRoute } from "vue-router";
import { Compass } from "@element-plus/icons-vue";

const route = useRoute();
const address = ref("http://localhost:8080");

watch(
  () => route.query,
  (query) => {
    if (typeof query.url === "string") {
      address.value = query.url;
    }
  },
  { immediate: true }
);

const copyAddress = () => {
  navigator.clipboard.writeText(address.value);
};
</script>

<style scoped>
.access-placeholder {
  min-height: 400px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #6b7280;
}

.access-title {
  font-size: 18px;
  font-weight: 600;
  color: #111827;
}

.access-desc {
  font-size: 13px;
}

.access-url {
  font-family: "JetBrains Mono", monospace;
  color: #111827;
}
</style>
