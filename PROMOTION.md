# rust-serv: A High-Performance Static File Server in Rust

**55,000+ requests/second. 3x faster than nginx. Zero configuration.**

## The Problem

Need a static file server that doesn't break a sweat under heavy load? nginx is great, but it's showing its age. Modern Rust async runtimes can do better.

## The Solution

rust-serv is a production-ready static file server built with Rust + Hyper + Tokio. It's not just fast—it's **really** fast.

### Performance Numbers

**vs nginx (1.29.5) on the same hardware:**

| Metric | nginx | rust-serv | Improvement |
|--------|-------|-----------|-------------|
| **QPS (small files)** | 17,467 | 55,818 | **3.2x** |
| **Latency (avg)** | 5.66ms | 1.74ms | **3.3x** |
| **P99 latency** | 6.67ms | 3.45ms | **1.9x** |
| **500 concurrent connections** | 17,340 QPS | 50,817 QPS | **2.9x** |

**Component benchmarks:**

- Path validation: 150-300ns (zero overhead security)
- MIME detection: ~400ns
- Compression decision: 2-7ns
- File read (1MB): ~3ms

### Features

**Core:**
✅ Static file serving
✅ Directory indexing
✅ HTTP/2 support
✅ WebSocket support
✅ Range requests (partial content)
✅ ETag caching (304 responses)

**Security:**
✅ Path traversal protection
✅ Basic Auth
✅ Rate limiting
✅ IP whitelist/blacklist
✅ Security headers (XSS, CSRF)
✅ Auto TLS via Let's Encrypt

**Performance:**
✅ Memory caching (LRU + TTL)
✅ Gzip & Brotli compression
✅ Bandwidth throttling (token bucket)

**Observability:**
✅ Prometheus metrics
✅ Structured logging
✅ Access logs (JSON/Common/Combined)
✅ Health checks & ready probes

**Advanced:**
✅ Virtual hosts (multi-site)
✅ Reverse proxy
✅ File upload (PUT/POST)
✅ Hot config reload

### Installation

```bash
# Via Cargo
cargo install rust-serv

# From source
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build --release
```

### Quick Start

```bash
# Serve current directory
rust-serv

# With config
rust-serv config.toml
```

### Configuration

```toml
port = 8080
host = "0.0.0.0"
root = "./public"
enable_compression = true

[tls]
enabled = true
cert_path = "./certs/cert.pem"
key_path = "./certs/key.pem"

[memory_cache]
enabled = true
max_entries = 10000
```

### Why rust-serv?

1. **Blazing fast** - 3x nginx on static files
2. **Memory safe** - Rust guarantees no buffer overflows
3. **Production ready** - 95%+ test coverage
4. **Developer friendly** - Single binary, zero dependencies
5. **Observable** - Prometheus metrics out of the box
6. **Secure by default** - Path validation, rate limiting, TLS

### Use Cases

- Static site hosting
- SPA (Single Page App) deployment
- API gateway for microservices
- Development server
- CDN origin server
- Kubernetes sidecar (health checks ready)

### Benchmarks

Full benchmarks: https://imnull.github.io/rust-serv/benchmark-vs-nginx.html

### Documentation

https://imnull.github.io/rust-serv/

### License

MIT OR Apache-2.0

---

**Star on GitHub:** https://github.com/imnull/rust-serv

**Install:** `cargo install rust-serv`

**Docs:** https://imnull.github.io/rust-serv/
