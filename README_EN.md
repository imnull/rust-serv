# rust-serv: High-Performance HTTP Static Server

[![Crates.io](https://img.shields.io/crates/v/rust-serv.svg)](https://crates.io/crates/rust-serv)
[![Downloads](https://img.shields.io/crates/d/rust-serv.svg)](https://crates.io/crates/rust-serv)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://imnull.github.io/rust-serv/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**简体中文** | **English**

**🚀 55,000+ requests/sec | 3x faster than nginx | 95%+ test coverage**

A high-performance, secure, and feature-rich HTTP static file server built in Rust. Powered by Hyper + Tokio with TDD methodology.

## ⚡ Performance

rust-serv significantly outperforms nginx for static file serving:

| Metric | nginx 1.29 | rust-serv 0.3 | Improvement |
|--------|-----------|--------------|-------------|
| **QPS** | 17,467 | **55,818** | **3.2x** |
| Avg Latency | 5.66 ms | **1.74 ms** | **3.3x** |
| P99 Latency | 6.67 ms | **3.45 ms** | **1.9x** |
| 500 concurrent QPS | 17,340 | **50,817** | **2.9x** |

Full benchmarks: [Component Performance](./docs/benchmarks.md) | [vs nginx Comparison](./docs/benchmark-vs-nginx.md)

## ✨ Features

### Core
- ✅ **Static file serving** - Efficient static file delivery
- ✅ **Directory indexing** - Auto-generated directory listings
- ✅ **Path security** - Prevents path traversal attacks
- ✅ **MIME detection** - Automatic file type detection
- ✅ **Configuration** - Flexible TOML config system
- ✅ **HTTP compression** - Gzip and Brotli support
- ✅ **HTTPS/TLS** - Secure encrypted connections
- ✅ **HTTP/2** - Modern protocol support
- ✅ **WebSocket** - Real-time communication
- ✅ **CORS** - Cross-origin resource sharing
- ✅ **Range requests** - Partial content delivery

### Performance
- ✅ **Memory caching** - LRU strategy, TTL expiration, thread-safe
- ✅ **ETag caching** - 304 Not Modified responses
- ✅ **Bandwidth throttling** - Token bucket algorithm

### Observability
- ✅ **Prometheus metrics** - QPS, latency, error rate monitoring
- ✅ **Access logs** - Common/Combined/JSON formats
- ✅ **Structured logging** - tracing-based logging
- ✅ **Management API** - Health checks, ready probes (K8s friendly)

### Security
- ✅ **Basic Auth** - HTTP Basic Authentication, path protection
- ✅ **Rate limiting** - DDoS protection
- ✅ **IP access control** - Whitelist/blacklist
- ✅ **Security headers** - XSS, CSRF protection
- ✅ **Auto TLS** - Let's Encrypt support

### Advanced
- ✅ **Virtual hosts** - Host-based multi-site routing
- ✅ **Reverse proxy** - Path routing with prefix stripping
- ✅ **File upload** - multipart parsing, size/extension limits
- ✅ **Custom error pages** - Beautiful templates
- ✅ **Hot reload** - Zero-downtime config updates

## 🚀 Quick Start

### Installation

**Option 1: Cargo (Recommended)**
```bash
cargo install rust-serv
```

**Option 2: From Source**
```bash
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build --release
```

### Usage

```bash
# Run with defaults
rust-serv

# Run with config file
rust-serv config.toml
```

### Configuration

Create `config.toml`:

```toml
# Basic config
port = 8080
host = "0.0.0.0"
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"

# TLS config
[tls]
enabled = true
cert_path = "./certs/cert.pem"
key_path = "./certs/key.pem"

# Memory cache
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
```

## 📚 Documentation

- **[Getting Started](./docs/en/getting-started.md)** - 5-minute guide
- **[Advanced Guide](./docs/en/advanced-guide.md)** - Core features in depth
- **[Expert Guide](./docs/en/expert-guide.md)** - Production deployment
- **[Configuration](./docs/en/configuration.md)** - Full parameter reference

## 🎯 Use Cases

- **Static sites** - Blogs, documentation, landing pages
- **SPAs** - React/Vue/Angular deployment
- **Development server** - Local testing with live reload
- **CDN origin** - High-performance origin server
- **Kubernetes sidecar** - Health checks ready
- **API gateway** - Reverse proxy for microservices

## 🔧 Architecture

Built with modern Rust async stack:
- **Hyper 1.5** - HTTP server
- **Tokio 1.42** - Async runtime
- **Tower** - Middleware stack
- **Rustls** - TLS implementation

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines.

## 📄 License

MIT OR Apache-2.0

## 🔗 Links

- **GitHub**: https://github.com/imnull/rust-serv
- **Crates.io**: https://crates.io/crates/rust-serv
- **Documentation**: https://imnull.github.io/rust-serv/
- **Benchmarks**: https://imnull.github.io/rust-serv/benchmark-vs-nginx.html

---

**Star ⭐ on GitHub if you find this useful!**
