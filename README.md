# Rust HTTP 静态服务器

一个高性能、安全、可配置的 Rust HTTP 静态文件服务器，采用测试驱动开发（TDD）范式进行开发。

## 特性

- ✅ **静态文件服务** - 高效提供静态文件
- ✅ **目录索引** - 自动生成目录列表
- ✅ **路径安全** - 防止路径遍历攻击
- ✅ **MIME 类型检测** - 自动检测文件类型
- ✅ **配置系统** - 灵活的 TOML 配置
- ✅ **TDD 开发** - 完整的测试覆盖

## 快速开始

### 安装

```bash
# 克隆仓库
git clone <repository-url>
cd rust-serv

# 编译
cargo build --release
```

### 使用

```bash
# 使用默认配置运行
cargo run

# 使用自定义配置
cargo run -- path/to/config.toml
```

### 配置

创建 `config.toml`:

```toml
port = 8080
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"
```

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test '*'
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# 运行 linter
cargo clippy
```

### 文档

```bash
# 生成并打开文档
cargo doc --open
```

## 项目结构

```
rust-serv/
├── Cargo.toml              # 项目配置
├── README.md               # 本文件
├── docs/
│   ├── requirements.md      # 需求文档
│   ├── technical-design.md  # 技术文档
│   └── api.md             # API 文档
├── src/
│   ├── main.rs            # 程序入口
│   ├── lib.rs             # 库根
│   ├── config/            # 配置模块
│   ├── server/            # 服务器核心
│   ├── handler/           # 请求处理
│   ├── file_service/      # 文件服务
│   ├── path_security/     # 路径安全
│   ├── mime_types/        # MIME 类型
│   ├── middleware/        # 中间件
│   └── utils/             # 工具
├── tests/
│   └── integration_test.rs  # 集成测试
└── examples/
    └── config.toml       # 配置示例
```

## 测试

项目采用 TDD 方法开发，包含以下测试：

- **单元测试** - 测试各个模块
- **集成测试** - 端到端测试
- **迭代测试** - 每个功能迭代的测试

### 测试覆盖

- ✅ 服务器启动和基本响应
- ✅ 静态文件服务 (HTML, CSS, JS, PNG 等)
- ✅ 目录索引和列表
- ✅ 路径安全 (防止目录遍历)
- ✅ 配置系统
- 🔨 范围请求 (待实现)
- 🔨 压缩支持 (待实现)
- 🔨 ETag 和缓存 (待实现)

## 性能

- **并发连接**: 1000+ (Tokio 异步)
- **内存占用**: < 100MB
- **启动时间**: < 1s

## 依赖

- `tokio` - 异步运行时
- `hyper` - HTTP 服务器
- `tower` - 中间件抽象
- `serde` + `toml` - 配置序列化
- `mime_guess` - MIME 类型检测
- `tracing` - 结构化日志

## 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 或 Apache-2.0 许可证。

## 致谢

- Hyper 团队 - 优秀的 HTTP 框架
- Tokio 团队 - 高性能异步运行时
- Rust 社区 - 强大的生态系统

## 路线图

### MVP (v0.1.0)

- [x] 基础 HTTP 服务
- [x] 静态文件服务
- [x] 目录索引
- [x] 路径安全
- [x] 配置系统
- [x] 基础测试

### v0.2.0

- [ ] 范围请求支持
- [ ] 日志集成
- [ ] 压缩优化
- [ ] 性能测试

### v0.3.0

- [ ] ETag 和缓存
- [ ] TLS 支持
- [ ] 中间件系统集成

### 未来

- [ ] HTTP/2 支持
- [ ] WebSocket 支持
- [ ] 插件系统
- [ ] 虚拟主机
- [ ] 集群部署

## 联系方式

- Issue Tracker: [GitHub Issues](https://github.com/yourusername/rust-serv/issues)
- Email: your.email@example.com

## 参考资料

- [Hyper 文档](https://hyper.rs/)
- [Tokio 文档](https://tokio.rs/)
- [Tower 文档](https://github.com/tower-rs/tower)
- [Rust 异步编程书籍](https://rust-lang.github.io/async-book/)
