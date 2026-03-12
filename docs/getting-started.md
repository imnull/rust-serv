# 快速入门指南

> 5分钟上手 rust-serv，从零开始搭建你的第一个 Web 服务

## 🦀 rust-serv 是什么？

**rust-serv** 是一个用 Rust 编写的**静态文件服务器**，简单来说：

> 把你电脑上的文件夹变成一个网站，让别人能通过浏览器访问。

就像 Python 的 `python -m http.server`，但更强大、更安全、更快速。

---

## ✨ 为什么选择它？

### 1. **简单到极致**
```bash
# 传统方式需要装 Nginx、写配置、改权限...
# rust-serv 只需一行：
rust-serv

# 你的网站已经在 http://localhost:8080 运行了！
```

### 2. **开箱即用**
不需要懂配置，不需要懂服务器：
- 支持所有文件类型（HTML、图片、视频、CSS、JS）
- 自动生成目录列表
- 自动压缩（更快加载）
- 自动缓存（减少流量）

### 3. **生产级安全**
Rust 的内存安全 + 内置防护：
- 防止路径遍历攻击（不会泄露系统文件）
- 速率限制（防止 DDoS）
- HTTPS 支持（加密传输）
- CORS 配置（跨域支持）

### 4. **开发者友好**
- 健康检查接口（K8s 部署就绪）
- Prometheus 指标（监控 QPS、延迟）
- 结构化日志（方便调试）
- 热重载配置（零停机更新）

### 5. **极致性能**
- Tokio 异步运行时（1000+ 并发连接）
- HTTP/2 支持（更快加载）
- 内存缓存（热点文件秒开）
- 带宽限速（公平分配）

---

## 🚀 5分钟 Hello World

### 步骤 1：安装

**方式 A：从 crates.io 安装（推荐）**
```bash
cargo install rust-serv
```

**方式 B：从源码编译**
```bash
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build --release
./target/release/rust-serv
```

**方式 C：Docker**
```bash
docker pull ghcr.io/imnull/rust-serv:latest
```

---

### 步骤 2：创建你的第一个网站

```bash
# 1. 创建一个文件夹
mkdir my-website
cd my-website

# 2. 创建一个 HTML 文件
cat > index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>我的第一个网站</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        h1 { font-size: 3em; text-align: center; }
        .feature {
            background: rgba(255,255,255,0.1);
            padding: 20px;
            margin: 10px 0;
            border-radius: 10px;
        }
    </style>
</head>
<body>
    <h1>🦀 Hello from rust-serv!</h1>
    
    <div class="feature">
        <h3>⚡ 极速</h3>
        <p>Tokio 异步运行时，轻松处理 1000+ 并发连接</p>
    </div>
    
    <div class="feature">
        <h3>🔒 安全</h3>
        <p>Rust 内存安全 + 内置防护</p>
    </div>
    
    <div class="feature">
        <h3>📊 可观测</h3>
        <p>内置 Prometheus 指标，实时监控</p>
    </div>
    
    <p style="text-align: center; margin-top: 50px;">
        访问 <a href="/stats" style="color: #fff;">/stats</a> 查看统计
    </p>
</body>
</html>
EOF

# 3. 启动服务器
rust-serv
```

**打开浏览器访问：**
- 主页：http://localhost:8080
- 统计：http://localhost:8080/stats
- 健康：http://localhost:8080/health

---

### 步骤 3：使用配置文件（可选）

创建 `config.toml`：

```toml
# 基础配置
port = 8080
root = "."
enable_indexing = true
enable_compression = true

# 管理端点
[management]
enabled = true
health_path = "/health"
ready_path = "/ready"
stats_path = "/stats"

# Prometheus 指标
[metrics]
enabled = true
path = "/metrics"

# 内存缓存
[memory_cache]
enabled = true
max_entries = 1000
max_size_mb = 50
ttl_secs = 300
```

启动：
```bash
rust-serv config.toml
```

---

## 🎬 实战示例

### 示例 1：分享文件给同事

```bash
cd /path/to/shared/files
rust-serv

# 同事访问 http://你的IP:8080
```

### 示例 2：本地开发前端

```bash
cd my-react-app/build
rust-serv --port 3000
```

### 示例 3：Docker 部署

```dockerfile
FROM ghcr.io/imnull/rust-serv:latest
COPY ./public /app/public
WORKDIR /app
EXPOSE 8080
CMD ["rust-serv"]
```

```bash
docker build -t my-website .
docker run -p 8080:8080 my-website
```

---

## 📊 查看运行状态

```bash
# 健康检查
curl http://localhost:8080/health
# {"status":"healthy"}

# 就绪探针
curl http://localhost:8080/ready
# {"status":"ready"}

# 运行时统计
curl http://localhost:8080/stats
# {
#   "active_connections": 5,
#   "total_requests": 1234,
#   "cache_hit_rate": 0.95,
#   "uptime_secs": 3600
# }

# Prometheus 指标
curl http://localhost:8080/metrics
```

---

## 🆚 对比其他方案

| 特性 | rust-serv | Nginx | Python | Caddy |
|------|-----------|-------|--------|-------|
| 安装复杂度 | ⭐ 简单 | ⭐⭐⭐ 复杂 | ⭐ 简单 | ⭐⭐ 中等 |
| 配置难度 | ⭐ 简单 | ⭐⭐⭐⭐ 困难 | ⭐ 简单 | ⭐⭐ 中等 |
| 性能 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| 内存安全 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## 🎯 适合谁用？

- ✅ **前端开发者** - 快速预览静态网站
- ✅ **后端开发者** - 学习 Rust 异步编程
- ✅ **DevOps 工程师** - 容器化部署
- ✅ **学生/教师** - 教学演示
- ✅ **开源爱好者** - 贡献代码

---

## 📚 下一步

- [进阶指南](./advanced-guide.md) - 学习更多功能
- [高级指南](./expert-guide.md) - 生产环境部署
- [配置参考](./configuration.md) - 完整配置参数

---

## 💬 总结

> **rust-serv 就像一把瑞士军刀：简单、可靠、功能齐全。**

一行命令，开始你的 Web 之旅：

```bash
cargo install rust-serv && rust-serv
```
