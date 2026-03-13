# rust-serv 插件系统

基于 WebAssembly 的热插拔插件系统。

## 概述

rust-serv 插件系统允许开发者编写自定义插件来扩展服务器功能，而无需修改核心代码或重启服务。

### 核心特性

- ✅ **热插拔** - 运行时加载/卸载插件
- ✅ **安全隔离** - 插件在 Wasm 沙箱中执行
- ✅ **高性能** - 接近原生性能（~10% 损耗）
- ✅ **易开发** - 提供简洁的 Rust SDK
- ✅ **可扩展** - 支持丰富的插件 API

## 快速开始

### 1. 安装依赖

```bash
# 添加 rust-wasm 目标
rustup target add wasm32-unknown-unknown
```

### 2. 创建插件

```rust
use rust_serv_plugin::{Plugin, PluginMetadata, PluginAction, PluginRequest, PluginResponse};

pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn on_request(&mut self, req: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        // 自定义逻辑
        if !req.headers().contains_key("X-API-Key") {
            return Ok(PluginAction::Intercept(
                PluginResponse::new(401).with_body("Unauthorized".to_string())
            ));
        }
        Ok(PluginAction::Continue)
    }
}

rust_serv_plugin::export_plugin!(MyPlugin);
```

### 3. 编译插件

```bash
cargo build --release --target wasm32-unknown-unknown
```

### 4. 配置加载

```toml
# config.toml
[plugins]
enabled = true
directory = "./plugins"

[[plugins.load]]
id = "com.example.my-plugin"
path = "./plugins/my_plugin.wasm"
enabled = true

[plugins.load.config]
api_key_header = "X-API-Key"
```

### 5. 启动服务

```bash
rust-serv config.toml
```

## 文档

- **[架构设计](./ARCHITECTURE.md)** - 系统架构和设计决策
- **[插件标准](./STANDARD.md)** - 插件接口和协议规范
- **[SDK 文档](../../plugin-sdk/)** - 开发 SDK 使用指南

## 示例插件

### 1. Add Header Plugin

为所有响应添加自定义 Header。

```toml
[[plugins.load]]
id = "com.example.add-header"
path = "./plugins/add_header.wasm"

[plugins.load.config]
header_name = "X-Powered-By"
header_value = "rust-serv"
```

### 2. Rate Limiter Plugin

基于 IP 的请求限流。

```toml
[[plugins.load]]
id = "com.example.rate-limiter"
path = "./plugins/rate_limiter.wasm"
priority = 200

[plugins.load.config]
requests_per_minute = 100
burst_size = 20
```

### 3. Auth Plugin

自定义认证逻辑。

```toml
[[plugins.load]]
id = "com.example.auth"
path = "./plugins/auth.wasm"
priority = 300

[plugins.load.config]
api_key_header = "X-API-Key"
whitelist = ["192.168.1.0/24"]
```

## 插件生命周期

```
┌─────────────┐
│   Load      │ on_load()
└─────────────┘
       ↓
┌─────────────┐
│   Request   │ on_request() → Action
└─────────────┘
       ↓
┌─────────────┐
│  Response   │ on_response() → Action
└─────────────┘
       ↓
┌─────────────┐
│   Unload    │ on_unload()
└─────────────┘
```

## 插件动作

插件可以返回以下动作：

| 动作 | 说明 |
|------|------|
| `Continue` | 继续执行下一个插件 |
| `Intercept` | 拦截并返回响应 |
| `ModifyRequest` | 修改请求并继续 |
| `ModifyResponse` | 修改响应并继续 |
| `Error` | 返回错误 |

## 插件优先级

插件按优先级（priority）排序执行：
- 数值越大，优先级越高
- 相同优先级按加载顺序执行
- 建议范围：0-1000

```toml
[[plugins.load]]
id = "auth"        # 先执行（priority = 300）
priority = 300

[[plugins.load]]
id = "rate-limit"  # 后执行（priority = 200）
priority = 200
```

## 性能

### 时间开销

| 操作 | 耗时 |
|------|------|
| 插件加载 | < 10ms |
| 请求处理 | < 100µs |
| 响应处理 | < 100µs |

### 资源限制

| 资源 | 限制 |
|------|------|
| 内存 | 16MB / plugin |
| CPU | 50% / core |
| 超时 | 100ms / request |

## 安全

### 权限控制

插件必须声明所需权限：

```toml
[[plugins.load]]
id = "my-plugin"
permissions = [
    "http-request:api.example.com",
    "file-read:/tmp/*"
]
```

### 沙箱隔离

- 每个 Wasm 插件独立实例
- 无法访问宿主内存
- 无法调用未声明的 Host 函数

## 调试

### 查看插件状态

```bash
# 列出所有插件
curl http://localhost:8080/_plugins

# 查看插件详情
curl http://localhost:8080/_plugins/{plugin_id}

# 插件统计
curl http://localhost:8080/_plugins/{plugin_id}/stats
```

### 动态管理

```bash
# 加载插件
curl -X POST http://localhost:8080/_plugins/load \
  -d '{"id":"new-plugin","path":"./plugins/new.wasm"}'

# 重载插件
curl -X POST http://localhost:8080/_plugins/{plugin_id}/reload

# 卸载插件
curl -X DELETE http://localhost:8080/_plugins/{plugin_id}
```

## 开发指南

### 项目结构

```
my-plugin/
├── Cargo.toml
├── src/
│   └── lib.rs
├── README.md
└── CONFIGURATION.md
```

### Cargo.toml

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
rust-serv-plugin = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = 3
lto = true
```

### 编译

```bash
cargo build --release --target wasm32-unknown-unknown
```

输出：`target/wasm32-unknown-unknown/release/my_plugin.wasm`

## 常见问题

### Q: 插件加载失败？

检查：
1. Wasm 文件路径是否正确
2. 插件元数据是否完整
3. 插件版本是否兼容

### Q: 插件执行超时？

解决方案：
1. 增加超时时间：`timeout_ms = 200`
2. 优化插件代码
3. 减少复杂逻辑

### Q: 如何调试插件？

1. 使用 `host_log()` 输出日志
2. 查看服务器日志
3. 使用调试接口

## 贡献

欢迎贡献插件！

1. Fork 项目
2. 创建插件目录：`plugins/my-plugin/`
3. 添加插件代码和文档
4. 提交 PR

## 许可证

MIT OR Apache-2.0
