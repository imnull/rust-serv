# Rust HTTP 静态服务器

一个高性能、安全、功能丰富的 Rust HTTP 静态文件服务器，采用测试驱动开发（TDD）范式开发，测试覆盖率 95%+。

## 特性

### 核心功能
- ✅ **静态文件服务** - 高效提供静态文件
- ✅ **目录索引** - 自动生成目录列表
- ✅ **路径安全** - 防止路径遍历攻击
- ✅ **MIME 类型检测** - 自动检测文件类型
- ✅ **配置系统** - 灵活的 TOML 配置
- ✅ **HTTP 压缩** - 支持 Gzip 和 Brotli 压缩
- ✅ **HTTPS/TLS 支持** - 安全加密连接
- ✅ **HTTP/2 支持** - 现代协议支持
- ✅ **WebSocket 支持** - 实时通信
- ✅ **CORS 跨域** - 跨域资源共享
- ✅ **Range 请求** - 部分内容传输

### 性能优化
- ✅ **内存缓存系统** - LRU 策略、TTL 过期、线程安全、缓存统计
- ✅ **ETag 缓存** - 304 Not Modified 响应
- ✅ **带宽限速** - Token Bucket 算法，全局/单 IP 限速

### 可观测性
- ✅ **Prometheus 指标** - Counter/Gauge/Histogram，监控 QPS、响应时间、错误率
- ✅ **访问日志持久化** - Common/Combined/JSON 格式
- ✅ **结构化日志** - 基于 tracing 的日志系统

### 安全特性
- ✅ **基础认证 (Basic Auth)** - HTTP Basic Auth，路径保护
- ✅ **请求频率限制** - 防止 DDoS 攻击
- ✅ **IP 访问控制** - 白名单/黑名单
- ✅ **安全头** - XSS、CSRF 防护

### 扩展功能
- ✅ **虚拟主机/多站点** - 基于 Host 头的站点路由
- ✅ **反向代理** - 路径路由到后端服务，前缀剥离
- ✅ **文件上传 (PUT/POST)** - multipart 解析，扩展名/大小限制
- ✅ **自定义错误页面** - 美观模板，支持 400-504 错误码
- ✅ **配置热重载** - 零停机配置更新

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/imnull/rust-serv.git
cd rust-serv

# 编译
cargo build --release
```

### 使用

```bash
# 使用默认配置运行
cargo run

# 使用自定义配置
cargo run -- path/to/config.toml
```

### 配置

创建 `config.toml`:

```toml
# 基础配置
port = 8080
host = "0.0.0.0"
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"

# TLS 配置
[tls]
enabled = true
cert_path = "./certs/cert.pem"
key_path = "./certs/key.pem"

# 内存缓存
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
ttl_secs = 3600

# Prometheus 指标
[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"

# 访问日志
[access_log]
enabled = true
path = "./logs/access.log"
format = "combined"

# 带宽限速
[throttle]
enabled = true
global_limit_bps = 10485760  # 10 MB/s
per_ip_limit_bps = 1048576   # 1 MB/s

# 虚拟主机
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"

# 反向代理
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true

# 基础认证
[[auth]]
path = "/admin"
users = [
    { username = "admin", password_hash = "hashed_password" }
]
```

## 模块文档

### 内存缓存 (memory_cache)

```rust
use rust_serv::{MemoryCache, CacheConfig};

let config = CacheConfig::new()
    .with_max_entries(10000)
    .with_max_size(100 * 1024 * 1024)  // 100 MB
    .with_ttl(3600);  // 1 hour

let cache = MemoryCache::new(config);
let cached = cache.get("/index.html").await;
```

### Prometheus 指标 (metrics)

```rust
use rust_serv::{MetricsCollector, Counter, Gauge, Histogram};

let collector = MetricsCollector::new("rust_serv");
let counter = collector.counter("requests_total");
counter.inc();

// GET /metrics 返回 Prometheus 格式
```

### 带宽限速 (throttle)

```rust
use rust_serv::{ThrottleLimiter, ThrottleConfig};

let config = ThrottleConfig::new()
    .enable()
    .with_global_limit(10 * 1024 * 1024)  // 10 MB/s
    .with_per_ip_limit(1 * 1024 * 1024);  // 1 MB/s per IP

let limiter = ThrottleLimiter::new(config);
let result = limiter.check("192.168.1.1", 1024).await;
```

### 虚拟主机 (vhost)

```rust
use rust_serv::{HostMatcher, VHostConfig};

let mut matcher = HostMatcher::new();
matcher.add_host(VHostConfig::new("blog.example.com", "/var/www/blog"));
matcher.add_host(VHostConfig::new("api.example.com", "/var/www/api"));

let vhost = matcher.match_host("blog.example.com");
```

### 反向代理 (proxy)

```rust
use rust_serv::{ProxyHandler, ProxyConfig};

let mut handler = ProxyHandler::new();
handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));

if handler.should_proxy("/api/users") {
    let target = handler.get_target_url("/api/users");
    // -> "http://localhost:3000/users"
}
```

### 文件上传 (file_upload)

```rust
use rust_serv::{UploadHandler, UploadConfig};

let config = UploadConfig::new("/uploads")
    .with_max_size(10 * 1024 * 1024)  // 10 MB
    .with_extensions(vec!["jpg", "png", "gif"])
    .with_unique_names(true);

let handler = UploadHandler::new(config);
let result = handler.handle_upload("photo.jpg", &data);
```

### 基础认证 (basic_auth)

```rust
use rust_serv::{Authenticator, Credentials};

let auth = Authenticator::new()
    .with_realm("Protected Area")
    .add_user("admin", "secret123");

if auth.authenticate(&request).await {
    // 认证成功
}
```

### 访问日志 (access_log)

```rust
use rust_serv::{AccessLogWriter, LogFormat};

let writer = AccessLogWriter::new("./logs/access.log")
    .with_format(LogFormat::Combined)
    .with_rotation(Rotation::Daily);

writer.log(&entry).await;
```

### 自定义错误页面 (error_pages)

```rust
use rust_serv::{ErrorPageHandler, ErrorTemplates};

let templates = ErrorTemplates::new()
    .with_custom_404("./errors/404.html")
    .with_custom_500("./errors/500.html");

let handler = ErrorPageHandler::new(templates);
```

## 项目结构

```
rust-serv/
├── Cargo.toml
├── README.md
├── ROADMAP.md
├── DEVELOPMENT_PROGRESS.md
├── docs/
│   ├── requirements.md
│   ├── technical-design.md
│   ├── performance.md
│   ├── tls.md
│   └── api.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/              # 配置模块
│   ├── config_reloader/     # 配置热重载
│   ├── server/              # 服务器核心
│   ├── handler/             # 请求处理
│   ├── file_service/        # 文件服务
│   ├── file_upload/         # 文件上传
│   ├── memory_cache/        # 内存缓存
│   ├── metrics/             # Prometheus 指标
│   ├── throttle/            # 带宽限速
│   ├── vhost/               # 虚拟主机
│   ├── proxy/               # 反向代理
│   ├── basic_auth/          # 基础认证
│   ├── access_log/          # 访问日志
│   ├── error_pages/         # 错误页面
│   ├── middleware/          # 中间件
│   ├── path_security/       # 路径安全
│   ├── mime_types/          # MIME 类型
│   └── utils/               # 工具
├── tests/
├── benches/
└── examples/
```

## 测试

项目采用 TDD 方法开发，拥有 **650+ 测试用例**，覆盖率 **95%+**。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行带覆盖率的测试
cargo llvm-cov --lib

# 运行基准测试
cargo bench
```

### 测试覆盖

| 模块 | 测试数 | 覆盖率 |
|------|--------|--------|
| memory_cache | 43 | 99%+ |
| metrics | 52 | 98%+ |
| access_log | 27 | 98%+ |
| basic_auth | 49 | 99%+ |
| error_pages | 25 | 95%+ |
| vhost | 29 | 100% |
| file_upload | 44 | 95%+ |
| proxy | 35 | 100% |
| throttle | 50 | 95%+ |
| **总计** | **650+** | **95%+** |

## 性能

- **并发连接**: 1000+ (Tokio 异步运行时)
- **内存占用**: < 100MB (基础操作)
- **启动时间**: < 1s
- **缓存命中**: > 95% (热点文件)
- **零警告**: 完美编译，无 clippy 警告

## 依赖

| 依赖 | 用途 |
|------|------|
| tokio | 异步运行时 |
| hyper | HTTP 服务器 |
| tower | 中间件抽象 |
| serde + toml | 配置序列化 |
| tracing | 结构化日志 |
| base64 | Base64 编解码 |
| tempfile | 测试临时文件 |

## 路线图

### ✅ v0.1.0 - MVP
- 静态文件服务
- 目录索引
- 路径安全
- 配置系统

### ✅ v0.2.0 - 功能增强
- Range 请求
- HTTP 压缩
- 性能优化

### ✅ v0.3.0 - 安全特性
- TLS/HTTPS 支持
- HTTP/2 支持
- WebSocket 支持
- CORS 跨域

### ✅ v0.4.0 - 可观测性 (2026-03-11)
- Prometheus 指标监控
- 访问日志持久化
- 配置热重载

### ✅ v0.5.0 - 性能优化 (2026-03-11)
- 内存缓存系统
- 带宽限速控制

### ✅ v0.6.0 - 扩展功能 (2026-03-11)
- 虚拟主机/多站点
- 反向代理
- 文件上传
- 基础认证
- 自定义错误页面

### 🔜 v0.7.0 - 未来
- 实时文件搜索
- 文件预览 (PDF/Markdown)
- 视频流媒体优化
- 分布式缓存 (Redis)

## 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 编写测试 (TDD)
4. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
5. 推送到分支 (`git push origin feature/AmazingFeature`)
6. 开启 Pull Request

### 开发规范
- 新功能必须包含测试 (覆盖率 ≥ 95%)
- 遵循 Rust 代码风格
- 更新相关文档

## 许可证

本项目采用 MIT 或 Apache-2.0 双许可证。

## 致谢

- Hyper 团队 - HTTP 框架
- Tokio 团队 - 异步运行时
- Tower 团队 - 中间件抽象
- Rust 社区 - 强大生态

## 联系方式

- GitHub: [https://github.com/imnull/rust-serv](https://github.com/imnull/rust-serv)
- Issues: [GitHub Issues](https://github.com/imnull/rust-serv/issues)
