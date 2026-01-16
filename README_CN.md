# Porta

Porta 是一个基于 [LibP2P](https://libp2p.io/) 的去中心化 P2P 网络应用，提供安全服务隧道、服务发现与订阅，并同时支持桌面 GUI 和无界面服务端模式。

[English](README.md)

## 功能特性

- 服务发布（HTTP/HTTPS/TCP/WebSocket）
- 服务发现与订阅
- Noise 加密通道
- 多跳中继路由
- 可选 SOCKS5/HTTP 代理
- 社区管理
- 桌面 GUI + 服务端模式

## 安装

### 桌面应用

| 平台 | 下载链接 |
|------|----------|
| macOS (Apple Silicon) | [Porta.dmg](https://github.com/porta-app/porta/releases/latest) |
| macOS (Intel) | [Porta.dmg](https://github.com/porta-app/porta/releases/latest) |
| Windows | [Porta.msi](https://github.com/porta-app/porta/releases/latest) |
| Linux | [Porta.AppImage](https://github.com/porta-app/porta/releases/latest) |

### 服务端二进制

```bash
curl -LO https://github.com/porta-app/porta/releases/latest/download/porta-server-linux-x64.tar.gz
tar xzf porta-server-linux-x64.tar.gz
./porta-server --config porta.toml
```

### 安装为 systemd 服务（Linux）

```bash
curl -LO https://github.com/porta-app/porta/releases/latest/download/porta-service-files.tar.gz
tar xzf porta-service-files.tar.gz
cd porta-service
sudo ./install.sh
```

## 快速开始（服务端）

```toml
# porta.toml
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "我的服务器"
role = "edge"

[database]
path = "/var/lib/porta/data.db"

[p2p]
tcp_port = 9000
quic_port = 9001
```

```bash
porta-server --config porta.toml
```

## 配置说明

| 配置节 | 配置项 | 默认值 | 说明 |
|--------|--------|--------|------|
| `server` | `listen_addr` | `0.0.0.0` | HTTP API 监听地址 |
| `server` | `port` | `8090` | HTTP API 端口 |
| `node` | `name` | `Porta Server` | 节点显示名称 |
| `node` | `role` | `edge` | 节点角色 |
| `database` | `path` | `porta.db` | SQLite 路径 |
| `p2p` | `tcp_port` | `9000` | TCP 端口 |
| `p2p` | `quic_port` | `9001` | QUIC 端口 |
| `p2p` | `mdns_enable` | `true` | 启用 mDNS |
| `p2p` | `dht_enable` | `true` | 启用 DHT |
| `logging` | `level` | `info` | 日志级别 |

## 命令行参数

```
porta-server [选项]

选项:
  -c, --config <文件>    配置文件路径 [默认: porta.toml]
      --listen <地址>    覆盖监听地址
  -p, --port <端口>      覆盖监听端口
      --log-level <级别> 覆盖日志级别
      --print-config     打印配置并退出
      --validate         验证配置并退出
```

## 开发与测试

```bash
npm test
npm run test:system
```

## 许可证

MIT，详见 [LICENSE](LICENSE)。
