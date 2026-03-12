# Advanced Guide

**English** | [中文](../advanced-guide.md)

---

> Master core features of rust-serv to boost your development efficiency

## 📖 Prerequisites

- Completed [Getting Started](./getting-started.md)
- Basic HTTP concepts understanding
- Familiar with command line operations

---

## 🎯 Learning Objectives

- ✅ Configure HTTPS/TLS
- ✅ Enable cache acceleration
- ✅ Configure virtual hosts
- ✅ Set up reverse proxy
- ✅ Enable authentication protection
- ✅ Monitoring and logging

---

## 1. Enable HTTPS

### Option A: Use Existing Certificate

```toml
# config.toml
port = 443
enable_tls = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
```

Generate self-signed certificate (for testing):
```bash
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=localhost"
```

### Option B: Use Let's Encrypt

1. Get certificate with certbot:
```bash
sudo certbot certonly --standalone -d example.com
```

2. Configure rust-serv:
```toml
enable_tls = true
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
tls_key = "/etc/letsencrypt/live/example.com/privkey.pem"
```

3. Auto-renewal (crontab):
```bash
0 0 1 * * certbot renew --quiet && systemctl restart rust-serv
```

---

## 2. Memory Cache Acceleration

### Basic Configuration

```toml
[memory_cache]
enabled = true
max_entries = 10000      # Max cached files
max_size_mb = 100        # Max cache size (MB)
ttl_secs = 3600          # Cache expiration (seconds)
```

### How It Works

```
Request → Check Cache
         ↓
    Cache Hit? → Yes → Immediate Return
         ↓
        No
         ↓
    Read File → Store in Cache → Return
```

### Cache Strategy

- **LRU Eviction**: Least recently used files are cleaned
- **TTL Expiration**: Auto-expire after time limit
- **Size Limit**: Auto-clean when capacity exceeded
- **Manual Cleanup**: Expired entries cleaned periodically

### Monitor Cache Performance

```bash
curl http://localhost:8080/stats | jq .cache_hit_rate
# 0.95 (95% hit rate)
```

---

## 3. Virtual Hosts (Multi-Site)

Host multiple websites:

```toml
# Default site
root = "/var/www/default"

# Virtual hosts
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"

[[vhosts]]
host = "*.example.com"  # Wildcard
root = "/var/www/wildcard"
```

### How It Works

```
Request Host: blog.example.com
    ↓
Match vhosts
    ↓
Return /var/www/blog/index.html
```

### Priority

1. Exact match > Wildcard match > Default site
2. First match wins

---

## 4. Reverse Proxy

Forward API requests to backend services:

```toml
[[proxies]]
path = "/api"              # Match path
target = "http://localhost:3000"  # Backend address
strip_prefix = true        # Remove /api prefix

[[proxies]]
path = "/graphql"
target = "http://localhost:4000"
strip_prefix = false
```

### Example

```
Frontend Request: GET /api/users
    ↓
rust-serv Proxy
    ↓
Backend Receives: GET /users (prefix stripped)
    ↓
Backend Response
    ↓
Return to Frontend
```

### Use Cases

- API Gateway
- Microservice aggregation
- Frontend-backend separation

---

## 5. Basic Authentication

Protect sensitive paths:

```toml
[[auth]]
path = "/admin"
users = [
    { username = "admin", password_hash = "hashed_password" }
]

[[auth]]
path = "/private"
users = [
    { username = "user1", password_hash = "hash1" },
    { username = "user2", password_hash = "hash2" }
]
```

### Generate Password Hash

```bash
# Using htpasswd (Apache tool)
htpasswd -nb admin password123
# admin:$apr1$...

# Or using base64 (simple cases)
echo -n "password123" | base64
```

### Access Method

```bash
# Browser will show login dialog
# Or
curl -u admin:password123 http://localhost:8080/admin
```

---

## 6. File Upload

Allow file uploads:

```toml
[[upload]]
path = "/upload"
max_size = 10485760  # 10 MB
allowed_extensions = ["jpg", "png", "pdf"]
unique_names = true  # Auto-generate unique filenames
```

### Usage

```bash
# Upload file
curl -X POST -F "file=@photo.jpg" http://localhost:8080/upload

# Response
{
  "success": true,
  "filename": "abc123.jpg",
  "size": 102400
}
```

---

## 7. Bandwidth Throttling

Prevent bandwidth saturation:

```toml
[throttle]
enabled = true
global_limit_bps = 10485760    # Global 10 MB/s
per_ip_limit_bps = 1048576     # Per IP 1 MB/s
```

### Token Bucket Algorithm

```
Bucket Capacity: 100 tokens
Refill Rate: 10 tokens/s

Request consumes token
    ↓
Bucket Empty → Reject or Wait
```

---

## 8. Monitoring & Logging

### Prometheus Metrics

```toml
[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"
```

Access metrics:
```bash
curl http://localhost:8080/metrics
```

### Access Log

```toml
[access_log]
enabled = true
path = "./logs/access.log"
format = "combined"  # common, combined, json
```

### Structured Logging

```toml
log_level = "info"  # error, warn, info, debug, trace
```

---

## 9. Hot Config Reload

Auto-reload when config changes:

```toml
[config_reloader]
enabled = true
watch_path = "./config.toml"
debounce_ms = 1000  # Debounce delay
```

---

## 10. Error Pages

Customize error pages:

```toml
[error_pages]
enabled = true

[error_pages.pages]
404 = "./errors/404.html"
500 = "./errors/500.html"
```

---

## 🎯 Practical: Complete Configuration Example

```toml
# config.toml - Advanced Configuration

# Basic
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"

# HTTPS
enable_tls = true
tls_cert = "./certs/cert.pem"
tls_key = "./certs/key.pem"

# Cache
[memory_cache]
enabled = true
max_entries = 5000
max_size_mb = 50
ttl_secs = 600

# Monitoring
[management]
enabled = true

[metrics]
enabled = true
path = "/metrics"

# Logging
[access_log]
enabled = true
path = "./logs/access.log"
format = "json"

# Throttling
[throttle]
enabled = true
global_limit_bps = 5242880  # 5 MB/s
per_ip_limit_bps = 524288   # 512 KB/s

# Virtual Hosts
[[vhosts]]
host = "static.example.com"
root = "/var/www/static"

# Reverse Proxy
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true

# Authentication
[[auth]]
path = "/admin"
users = [{ username = "admin", password_hash = "hashed" }]

# Hot Reload
[config_reloader]
enabled = true
```

---

## 📚 Next Steps

- [Expert Guide](./expert-guide.md) - Production deployment
- [Configuration Reference](./configuration.md) - Complete parameter documentation

---

## 💡 Tips

1. **Test Configuration**: Use `--test-config` to validate before applying
2. **Monitor Cache**: Regularly check hit rate and adjust parameters
3. **Log Rotation**: Configure automatic log file rotation
4. **Security Hardening**: Enable rate limiting and IP filtering
5. **Performance Tuning**: Adjust cache size based on actual load
