# Plugin 标准 (Plugin Standard)

## 概述

本文档定义了 rust-serv 插件的标准接口、协议和规范。

## 插件元数据

每个插件必须包含元数据：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件唯一标识符
    pub id: String,
    
    /// 插件名称
    pub name: String,
    
    /// 插件版本（语义化版本）
    pub version: Version,
    
    /// 插件描述
    pub description: String,
    
    /// 作者信息
    pub author: String,
    
    /// 插件主页
    pub homepage: Option<String>,
    
    /// 许可证
    pub license: String,
    
    /// rust-serv 最低版本要求
    pub min_server_version: Version,
    
    /// 插件优先级（数值越大优先级越高）
    pub priority: i32,
    
    /// 插件能力声明
    pub capabilities: Vec<Capability>,
    
    /// 插件依赖
    pub dependencies: Vec<PluginDependency>,
    
    /// 所需权限
    pub permissions: Vec<Permission>,
    
    /// 配置 Schema
    pub config_schema: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_req: VersionReq,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Capability {
    /// 可以修改请求
    ModifyRequest,
    
    /// 可以修改响应
    ModifyResponse,
    
    /// 可以拦截请求
    InterceptRequest,
    
    /// 可以访问配置
    AccessConfig,
    
    /// 可以记录日志
    Logging,
    
    /// 可以上报指标
    Metrics,
    
    /// 可以访问外部资源
    NetworkAccess,
    
    /// 可以访问文件系统
    FileSystemAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    /// 读取环境变量
    ReadEnv,
    
    /// 发起 HTTP 请求
    HttpRequest { allowed_hosts: Vec<String> },
    
    /// 读取文件
    FileRead { allowed_paths: Vec<String> },
    
    /// 写入文件
    FileWrite { allowed_paths: Vec<String> },
    
    /// 访问网络
    NetworkAccess { allowed_ports: Vec<u16> },
}
```

**配置示例：**

```json
{
  "id": "com.example.rate-limiter",
  "name": "Rate Limiter",
  "version": "1.0.0",
  "description": "Advanced rate limiting with multiple strategies",
  "author": "Your Name <you@example.com>",
  "license": "MIT",
  "min_server_version": "0.3.0",
  "priority": 100,
  "capabilities": [
    "ModifyRequest",
    "InterceptRequest"
  ],
  "permissions": [
    {
      "HttpRequest": {
        "allowed_hosts": ["api.example.com"]
      }
    }
  ]
}
```

## 插件配置

### 配置文件格式

插件配置使用 TOML 格式：

```toml
# 插件全局配置
[plugins]
enabled = true
directory = "./plugins"
auto_reload = true
max_plugins = 100
default_timeout_ms = 100

# 加载特定插件
[[plugins.load]]
id = "com.example.rate-limiter"
path = "./plugins/rate_limiter.wasm"
enabled = true
priority = 100

# 插件自定义配置
[plugins.load.config]
requests_per_minute = 100
burst_size = 20
strategy = "sliding_window"

[[plugins.load]]
id = "com.example.auth"
path = "./plugins/auth.wasm"
enabled = true
priority = 200

[plugins.load.config]
api_key_header = "X-API-Key"
whitelist = ["192.168.1.0/24"]
```

### 动态配置 API

```rust
// 通过 HTTP API 动态管理插件
POST /_plugins/load
{
  "id": "com.example.custom",
  "path": "./plugins/custom.wasm",
  "config": { ... }
}

DELETE /_plugins/{plugin_id}

POST /_plugins/{plugin_id}/reload

GET /_plugins/{plugin_id}/config

PUT /_plugins/{plugin_id}/config
{
  "new_config": { ... }
}
```

## 插件生命周期

### 生命周期钩子

插件必须实现以下钩子：

```rust
pub trait Plugin {
    /// 插件元数据
    fn metadata(&self) -> &PluginMetadata;
    
    /// 插件加载时调用
    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    
    /// 配置变更时调用
    fn on_config_change(&mut self, new_config: &PluginConfig) -> Result<(), PluginError>;
    
    /// 请求到达时调用（可能被多次调用）
    fn on_request(&mut self, req: &mut PluginRequest) -> Result<PluginAction, PluginError>;
    
    /// 响应返回前调用（可能被多次调用）
    fn on_response(&mut self, res: &mut PluginResponse) -> Result<PluginAction, PluginError>;
    
    /// 插件卸载前调用
    fn on_unload(&mut self) -> Result<(), PluginError>;
}
```

### 执行顺序

1. **加载阶段**
   ```
   Server Start → Load Config → For each plugin:
     on_load() → Initialize plugin
   ```

2. **请求处理阶段**
   ```
   HTTP Request → For each plugin (by priority):
     on_request() → Decide action
   
   If action == Continue:
     Handle request → For each plugin (by priority):
       on_response() → Decide action
   
   Return response
   ```

3. **卸载阶段**
   ```
   Server Shutdown → For each plugin:
     on_unload() → Cleanup resources
   ```

### 插件动作

```rust
#[derive(Debug, Clone)]
pub enum PluginAction {
    /// 继续执行下一个插件
    Continue,
    
    /// 拦截并返回响应
    Intercept(PluginResponse),
    
    /// 修改请求并继续
    ModifyRequest(PluginRequest),
    
    /// 修改响应并继续
    ModifyResponse(PluginResponse),
    
    /// 发生错误，返回错误响应
    Error(PluginError),
}
```

## 通信协议

### Host → Plugin 通信

#### 初始化

```c
// 插件初始化
int32_t plugin_init(
    int32_t config_ptr,    // JSON 配置字符串指针
    int32_t config_len,    // 配置长度
    int32_t meta_ptr,      // 输出：元数据指针
    int32_t meta_len_ptr   // 输出：元数据长度指针
);
// 返回：0=成功，非0=错误码
```

#### 处理请求

```c
// 处理 HTTP 请求
int32_t plugin_on_request(
    int32_t req_ptr,       // JSON 请求字符串指针
    int32_t req_len,       // 请求长度
    int32_t action_ptr,    // 输出：动作指针
    int32_t action_len_ptr // 输出：动作长度指针
);
// 返回：0=成功，非0=错误码
```

#### 处理响应

```c
// 处理 HTTP 响应
int32_t plugin_on_response(
    int32_t res_ptr,       // JSON 响应字符串指针
    int32_t res_len,       // 响应长度
    int32_t action_ptr,    // 输出：动作指针
    int32_t action_len_ptr // 输出：动作长度指针
);
// 返回：0=成功，非0=错误码
```

### Plugin → Host 通信

```c
// 日志记录
void host_log(
    int32_t level,         // 日志级别：0=debug, 1=info, 2=warn, 3=error
    int32_t msg_ptr,       // 日志消息指针
    int32_t msg_len        // 消息长度
);

// 获取配置值
int32_t host_get_config(
    int32_t key_ptr,       // 配置键指针
    int32_t key_len,       // 键长度
    int32_t val_ptr,       // 输出：值指针
    int32_t val_len_ptr    // 输出：值长度指针
);
// 返回：0=成功，1=不存在，非0=错误

// 设置响应头
void host_set_header(
    int32_t name_ptr,      // Header 名指针
    int32_t name_len,      // 名长度
    int32_t val_ptr,       // Header 值指针
    int32_t val_len        // 值长度
);

// 上报 Counter 指标
void host_metrics_counter(
    int32_t name_ptr,      // 指标名指针
    int32_t name_len,      // 名长度
    double value           // 值
);

// 上报 Gauge 指标
void host_metrics_gauge(
    int32_t name_ptr,      // 指标名指针
    int32_t name_len,      // 名长度
    double value           // 值
);
```

## 插件数据结构

### PluginRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// HTTP 方法
    pub method: String,
    
    /// 请求路径
    pub path: String,
    
    /// 查询参数
    pub query: HashMap<String, String>,
    
    /// 请求头
    pub headers: HashMap<String, String>,
    
    /// 请求体（Base64 编码）
    pub body: Option<String>,
    
    /// 客户端 IP
    pub client_ip: String,
    
    /// 请求 ID
    pub request_id: String,
    
    /// 协议版本
    pub version: String,
    
    /// Host 头
    pub host: String,
    
    /// 扩展字段
    pub extensions: HashMap<String, Value>,
}
```

### PluginResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// HTTP 状态码
    pub status: u16,
    
    /// 响应头
    pub headers: HashMap<String, String>,
    
    /// 响应体（Base64 编码）
    pub body: Option<String>,
    
    /// 扩展字段
    pub extensions: HashMap<String, Value>,
}
```

## 错误处理

### 错误码

```rust
pub enum PluginErrorCode {
    Success = 0,
    
    // 初始化错误 (1000-1999)
    InitFailed = 1000,
    InvalidConfig = 1001,
    MissingDependency = 1002,
    
    // 执行错误 (2000-2999)
    ExecutionError = 2000,
    Timeout = 2001,
    MemoryLimitExceeded = 2002,
    
    // 权限错误 (3000-3999)
    PermissionDenied = 3000,
    
    // 协议错误 (4000-4999)
    InvalidInput = 4000,
    SerializationError = 4001,
}
```

### 错误响应

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    pub code: PluginErrorCode,
    pub message: String,
    pub details: Option<Value>,
}
```

## 性能规范

### 时间限制

| 操作 | 超时时间 | 说明 |
|------|---------|------|
| `plugin_init` | 1s | 插件初始化 |
| `on_request` | 100ms | 处理单个请求 |
| `on_response` | 100ms | 处理单个响应 |
| `on_load` | 10s | 加载插件 |
| `on_unload` | 1s | 卸载插件 |

### 资源限制

| 资源 | 限制 | 说明 |
|------|------|------|
| 内存 | 16MB | 每个 Wasm 实例 |
| CPU | 50% | 单核百分比 |
| 文件描述符 | 10 | 每个插件 |
| 网络连接 | 5 | 每个插件 |

## 安全规范

### 1. 沙箱隔离

- 每个 Wasm 插件运行在独立实例
- 无法访问宿主内存
- 无法调用未声明的 Host 函数

### 2. 权限控制

- 插件必须声明所需权限
- Host 运行时验证权限
- 越权操作将被拒绝

### 3. 资源限制

- 内存、CPU、网络限制
- 防止资源耗尽攻击
- 自动终止异常插件

### 4. 代码签名（可选）

```toml
[plugins.verification]
enabled = true
public_key = "path/to/pubkey.pem"
```

## 版本兼容性

### 语义化版本

插件版本遵循 [SemVer 2.0.0](https://semver.org/)：
- MAJOR: 不兼容的 API 变更
- MINOR: 向后兼容的功能增加
- PATCH: 向后兼容的问题修复

### 兼容性检查

```rust
pub fn is_compatible(
    plugin_version: &Version,
    server_version: &Version,
) -> bool {
    // 检查最低版本要求
    // 检查 API 兼容性
}
```

## 测试规范

### 单元测试

每个插件必须包含单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_on_request() {
        let mut plugin = MyPlugin::new();
        let mut req = PluginRequest::default();
        let action = plugin.on_request(&mut req).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }
}
```

### 集成测试

```bash
# 使用 rust-serv 插件测试框架
cargo test --plugin ./target/wasm32-unknown-unknown/release/my_plugin.wasm
```

## 文档规范

每个插件必须包含：

1. **README.md** - 插件说明
2. **CONFIGURATION.md** - 配置文档
3. **API.md** - API 文档
4. **EXAMPLES.md** - 使用示例
5. **CHANGELOG.md** - 变更日志

## 示例插件

见 `/plugin-sdk/examples/` 目录。

## 版本历史

- **v1.0.0** (2026-03-13) - 初始标准定义
