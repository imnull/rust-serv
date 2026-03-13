# Configuration Reference

**English** | [中文](../configuration.md)

---

> Complete configuration parameter documentation for rust-serv

## 📖 Table of Contents

- [Basic Configuration](#basic-configuration)
- [TLS/HTTPS](#tlshttps)
- [Memory Cache](#memory-cache)
- [Prometheus Metrics](#prometheus-metrics)
- [Management API](#management-api)
- [Access Log](#access-log)
- [Bandwidth Throttling](#bandwidth-throttling)
- [Virtual Hosts](#virtual-hosts)
- [Reverse Proxy](#reverse-proxy)
- [File Upload](#file-upload)
- [Basic Authentication](#basic-authentication)
- [Security](#security)
- [Auto TLS](#auto-tls)
- [Hot Reload](#hot-reload)
- [Error Pages](#error-pages)

---

## Basic Configuration

### `port`

**Type**: `u16`  
**Default**: `8080`  
**Description**: Server listening port

```toml
port = 8080
```

---

### `root`

**Type**: `String`  
**Default**: `"."`  
**Description**: Static file root directory

```toml
root = "/var/www/html"
# or
root = "./public"
```

---

### `enable_indexing`

**Type**: `bool`  
**Default**: `true`  
**Description**: Enable directory indexing

```toml
enable_indexing = true
```

---

### `enable_compression`

**Type**: `bool`  
**Default**: `true`  
**Description**: Enable compression (Gzip/Brotli)

```toml
enable_compression = true
```

---

### `log_level`

**Type**: `String`  
**Default**: `"info"`  
**Options**: `error`, `warn`, `info`, `debug`, `trace`  
**Description**: Log level

```toml
log_level = "info"
```

---

### `max_connections`

**Type**: `usize`  
**Default**: `1000`  
**Description**: Maximum concurrent connections

```toml
max_connections = 5000
```

---

### `connection_timeout_secs`

**Type**: `u64`  
**Default**: `30`  
**Description**: Connection timeout in seconds

```toml
connection_timeout_secs = 60
```

---

## TLS/HTTPS

### `enable_tls`

**Type**: `bool`  
**Default**: `false`  
**Description**: Enable HTTPS

```toml
enable_tls = true
```

---

### `tls_cert`

**Type**: `Option<String>`  
**Default**: `None`  
**Description**: TLS certificate file path

```toml
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
```

---

### `tls_key`

**Type**: `Option<String>`  
**Default**: `None`  
**Description**: TLS private key file path

```toml
tls_key = "/etc/letsencrypt/live/example.com/privkey.pem"
```

---

## Memory Cache

### `[memory_cache]`

Memory cache configuration block.

```toml
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
ttl_secs = 3600
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | bool | false | Enable memory cache |
| `max_entries` | usize | 10000 | Max cached entries |
| `max_size_mb` | usize | 100 | Max cache size (MB) |
| `ttl_secs` | u64 | 3600 | Cache TTL (seconds) |

---

## Prometheus Metrics

### `[metrics]`

Prometheus metrics configuration.

```toml
[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | bool | false | Enable metrics |
| `path` | String | "/metrics" | Metrics endpoint path |
| `namespace` | String | "rust_serv" | Metrics namespace prefix |

---

## Management API

### `[management]`

Management endpoints configuration.

```toml
[management]
enabled = true
health_path = "/health"
ready_path = "/ready"
stats_path = "/stats"
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | bool | false | Enable management API |
| `health_path` | String | "/health" | Health check endpoint |
| `ready_path` | String | "/ready" | Readiness probe endpoint |
| `stats_path` | String | "/stats" | Statistics endpoint |

---

## Access Log

### `[access_log]`

Access log configuration.

```toml
[access_log]
enabled = true
path = "./logs/access.log"
format = "combined"
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | bool | false | Enable access log |
| `path` | String | "./access.log" | Log file path |
| `format` | String | "combined" | Log format (common/combined/json) |

---

## Bandwidth Throttling

### `[throttle]`

Bandwidth throttling configuration.

```toml
[throttle]
enabled = true
global_limit_bps = 10485760   # 10 MB/s
per_ip_limit_bps = 1048576    # 1 MB/s
```

---

## Virtual Hosts

### `[[vhosts]]`

Virtual host configuration array.

```toml
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"
```

---

## Reverse Proxy

### `[[proxies]]`

Reverse proxy configuration.

```toml
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true
```

---

## File Upload

### `[[upload]]`

File upload configuration.

```toml
[[upload]]
path = "/upload"
max_size = 10485760
allowed_extensions = ["jpg", "png", "pdf"]
unique_names = true
```

---

## Basic Authentication

### `[[auth]]`

Basic authentication configuration.

```toml
[[auth]]
path = "/admin"
users = [
  { username = "admin", password_hash = "hashed_password" }
]
```

---

## Security

### `[security]`

Security configuration.

```toml
[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60
ip_allowlist = ["192.168.1.0/24"]
ip_blocklist = ["10.0.0.100"]
```

---

## Auto TLS

### `[auto_tls]`

Automatic TLS certificate configuration.

```toml
[auto_tls]
enabled = true
domains = ["example.com"]
email = "admin@example.com"
challenge_type = "http-01"
cache_dir = "./certs"
renew_before_days = 30
```

---

## Hot Reload

### `[config_reloader]`

Configuration hot reload.

```toml
[config_reloader]
enabled = true
watch_path = "./config.toml"
debounce_ms = 1000
```

---

## Error Pages

### `[error_pages]`

Custom error pages.

```toml
[error_pages]
enabled = true

[error_pages.pages]
404 = "./errors/404.html"
500 = "./errors/500.html"
```

---

## Complete Example

```toml
# rust-serv Complete Configuration Example

# ===== Basic =====
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"
max_connections = 10000
connection_timeout_secs = 60

# ===== TLS =====
enable_tls = true
tls_cert = "./certs/cert.pem"
tls_key = "./certs/key.pem"

# ===== Cache =====
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
ttl_secs = 3600

# ===== Monitoring =====
[management]
enabled = true

[metrics]
enabled = true

# ===== Logging =====
[access_log]
enabled = true
path = "./logs/access.log"
format = "json"

# ===== Throttling =====
[throttle]
enabled = true
global_limit_bps = 10485760
per_ip_limit_bps = 1048576

# ===== Virtual Hosts =====
[[vhosts]]
host = "static.example.com"
root = "/var/www/static"

# ===== Proxy =====
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true

# ===== Auth =====
[[auth]]
path = "/admin"
users = [{ username = "admin", password_hash = "hashed" }]

# ===== Security =====
[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60

# ===== Auto TLS =====
[auto_tls]
enabled = false
domains = ["example.com"]
email = "admin@example.com"

# ===== Hot Reload =====
[config_reloader]
enabled = true
```

---

## 📚 Related Documentation

- [Getting Started](./getting-started.md)
- [Advanced Guide](./advanced-guide.md)
- [Expert Guide](./expert-guide.md)
