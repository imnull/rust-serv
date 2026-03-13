# Getting Started Guide

**English** | [中文](../getting-started.md)

---

> Get started with rust-serv in 5 minutes - from zero to your first web service

## 🦀 What is rust-serv?

**rust-serv** is a **static file server** written in Rust. Simply put:

> Turn any folder on your computer into a website accessible via browser.

Like Python's `python -m http.server`, but more powerful, secure, and fast.

---

## ✨ Why Choose It?

### 1. **Extremely Simple**
```bash
# Traditional methods require installing Nginx, writing configs, changing permissions...
# rust-serv needs only one line:
rust-serv

# Your website is now running at http://localhost:8080!
```

### 2. **Batteries Included**
No need to understand configuration or servers:
- Supports all file types (HTML, images, videos, CSS, JS)
- Auto-generated directory listings
- Automatic compression (faster loading)
- Automatic caching (reduced bandwidth)

### 3. **Production-Grade Security**
Rust's memory safety + built-in protections:
- Path traversal prevention (won't leak system files)
- Rate limiting (prevent DDoS)
- HTTPS support (encrypted transmission)
- CORS configuration (cross-origin support)

### 4. **Developer-Friendly**
- Health check endpoints (K8s ready)
- Prometheus metrics (monitor QPS, latency)
- Structured logging (easy debugging)
- Hot config reload (zero downtime)

### 5. **Extreme Performance**
- Tokio async runtime (1000+ concurrent connections)
- HTTP/2 support (faster loading)
- Memory cache (instant hot file access)
- Bandwidth throttling (fair distribution)

---

## 🚀 5-Minute Hello World

### Step 1: Install

**Option A: From crates.io (Recommended)**
```bash
cargo install rust-serv
```

**Option B: Build from Source**
```bash
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build --release
./target/release/rust-serv
```

**Option C: Docker**
```bash
docker pull ghcr.io/imnull/rust-serv:latest
```

---

### Step 2: Create Your First Website

```bash
# 1. Create a folder
mkdir my-website
cd my-website

# 2. Create an HTML file
cat > index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>My First Website</title>
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
        <h3>⚡ Blazing Fast</h3>
        <p>Tokio async runtime handles 1000+ concurrent connections easily</p>
    </div>
    
    <div class="feature">
        <h3>🔒 Secure</h3>
        <p>Rust memory safety + built-in protections</p>
    </div>
    
    <div class="feature">
        <h3>📊 Observable</h3>
        <p>Built-in Prometheus metrics for real-time monitoring</p>
    </div>
    
    <p style="text-align: center; margin-top: 50px;">
        Visit <a href="/stats" style="color: #fff;">/stats</a> to see server statistics
    </p>
</body>
</html>
EOF

# 3. Start the server
rust-serv
```

**Open your browser and visit:**
- Homepage: http://localhost:8080
- Statistics: http://localhost:8080/stats
- Health: http://localhost:8080/health

---

### Step 3: Use Configuration (Optional)

Create `config.toml`:

```toml
# Basic configuration
port = 8080
root = "."
enable_indexing = true
enable_compression = true

# Management endpoints
[management]
enabled = true
health_path = "/health"
ready_path = "/ready"
stats_path = "/stats"

# Prometheus metrics
[metrics]
enabled = true
path = "/metrics"

# Memory cache
[memory_cache]
enabled = true
max_entries = 1000
max_size_mb = 50
ttl_secs = 300
```

Start with config:
```bash
rust-serv config.toml
```

---

## 🎬 Practical Examples

### Example 1: Share Files with Colleagues

```bash
cd /path/to/shared/files
rust-serv

# Colleagues visit http://YOUR_IP:8080
```

### Example 2: Local Frontend Development

```bash
cd my-react-app/build
rust-serv --port 3000
```

### Example 3: Docker Deployment

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

## 📊 Check Runtime Status

Visit management endpoints:

```bash
# Health check
curl http://localhost:8080/health
# {"status":"healthy"}

# Readiness probe
curl http://localhost:8080/ready
# {"status":"ready"}

# Runtime statistics
curl http://localhost:8080/stats
# {
#   "active_connections": 5,
#   "total_requests": 1234,
#   "cache_hit_rate": 0.95,
#   "uptime_secs": 3600
# }

# Prometheus metrics
curl http://localhost:8080/metrics
```

---

## 🆚 Comparison with Other Solutions

| Feature | rust-serv | Nginx | Python | Caddy |
|---------|-----------|-------|--------|-------|
| Installation | ⭐ Easy | ⭐⭐⭐ Complex | ⭐ Easy | ⭐⭐ Medium |
| Configuration | ⭐ Easy | ⭐⭐⭐⭐ Hard | ⭐ Easy | ⭐⭐ Medium |
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ Low | ⭐⭐⭐⭐ High |
| Memory Safety | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ C lang | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Batteries Included | ⭐⭐⭐⭐⭐ Yes | ⭐⭐ Needs config | ⭐⭐⭐⭐ Yes | ⭐⭐⭐⭐ Yes |
| HTTPS | ⭐⭐⭐⭐ Supported | ⭐⭐⭐ Manual | ⭐ Not supported | ⭐⭐⭐⭐⭐ Auto |

---

## 🎯 Who Should Use It?

- ✅ **Frontend Developers** - Quick static site preview
- ✅ **Backend Developers** - Learn Rust async programming
- ✅ **DevOps Engineers** - Containerized deployment
- ✅ **Students/Teachers** - Teaching demos, quick file sharing
- ✅ **Open Source Enthusiasts** - Contribute code, learn TDD

---

## 📚 Next Steps

- [Advanced Guide](./advanced-guide.md) - Learn more features
- [Expert Guide](./expert-guide.md) - Production deployment
- [Configuration Reference](./configuration.md) - Complete parameter docs

---

## 💬 Summary

> **rust-serv is like a Swiss Army knife: simple, reliable, feature-complete.**

No need to:
- Learn Nginx config syntax
- Worry about C buffer overflows
- Install heavy dependencies

**Start your web journey with one command:**

```bash
cargo install rust-serv && rust-serv
```
