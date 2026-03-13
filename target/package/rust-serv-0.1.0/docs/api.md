# Rust HTTP 静态服务器 - API 文档

## 公共 API

### Config

服务器配置结构体。

```rust
pub struct Config {
    pub port: u16,              // 服务器端口 (默认: 8080)
    pub root: PathBuf,           // 根目录 (默认: ".")
    pub enable_indexing: bool,     // 启用目录索引 (默认: true)
    pub enable_compression: bool,  // 启用压缩 (默认: true)
    pub log_level: String,        // 日志级别 (默认: "info")
}
```

### Server

HTTP 服务器。

```rust
impl Server {
    pub fn new(config: Config) -> Self;
    pub async fn run(&self) -> Result<()>;
}
```

### Handler

HTTP 请求处理器。

```rust
impl Handler {
    pub fn new(config: Arc<Config>) -> Self;
    pub async fn handle_request(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible>;
}
```

### FileService

文件服务。

```rust
impl FileService {
    pub fn read_file(path: &Path) -> Result<Vec<u8>>;
    pub fn is_directory(path: &Path) -> bool;
    pub fn list_directory(path: &Path) -> Result<Vec<FileMetadata>>;
}
```

### PathValidator

路径验证器，防止路径遍历攻击。

```rust
impl PathValidator {
    pub fn new(root: PathBuf) -> Self;
    pub fn validate(&self, path: &Path) -> Result<PathBuf>;
}
```

### MimeDetector

MIME 类型检测器。

```rust
impl MimeDetector {
    pub fn detect(path: &Path) -> Mime;
}
```

## HTTP 响应码

- `200 OK`: 成功返回文件内容
- `206 Partial Content`: 范围请求成功（待实现）
- `304 Not Modified`: 缓存命中（待实现）
- `400 Bad Request`: 无效请求
- `403 Forbidden`: 路径安全违规
- `404 Not Found`: 文件不存在
- `500 Internal Server Error`: 服务器内部错误

## 配置文件格式 (TOML)

```toml
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"
```

## 错误处理

所有函数返回 `Result<T>`，其中 `Error` 类型包括：

```rust
pub enum Error {
    Config(String),
    Io(io::Error),
    Http(String),
    PathSecurity(String),
    NotFound(String),
    Forbidden(String),
    Internal(String),
    AddrParse(AddrParseError),
}
```

## 使用示例

### 基本用法

```rust
use rust_serv::{Config, Server};

#[tokio::main]
async fn main() -> rust_serv::error::Result<()> {
    let config = Config::default();
    let server = Server::new(config);
    server.run().await?;
    Ok(())
}
```

### 自定义配置

```rust
use rust_serv::{Config, Server};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> rust_serv::error::Result<()> {
    let config = Config {
        port: 9000,
        root: PathBuf::from("/var/www"),
        enable_indexing: true,
        enable_compression: true,
        log_level: "debug".to_string(),
    };

    let server = Server::new(config);
    server.run().await?;
    Ok(())
}
```

## 中间件

### LoggingLayer

记录所有请求。

```rust
use rust_serv::middleware::LoggingLayer;

let layer = LoggingLayer;
```

### CompressionLayer

压缩响应内容。

```rust
use rust_serv::middleware::CompressionLayer;

let layer = CompressionLayer::new(true);
```

### CacheLayer

处理缓存控制。

```rust
use rust_serv::middleware::CacheLayer;

let layer = CacheLayer;
```

## 实现状态

### ✅ 已实现

- 基础 HTTP 服务
- 静态文件服务
- 目录索引
- 路径安全
- 配置系统 (Config 结构体)

### 🔨 部分实现

- 日志和监控 (中间件结构存在，需集成)
- 压缩支持 (中间件结构存在，需集成)
- 缓存控制 (中间件结构存在，需集成)

### 📋 待实现

- 范围请求支持 (HTTP Range header)
- ETag 和缓存 (HTTP 缓存头)
- TLS 支持 (HTTPS)
- CORS 支持
- 速率限制
- 认证

## 扩展指南

### 添加自定义中间件

实现 `Layer` trait:

```rust
use tower::Layer;

pub struct CustomLayer;

impl<S> Layer<S> for CustomLayer {
    type Service = CustomService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CustomService { inner }
    }
}
```

### 自定义 MIME 类型

扩展 `MimeDetector`:

```rust
impl MimeDetector {
    pub fn detect(path: &Path) -> Mime {
        // 自定义逻辑
    }
}
```

## 性能考虑

1. **并发**: 使用 Tokio 异步运行时
2. **内存**: 使用 `Full<Bytes>` 避免不必要的数据复制
3. **文件 I/O**: 大文件应使用流式传输 (TODO)
4. **连接**: HTTP/1.1 Keep-Alive 连接复用

## 安全建议

1. 始终使用 `PathValidator` 验证路径
2. 限制可访问的根目录
3. 启用日志监控可疑活动
4. 考虑添加速率限制
5. 在生产环境使用 TLS

## 测试

运行所有测试:

```bash
cargo test
```

运行特定测试:

```bash
cargo test --test integration_test
cargo test --test iteration2_static_files
cargo test --test iteration3_directory_index
```

## 性能测试

运行基准测试:

```bash
cargo bench
```

## 代码质量

运行 linter:

```bash
cargo clippy
cargo fmt
```

## 文档

生成文档:

```bash
cargo doc --open
```
