# 掘金发布步骤

## 1. 访问掘金
打开 https://juejin.cn/

## 2. 登录/注册
- 如果没有账号，先注册（支持微信、GitHub、微博登录）
- 已有账号直接登录

## 3. 点击"写文章"
- 右上角有"写文章"或"发布"按钮
- 进入 Markdown 编辑器

## 4. 填写内容

**标题:**
```
我用 Rust 写了一个比 nginx 快 3 倍的静态文件服务器
```

**摘要:**
```
rust-serv 是一个高性能 HTTP 静态文件服务器，基准测试显示 QPS 达到 55,818，比 nginx 17,467 快 3 倍。支持 HTTP/2、WebSocket、TLS、缓存、压缩、监控等完整功能。
```

**封面图:**
上传一张图片（可用 README 截图或项目 logo）

**分类:**
选择"后端"

**标签 (至少3个):**
- Rust
- 性能优化
- nginx
- Web服务器
- 开源项目

**正文内容 (复制粘贴):**

```markdown
## 项目背景

最近在做一个静态文件服务器项目，用 Rust 写了一个名为 rust-serv 的高性能 HTTP 服务器。做完基准测试后，结果让我很惊喜：**比 nginx 快了 3 倍**。

## 性能数据

在相同硬件环境下测试：

| 指标 | nginx 1.29 | rust-serv 0.3 | 提升 |
|------|-----------|--------------|------|
| **QPS** | 17,467 | **55,818** | **3.2x** |
| 平均延迟 | 5.66 ms | **1.74 ms** | **3.3x** |
| P99 延迟 | 6.67 ms | **3.45 ms** | **1.9x** |
| 500 并发 QPS | 17,340 | **50,817** | **2.9x** |

测试工具：oha
测试环境：macOS Darwin，Rust 1.82

## 技术栈

基于 Rust 现代异步技术栈构建：
- **Hyper 1.5** - HTTP 服务器
- **Tokio 1.42** - 异步运行时
- **Tower** - 中间件栈
- **Rustls** - TLS 实现
- **测试覆盖率 95%+**

## 主要特性

### 核心功能
✅ 静态文件服务
✅ 目录索引
✅ 路径安全（防遍历攻击）
✅ MIME 类型检测
✅ HTTP 压缩（Gzip + Brotli）
✅ HTTPS/TLS 支持
✅ HTTP/2 支持
✅ WebSocket 支持
✅ CORS 跨域
✅ Range 请求（断点续传）

### 性能优化
✅ 内存缓存（LRU 策略 + TTL）
✅ ETag 缓存（304 响应）
✅ 带宽限速（Token Bucket）

### 可观测性
✅ Prometheus 指标
✅ 访问日志持久化
✅ 结构化日志
✅ 管理接口（健康检查）

### 安全特性
✅ 基础认证
✅ 请求频率限制
✅ IP 访问控制
✅ 安全头（XSS、CSRF）
✅ 自动 HTTPS 证书（Let's Encrypt）

### 高级功能
✅ 虚拟主机/多站点
✅ 反向代理
✅ 文件上传
✅ 自定义错误页面
✅ 配置热重载

## 组件性能

除了端到端测试，还做了组件级 benchmark：

| 组件 | 性能 |
|------|------|
| **路径验证** | 150-300ns |
| **MIME 检测** | ~400ns |
| **压缩决策** | 2-7ns |
| **ETag 生成** | ~5µs |
| **文件读取 (1MB)** | ~3ms |

## 安装使用

### 安装

```bash
# 通过 Cargo 安装（推荐）
cargo install rust-serv

# 从源码编译
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build --release
```

### 运行

```bash
# 使用默认配置
rust-serv

# 使用配置文件
rust-serv config.toml
```

### 配置示例

```toml
# 基础配置
port = 8080
host = "0.0.0.0"
root = "./public"
enable_indexing = true
enable_compression = true
log_level = "info"

# TLS 配置
[tls]
enabled = true
cert_path = "./certs/cert.pem"
key_path = "./certs/key.pem"

# 内存缓存
[memory_cache]
enabled = true
max_entries = 10000
max_size_mb = 100
```

## 适用场景

- **静态网站** - 博客、文档、落地页
- **SPA 部署** - React/Vue/Angular 应用
- **开发服务器** - 本地测试
- **CDN 源站** - 高性能源服务器
- **Kubernetes 边车** - 健康检查就绪
- **API 网关** - 微服务反向代理

## 为什么这么快？

1. **零拷贝 I/O** - Hyper 的零拷贝技术
2. **异步运行时** - Tokio 高效调度
3. **优化的路径验证** - 150ns 级别
4. **高效压缩管线** - 流式压缩
5. **LRU 缓存** - 热点数据缓存

## 项目状态

- ✅ 已发布到 crates.io (v0.3.0)
- ✅ 完整文档（GitHub Pages）
- ✅ MIT + Apache-2.0 双许可证
- ✅ 测试覆盖率 95%+
- ✅ 生产就绪

## 链接

- **GitHub**: https://github.com/imnull/rust-serv
- **文档**: https://imnull.github.io/rust-serv/
- **性能测试**: https://imnull.github.io/rust-serv/benchmark-vs-nginx.html
- **Crates.io**: https://crates.io/crates/rust-serv

## 欢迎反馈

这是我的第一个 Rust 开源项目，欢迎各位大佬：
- ⭐ Star 支持
- 🐛 提 Issue 报 Bug
- 🔧 提 PR 贡献代码
- 💬 留言讨论技术实现

如果觉得有用，帮忙点个 Star，感谢！

---

**相关标签:** #Rust #nginx #性能优化 #Web服务器 #开源项目
```

## 5. 发布设置

**定时发布:** 不需要，直接发布

**自动生成目录:** 勾选（掘金会自动从标题生成）

**声明原创:** 勾选

**赞赏:** 可选（如果开通了）

## 6. 点击"发布文章"

检查预览，确认无误后点击发布。

## 7. 发布后

- 分享到微信/朋友圈
- 回复评论互动
- 关注数据表现

## 最佳发布时间

掘金活跃时间：
- **工作日早上 9-11 点**（上班摸鱼时间）
- **工作日下午 3-5 点**（下午茶时间）
- **工作日晚上 8-10 点**（晚间阅读）

**推荐发布时间：今天晚上 8 点左右**

## 注意事项

1. 掘金对原创内容友好，容易被推荐
2. 标题要有吸引力，数据要亮眼
3. 代码块要格式正确
4. 配图能提升阅读体验
5. 及时回复评论，增加互动
