# Reddit / Hacker News Post Templates

## r/rust

**Title:** [Show] rust-serv: A static file server that's 3x faster than nginx (55K req/s)

**Body:**
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

---

## Hacker News

**Title:** Show HN: rust-serv – Static file server 3x faster than nginx (55K req/s)

**Body:**
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
- Static file serving (obviously)
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

---

## Twitter/X Thread

1/5
🚀 Just released rust-serv: a static file server built in #rustlang

Benchmarks show 3x faster than nginx:
- 55K req/s vs 17K req/s
- 70% lower latency
- 100% success at 500 concurrent connections

github.com/imnull/rust-serv

2/5
Features:
✅ HTTP/2 + WebSocket
✅ Memory caching (LRU+TTL)
✅ Gzip/Brotli
✅ Prometheus metrics
✅ Rate limiting
✅ Virtual hosts
✅ Hot reload
✅ Auto TLS (Let's Encrypt)

3/5
Built on:
- Hyper 1.5
- Tokio 1.42
- Tower middleware
- Rustls

95%+ test coverage, single binary, zero runtime deps.

4/5
Install in seconds:
```bash
cargo install rust-serv
rust-serv
```

That's it. Your static files are now being served at 55K requests/second.

5/5
Full benchmarks: imnull.github.io/rust-serv/benchma…
Docs: imnull.github.io/rust-serv/

Star it on GitHub if you find it useful! ⭐
github.com/imnull/rust-serv

#rustlang #webdev #performance

---

## Dev.to Article Title

"I built a static file server 3x faster than nginx. Here's how."
