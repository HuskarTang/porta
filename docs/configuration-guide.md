# Porta 配置指南

## 配置文件位置

### 默认配置文件路径

Porta 会在以下位置查找配置文件：

**服务器模式 (porta-server)**:
- macOS: `~/Library/Application Support/porta/config.toml`
- Linux: `~/.config/porta/config.toml`
- Windows: `%APPDATA%\Porta\config.toml`

**桌面应用 (Porta.app)**:
- 使用应用数据目录，由 Tauri 自动管理

### 自定义配置文件

可以通过 `--config` 参数指定配置文件路径：

```bash
porta-server --config /path/to/your/config.toml
```

## 配置文件格式

配置文件使用 TOML 格式。首次运行时，如果配置文件不存在，将自动创建默认配置。

### 完整配置示例

```toml
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "Porta Server"
role = "edge"  # 或 "community"

[database]
path = "~/Library/Application Support/porta/porta.db"  # macOS 示例

[p2p]
tcp_port = 9000
quic_port = 9001
mdns_enable = true
dht_enable = true
external_addrs = []

[logging]
level = "info"  # trace, debug, info, warn, error
format = "compact"  # compact, pretty, json
```

## 配置项说明

### [server] - 服务器配置

- `listen_addr`: HTTP API 监听地址（默认: `0.0.0.0`）
- `port`: HTTP API 监听端口（默认: `8090`）

### [node] - 节点配置

- `name`: 节点显示名称（默认: `"Porta Server"`）
- `role`: 节点角色，可选值：
  - `"edge"`: 边缘节点，用于访问和使用服务
  - `"community"`: 社区节点，用于管理和分发服务

### [database] - 数据库配置

- `path`: SQLite 数据库文件路径
  - 默认使用用户数据目录
  - 可以使用相对路径或绝对路径
  - 特殊值 `:memory:` 表示使用内存数据库（测试用）

### [p2p] - P2P 网络配置

- `tcp_port`: TCP 监听端口（默认: `9000`）
- `quic_port`: QUIC 监听端口（默认: `9001`）
- `mdns_enable`: 启用 mDNS 本地发现（默认: `true`）
- `dht_enable`: 启用 DHT 分布式发现（默认: `true`）
- `external_addrs`: 外部地址列表，用于 NAT 穿透（默认: `[]`）

### [logging] - 日志配置

- `level`: 日志级别
  - `"trace"`: 最详细
  - `"debug"`: 调试信息
  - `"info"`: 一般信息（默认）
  - `"warn"`: 警告
  - `"error"`: 仅错误

- `format`: 日志格式
  - `"compact"`: 紧凑格式，包含文件和行号（默认）
  - `"pretty"`: 易读格式，适合开发
  - `"json"`: JSON 格式，适合日志收集

## 命令行参数

命令行参数可以覆盖配置文件中的设置：

```bash
# 指定配置文件
porta-server --config /path/to/config.toml

# 覆盖监听地址
porta-server --listen 127.0.0.1

# 覆盖端口
porta-server --port 9090

# 覆盖日志级别
porta-server --log-level debug

# 打印当前配置
porta-server --print-config

# 验证配置文件
porta-server --validate
```

## 使用场景

### 开发环境

```toml
[server]
port = 8090

[node]
role = "edge"

[logging]
level = "debug"
format = "pretty"
```

### 生产环境 - 边缘节点

```toml
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "Production Edge Node"
role = "edge"

[p2p]
tcp_port = 9000
quic_port = 9001
external_addrs = ["<your-public-ip>:9000"]

[logging]
level = "info"
format = "json"
```

### 生产环境 - 社区节点

```toml
[server]
listen_addr = "0.0.0.0"
port = 8091

[node]
name = "Community Hub"
role = "community"

[p2p]
tcp_port = 9010
quic_port = 9011
external_addrs = ["<your-public-ip>:9010"]

[logging]
level = "info"
format = "json"
```

## 配置文件管理

### 查看当前配置

```bash
porta-server --print-config
```

### 验证配置文件

```bash
porta-server --config /path/to/config.toml --validate
```

### 重置为默认配置

删除配置文件，下次启动时会自动创建：

```bash
# macOS
rm ~/Library/Application\ Support/porta/config.toml

# Linux
rm ~/.config/porta/config.toml

# Windows
del %APPDATA%\Porta\config.toml
```

## 数据库位置

数据库文件默认存储在：

**服务器模式**:
- macOS: `~/Library/Application Support/porta/porta.db`
- Linux: `~/.local/share/porta/porta.db`
- Windows: `%LOCALAPPDATA%\porta\porta.db`

**桌面应用**:
- 由 Tauri 管理，位于应用数据目录

## 环境变量

以下环境变量用于内部配置传递，不建议手动设置：

- `PORTA_ROLE`: 节点角色
- `PORTA_DB`: 数据库路径
- `PORTA_P2P_TCP_PORT`: P2P TCP 端口
- `PORTA_NODE_NAME`: 节点名称

## 故障排查

### 配置文件不生效

1. 检查配置文件路径是否正确
2. 验证配置文件语法：`porta-server --validate`
3. 查看启动日志，确认加载的配置文件路径

### 端口冲突

如果默认端口被占用，可以在配置文件中修改：

```toml
[server]
port = 9090  # HTTP API 端口

[p2p]
tcp_port = 9100  # P2P TCP 端口
```

### 数据库权限问题

确保数据库目录有写权限：

```bash
# macOS/Linux
chmod 755 ~/Library/Application\ Support/porta
```

## 相关文档

- [日志系统改进文档](./logging-improvements.md)
- [DialFailure 错误分析](./dial-failure-analysis.md)
- [系统测试指南](../scripts/README.md)
