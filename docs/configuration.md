# 配置参数参考

> rust-serv 完整配置参数说明

## 📖 目录

- [基础配置](#基础配置)
- [TLS/HTTPS](#tlshttps)
- [内存缓存](#内存缓存)
- [Prometheus 指标](#prometheus-指标)
- [管理 API](#管理-api)
- [访问日志](#访问日志)
- [带宽限速](#带宽限速)
- [虚拟主机](#虚拟主机)
- [反向代理](#反向代理)
- [文件上传](#文件上传)
- [基础认证](#基础认证)
- [安全配置](#安全配置)
- [自动 TLS](#自动-tls)
- [配置热重载](#配置热重载)
- [错误页面](#错误页面)

---

## 基础配置

### `port`

**类型**: `u16`  
**默认值**: `8080`  
**说明**: 服务器监听端口

```toml
port = 8080
```

---

### `root`

**类型**: `String`  
**默认值**: `"."`  
**说明**: 静态文件根目录（相对或绝对路径）

```toml
root = "/var/www/html"
# 或
root = "./public"
```

---

### `enable_indexing`

**类型**: `bool`  
**默认值**: `true`  
**说明**: 是否启用目录索引（显示文件列表）

```toml
enable_indexing = true
```

效果：
```
访问 /docs/ 显示：
📁 docs/
  📄 index.html
  📄 readme.md
  📁 images/
```

---

### `enable_compression`

**类型**: `bool`  
**默认值**: `true`  
**说明**: 是否启用压缩（Gzip/Brotli）

```toml
enable_compression = true
```

支持：
- Gzip（默认）
- Brotli（如果客户端支持）

---

### `log_level`

**类型**: `String`  
**默认值**: `"info"`  
**可选值**: `error`, `warn`, `info`, `debug`, `trace`  
**说明**: 日志级别

```toml
log_level = "info"
```

---

### `max_connections`

**类型**: `usize`  
**默认值**: `1000`  
**说明**: 最大并发连接数

```toml
max_connections = 5000
```

---

### `connection_timeout_secs`

**类型**: `u64`  
**默认值**: `30`  
**说明**: 连接超时时间（秒）

```toml
connection_timeout_secs = 60
```

---

### `max_body_size`

**类型**: `u64`  
**默认值**: `10485760` (10 MB)  
**说明**: 请求体最大大小（字节）

```toml
max_body_size = 52428800  # 50 MB
```

---

### `max_headers`

**类型**: `usize`  
**默认值**: `100`  
**说明**: 请求头最大数量

```toml
max_headers = 200
```

---

## TLS/HTTPS

### `enable_tls`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用 HTTPS

```toml
enable_tls = true
```

---

### `tls_cert`

**类型**: `Option<String>`  
**默认值**: `None`  
**说明**: TLS 证书文件路径

```toml
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
```

---

### `tls_key`

**类型**: `Option<String>`  
**默认值**: `None`  
**说明**: TLS 私钥文件路径

```toml
tls_key = "/etc/letsencrypt/live/example.com/privkey.pem"
```

---

## 内存缓存

### `[memory_cache]`

内存缓存配置块。

```toml
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
ttl_secs = 3600
```

---

#### `memory_cache.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用内存缓存

---

#### `memory_cache.max_entries`

**类型**: `usize`  
**默认值**: `10000`  
**说明**: 最大缓存条目数

---

#### `memory_cache.max_size_mb`

**类型**: `usize`  
**默认值**: `100`  
**说明**: 最大缓存大小（MB）

---

#### `memory_cache.ttl_secs`

**类型**: `u64`  
**默认值**: `3600`  
**说明**: 缓存过期时间（秒）

---

## Prometheus 指标

### `[metrics]`

Prometheus 指标配置块。

```toml
[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"
```

---

#### `metrics.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用 Prometheus 指标

---

#### `metrics.path`

**类型**: `String`  
**默认值**: `"/metrics"`  
**说明**: 指标端点路径

---

#### `metrics.namespace`

**类型**: `String`  
**默认值**: `"rust_serv"`  
**说明**: 指标命名空间前缀

---

## 管理 API

### `[management]`

管理端点配置块。

```toml
[management]
enabled = true
health_path = "/health"
ready_path = "/ready"
stats_path = "/stats"
```

---

#### `management.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用管理端点

---

#### `management.health_path`

**类型**: `String`  
**默认值**: `"/health"`  
**说明**: 健康检查端点路径

返回示例：
```json
{"status":"healthy"}
```

---

#### `management.ready_path`

**类型**: `String`  
**默认值**: `"/ready"`  
**说明**: 就绪探针端点路径

返回示例：
```json
{"status":"ready"}
```

---

#### `management.stats_path`

**类型**: `String`  
**默认值**: `"/stats"`  
**说明**: 统计信息端点路径

返回示例：
```json
{
  "active_connections": 5,
  "total_requests": 1234,
  "cache_hit_rate": 0.95,
  "uptime_secs": 3600,
  "bytes_sent": 1024000,
  "bytes_received": 51200
}
```

---

## 访问日志

### `[access_log]`

访问日志配置块。

```toml
[access_log]
enabled = true
path = "./logs/access.log"
format = "combined"
```

---

#### `access_log.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用访问日志

---

#### `access_log.path`

**类型**: `String`  
**默认值**: `"./access.log"`  
**说明**: 日志文件路径

---

#### `access_log.format`

**类型**: `String`  
**默认值**: `"combined"`  
**可选值**: `common`, `combined`, `json`  
**说明**: 日志格式

**Common 格式**:
```
192.168.1.1 - - [12/Mar/2026:10:00:00 +0800] "GET /index.html HTTP/1.1" 200 1234
```

**Combined 格式**:
```
192.168.1.1 - - [12/Mar/2026:10:00:00 +0800] "GET /index.html HTTP/1.1" 200 1234 "http://example.com" "Mozilla/5.0"
```

**JSON 格式**:
```json
{
  "ip": "192.168.1.1",
  "method": "GET",
  "path": "/index.html",
  "status": 200,
  "size": 1234,
  "duration_ms": 5,
  "user_agent": "Mozilla/5.0"
}
```

---

## 带宽限速

### `[throttle]`

带宽限速配置块。

```toml
[throttle]
enabled = true
global_limit_bps = 10485760   # 10 MB/s
per_ip_limit_bps = 1048576    # 1 MB/s
```

---

#### `throttle.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用带宽限速

---

#### `throttle.global_limit_bps`

**类型**: `u64`  
**默认值**: `0` (无限制)  
**说明**: 全局带宽限制（字节/秒）

---

#### `throttle.per_ip_limit_bps`

**类型**: `u64`  
**默认值**: `0` (无限制)  
**说明**: 单 IP 带宽限制（字节/秒）

---

## 虚拟主机

### `[[vhosts]]`

虚拟主机配置数组。

```toml
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"
```

---

#### `vhosts.host`

**类型**: `String`  
**说明**: 域名（支持通配符 `*`）

```toml
host = "*.example.com"  # 匹配所有子域名
```

---

#### `vhosts.root`

**类型**: `String`  
**说明**: 站点根目录

---

## 反向代理

### `[[proxies]]`

反向代理配置数组。

```toml
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true
```

---

#### `proxies.path`

**类型**: `String`  
**说明**: 代理路径前缀

---

#### `proxies.target`

**类型**: `String`  
**说明**: 后端服务地址

---

#### `proxies.strip_prefix`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否移除路径前缀

示例：
```
strip_prefix = false:
  请求 /api/users → 后端 /api/users

strip_prefix = true:
  请求 /api/users → 后端 /users
```

---

## 文件上传

### `[[upload]]`

文件上传配置数组。

```toml
[[upload]]
path = "/upload"
max_size = 10485760
allowed_extensions = ["jpg", "png", "pdf"]
unique_names = true
```

---

#### `upload.path`

**类型**: `String`  
**说明**: 上传端点路径

---

#### `upload.max_size`

**类型**: `u64`  
**默认值**: `10485760` (10 MB)  
**说明**: 最大文件大小（字节）

---

#### `upload.allowed_extensions`

**类型**: `Vec<String>`  
**默认值**: `[]` (所有类型)  
**说明**: 允许的文件扩展名

---

#### `upload.unique_names`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否自动生成唯一文件名

---

## 基础认证

### `[[auth]]`

基础认证配置数组。

```toml
[[auth]]
path = "/admin"
users = [
  { username = "admin", password_hash = "hashed_password" }
]
```

---

#### `auth.path`

**类型**: `String`  
**说明**: 需要认证的路径

---

#### `auth.users`

**类型**: `Array`  
**说明**: 用户列表

```toml
users = [
  { username = "admin", password_hash = "$apr1$..." },
  { username = "user", password_hash = "$apr1$..." }
]
```

---

## 安全配置

### `[security]`

安全配置块。

```toml
[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60
ip_allowlist = ["192.168.1.0/24"]
ip_blocklist = ["10.0.0.100"]
```

---

#### `security.enable_rate_limit`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用速率限制

---

#### `security.rate_limit_max_requests`

**类型**: `usize`  
**默认值**: `100`  
**说明**: 时间窗口内最大请求数

---

#### `security.rate_limit_window_secs`

**类型**: `u64`  
**默认值**: `60`  
**说明**: 时间窗口大小（秒）

---

#### `security.ip_allowlist`

**类型**: `Vec<String>`  
**默认值**: `[]` (所有允许)  
**说明**: IP 白名单（支持 CIDR）

---

#### `security.ip_blocklist`

**类型**: `Vec<String>`  
**默认值**: `[]`  
**说明**: IP 黑名单

---

## 自动 TLS

### `[auto_tls]`

自动 TLS 配置块。

```toml
[auto_tls]
enabled = true
domains = ["example.com", "www.example.com"]
email = "admin@example.com"
challenge_type = "http-01"
cache_dir = "./certs"
renew_before_days = 30
```

---

#### `auto_tls.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用自动 TLS

---

#### `auto_tls.domains`

**类型**: `Vec<String>`  
**说明**: 域名列表

---

#### `auto_tls.email`

**类型**: `String`  
**说明**: Let's Encrypt 注册邮箱

---

#### `auto_tls.challenge_type`

**类型**: `String`  
**默认值**: `"http-01"`  
**可选值**: `http-01`, `dns-01`  
**说明**: ACME 挑战类型

---

#### `auto_tls.cache_dir`

**类型**: `String`  
**默认值**: `"./certs"`  
**说明**: 证书缓存目录

---

#### `auto_tls.renew_before_days`

**类型**: `u32`  
**默认值**: `30`  
**说明**: 提前多少天续期

---

## 配置热重载

### `[config_reloader]`

配置热重载配置块。

```toml
[config_reloader]
enabled = true
watch_path = "./config.toml"
debounce_ms = 1000
```

---

#### `config_reloader.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用热重载

---

#### `config_reloader.watch_path`

**类型**: `String`  
**说明**: 监听的配置文件路径

---

#### `config_reloader.debounce_ms`

**类型**: `u64`  
**默认值**: `1000`  
**说明**: 防抖延迟（毫秒）

---

## 错误页面

### `[error_pages]`

自定义错误页面配置块。

```toml
[error_pages]
enabled = true

[error_pages.pages]
404 = "./errors/404.html"
500 = "./errors/500.html"
503 = "./errors/503.html"
```

---

#### `error_pages.enabled`

**类型**: `bool`  
**默认值**: `false`  
**说明**: 是否启用自定义错误页面

---

#### `error_pages.pages`

**类型**: `Map<u16, String>`  
**说明**: 错误码到 HTML 文件的映射

支持错误码：`400`, `401`, `403`, `404`, `405`, `500`, `502`, `503`, `504`

---

## 完整示例

```toml
# rust-serv 完整配置示例
# 保存为 config.toml

# ===== 基础配置 =====
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"
max_connections = 10000
connection_timeout_secs = 60
max_body_size = 52428800
max_headers = 100

# ===== TLS =====
enable_tls = true
tls_cert = "./certs/cert.pem"
tls_key = "./certs/key.pem"

# ===== 缓存 =====
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
ttl_secs = 3600

# ===== 监控 =====
[management]
enabled = true
health_path = "/health"
ready_path = "/ready"
stats_path = "/stats"

[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"

# ===== 日志 =====
[access_log]
enabled = true
path = "./logs/access.log"
format = "json"

# ===== 限速 =====
[throttle]
enabled = true
global_limit_bps = 10485760
per_ip_limit_bps = 1048576

# ===== 虚拟主机 =====
[[vhosts]]
host = "static.example.com"
root = "/var/www/static"

[[vhosts]]
host = "docs.example.com"
root = "/var/www/docs"

# ===== 反向代理 =====
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true

# ===== 认证 =====
[[auth]]
path = "/admin"
users = [
  { username = "admin", password_hash = "hashed" }
]

# ===== 安全 =====
[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60
ip_allowlist = []
ip_blocklist = ["10.0.0.100"]

# ===== 自动 TLS =====
[auto_tls]
enabled = false
domains = ["example.com"]
email = "admin@example.com"
challenge_type = "http-01"
cache_dir = "./certs"
renew_before_days = 30

# ===== 热重载 =====
[config_reloader]
enabled = true
watch_path = "./config.toml"
debounce_ms = 1000

# ===== 错误页面 =====
[error_pages]
enabled = true

[error_pages.pages]
404 = "./errors/404.html"
500 = "./errors/500.html"
```

---

## 📚 相关文档

- [快速入门](./getting-started.md)
- [进阶指南](./advanced-guide.md)
- [高级指南](./expert-guide.md)
