# 插件系统实现计划

## 总览

基于 WebAssembly 的插件系统实现路线图。

## Phase 1: 核心框架 (Week 1-2)

### 目标
建立插件系统的基础架构

### 任务

#### 1.1 项目结构
- [ ] 创建 `rust-serv-plugin` crate
- [ ] 设置目录结构
- [ ] 配置依赖

```toml
[dependencies]
wasmtime = "15"
wasmtime-wasi = "15"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["sync", "time"] }
notify = "7"  # 文件监听
base64 = "0.22"
```

#### 1.2 核心接口定义
- [ ] `Plugin` trait
- [ ] `PluginMetadata` 结构
- [ ] `PluginConfig` 结构
- [ ] `PluginAction` 枚举
- [ ] `PluginError` 错误类型

文件：`src/plugin/traits.rs`

#### 1.3 Wasm 加载器
- [ ] `PluginLoader` 实现
- [ ] Wasm 模块编译
- [ ] 实例化管理
- [ ] 缓存机制

文件：`src/plugin/loader.rs`

```rust
pub struct PluginLoader {
    engine: wasmtime::Engine,
    module_cache: HashMap<PathBuf, Module>,
}

impl PluginLoader {
    pub fn compile(&mut self, path: &Path) -> Result<Module>;
    pub fn instantiate(&mut self, module: &Module) -> Result<Instance>;
}
```

#### 1.4 插件管理器
- [ ] `PluginManager` 实现
- [ ] 插件加载/卸载
- [ ] 生命周期管理
- [ ] 执行调度

文件：`src/plugin/manager.rs`

```rust
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    loader: PluginLoader,
    config: PluginConfig,
    execution_order: Vec<String>,
}
```

#### 1.5 序列化层
- [ ] Host → Plugin 通信
- [ ] Plugin → Host 通信
- [ ] JSON 序列化
- [ ] Base64 编码

文件：`src/plugin/serde.rs`

### 交付物
- 可编译的核心框架
- 基础单元测试
- 技术文档

## Phase 2: SDK 和示例 (Week 3)

### 目标
提供开发者友好的 SDK 和示例插件

### 任务

#### 2.1 Plugin SDK
- [ ] `rust-serv-plugin` crate 发布
- [ ] 插件宏 `export_plugin!`
- [ ] 辅助函数
- [ ] 文档注释

文件：`plugin-sdk/src/lib.rs`

#### 2.2 Host Functions
- [ ] `host_log()` - 日志
- [ ] `host_get_config()` - 获取配置
- [ ] `host_set_header()` - 设置 Header
- [ ] `host_metrics_*()` - 指标上报

文件：`src/plugin/host.rs`

```rust
pub fn define_host_functions(store: &mut Store<HostState>) -> Result<WasmFunctions> {
    // 定义所有 Host 函数
}
```

#### 2.3 示例插件

##### 示例 1: Add Header Plugin
- [ ] 实现插件
- [ ] 配置示例
- [ ] 测试用例

文件：`plugin-sdk/examples/add_header.rs`

##### 示例 2: Rate Limiter Plugin
- [ ] 实现插件
- [ ] 配置示例
- [ ] 测试用例

文件：`plugin-sdk/examples/rate_limiter.rs`

##### 示例 3: Auth Plugin
- [ ] 实现插件
- [ ] 配置示例
- [ ] 测试用例

文件：`plugin-sdk/examples/auth.rs`

#### 2.4 开发文档
- [ ] 快速开始指南
- [ ] API 文档
- [ ] 最佳实践
- [ ] 常见问题

### 交付物
- 可用的 SDK
- 3+ 示例插件
- 完整开发文档

## Phase 3: 集成和测试 (Week 4)

### 目标
集成到主服务并进行全面测试

### 任务

#### 3.1 PluginMiddleware
- [ ] 实现 Tower 中间件
- [ ] 集成到请求处理链
- [ ] 执行顺序控制
- [ ] 错误处理

文件：`src/middleware/plugin.rs`

```rust
pub struct PluginMiddleware {
    manager: Arc<RwLock<PluginManager>>,
}

impl<S> Service<Request<Body>> for PluginMiddleware<S> {
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // 执行插件链
    }
}
```

#### 3.2 配置系统
- [ ] 插件配置解析
- [ ] 动态重载支持
- [ ] 配置验证

文件：`src/config/plugin.rs`

#### 3.3 文件监听
- [ ] 使用 `notify` crate
- [ ] 自动重载变更的插件
- [ ] 防抖处理

文件：`src/plugin/watcher.rs`

#### 3.4 测试

##### 单元测试
- [ ] PluginLoader 测试
- [ ] PluginManager 测试
- [ ] 序列化测试

##### 集成测试
- [ ] 端到端测试
- [ ] 热插拔测试
- [ ] 并发测试

##### 性能测试
- [ ] 加载时间测试
- [ ] 执行开销测试
- [ ] 内存占用测试

### 交付物
- 集成到主服务
- 80%+ 测试覆盖率
- 性能基准

## Phase 4: 生产化 (Week 5-6)

### 目标
生产环境就绪

### 任务

#### 4.1 错误处理
- [ ] 插件崩溃隔离
- [ ] 超时处理
- [ ] 降级策略
- [ ] 错误恢复

文件：`src/plugin/error.rs`

#### 4.2 监控指标
- [ ] Prometheus 集成
- [ ] 插件级指标
- [ ] 性能监控

```rust
// 新增指标
plugin_load_duration_seconds
plugin_request_duration_seconds
plugin_error_total
plugin_memory_bytes
plugin_active_count
```

#### 4.3 管理接口
- [ ] REST API 实现
- [ ] 插件列表
- [ ] 插件详情
- [ ] 动态加载/卸载

文件：`src/management/plugins.rs`

```rust
// GET /_plugins
// GET /_plugins/{id}
// POST /_plugins/load
// POST /_plugins/{id}/reload
// DELETE /_plugins/{id}
```

#### 4.4 安全加固
- [ ] 权限验证
- [ ] 资源限制
- [ ] 沙箱配置
- [ ] 代码签名（可选）

#### 4.5 文档完善
- [ ] 用户指南
- [ ] 管理员指南
- [ ] 架构文档
- [ ] API 文档

### 交付物
- 生产就绪的插件系统
- 完整文档
- 示例和最佳实践

## Phase 5: 生态建设 (持续)

### 目标
构建插件生态系统

### 任务

#### 5.1 官方插件库
- [ ] rate-limiter
- [ ] basic-auth
- [ ] jwt-auth
- [ ] cors
- [ ] compression
- [ ] cache
- [ ] logging
- [ ] metrics

#### 5.2 插件模板
- [ ] Cookiecutter 模板
- [ ] 快速脚手架
- [ ] CI/CD 示例

#### 5.3 插件市场（未来）
- [ ] 插件仓库
- [ ] 版本管理
- [ ] 依赖解析
- [ ] 搜索功能

## 里程碑

| 阶段 | 完成时间 | 交付物 |
|------|---------|--------|
| Phase 1 | Week 2 | 核心框架 |
| Phase 2 | Week 3 | SDK + 示例 |
| Phase 3 | Week 4 | 集成测试 |
| Phase 4 | Week 6 | 生产就绪 |
| Phase 5 | 持续 | 生态建设 |

## 风险和缓解

### 风险 1: Wasm 性能损耗
- **影响**: 请求处理延迟
- **缓解**: 
  - 使用 AOT 编译
  - 优化序列化
  - 缓存 Wasm 实例

### 风险 2: 插件崩溃影响主服务
- **影响**: 服务不稳定
- **缓解**:
  - 隔离每个插件实例
  - 实现熔断机制
  - 降级策略

### 风险 3: 安全漏洞
- **影响**: 宿主被攻击
- **缓解**:
  - 严格权限控制
  - 资源限制
  - 定期安全审计

## 成功指标

### 性能指标
- 插件加载时间 < 10ms
- 请求处理开销 < 100µs
- 内存占用 < 5MB/plugin

### 质量指标
- 测试覆盖率 > 80%
- 零 P0 级 Bug
- 文档完整度 > 90%

### 生态指标
- 10+ 官方插件
- 50+ 第三方插件（6 个月内）
- 社区活跃度

## 后续增强

### v1.1
- [ ] 插件依赖管理
- [ ] 插件配置 Schema 验证
- [ ] 插件性能分析工具

### v1.2
- [ ] 远程插件支持（gRPC/HTTP）
- [ ] 插件编排工作流
- [ ] 插件测试框架

### v2.0
- [ ] 插件市场
- [ ] AI 辅助插件开发
- [ ] 可视化插件编辑器

## 参考资源

- [Wasmtime 文档](https://docs.wasmtime.dev/)
- [WebAssembly 规范](https://webassembly.org/)
- [Extism 插件系统](https://extism.org/)
- [Proxy-Wasm 标准](https://github.com/proxy-wasm)
