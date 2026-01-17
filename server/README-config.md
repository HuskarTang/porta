# Porta 配置文件说明

## 快速开始

### 使用预配置的配置文件

项目提供了两个预配置的配置文件，可以直接使用：

#### 1. 启动社区节点 (Community Node)

```bash
cd server
./target/release/porta-server --config community.toml
```

社区节点将：
- 监听 HTTP API 端口: **8091**
- 监听 P2P TCP 端口: **9010**
- 监听 P2P QUIC 端口: **9011**
- 数据库文件: `porta-community.db`

#### 2. 启动边缘节点 (Edge Node)

```bash
cd server
./target/release/porta-server --config edge.toml
```

边缘节点将：
- 监听 HTTP API 端口: **8090**
- 监听 P2P TCP 端口: **9000**
- 监听 P2P QUIC 端口: **9001**
- 数据库文件: `porta-edge.db`

### 配置文件位置

配置文件应该放在 `server/` 目录下，或者使用绝对路径：

```bash
# 使用相对路径（从 server 目录）
./target/release/porta-server --config community.toml

# 使用绝对路径
./target/release/porta-server --config /path/to/community.toml
```

### 端口说明

为了避免端口冲突，两个配置文件使用了不同的端口：

| 节点类型 | HTTP API | P2P TCP | P2P QUIC |
|---------|----------|---------|----------|
| Community | 8091 | 9010 | 9011 |
| Edge | 8090 | 9000 | 9001 |

### 连接社区节点

当边缘节点需要连接到社区节点时，使用以下 multiaddr：

```
/ip4/127.0.0.1/tcp/9010/p2p/<COMMUNITY_NODE_ID>
```

其中 `<COMMUNITY_NODE_ID>` 是社区节点的 Peer ID，可以通过以下命令获取：

```bash
curl http://localhost:8091/porta/node/info | jq -r '.data.node_id'
```

### 自定义配置

如果需要修改配置，可以直接编辑对应的 `.toml` 文件：

```bash
# 编辑社区节点配置
vim server/community.toml

# 编辑边缘节点配置
vim server/edge.toml
```

### 测试脚本

使用提供的测试脚本可以自动启动两个节点并测试连接：

```bash
./scripts/test-config-and-join.sh
```

### 注意事项

1. **端口冲突**: 确保没有其他服务占用这些端口
2. **防火墙**: 如果需要在不同机器间连接，确保防火墙允许相应端口
3. **数据库路径**: 数据库文件会创建在配置文件所在目录，或使用绝对路径
4. **外部访问**: 如果需要外部访问，取消注释 `external_addrs` 并填入公网 IP

### 示例：启动两个节点

**终端 1 - 社区节点**:
```bash
cd server
./target/release/porta-server --config community.toml
```

**终端 2 - 边缘节点**:
```bash
cd server
./target/release/porta-server --config edge.toml
```

**终端 3 - 测试连接**:
```bash
# 获取社区节点 ID
COMMUNITY_NODE_ID=$(curl -s http://localhost:8091/porta/node/info | jq -r '.data.node_id')

# 添加社区
curl -X POST http://localhost:8090/porta/community/add \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"测试社区\",
    \"description\": \"本地测试社区\",
    \"multiaddr\": \"/ip4/127.0.0.1/tcp/9010/p2p/$COMMUNITY_NODE_ID\"
  }"

# 加入社区
COMMUNITY_ID=$(curl -s http://localhost:8090/porta/community/list | jq -r '.data[0].id')
curl -X POST http://localhost:8090/porta/community/connect \
  -H "Content-Type: application/json" \
  -d "{\"id\": \"$COMMUNITY_ID\"}"
```
