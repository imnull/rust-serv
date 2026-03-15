# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-03-15

### Added
- **WebAssembly Plugin System** - 完整的热插拔插件系统
  - Plugin SDK - 4 核心模块 (types, error, host, lib)
  - Plugin Runtime - executor, loader, manager, watcher
  - PluginMiddleware - Tower 中间件集成
  - 管理 API - `/_plugins/*` RESTful 接口
  - 9 个示例插件 (CORS, 限流, JWT 认证等)
  - 23 个集成测试
- 1050+ 单元测试
- 93%+ 测试覆盖率
- 完整插件开发文档

### Changed
- 更新 ROADMAP.md 包含插件系统规划
- 新增 RELEASE_ASSESSMENT.md 上线评估报告
- 优化测试覆盖率和代码质量

## [0.3.0] - 2026-03-12

### Added
- Complete documentation with GitHub Pages
- Comprehensive benchmark suite
- Performance comparison with nginx (3x faster)
- CONTRIBUTING.md guide
- MIT and Apache-2.0 dual licensing

### Performance
- 55,000+ requests/second (3x faster than nginx)
- 70% lower latency
- 100% success rate at 500 concurrent connections

### Changed
- Improved README with performance highlights
- Added benchmark documentation
- Enhanced project structure

## [0.2.0] - 2026-03-10

### Added
- Auto TLS implementation with Let's Encrypt support
- Enhanced security features
- Improved configuration system

## [0.1.0] - 2026-03-05

### Added
- Initial release
- Static file serving
- Directory indexing
- HTTP/2 support
- WebSocket support
- TLS/HTTPS support
- Memory caching (LRU + TTL)
- Gzip and Brotli compression
- Prometheus metrics
- Rate limiting
- IP access control
- Basic authentication
- Virtual hosts
- Reverse proxy
- File upload support
- Custom error pages
- Hot configuration reload
- 95%+ test coverage

### Performance
- Built on Hyper 1.5 + Tokio 1.42
- Zero-copy I/O
- Efficient async runtime
- Optimized path validation

---

## Release Notes

### v0.3.0 - Performance & Documentation

This release focuses on proving and documenting rust-serv's performance advantages:

**Key Highlights:**
- **3x faster than nginx** in static file serving benchmarks
- **55K+ req/s** vs nginx's 17K req/s
- Comprehensive documentation site
- Ready for production use

**What's Next:**
- Docker image (in progress)
- Homebrew formula
- More deployment options
- Community feedback integration

---

For more details on each release, see the [GitHub releases page](https://github.com/imnull/rust-serv/releases).
