# 进阶指南

[English](./en/advanced-guide.md) | **中文**

---

> 掌握 rust-serv 的核心功能，提升你的开发效率

## 📖 前置知识

- 已完成[快速入门](./getting-started.md)
- 了解基本 HTTP 概念
- 熟悉命令行操作

---

## 🎯 本章目标

- ✅ 配置 HTTPS/TLS
- ✅ 启用缓存加速
- ✅ 配置虚拟主机
- ✅ 设置反向代理
- ✅ 启用认证保护
- ✅ 监控与日志

---

## 1. 启用 HTTPS

### 方式 A：使用现有证书

```toml
# config.toml
port = 443
enable_tls = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
```

生成自签名证书（测试用）：
```bash
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=localhost"
```

### 方式 B：使用 Let's Encrypt

1. 用 certbot 获取证书：
```bash
sudo certbot certonly --standalone -d example.com
```

2. 配置 rust-serv：
```toml
enable_tls = true
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
tls_key = "/etc/letsencrypt/live/example.com/privkey.pem"
```

3. 自动续期（crontab）：
```bash
0 0 1 * * certbot renew --quiet && systemctl restart rust-serv
```

### 方式 C：使用自动 TLS（占位符）

```toml
[auto_tls]
enabled = true
domains = ["example.com"]
email = "admin@example.com"
cache_dir = "./certs"
```

> ⚠️ 注意：完整 ACME 实现计划在 v0.4.0，当前版本请使用 certbot

---

## 2. 内存缓存加速

### 基础配置

```toml
[memory_cache]
enabled = true
max_entries = 10000      # 最大缓存文件数
max_size_mb = 100        # 最大缓存大小 (MB)
ttl_secs = 3600          # 缓存过期时间 (秒)
```

### 工作原理

```
请求 → 检查缓存
         ↓
    缓存命中? → 是 → 立即返回
         ↓
        否
         ↓
    读取文件 → 存入缓存 → 返回
```

### 缓存策略

- **LRU 淘汰**：最近最少使用的文件会被清理
- **TTL 过期**：超过时间自动失效
- **大小限制**：超出容量自动清理
- **手动清理**：过期条目定期清理

### 监控缓存效果

```bash
curl http://localhost:8080/stats | jq .cache_hit_rate
# 0.95 (95% 命中率)
```

---

## 3. 虚拟主机（多站点）

托管多个网站：

```toml
# 默认站点
root = "/var/www/default"

# 虚拟主机
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"

[[vhosts]]
host = "*.example.com"  # 通配符
root = "/var/www/wildcard"
```

### 工作原理

```
请求 Host: blog.example.com
    ↓
匹配 vhosts
    ↓
返回 /var/www/blog/index.html
```

### 优先级

1. 精确匹配 > 通配符匹配 > 默认站点
2. 第一个匹配的生效

---

## 4. 反向代理

将 API 请求转发到后端服务：

```toml
[[proxies]]
path = "/api"              # 匹配路径
target = "http://localhost:3000"  # 后端地址
strip_prefix = true        # 移除 /api 前缀

[[proxies]]
path = "/graphql"
target = "http://localhost:4000"
strip_prefix = false
```

### 示例

```
前端请求: GET /api/users
    ↓
rust-serv 代理
    ↓
后端收到: GET /users (prefix stripped)
    ↓
后端响应
    ↓
返回前端
```

### 使用场景

- API 网关
- 微服务聚合
- 前后端分离

---

## 5. 基础认证

保护敏感路径：

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

### 生成密码哈希

```bash
# 使用 htpasswd (Apache 工具)
htpasswd -nb admin password123
# admin:$apr1$...

# 或使用 base64 (简单场景)
echo -n "password123" | base64
```

### 访问方式

```bash
# 浏览器会弹出登录框
# 或
curl -u admin:password123 http://localhost:8080/admin
```

---

## 6. 文件上传

允许上传文件：

```toml
[[upload]]
path = "/upload"
max_size = 10485760  # 10 MB
allowed_extensions = ["jpg", "png", "pdf"]
unique_names = true  # 自动生成唯一文件名
```

### 使用方式

```bash
# 上传文件
curl -X POST -F "file=@photo.jpg" http://localhost:8080/upload

# 返回
{
  "success": true,
  "filename": "abc123.jpg",
  "size": 102400
}
```

---

## 7. 带宽限速

防止带宽被占满：

```toml
[throttle]
enabled = true
global_limit_bps = 10485760    # 全局 10 MB/s
per_ip_limit_bps = 1048576     # 单 IP 1 MB/s
```

### Token Bucket 算法

```
桶容量: 100 tokens
补充速率: 10 tokens/s

请求消耗 token
    ↓
桶空了 → 拒绝或等待
```

---

## 8. 监控与日志

### Prometheus 指标

```toml
[metrics]
enabled = true
path = "/metrics"
namespace = "rust_serv"
```

访问指标：
```bash
curl http://localhost:8080/metrics
```

指标示例：
```
# HELP rust_serv_requests_total Total number of requests
# TYPE rust_serv_requests_total counter
rust_serv_requests_total{method="GET",status="200"} 1234

# HELP rust_serv_request_duration_seconds Request duration
# TYPE rust_serv_request_duration_seconds histogram
rust_serv_request_duration_seconds_bucket{le="0.01"} 100
rust_serv_request_duration_seconds_bucket{le="0.05"} 500
```

### 访问日志

```toml
[access_log]
enabled = true
path = "./logs/access.log"
format = "combined"  # common, combined, json
```

格式示例：
```
# Combined
192.168.1.1 - - [12/Mar/2026:10:00:00 +0800] "GET /index.html HTTP/1.1" 200 1234 "http://example.com" "Mozilla/5.0"

# JSON
{"ip":"192.168.1.1","method":"GET","path":"/index.html","status":200,"size":1234,"duration_ms":5}
```

### 结构化日志

```toml
log_level = "info"  # error, warn, info, debug, trace
```

日志输出：
```
2026-03-12T10:00:00.000Z INFO rust_serv::server Starting server on 0.0.0.0:8080
2026-03-12T10:00:05.123Z INFO rust_serv::handler Request method=GET path=/index.html status=200 duration_ms=5
```

---

## 9. 配置热重载

修改配置后自动生效：

```toml
[config_reloader]
enabled = true
watch_path = "./config.toml"
debounce_ms = 1000  # 防抖延迟
```

工作原理：
```
1. 监听配置文件变化
2. 等待防抖时间
3. 验证新配置
4. 平滑切换
5. 无需重启
```

---

## 10. 错误页面

自定义错误页面：

```toml
[error_pages]
enabled = true

[error_pages.pages]
404 = "./errors/404.html"
500 = "./errors/500.html"
502 = "./errors/502.html"
503 = "./errors/503.html"
```

404.html 示例：
```html
<!DOCTYPE html>
<html>
<head>
    <title>404 - 页面未找到</title>
    <style>
        body { 
            text-align: center; 
            padding: 50px; 
            font-family: Arial;
        }
        h1 { font-size: 5em; color: #e74c3c; }
    </style>
</head>
<body>
    <h1>404</h1>
    <p>抱歉，您访问的页面不存在</p>
    <a href="/">返回首页</a>
</body>
</html>
```

---

## 🎯 实战：完整配置示例

```toml
# config.toml - 进阶配置

# 基础
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"

# HTTPS
enable_tls = true
tls_cert = "./certs/cert.pem"
tls_key = "./certs/key.pem"

# 缓存
[memory_cache]
enabled = true
max_entries = 5000
max_size_mb = 50
ttl_secs = 600

# 监控
[management]
enabled = true

[metrics]
enabled = true
path = "/metrics"

# 日志
[access_log]
enabled = true
path = "./logs/access.log"
format = "json"

# 限速
[throttle]
enabled = true
global_limit_bps = 5242880  # 5 MB/s
per_ip_limit_bps = 524288   # 512 KB/s

# 虚拟主机
[[vhosts]]
host = "static.example.com"
root = "/var/www/static"

[[vhosts]]
host = "docs.example.com"
root = "/var/www/docs"

# 反向代理
[[proxies]]
path = "/api"
target = "http://localhost:3000"
strip_prefix = true

# 认证
[[auth]]
path = "/admin"
users = [{ username = "admin", password_hash = "hashed" }]

# 热重载
[config_reloader]
enabled = true
```

---

## 📚 下一步

- [高级指南](./expert-guide.md) - 生产环境部署
- [配置参考](./configuration.md) - 完整参数说明

---

## 💡 小贴士

1. **测试配置**：修改配置后先用 `--test-config` 验证
2. **监控缓存**：定期检查命中率，调整参数
3. **日志轮转**：配置日志文件自动切割
4. **安全加固**：启用速率限制和 IP 过滤
5. **性能优化**：根据实际负载调整缓存大小
