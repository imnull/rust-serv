# 性能基准测试

## 概述

本项目包含完整的性能基准测试套件，使用 [Criterion](https://github.com/bheisler/criterion.rs) 框架构建，用于测量和优化关键组件的性能。

## 运行基准测试

### 安装依赖

基准测试需要额外的依赖（已在 `Cargo.toml` 中配置）：

```toml
[dev-dependencies]
criterion = "0.5"
tempfile = "3.14"
```

### 运行所有基准测试

```bash
# 运行完整的基准测试套件
cargo bench

# 运行特定基准测试组
cargo bench -- bench_file_service
cargo bench -- bench_gzip_compression
cargo bench -- bench_brotli_compression
```

### 查看详细输出

```bash
# 保存基准测试结果
cargo bench -- --save-baseline main

# 比较不同运行结果
cargo bench -- --baseline main
```

## 基准测试类别

### 1. 文件服务性能 (`file_service_read`)

测试不同大小的文件读取性能：

- **小文件** (1KB): 典型配置文件、JSON 数据
- **中文件** (10KB): HTML 页面、脚本文件
- **大文件** (100KB): 图片资源、文档
- **超大文件** (1MB): 媒体文件、归档

**预期性能:**
- 所有操作应在微秒级别完成
- 文件大小不应线性影响读取时间（使用零拷贝技术）

### 2. 压缩性能 (`gzip_compression`, `brotli_compression`)

测试不同数据模式的压缩性能：

- **重复数据** (`repetitive`): 测试最佳压缩场景
- **文本数据** (`text`): 模拟 HTML/CSS 内容
- **JSON 数据** (`json`): API 响应数据
- **HTML 数据** (`html`): 网页内容

**预期性能:**
- 重复数据应获得 10-100 倍压缩比
- Brotli 通常比 Gzip 慢但压缩率更高
- 压缩时间应与输入大小呈线性关系

### 3. 路径验证性能 (`path_validation`)

测试路径安全检查性能：

- **有效路径** (`valid`): 正常文件路径
- **嵌套路径** (`nested`): 深层目录结构
- **路径遍历攻击** (`traversal`): `../../../etc/passwd`
- **深层路径** (`deep`): 长路径验证

**预期性能:**
- 所有验证应在纳秒级别完成
- 路径复杂度不应显著影响验证时间
- 恶意路径应被快速拒绝

### 4. MIME 类型检测性能 (`mime_detection`)

测试文件类型识别性能：

- **HTML/CSS/JS**: 网页资源
- **图片格式**: PNG, JPG 等
- **数据格式**: JSON, XML 等

**预期性能:**
- 基于扩展名的检测应在纳秒级别完成
- 不应进行文件系统访问
- 支持数百种文件类型

### 5. 压缩决策性能 (`compression_decision`)

测试内容类型过滤性能：

- **可压缩类型**: text/html, text/css, application/json
- **跳过类型**: image/*, video/*, audio/*
- **已压缩类型**: application/gzip, application/zip

**预期性能:**
- 决策逻辑应在纳秒级别完成
- 使用字符串前缀匹配而非正则表达式
- 避免不必要的内存分配

### 6. ETag 生成性能 (`etag_generation`)

测试 ETag 标识符生成性能：

**预期性能:**
- 使用文件元数据（大小、修改时间）
- 避免读取文件内容
- 生成时间应在微秒级别

### 7. 目录列表性能 (`directory_listing`)

测试不同规模的目录遍历性能：

- **小目录** (10 文件)
- **中等目录** (100 文件)
- **大目录** (1000 文件)

**预期性能:**
- 列表时间应与文件数呈线性关系
- 跳过隐藏文件和特殊文件
- 按类型和名称排序

## 性能优化建议

### 文件服务优化

1. **使用内存映射**：对大文件使用 `memmap2`
2. **零拷贝技术**：避免不必要的数据复制
3. **异步 I/O**：使用 Tokio 的异步文件操作

### 压缩优化

1. **智能压缩**：仅压缩可受益的内容
2. **压缩级别**：平衡压缩率与速度
3. **缓存压缩结果**：对频繁访问的文件

### 路径验证优化

1. **避免系统调用**：仅使用字符串操作
2. **早期拒绝**：快速识别恶意路径
3. **缓存验证结果**：对重复路径

### MIME 检测优化

1. **预编译映射**：使用静态查找表
2. **避免正则表达式**：使用字符串前缀匹配
3. **默认类型**：快速处理未知类型

## 性能指标

### 基准性能目标

| 操作类型 | 预期性能 | 备注 |
|---------|-----------|------|
| 文件读取 (1KB) | < 10 μs | 小文件快速访问 |
| 文件读取 (1MB) | < 10 ms | 大文件高效传输 |
| Gzip 压缩 (100KB) | < 5 ms | 文本内容压缩 |
| Brotli 压缩 (100KB) | < 20 ms | 高压缩率场景 |
| 路径验证 | < 100 ns | 安全检查 |
| MIME 检测 | < 50 ns | 类型识别 |
| ETag 生成 | < 1 μs | 缓存标识 |
| 目录列表 (100 文件) | < 1 ms | 快速遍历 |

### 内存使用目标

- **启动内存**: < 50 MB
- **每连接内存**: < 10 KB
- **空闲内存**: < 20 MB
- **峰值内存**: < 100 MB (1000 并发连接)

### 并发性能目标

- **并发连接数**: 1000+
- **请求吞吐量**: 10,000+ req/s
- **响应时间**: < 10 ms (P95)
- **错误率**: < 0.1%

## 持续性能监控

### CI/CD 集成

```yaml
# .github/workflows/benchmark.yml
name: Benchmarks
on: [push, pull_request]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo bench
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: cargo
```

### 性能回归检测

1. **基准基线**：建立性能基线
2. **自动比较**：每次 PR 比较性能变化
3. **失败阈值**：性能下降 > 5% 时告警
4. **趋势分析**：跟踪长期性能趋势

## 性能调优工具

### Profiling

```bash
# 使用火焰图分析
cargo install flamegraph
cargo flamegraph --bench performance

# 使用 perf (Linux)
perf record -g cargo bench
perf report

# 使用 dtrace (macOS)
dtrace -n 'profile-97 /pid == $target/ { @[ustack()] = count(); }' -c cargo bench
```

### 内存分析

```bash
# 使用 valgrind
valgrind --tool=massif cargo bench

# 使用 heaptrack
heaptrack cargo bench
```

### 网络性能测试

```bash
# 使用 wrk 进行负载测试
wrk -t4 -c100 -d30s http://localhost:8080/

# 使用 ab (Apache Benchmark)
ab -n 10000 -c 100 http://localhost:8080/

# 使用 hey
hey -n 10000 -c 100 http://localhost:8080/
```

## 参考资源

- [Criterion 文档](https://bheisler.github.io/criterion.rs/book/)
- [Rust 性能优化指南](https://nnethercote.github.io/perf-book/)
- [Tokio 性能最佳实践](https://tokio.rs/tokio/topics/performance)
- [HTTP 性能优化](https://developer.mozilla.org/en-US/docs/Web/Performance)

## 贡献

添加新的基准测试：

1. 在 `benches/performance.rs` 中添加基准函数
2. 使用 `black_box()` 防止编译器优化
3. 在相应的 `criterion_group!` 中注册
4. 更新本文档
5. 运行基准测试确保正常工作

基准测试命名约定：
- `bench_<component>`: 功能基准测试
- `bench_<feature>_performance`: 特定功能性能测试
- 使用 `BenchmarkId` 标记不同输入规模
