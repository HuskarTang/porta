# 日志系统改进总结

## 改进内容

### 1. 添加文件和行号到日志输出

**修改文件**: `server/src/main.rs`

在所有日志格式（JSON、Pretty、Compact）中添加了：
- `with_file(true)` - 显示源文件路径
- `with_line_number(true)` - 显示行号
- `with_target(true)` - 显示模块路径

**示例输出**:
```
[2026-01-17T15:30:00.123Z INFO  porta_backend::app] [app.rs:332] [社区连接] 开始连接社区: id=community-xxx
```

### 2. 完善连接流程日志

**修改文件**: 
- `backend/src/app.rs` - 社区连接逻辑
- `backend/src/p2p/node.rs` - P2P 节点处理

**添加的日志标签**:
- `[社区连接]` - 社区连接相关操作
- `[P2P]` - P2P 网络层操作
- `[API]` - API 请求处理

**记录的步骤**:
1. 开始连接社区
2. 查找社区信息
3. 解析 multiaddr
4. 开始拨号
5. 拨号成功/失败
6. 发送 Hello 请求
7. 收到 Hello 响应
8. 连接建立/关闭事件

### 3. 改进错误消息

**修改文件**: `backend/src/p2p/node.rs`

将技术错误转换为用户友好的消息：

| 原始错误 | 用户友好消息 |
|---------|------------|
| `DialFailure` | "无法连接到目标节点，请检查网络连接和节点地址" |
| `Timeout` | "请求超时，目标节点可能无响应" |
| `ConnectionClosed` | "连接已关闭" |
| `UnsupportedProtocols` | "目标节点不支持请求的协议" |
| `Io(_)` | "IO 错误: {详细错误}" |

### 4. 连接事件日志

**修改文件**: `backend/src/p2p/node.rs`

添加了对以下 SwarmEvent 的日志记录：
- `ConnectionEstablished` - 连接建立
- `ConnectionClosed` - 连接关闭
- `NewListenAddr` - 新监听地址

## 使用方法

### 查看日志

**服务器模式**:
```bash
# 查看实时日志
tail -f /path/to/logfile

# 或使用 systemd
journalctl -u porta-server -f
```

**桌面模式**:
日志输出到控制台，可以通过开发者工具查看。

### 日志级别

通过配置文件或环境变量设置：

```toml
[logging]
level = "info"  # debug, info, warn, error
format = "compact"  # compact, pretty, json
```

或环境变量：
```bash
RUST_LOG=porta_backend=debug,porta_server=info
```

### 过滤日志

使用标签过滤：
```bash
# 只看社区连接相关
grep "\[社区连接\]" logfile

# 只看 P2P 相关
grep "\[P2P\]" logfile

# 只看错误
grep "ERROR" logfile
```

## 调试 DialFailure 错误

当遇到 `DialFailure` 错误时，查看日志中的以下信息：

1. **社区信息**:
   ```
   [社区连接] 查找社区信息: id=xxx
   [社区连接] 解析 multiaddr: id=xxx, addr=/ip4/...
   ```

2. **拨号过程**:
   ```
   [P2P] 开始拨号到 peer: ... (地址: /ip4/...)
   [P2P] 拨号失败: 无法连接到 ...
   ```

3. **连接事件**:
   ```
   [P2P] 连接已建立: peer=...
   [P2P] 连接已关闭: peer=..., cause=...
   ```

根据这些日志信息，可以快速定位问题：
- 如果 multiaddr 解析失败 → 检查 multiaddr 格式
- 如果拨号失败 → 检查目标节点是否运行、网络是否可达
- 如果连接建立后立即关闭 → 检查协议兼容性

## 相关文件

- `server/src/main.rs` - 日志初始化
- `backend/src/app.rs` - 社区连接逻辑和日志
- `backend/src/p2p/node.rs` - P2P 节点和连接处理
- `backend/src/routes/community.rs` - API 路由日志
- `docs/dial-failure-analysis.md` - DialFailure 错误分析
