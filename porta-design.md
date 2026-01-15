# Porta 详细设计文档（DDD）
文档名称：Porta 详细设计  
版本：0.1  
状态：草案  
维护人：待定  
最后更新：2026-01-15  
关联需求：`porta-requirement.md`  

---

# 1. 设计目标
- 将 SRS 中的需求拆解为可实现的模块、接口与数据结构
- 明确前后端边界、协议边界与依赖关系
- 为 AI 模型提供可分阶段交付的实现计划与里程碑

---

# 2. 系统架构设计
## 2.1 总体架构
- **前端 UI**：Vue + TS + ElementPlus（桌面 GUI 与 Web 管理共用）
- **后台服务**：Rust 服务进程，提供 API、状态管理与本地持久化
- **P2P 网络层**：基于 LibP2P，承担节点连接、服务注册/发现、路由与流量转发
- **运行模式**：GUI 模式 / 后台服务模式（同一前端资源，启动方式不同）

## 2.2 模块分解
- **node-core**：节点身份、配置、生命周期
- **community-manager**：社区节点列表与连接管理
- **service-catalog**：服务定义、发布、订阅、发现
- **tunnel-manager**：隧道建立与维护（含 hole punching）
- **port-mapper**：端口映射与 HTTP/HTTPS 访问适配
- **secure-routing**：安全服务映射（多跳代理）
- **omega-proxy**：本地代理监听与转发
- **ui-shell**：前端路由、状态管理与操作入口
- **api-gateway**：前后端 HTTP API 与鉴权
- **storage**：配置、服务订阅表、缓存

---

# 3. 组件与接口设计
## 3.1 前端组件
### 3.1.1 节点配置页
- 输入：节点名称、key 文件、监听端口、外部地址
- 输出：保存配置、生成/导入密钥、配置校验

### 3.1.2 社区节点管理页
- 输入：Multiaddr/二维码/UUID/PeerID
- 输出：社区列表、连接状态、删除/更新

### 3.1.3 服务发现与订阅
- 输入：社区筛选、搜索条件
- 输出：服务列表、订阅操作

### 3.1.4 服务连接与访问
- 输入：订阅服务选择
- 输出：连接状态、断开、访问按钮

### 3.1.5 服务发布与管理
- 输入：服务协议、端口、名称、描述
- 输出：已发布列表、下架/删除

### 3.1.6 社区管理（服务/节点）
- 输入：服务公告开关、节点封禁/解封
- 输出：状态同步

## 3.2 后端服务接口（HTTP API）
说明：接口路径为建议草案，后续可按实际项目调整。

### 节点与配置
- `GET /porta/node/info`
- `POST /porta/node/config`
- `POST /porta/node/key/import`
- `POST /porta/node/key/generate`

### 社区节点管理
- `GET /porta/community/list`
- `POST /porta/community/add`
- `POST /porta/community/remove`
- `POST /porta/community/connect`

### 服务发现/订阅
- `GET /porta/service/discover?communityId=`
- `POST /porta/service/subscribe`
- `GET /porta/service/subscriptions`

### 服务连接/访问
- `POST /porta/service/connect`
- `POST /porta/service/disconnect`
- `GET /porta/service/sessions`
- `POST /porta/service/access`（HTTP/HTTPS 打开端口映射）

### 服务发布
- `POST /porta/service/publish`
- `POST /porta/service/unpublish`
- `DELETE /porta/service/remove`

### 社区管理
- `POST /porta/community/service/announce`
- `POST /porta/community/service/disable`
- `POST /porta/community/node/ban`
- `POST /porta/community/node/unban`

### Omega 代理
- `POST /porta/proxy/enable`
- `POST /porta/proxy/disable`
- `GET /porta/proxy/status`

---

# 4. 数据模型设计
## 4.1 NodeConfig
```
name: string
key_path: string
tcp_listen_enable: bool
tcp_listen_port: number
quci_listen_enable: bool
quci_listen_port: number

listen_port: number
external_addr: []string
mode: "gui" | "service"
```

## 4.2 CommunityNode
```
id: string
name: string
multiaddr: string
peer_id: string
status: "disconnected" | "connecting" | "connected"
last_seen: datetime
```

## 4.3 ServiceDescriptor
```
uuid: string
name: string
protocol: "http" | "https" | "tcp" | "udp" | "omega" | "ws"
port: number
description: string
node_id: string
```

## 4.4 Subscription
```
service_uuid: string
community_id: string
status: "active" | "paused"
created_at: datetime
```

## 4.5 Session
```
session_id: string
service_uuid: string
local_port: number
remote_peer: string
state: "connecting" | "connected" | "closed" | "error"
```

---

# 5. 关键流程设计
## 5.1 社区节点连接
1. 用户添加社区节点 Multiaddr
2. Backend 调用 LibP2P 建立连接
3. 完成注册与能力同步
4. 前端更新连接状态

## 5.2 服务发现与订阅
1. 前端请求服务发现列表
2. Backend 与 CommunityNode 协议交互获取服务
3. 用户选择订阅，保存到订阅表

## 5.3 服务连接与访问
1. 用户点击连接
2. Backend 建立隧道并记录 Session
3. HTTP/HTTPS 服务触发端口映射
4. 返回本地访问地址给前端打开浏览器

## 5.4 安全服务映射
1. 选择目标服务
2. 选择至少两个中间节点
3. 建立多跳代理链路
4. 在本地建立映射端口

## 5.5 Omega 代理
1. 用户启用代理
2. Backend 启动本地监听端口
3. 作为服务发布到社区

---

# 6. 安全与权限设计
- 全链路启用 Noise + TLS
- 配置与密钥文件本地加密存储
- 访问控制策略：初期默认本地用户可操作
- 后续可扩展 Token/porta-Key 认证

---

# 7. 运行模式与部署
- GUI 模式：启动桌面 UI + 后台服务
- 服务模式：仅启动后台服务 + Web 管理端
- 统一配置文件路径适配各平台规范

---

# 8. 分阶段实施计划（面向 AI 模型）
说明：每阶段完成可独立交付、可测试的最小闭环功能。

## Phase 1：基础骨架与配置能力
- Rust 后端服务框架与配置管理
- 节点配置页面（UI）
- key 文件导入/生成接口
- 本地配置持久化
- API 网关与前后端通讯打通

## Phase 2：社区节点管理与连接
- 社区节点列表、添加/删除 UI
- 后端保存社区节点信息
- LibP2P 连接管理与状态同步
- 服务发现 API 的基础框架

## Phase 3：服务发现与订阅
- 服务发现协议联通
- 前端服务列表与订阅管理
- 订阅表本地存储

## Phase 4：服务连接与访问（基础映射）
- 隧道建立与 Session 管理
- TCP/HTTP 服务映射
- 本地端口访问联通

## Phase 5：服务发布与社区管理
- 本地服务发布 UI
- 后端服务注册协议
- 社区节点公告管理

## Phase 6：安全映射与 Omega 代理
- 多跳代理链路构建
- 安全服务映射 UI/后端
- Omega 代理启停与发布

## Phase 7：稳定性与优化
- 重连、心跳、断线恢复
- 性能优化与错误处理
- 打包与跨平台适配

---

# 9. 可测试性与验收映射
每个 Phase 对应的验收点需要与 `porta-requirement.md` 的 7.1 功能验收对齐，确保阶段交付可量化验证。

---

# 10. 风险与待补充点
- 关键协议细节（服务注册/发现/路由）需进一步定义
- Session 生命周期管理策略需要明确
- 安全服务映射的多跳选择策略需设计
- 代理访问权限与认证方式需补充
