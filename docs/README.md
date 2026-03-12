# rust-serv Documentation

> High Performance HTTP Static Server in Rust

[![Crates.io](https://img.shields.io/crates/v/rust-serv.svg)](https://crates.io/crates/rust-serv)
[![Documentation](https://docs.rs/rust-serv/badge.svg)](https://docs.rs/rust-serv)
[![CI](https://github.com/imnull/rust-serv/workflows/CI/badge.svg)](https://github.com/imnull/rust-serv/actions)
[![Coverage](https://img.shields.io/badge/coverage-95.69%25-brightgreen)]()

---

## 🦀 About

**rust-serv** is a high-performance, secure, and feature-rich HTTP static file server written in Rust.

### Key Features

| Feature | Description |
|---------|-------------|
| ⚡ **Performance** | Tokio async runtime, 1000+ concurrent connections |
| 🔒 **Security** | Rust memory safety + built-in protections |
| 📊 **Observable** | Built-in Prometheus metrics & health checks |
| 🚀 **Easy** | One command to start, zero config needed |
| 🌐 **Production Ready** | K8s friendly, Docker support |

### Statistics

| Metric | Value |
|--------|-------|
| Test Coverage | **95.69%** |
| Test Cases | **824** |
| Version | **0.3.0** |
| Lines of Code | **21,180** |

---

## 🚀 Quick Start

### Installation

```bash
# From crates.io
cargo install rust-serv

# Or with Docker
docker pull ghcr.io/imnull/rust-serv:latest
```

### Usage

```bash
# Start with default config
rust-serv

# Or with custom config
rust-serv config.toml
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](getting-started.md) | 5-minute tutorial for beginners |
| [Advanced Guide](advanced-guide.md) | Core features and configuration |
| [Expert Guide](expert-guide.md) | Production deployment and tuning |
| [Configuration](configuration.md) | Complete parameter reference |

---

## 🔗 Links

- **GitHub**: [https://github.com/imnull/rust-serv](https://github.com/imnull/rust-serv)
- **Crates.io**: [https://crates.io/crates/rust-serv](https://crates.io/crates/rust-serv)
- **API Docs**: [https://docs.rs/rust-serv](https://docs.rs/rust-serv)

---

## 📄 License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/imnull/rust-serv/blob/main/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/imnull/rust-serv/blob/main/LICENSE-MIT))

at your option.
