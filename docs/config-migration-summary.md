# 配置系统迁移总结

## 完成的改进

### 1. 统一配置管理

所有配置现在从 TOML 文件加载，而不是从环境变量或硬编码值。

**配置文件位置**:
- 服务器模式: 用户配置目录下的 `porta/config.toml`
  - macOS: `~/Library/Application Support/porta/config.toml`
  - Linux: `~/.config/porta/config.toml`
  - Windows: `%APPDATA%\Porta\config.toml`
- 桌面应用: Tauri 应用数据目录

### 2. 数据库路径优化

数据库文件现在默认存储在用户数据目录：
- macOS: `~/Library/Application Support/porta/porta.db`
- Linux: `~/.local/share/porta/porta.db`
- Windows: `%LOCALAPPDATA%\porta\porta.db`

### 3. 配置文件自动创建

首次运行时，如果配置文件不存在，会自动创建默认配置并写入文件。

### 4. 命令行参数支持

```bash
# 指定自定义配置文件
porta-server --config /path/to/config.toml

# 覆盖特定配置项
porta-server --port 9090 --log-level debug

# 打印当前配置
porta-server --print-config

# 验证配置文件
porta-server --validate
```

### 5. 日志增强

配置加载过程中会记录详细日志：
```
INFO Creating default config at: ~/Library/Application Support/porta/config.toml
INFO Loading config from: ~/Library/Application Support/porta/config.toml
INFO Config file: ~/Library/Application Support/porta/config.toml
INFO Database: ~/Library/Application Support/porta/porta.db
```

## 代码变更

### server/src/config.rs

```rust
// 新增方法
pub fn default_config_dir() -> Result<PathBuf>
pub fn default_config_path() -> Result<PathBuf>

// 更新方法
pub fn load_or_create_default<P: AsRef<Path>>(path: P) -> Result<(Self, bool)>

// 默认路径使用 dirs crate
fn default_db_path() -> String {
    if let Some(data_dir) = dirs::data_local_dir() {
        // Use user data directory
    }
}
```

### server/src/main.rs

```rust
// Args 结构更新
struct Args {
    /// 配置文件路径现在是 Option，使用默认路径如果未指定
    #[arg(short, long)]
    config: Option<PathBuf>,
    // ...
}

// 配置加载逻辑
let config_path = if let Some(path) = args.config {
    path
} else {
    Config::default_config_path()?
};
```

### src-tauri/src/main.rs

```rust
// 设置数据库路径和 P2P 端口
std::env::set_var("PORTA_DB", db_path.to_string_lossy().to_string());
std::env::set_var("PORTA_ROLE", "edge");
std::env::set_var("PORTA_P2P_TCP_PORT", "9000");

tracing::info!("Database path: {}", db_path.display());
tracing::info!("Node role: edge");
```

## 测试结果

### 配置系统测试 ✅

```bash
# 配置文件自动创建
$ porta-server --print-config
Default config created at: ~/Library/Application Support/porta/config.toml

# 配置正确加载
[server]
listen_addr = "0.0.0.0"
port = 8090

[node]
name = "Porta Server"
role = "edge"

[database]
path = "/Users/user/Library/Application Support/porta/porta.db"

[p2p]
tcp_port = 9000
quic_port = 9001
mdns_enable = true
dht_enable = true
external_addrs = []

[logging]
level = "info"
format = "compact"
```

### 节点启动测试 ✅

- CommunityNode 启动成功，P2P 端口 9000
- EdgeNode 启动成功，P2P 端口 9000
- 社区添加成功
- 配置正确加载并应用

### 社区连接测试 ❌

**问题**: 连接失败，DialFailure

**原因**: 
1. 两个节点使用相同的 P2P 端口（9000）
2. 在同一台机器上测试时，第二个节点无法绑定到已被占用的端口
3. 虽然节点启动了，但实际 P2P 监听可能失败

**日志证据**:
```
EdgeNode: [P2P] Listening on: /ip4/0.0.0.0/tcp/9000
CommunityNode: [P2P] Listening on: /ip4/0.0.0.0/tcp/9000
```

## 已知问题

### 1. 端口冲突

**问题描述**: 当在同一台机器上运行多个节点实例时，它们会尝试绑定相同的 P2P 端口。

**解决方案**:
- 为不同节点创建不同的配置文件
- 在配置文件中指定不同的 P2P 端口

示例配置:

**community-node.toml**:
```toml
[server]
port = 8091

[node]
role = "community"

[p2p]
tcp_port = 9010
quic_port = 9011
```

**edge-node.toml**:
```toml
[server]
port = 8090

[node]
role = "edge"

[p2p]
tcp_port = 9000
quic_port = 9001
```

**启动方式**:
```bash
# 社区节点
porta-server --config community-node.toml

# 边缘节点
porta-server --config edge-node.toml
```

### 2. 环境变量传递

目前配置通过环境变量传递给后端（PORTA_DB, PORTA_ROLE等）。

**改进方向**: 
- 将配置对象直接传递给后端
- 减少环境变量依赖

## 文档

新增文档：
- `docs/configuration-guide.md` - 完整配置指南
- `docs/config-migration-summary.md` - 本文件
- `scripts/test-config-and-join.sh` - 配置和连接测试脚本

更新文档：
- `docs/logging-improvements.md` - 日志改进说明
- `docs/dial-failure-analysis.md` - DialFailure 错误分析

## 下一步

1. 修复端口冲突问题
   - 检查端口是否已被占用
   - 提供更好的错误提示
   - 支持自动分配可用端口

2. 改进配置传递
   - 直接传递配置对象给后端
   - 减少环境变量使用

3. 完善测试
   - 多节点测试脚本使用不同端口
   - 端到端测试自动化

## 相关命令

```bash
# 查看配置
porta-server --print-config

# 验证配置
porta-server --validate

# 使用自定义配置
porta-server --config /path/to/config.toml

# 运行测试
./scripts/test-config-and-join.sh

# 清理配置（重置）
rm ~/Library/Application\ Support/porta/config.toml
rm ~/Library/Application\ Support/porta/porta.db
```
