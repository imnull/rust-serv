# 测试覆盖率提升计划 - 目标 95%+

## 当前状态

| 模块 | 当前行覆盖 | 目标 |
|------|-----------|------|
| server/server.rs | 83.58% | 95%+ |
| server/http2.rs | 82.35% | 95%+ |
| server/websocket.rs | 83.26% | 95%+ |
| server/tls.rs | 90.18% | 95%+ |
| handler/handler.rs | 92.55% | 95%+ |
| handler/compress.rs | 93.70% | 95%+ |
| handler/range.rs | 96.81% | ✅ |
| middleware/security.rs | 85.03% | 95%+ |
| path_security/validator.rs | 92.47% | 95%+ |

---

## 实施计划

### 阶段 1: server/server.rs (83.58% → 95%+)

**文件**: `src/server/server.rs`

**需要添加的测试**:

1. `test_serve_connection_success` - 测试连接服务成功
2. `test_serve_connection_error` - 测试连接服务错误处理
3. `test_run_without_tls` - 测试无 TLS 运行
4. `test_run_with_valid_tls` - 测试带有效 TLS 运行
5. `test_connection_limit_enforcement` - 测试连接限制执行
6. `test_connection_timeout` - 测试连接超时
7. `test_concurrent_connections` - 测试并发连接

### 阶段 2: server/http2.rs (82.35% → 95%+)

**文件**: `src/server/http2.rs`

**需要添加的测试**:

1. `test_push_with_invalid_content_length` - 无效 Content-Length
2. `test_client_push_wrong_accept_encoding` - 错误的 Accept-Encoding
3. `test_create_response_builder_failure` - 响应构建失败
4. `test_handle_push_empty_headers` - 空头处理
5. `test_priority_ordering_edge_cases` - 优先级排序边缘情况

### 阶段 3: server/websocket.rs (83.26% → 95%+)

**文件**: `src/server/websocket.rs`

**需要添加的测试**:

1. `test_handshake_with_invalid_header_value` - 无效头值
2. `test_send_to_connection_failure` - 发送失败
3. `test_broadcast_partial_failure` - 部分广播失败
4. `test_connection_cleanup_on_error` - 错误时连接清理
5. `test_message_conversion_edge_cases` - 消息转换边缘情况

### 阶段 4: server/tls.rs (90.18% → 95%+)

**文件**: `src/server/tls.rs`

**需要添加的测试**:

1. `test_load_tls_config_success` - 成功加载 TLS 配置
2. `test_load_tls_config_with_chain` - 加载证书链

### 阶段 5: handler/handler.rs (92.55% → 95%+)

**文件**: `src/handler/handler.rs`

**需要添加的测试**:

1. `test_url_decode_failure` - URL 解码失败
2. `test_directory_without_index_and_indexing_disabled` - 禁用索引
3. `test_range_request_parse_error` - Range 解析错误
4. `test_compression_failure_fallback` - 压缩失败回退
5. `test_cache_control_headers` - 缓存控制头

### 阶段 6: handler/compress.rs (93.70% → 95%+)

**文件**: `src/handler/compress.rs`

**需要添加的测试**:

1. `test_parse_accept_encoding_case_insensitive` - 大小写不敏感
2. `test_parse_accept_encoding_with_unknown` - 未知编码器
3. `test_should_skip_compression_no_extension` - 无扩展名
4. `test_compress_empty_data` - 空数据压缩

### 阶段 7: middleware/security.rs (85.03% → 95%+)

**文件**: `src/middleware/security.rs`

**需要添加的测试**:

1. `test_rate_limit_concurrent_requests` - 并发速率限制
2. `test_rate_limit_time_window_boundary` - 时间窗口边界
3. `test_ip_ipv4_mapped_to_ipv6` - IPv4 映射到 IPv6
4. `test_request_header_size_limit` - 请求头大小限制
5. `test_security_header_invalid_value` - 无效安全头值
6. `test_cleanup_expired_states` - 清理过期状态

### 阶段 8: path_security/validator.rs (92.47% → 95%+)

**文件**: `src/path_security/validator.rs`

**需要添加的测试**:

1. `test_symlink_outside_root` - 符号链接指向外部
2. `test_canonicalize_permission_denied` - 权限拒绝
3. `test_empty_path_handling` - 空路径处理
4. `test_multiple_slashes` - 多个斜杠

---

## 关键文件

- `src/server/server.rs`
- `src/server/http2.rs`
- `src/server/websocket.rs`
- `src/server/tls.rs`
- `src/handler/handler.rs`
- `src/handler/compress.rs`
- `src/middleware/security.rs`
- `src/path_security/validator.rs`

---

## 验证方法

1. 运行 `cargo llvm-cov --workspace --all-features` 查看覆盖率
2. 确保所有测试通过: `cargo test --all`
3. 检查每个模块覆盖率是否达到 95%+
