# 🚀 发布清单 - 直接复制粘贴

## 1. Reddit r/rust

**链接:** https://www.reddit.com/r/rust/submit

**标题:** `[Show] rust-serv: A static file server that's 3x faster than nginx (55K req/s)`

**内容:**
```
Hi r/rust,

I built rust-serv, a high-performance static file server in Rust, and I'm excited to share the benchmarks.

**The numbers:**
- 55,818 req/s vs nginx's 17,467 req/s (same hardware)
- 70% lower latency
- 100% success rate at 500 concurrent connections

**Tech stack:**
- Hyper 1.5 + Tokio 1.42
- Tower middleware stack
- Rustls for TLS
- 95%+ test coverage

**Features:**
- HTTP/2, WebSocket, Range requests
- Memory caching (LRU + TTL)
- Gzip/Brotli compression
- Prometheus metrics
- Rate limiting & IP filtering
- Virtual hosts
- Reverse proxy
- Hot config reload
- Auto TLS (Let's Encrypt)

**Install:**
```bash
cargo install rust-serv
```

**Links:**
- GitHub: https://github.com/imnull/rust-serv
- Docs: https://imnull.github.io/rust-serv/
- Benchmarks: https://imnull.github.io/rust-serv/benchmark-vs-nginx.html

Happy to answer questions about the architecture, performance optimizations, or anything else!
```

---

## 2. Hacker News

**链接:** https://news.ycombinator.com/submit

**标题:** `Show HN: rust-serv – Static file server 3x faster than nginx (55K req/s)`

**内容:**
```
Hi HN,

I built rust-serv, a static file server in Rust that outperforms nginx by 3x in my benchmarks.

**Performance comparison (same hardware):**
- rust-serv: 55,818 req/s
- nginx 1.29.5: 17,467 req/s
- Latency: 1.74ms vs 5.66ms

**Why it's fast:**
- Built on Hyper 1.5 with zero-copy I/O
- Tokio async runtime
- Optimized path validation (150ns)
- Efficient compression pipeline
- LRU memory cache

**What it does:**
- Static file serving
- HTTP/2, WebSocket, Range requests
- TLS with auto Let's Encrypt
- Virtual hosts, reverse proxy
- Prometheus metrics
- Rate limiting, IP filtering
- Hot reload

**Use cases:**
- Static sites & SPAs
- Development server
- CDN origin
- K8s sidecar

It's production-ready with 95%+ test coverage and MIT/Apache-2.0 licensed.

GitHub: https://github.com/imnull/rust-serv
Docs: https://imnull.github.io/rust-serv/

I'd love feedback on the architecture, benchmarking methodology, or potential improvements.
```

---

## 3. Twitter/X Thread

**Tweet 1:**
```
🚀 Just released rust-serv: a static file server built in #rustlang

Benchmarks show 3x faster than nginx:
- 55K req/s vs 17K req/s
- 70% lower latency
- 100% success at 500 concurrent connections

github.com/imnull/rust-serv
```

**Tweet 2:**
```
Features:
✅ HTTP/2 + WebSocket
✅ Memory caching (LRU+TTL)
✅ Gzip/Brotli
✅ Prometheus metrics
✅ Rate limiting
✅ Virtual hosts
✅ Hot reload
✅ Auto TLS (Let's Encrypt)
```

**Tweet 3:**
```
Built on:
- Hyper 1.5
- Tokio 1.42
- Tower middleware
- Rustls

95%+ test coverage, single binary, zero runtime deps.
```

**Tweet 4:**
```
Install in seconds:

cargo install rust-serv
rust-serv

That's it. Your static files are now being served at 55K requests/second.
```

**Tweet 5:**
```
Full benchmarks: imnull.github.io/rust-serv/benchmark-vs-nginx.html
Docs: imnull.github.io/rust-serv/

Star it on GitHub if you find it useful! ⭐
github.com/imnull/rust-serv

#rustlang #webdev #performance
```

---

## 4. 中文社区 - V2EX

**链接:** https://www.v2ex.com/new

**节点:** 酷工作

**标题:** `rust-serv - 一个比 nginx 快 3 倍的静态文件服务器 (55K req/s)`

**内容:**
```
大家好，

我用 Rust 写了一个高性能的静态文件服务器，基准测试显示比 nginx 快 3 倍。

**性能数据:**
- rust-serv: 55,818 req/s
- nginx 1.29: 17,467 req/s
- 延迟降低 70%

**技术栈:**
- Hyper 1.5 + Tokio 1.42
- Tower 中间件
- Rustls TLS
- 测试覆盖率 95%+

**主要特性:**
- HTTP/2、WebSocket、断点续传
- 内存缓存 (LRU + TTL)
- Gzip/Brotli 压缩
- Prometheus 监控
- 限流、IP 过滤
- 虚拟主机、反向代理
- 热重载配置
- Let's Encrypt 自动证书

**安装:**
cargo install rust-serv

**链接:**
- GitHub: https://github.com/imnull/rust-serv
- 文档: https://imnull.github.io/rust-serv/
- 性能测试: https://imnull.github.io/rust-serv/benchmark-vs-nginx.html

欢迎反馈和建议！
```

---

## 5. 掘金

**标题:** `我用 Rust 写了一个比 nginx 快 3 倍的静态文件服务器`

**标签:** Rust, nginx, 性能优化, Web服务器

---

## 6. awesome-rust PR

**操作步骤:**
1. 访问 https://github.com/rust-unofficial/awesome-rust
2. Fork 仓库
3. 编辑 README.md
4. 在 "Web / Server" 部分添加:
```
- [rust-serv](https://github.com/imnull/rust-serv) - High-performance HTTP static file server. 3x faster than nginx, HTTP/2, WebSocket, TLS, caching, metrics. [![Crates.io](https://img.shields.io/crates/v/rust-serv.svg)](https://crates.io/crates/rust-serv)
```
5. 提交 PR

---

## 📋 发布顺序建议

1. **现在:** Reddit r/rust（最活跃的 Rust 社区）
2. **1小时后:** Hacker News（需要时间审核）
3. **同时:** Twitter/X 线程
4. **今晚:** V2EX、掘金（中文社区活跃时间）
5. **周末:** 提交 awesome-rust PR（等社区反馈后）
