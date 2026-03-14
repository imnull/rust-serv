# rust-serv-plugin 示例插件

本目录包含 rust-serv WebAssembly 插件系统的示例插件。

## 示例列表

### 1. Add Header Plugin (`add_header.rs`)
为所有响应添加自定义 Header。

```toml
[[plugins.load]]
id = "com.example.add-header"
path = "./plugins/add_header.wasm"
priority = 50

[plugins.load.config]
header_name = "X-Powered-By"
header_value = "rust-serv"
```

### 2. Rate Limiter Plugin (`rate_limiter.rs`)
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

### 3. CORS Plugin (`cors.rs`)
跨域资源共享支持。

```toml
[[plugins.load]]
id = "com.example.cors"
path = "./plugins/cors.wasm"
priority = 150

[plugins.load.config]
allowed_origins = ["https://example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization"]
max_age = 86400
allow_credentials = true
```

### 4. JWT Auth Plugin (`jwt_auth.rs`)
JWT 认证验证。

```toml
[[plugins.load]]
id = "com.example.jwt-auth"
path = "./plugins/jwt_auth.wasm"
priority = 300

[plugins.load.config]
secret = "your-jwt-secret"
header_name = "Authorization"
prefix = "Bearer "
excluded_paths = ["/health", "/public"]
```

### 5. URL Rewrite Plugin (`url_rewrite.rs`)
URL 重写和重定向。

```toml
[[plugins.load]]
id = "com.example.url-rewrite"
path = "./plugins/url_rewrite.wasm"
priority = 100

[[plugins.load.config.rules]]
from = "/old-api/*"
to = "/api/v1/*"
type = "rewrite"

[[plugins.load.config.rules]]
from = "/docs"
to = "/documentation"
type = "redirect"
status = 301
```

### 6. IP Blacklist Plugin (`ip_blacklist.rs`)
IP 黑名单阻止。

```toml
[[plugins.load]]
id = "com.example.ip-blacklist"
path = "./plugins/ip_blacklist.wasm"
priority = 310

[plugins.load.config]
blacklist = ["192.168.1.100", "10.0.0.0/24"]
deny_message = "Your IP has been blocked"
return_444 = false
```

### 7. Request Modifier Plugin (`request_modifier.rs`)
请求头修改。

```toml
[[plugins.load]]
id = "com.example.request-modifier"
path = "./plugins/request_modifier.wasm"
priority = 80

[plugins.load.config.add_headers]
"X-Request-ID" = "${REQUEST_ID}"
"X-Real-IP" = "${CLIENT_IP}"

remove_headers = ["X-Internal-Token"]
```

### 8. Cache Control Plugin (`cache_control.rs`)
动态缓存控制。

```toml
[[plugins.load]]
id = "com.example.cache-control"
path = "./plugins/cache_control.wasm"
priority = 90

[plugins.load.config]
default_max_age = 3600

[[plugins.load.config.rules]]
path_pattern = "/api/*"
max_age = 0
no_cache = true

[[plugins.load.config.rules]]
path_pattern = "/static/*"
max_age = 86400
immutable = true
```

### 9. Response Modifier Plugin (`response_modifier.rs`)
响应头修改和安全头添加。

```toml
[[plugins.load]]
id = "com.example.response-modifier"
path = "./plugins/response_modifier.wasm"
priority = 110

[plugins.load.config.add_headers]
"X-Frame-Options" = "DENY"
"X-Content-Type-Options" = "nosniff"

remove_headers = ["X-Powered-By", "Server"]
```

## 编译插件

```bash
# 添加 wasm 目标
rustup target add wasm32-unknown-unknown

# 编译单个插件
cargo build --release --example add_header --target wasm32-unknown-unknown

# 编译后的文件位于
target/wasm32-unknown-unknown/release/examples/add_header.wasm
```

## 开发新插件

1. 创建新的 Rust 项目
2. 添加依赖：`rust-serv-plugin = "0.1.0"`
3. 实现 `Plugin` trait
4. 使用 `export_plugin!` 宏导出
5. 编译为 WebAssembly

### 最小插件示例

```rust
use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::*;

pub struct MyPlugin;

impl Default for MyPlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        // 返回元数据
        &METADATA
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        // 处理请求
        Ok(PluginAction::Continue)
    }
}

static METADATA: PluginMetadata = PluginMetadata {
    id: "com.example.my-plugin".to_string(),
    name: "My Plugin".to_string(),
    version: "1.0.0".to_string(),
    description: "My first plugin".to_string(),
    author: "Your Name".to_string(),
    homepage: None,
    license: "MIT".to_string(),
    min_server_version: "0.3.0".to_string(),
    priority: 100,
    capabilities: vec![],
    permissions: vec![],
};

export_plugin!(MyPlugin);
```

## 优先级建议

| 插件类型 | 建议优先级 | 说明 |
|---------|-----------|------|
| IP 黑名单 | 300+ | 尽早拦截恶意请求 |
| JWT 认证 | 250-300 | 认证应在业务逻辑前 |
| 速率限制 | 200-250 | 限流在认证后 |
| CORS | 150-200 | 跨域处理 |
| 响应修改 | 100-150 | 在生成响应后 |
| 添加 Header | 50-100 | 最后的修饰 |
| 日志 | 10-50 | 记录所有内容 |

## 更多信息

- [插件系统架构](../../docs/plugin-system/ARCHITECTURE.md)
- [插件开发指南](../../docs/plugin-system/README.md)
- [API 文档](https://docs.rs/rust-serv-plugin)
