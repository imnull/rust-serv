# WebAssembly 插件系统 - 架构设计

## 概述

rust-serv 插件系统基于 WebAssembly (Wasm) 实现，支持运行时热插拔、安全隔离和高性能执行。

## 设计目标

### 核心目标
1. **热插拔** - 运行时加载/卸载插件，无需重启服务
2. **安全性** - 插件在沙箱中执行，无法访问宿主敏感资源
3. **高性能** - 插件执行开销 < 100µs，接近原生性能
4. **易用性** - 提供简洁的 Rust SDK，开发者友好
5. **可扩展** - 支持多种插件类型和丰富的 API

### 非功能性需求
- 插件加载时间 < 10ms
- 内存占用 1-5MB per plugin
- 插件崩溃不影响主服务
- 支持插件版本管理和依赖解析

## 架构层次

```
┌─────────────────────────────────────────────┐
│           rust-serv HTTP Server             │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│          Plugin Middleware Layer            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Plugin 1 │  │ Plugin 2 │  │ Plugin N │  │
│  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│         Plugin Runtime (Wasmtime)           │
│  ┌────────────────────────────────────────┐ │
│  │  Wasm Instance 1  │  Wasm Instance N   │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│           Plugin Host Interface             │
│  - Request/Response API                     │
│  - Configuration API                        │
│  - Logging API                              │
│  - Metrics API                              │
└─────────────────────────────────────────────┘
```

## 核心组件

### 1. PluginManager

插件管理器，负责插件的生命周期管理。

```rust
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    loader: PluginLoader,
    config: PluginConfig,
    watcher: Option<PluginWatcher>,
}

impl PluginManager {
    pub fn load(&mut self, path: &Path) -> Result<String>;
    pub fn unload(&mut self, name: &str) -> Result<()>;
    pub fn reload(&mut self, name: &str) -> Result<()>;
    pub fn execute(&mut self, event: PluginEvent) -> Result<PluginAction>;
    pub fn list(&self) -> Vec<&PluginMetadata>;
}
```

**职责：**
- 插件加载、卸载、重载
- 插件状态管理
- 执行顺序控制（优先级）
- 错误隔离和恢复

### 2. PluginLoader

Wasm 插件加载器，负责编译和实例化 Wasm 模块。

```rust
pub struct PluginLoader {
    engine: Engine,
    module_cache: HashMap<String, Module>,
}

impl PluginLoader {
    pub fn load_from_file(&mut self, path: &Path) -> Result<LoadedPlugin>;
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<LoadedPlugin>;
    pub fn validate(&self, bytes: &[u8]) -> Result<PluginMetadata>;
}
```

**职责：**
- Wasm 模块编译
- 模块缓存
- 插件验证（签名、版本、权限）
- 实例化配置

### 3. LoadedPlugin

已加载的插件实例。

```rust
pub struct LoadedPlugin {
    metadata: PluginMetadata,
    instance: Instance,
    store: Store<PluginState>,
    config: Value,
    stats: PluginStats,
}

pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub priority: u32,
    pub permissions: Vec<Permission>,
}

pub struct PluginStats {
    pub load_time: Instant,
    pub request_count: u64,
    pub error_count: u64,
    pub avg_latency_us: f64,
}
```

### 4. PluginWatcher

文件监视器，监听插件目录变化，自动重载插件。

```rust
pub struct PluginWatcher {
    watcher: RecommendedWatcher,
    manager: Arc<Mutex<PluginManager>>,
}

impl PluginWatcher {
    pub fn watch(&mut self, dir: &Path) -> Result<()>;
    pub fn unwatch(&mut self, dir: &Path) -> Result<()>;
}
```

**支持的文件事件：**
- Create - 新插件文件
- Modify - 插件更新
- Remove - 插件删除

### 5. PluginMiddleware

Tower 中间件层，拦截 HTTP 请求并执行插件。

```rust
pub struct PluginMiddleware {
    manager: Arc<Mutex<PluginManager>>,
}

impl<S> Service<Request<Body>> for PluginMiddleware<S> {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request<Body>) -> Self::Future;
}
```

## 插件生命周期

### 1. 加载流程

```
用户请求加载插件
    ↓
PluginLoader 验证 Wasm 模块
    ↓
提取 PluginMetadata
    ↓
检查依赖和权限
    ↓
编译 Wasm 为 Module
    ↓
创建 Wasm Instance
    ↓
调用 plugin_init()
    ↓
注册到 PluginManager
    ↓
返回插件 ID
```

### 2. 执行流程

```
HTTP 请求到达
    ↓
PluginMiddleware 拦截
    ↓
PluginManager 按优先级排序插件
    ↓
For each plugin:
    调用 plugin_on_request()
    ↓
    返回 PluginAction
    ↓
    If Action == Intercept:
        返回响应，终止链
    Else If Action == ModifyRequest:
        修改请求
    Else:
        继续下一个插件
    ↓
传递到下游 handler
    ↓
响应返回时调用 plugin_on_response()
```

### 3. 卸载流程

```
用户请求卸载插件
    ↓
调用 plugin_on_unload()
    ↓
释放插件资源
    ↓
从 PluginManager 移除
    ↓
Wasm Instance 被垃圾回收
```

## 插件通信

### Host → Plugin

通过 Wasm 导出函数调用：

```rust
// 插件必须实现的函数
#[no_mangle]
pub extern "C" fn plugin_init(config_ptr: i32, config_len: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_request(req_ptr: i32, req_len: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_response(res_ptr: i32, res_len: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_unload() -> i32;
```

### Plugin → Host

通过 Wasm 导入函数调用：

```rust
// Host 提供给插件的函数
#[link(wasm_import_module = "host")]
extern "C" {
    fn host_log(level: i32, msg_ptr: i32, msg_len: i32);
    fn host_get_config(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32;
    fn host_set_header(name_ptr: i32, name_len: i32, val_ptr: i32, val_len: i32);
    fn host_metrics_counter(name_ptr: i32, name_len: i32, value: f64);
    fn host_metrics_gauge(name_ptr: i32, name_len: i32, value: f64);
}
```

## 数据传递

### 序列化格式

使用 JSON 进行跨边界通信，未来可考虑更高效的格式（MessagePack、CBOR）。

```rust
// Request 结构
#[derive(Serialize, Deserialize)]
pub struct PluginRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

// Response 结构
#[derive(Serialize, Deserialize)]
pub struct PluginResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

// Action 结构
#[derive(Serialize, Deserialize)]
pub enum PluginAction {
    Continue,
    Intercept(PluginResponse),
    ModifyRequest(PluginRequest),
    ModifyResponse(PluginResponse),
    Error(String),
}
```

## 安全模型

### 1. Capability-based Security

插件必须声明所需权限：

```toml
[permissions]
# 网络访问
network = true

# 文件系统（只读）
fs_read = ["/etc/whitelist.txt"]

# 环境变量
env = ["API_KEY"]

# 系统调用
syscalls = ["clock_gettime"]
```

### 2. 资源限制

- **内存限制**: 每个 Wasm 实例最多 16MB
- **执行时间**: 单次调用最多 100ms
- **堆栈深度**: 最多 1000 层
- **文件描述符**: 不允许直接访问

### 3. 隔离机制

- Wasm 实例之间完全隔离
- 插件无法直接访问宿主内存
- 所有通信通过序列化进行
- 插件崩溃不会影响主进程

## 配置系统

### 主配置文件

```toml
[plugins]
enabled = true
directory = "./plugins"
auto_reload = true
max_plugins = 100

[[plugins.plugin]]
name = "rate-limiter"
path = "./plugins/rate_limiter.wasm"
enabled = true
priority = 100
config = { rpm = 1000, burst = 100 }

[[plugins.plugin]]
name = "custom-auth"
path = "./plugins/auth.wasm"
enabled = false
priority = 50
config = { api_key = "${API_KEY}" }
```

### 运行时 API

```rust
// 管理接口
POST /_plugins/load
{
  "name": "my-plugin",
  "path": "/path/to/plugin.wasm",
  "config": {}
}

POST /_plugins/unload
{
  "name": "my-plugin"
}

POST /_plugins/reload
{
  "name": "my-plugin"
}

GET /_plugins
GET /_plugins/{name}/stats
```

## 性能优化

### 1. 模块缓存

编译后的 Wasm Module 缓存到内存，避免重复编译。

### 2. 预编译

启动时预编译常用插件，减少首次请求延迟。

### 3. 批量执行

多个插件的 `on_request` 可以批量执行，减少序列化开销。

### 4. 惰性加载

仅在需要时才实例化插件，节省内存。

## 错误处理

### 插件错误

- **超时**: 插件执行超时，记录错误并继续
- **崩溃**: Wasm 运行时错误，隔离插件并降级
- **资源耗尽**: 达到限制，拒绝新插件加载

### 降级策略

- 插件失败时返回 fallback 响应
- 可配置跳过故障插件
- 自动禁用频繁出错的插件

## 监控和调试

### 指标

```
plugin_load_duration_seconds
plugin_request_duration_seconds
plugin_error_total
plugin_memory_bytes
plugin_active_count
```

### 日志

```
[INFO] plugin loaded: rate-limiter v1.0.0
[DEBUG] plugin rate-limiter executed in 23µs
[WARN] plugin auth timeout after 100ms
[ERROR] plugin rate-limiter crashed: wasm trap
```

### 调试接口

```rust
GET /_plugins/{name}/debug
{
  "stack_trace": "...",
  "memory_usage": 2048576,
  "last_error": "...",
  "execution_log": [...]
}
```

## 扩展性

### 未来增强

1. **插件市场** - 类似 npm/crates.io 的插件仓库
2. **插件热更新** - 自动检测并更新插件
3. **分布式插件** - 支持远程插件（gRPC/HTTP）
4. **插件编排** - 多插件协同工作流
5. **插件依赖** - 插件间依赖管理

### 插件类型扩展

- **Transformer Plugins** - 转换请求/响应
- **Auth Plugins** - 认证授权
- **Metrics Plugins** - 自定义监控
- **Storage Plugins** - 自定义存储后端
- **Cache Plugins** - 自定义缓存策略

## 实现计划

### Phase 1: 核心框架 (Week 1-2)
- [ ] Plugin trait 定义
- [ ] PluginLoader 实现
- [ ] PluginManager 实现
- [ ] 基础序列化

### Phase 2: SDK 和示例 (Week 3)
- [ ] rust-serv-plugin crate
- [ ] 插件宏
- [ ] 示例插件（rate-limiter, auth）
- [ ] 开发文档

### Phase 3: 集成和测试 (Week 4)
- [ ] PluginMiddleware 实现
- [ ] 配置系统集成
- [ ] 测试覆盖
- [ ] 性能优化

### Phase 4: 生产化 (Week 5-6)
- [ ] 错误处理
- [ ] 监控指标
- [ ] 管理接口
- [ ] 文档完善

## 参考实现

- [Extism](https://extism.org/) - 通用插件系统
- [Wasmtime](https://wasmtime.dev/) - Wasm 运行时
- [Proxy-Wasm](https://github.com/proxy-wasm) - Envoy 插件标准
