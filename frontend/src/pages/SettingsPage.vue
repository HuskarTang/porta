<template>
  <div>
    <h2 class="page-title">设置</h2>
    <p class="page-subtitle">配置节点与网络参数</p>

    <el-card class="table-card" shadow="never">
      <template #header>
        <div>
          <div class="section-title">节点配置</div>
          <div class="section-subtitle">节点标识和密钥管理</div>
        </div>
      </template>
      <el-form :model="nodeInfo" label-width="120px">
        <el-form-item label="节点名称">
          <el-input v-model="nodeInfo.name" placeholder="设置当前节点的可读名称" />
        </el-form-item>
        <el-row :gutter="16">
          <el-col :span="8">
            <el-form-item label="节点 ID">
              <el-input :model-value="nodeInfo.node_id" readonly />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="UUID">
              <el-input :model-value="nodeInfo.uuid" readonly />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="密钥文件">
              <el-input :model-value="nodeInfo.key_path" readonly />
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item>
          <el-button>导出密钥</el-button>
          <el-button>导入密钥</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card class="table-card" shadow="never" style="margin-top: 20px">
      <template #header>
        <div>
          <div class="section-title">网络配置</div>
          <div class="section-subtitle">LibP2P 网络与协议设置</div>
        </div>
      </template>

      <el-form label-width="140px">
        <el-form-item label="启用 TCP">
          <el-switch v-model="nodeInfo.tcp_listen_enable" />
          <el-input
            v-model.number="nodeInfo.tcp_listen_port"
            style="width: 140px; margin-left: 12px"
            placeholder="TCP 端口"
          />
          <span class="hint">0 = 系统随机选择</span>
        </el-form-item>
        <el-form-item label="启用 QUIC">
          <el-switch v-model="nodeInfo.quci_listen_enable" />
          <el-input
            v-model.number="nodeInfo.quci_listen_port"
            style="width: 140px; margin-left: 12px"
            placeholder="QUIC 端口"
          />
          <span class="hint">0 = 系统随机选择</span>
        </el-form-item>
      </el-form>

      <div class="section-divider" />

      <div class="section-title">外网访问地址（可选）</div>
      <el-table :data="externalTable" style="width: 100%; margin-top: 12px">
        <el-table-column label="地址或域名" prop="host" />
        <el-table-column label="协议" prop="protocol" width="120" />
        <el-table-column label="端口" prop="port" width="120" />
        <el-table-column label="操作" width="100">
          <template #default="scope">
            <el-button link type="danger" @click="removeExternal(scope.$index)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="toolbar" style="margin-top: 12px">
        <el-input v-model="newExternal.host" placeholder="地址或域名" />
        <el-select v-model="newExternal.protocol" placeholder="协议" style="width: 120px">
          <el-option label="TCP" value="TCP" />
          <el-option label="QUIC" value="QUIC" />
        </el-select>
        <el-input v-model="newExternal.port" placeholder="端口" style="width: 120px" />
        <el-button type="primary" @click="addExternal">添加</el-button>
      </div>

      <div class="section-divider" />

      <el-form label-width="140px">
        <el-form-item label="启用 mDNS 发现">
          <el-switch v-model="nodeInfo.mdns_enable" />
        </el-form-item>
        <el-form-item label="启用 DHT">
          <el-switch v-model="nodeInfo.dht_enable" />
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive } from "vue";
import { fetchNodeInfo } from "../services/api";

interface ExternalAddress {
  host: string;
  protocol: string;
  port: string;
}

const nodeInfo = reactive({
  name: "",
  node_id: "",
  uuid: "",
  key_path: "",
  tcp_listen_enable: true,
  tcp_listen_port: 0,
  quci_listen_enable: true,
  quci_listen_port: 0,
  external_addr: [] as string[],
  mdns_enable: true,
  dht_enable: true
});

const externalTable = reactive<ExternalAddress[]>([]);
const newExternal = reactive<ExternalAddress>({
  host: "",
  protocol: "TCP",
  port: ""
});

const addExternal = () => {
  if (!newExternal.host || !newExternal.port) return;
  externalTable.push({ ...newExternal });
  newExternal.host = "";
  newExternal.port = "";
};

const removeExternal = (index: number) => {
  externalTable.splice(index, 1);
};

onMounted(async () => {
  const info = await fetchNodeInfo();
  Object.assign(nodeInfo, info);
  externalTable.splice(0, externalTable.length);
  info.external_addr.forEach((item) => {
    const [host, port] = item.split(":");
    externalTable.push({ host, protocol: "TCP", port: port ?? "" });
  });
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

.hint {
  color: #9ca3af;
  font-size: 12px;
  margin-left: 12px;
}

.section-divider {
  height: 1px;
  background: #e5e7eb;
  margin: 16px 0;
}
</style>
