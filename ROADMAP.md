# Rust Serv 开发路线图

> 当前版本: 0.1.0  
> 测试覆盖率: 96.11% ✅

---

## 已完成功能 ✅

### 核心功能
- [x] 静态文件服务
- [x] 目录索引 (Directory Listing)
- [x] ETag 缓存机制
- [x] Range 请求 (Partial Content)
- [x] 压缩支持 (Gzip/Brotli)
- [x] TLS/HTTPS 支持
- [x] HTTP/2 支持
- [x] WebSocket 支持
- [x] CORS 跨域支持
- [x] 安全中间件 (速率限制、IP控制)
- [x] 日志系统
- [x] 测试覆盖率 95%+

---

## 建议开发功能 🚀

### 优先级: 高 🔴

#### 1. 内存缓存系统 (In-Memory Cache)
**描述**: 将频繁访问的文件缓存到内存，减少磁盘 I/O  
**价值**: 显著提升性能，特别是小文件访问  
**技术点**:
- LRU 缓存策略
- 最大内存限制
- 文件变更监听
- 缓存命中率统计

```rust
pub struct MemoryCache {
    cache: Arc<RwLock<LruCache<PathBuf, CachedFile>>>,
    max_size: usize,
    current_size: AtomicUsize,
}
```

#### 2. Prometheus 指标监控
**描述**: 暴露 HTTP 指标供 Prometheus 采集  
**价值**: 生产环境监控必备  
**指标**:
- 请求数/秒 (QPS)
- 响应时间分布 (P50/P95/P99)
- 缓存命中率
- 活跃连接数
- 错误率

```
GET /metrics -> Prometheus format
```

#### 3. 配置文件热重载
**描述**: 无需重启服务即可更新配置  
**价值**: 生产环境零停机配置更新  
**实现**:
- 文件系统监听 (notify crate)
- 配置 diff 检测
- 平滑过渡

---

### 优先级: 中 🟡

#### 4. 文件上传支持 (PUT/POST)
**描述**: 支持通过 HTTP 上传文件到服务器  
**价值**: 扩展使用场景，不仅限于静态服务  
**功能**:
- 单文件上传
- 多文件批量上传
- 分片上传 (大文件)
- 上传进度回调

```rust
async fn handle_upload(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
    // 处理 multipart/form-data
}
```

#### 5. 虚拟主机 / 多站点支持
**描述**: 根据 Host 头提供不同站点的内容  
**价值**: 一机多站，节省资源  
**配置**:
```toml
[[vhosts]]
host = "blog.example.com"
root = "/var/www/blog"

[[vhosts]]
host = "api.example.com"
root = "/var/www/api"
```

#### 6. 反向代理功能
**描述**: 将特定路径转发到后端服务  
**价值**: 作为统一入口网关  
**配置**:
```toml
[[proxies]]
path = "/api"
target = "http://localhost:3000"
```

#### 7. 访问日志持久化
**描述**: 将访问日志写入文件或发送到远程  
**价值**: 审计、分析、故障排查  
**格式**: Common Log Format / JSON

```
127.0.0.1 - - [05/Mar/2026:10:30:00 +0800] "GET /index.html HTTP/1.1" 200 1234
```

#### 8. 基础认证 (Basic Auth)
**描述**: 为特定路径添加访问控制  
**价值**: 保护敏感内容  
**配置**:
```toml
[[auth]]
path = "/admin"
users = [
    { username = "admin", password_hash = "..." }
]
```

---

### 优先级: 低 🟢

#### 9. 实时文件搜索
**描述**: 通过 API 搜索服务器上的文件  
**价值**: 方便查找资源  
**接口**:
```
GET /search?q=keyword&dir=/docs
```

#### 10. 视频流媒体优化
**描述**: 针对视频文件的 HLS/DASH 支持  
**价值**: 更好的视频播放体验  
**功能**:
- 自适应码率
- 切片生成
- 预加载

#### 11. 分布式缓存 (Redis)
**描述**: 多实例共享缓存  
**价值**: 集群部署时保持缓存一致  

#### 12. 服务发现集成
**描述**: 对接 Consul/etcd  
**价值**: 微服务架构支持

#### 13. 自定义错误页面
**描述**: 配置 404/500 等错误页面的模板  
**价值**: 品牌一致性

```toml
[error_pages]
404 = "/custom/404.html"
500 = "/custom/500.html"
```

#### 14. 文件预览功能
**描述**: 浏览器直接预览 PDF、图片、Markdown  
**价值**: 提升用户体验  

#### 15. 限速控制 (Bandwidth Throttling)
**描述**: 限制单个连接/IP 的下载速度  
**价值**: 防止带宽被单个用户占满

---

## 推荐开发顺序

### Phase 1: 性能优化 (2-3 周)
1. 内存缓存系统
2. Prometheus 指标监控
3. 配置文件热重载

### Phase 2: 功能增强 (2-3 周)
4. 文件上传支持
5. 访问日志持久化
6. 基础认证

### Phase 3: 扩展功能 (2-3 周)
7. 虚拟主机/多站点
8. 反向代理
9. 自定义错误页面

---

## 技术选型建议

| 功能 | 推荐 crate | 说明 |
|------|-----------|------|
| 内存缓存 | `lru` / `moka` | 高性能 LRU 缓存 |
| 指标监控 | `metrics` + `metrics-exporter-prometheus` | 标准指标库 |
| 文件监听 | `notify` | 跨平台文件系统事件 |
| 上传处理 | `multer` / `multipart` | 表单解析 |
| 认证 | `argon2` / `bcrypt` | 密码哈希 |
| 日志 | `tracing-appender` | 日志持久化 |

---

## 贡献指南

1. 选择功能前先开 Issue 讨论
2. 遵循现有代码风格
3. 新功能必须包含测试 (覆盖率 ≥ 95%)
4. 更新文档

---

*最后更新: 2026-03-05*
