# TLS/HTTPS 支持

## 概述

rust-serv 支持 HTTPS/TLS 加密连接，提供安全的文件传输。TLS 实现基于 Rustls，这是一个现代化的、内存安全的 TLS 库。

## 配置

### 启用 TLS

在配置文件 `config.toml` 中启用 TLS：

```toml
enable_tls = true
```

### 指定证书和密钥

TLS 需要证书文件和私钥文件：

```toml
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
```

### 完整配置示例

```toml
port = 443                    # HTTPS 默认端口
root = "./public"
enable_indexing = true
enable_compression = true
enable_tls = true            # 启用 TLS
tls_cert = "./cert.pem"      # 证书文件路径
tls_key = "./key.pem"       # 私钥文件路径
log_level = "info"
```

## 证书生成

### 使用 OpenSSL 生成证书

```bash
# 生成私钥
openssl genrsa -out key.pem 2048

# 生成自签名证书
openssl req -new -key key.pem -out cert.pem -days 365 -x509 \
  -subj "/C=CN/ST=State/L=City/O=Organization/CN=localhost"
```

### 使用 Certbot (Let's Encrypt)

```bash
# 安装 certbot
sudo apt-get install certbot

# 生成证书
sudo certbot certonly --standalone -d yourdomain.com

# 证书通常位于 /etc/letsencrypt/live/yourdomain.com/
```

### 生成开发测试证书

```bash
# 使用 OpenSSL 生成自签名证书
openssl req -x509 -newkey rsa:2048 -keyout key.pem \
  -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,DNS:127.0.0.1"
```

## 使用 HTTPS 运行服务器

### 1. 准备证书文件

确保证书文件位于服务器可访问的位置：

```bash
# 验证文件存在
ls -la cert.pem key.pem
```

### 2. 启动 HTTPS 服务器

```bash
# 使用 TLS 配置启动
cargo run --config.toml

# 或使用命令行参数（如果支持）
cargo run -- --enable-tls --cert cert.pem --key key.pem
```

### 3. 验证 HTTPS 连接

```bash
# 使用 curl 测试 HTTPS 连接
curl -k https://localhost:443/

# 使用浏览器访问
https://localhost:443/
```

**注意**: 使用自签名证书时，浏览器会显示安全警告，需要手动信任证书。

## 客户端证书验证

### 生产环境

在生产环境中，使用受信任的证书颁发机构（CA）签名的证书，如 Let's Encrypt、DigiCert 等。

### 开发环境

在开发环境中，可以：
- 使用自签名证书并让客户端忽略证书错误
- 使用 `-k` 参数（curl）或 `--insecure` 参数
- 将自签名证书添加到浏览器/操作系统的信任存储

## TLS 配置验证

### 配置检查

服务器启动时会自动验证：

1. **证书文件存在性** - 证书文件必须存在
2. **私钥文件存在性** - 私钥文件必须存在
3. **文件可读性** - 两个文件都必须可读
4. **同时配置** - `tls_cert` 和 `tls_key` 必须同时配置或同时省略

### 错误处理

如果 TLS 配置无效，服务器会在启动时返回详细错误：

```
Error: Certificate file not found: /path/to/cert.pem
Error: Private key file not found: /path/to/key.pem
Error: Both tls_cert and tls_key must be specified together or both omitted
```

## 安全建议

### 证书管理

1. **定期更新证书** - 证书通常有有效期（如 90 天）
2. **使用强加密算法** - 推荐使用 RSA 2048 位或更长
3. **保护私钥** - 私钥文件权限应设为只读，仅限特定用户访问
4. **证书撤销** - 配置 CRL（证书撤销列表）或 OCSP（在线证书状态协议）

### 加密套件选择

rust-serv 默认使用安全配置：

- **协议** - TLS 1.2 和 TLS 1.3
- **加密算法** - AES-256-GCM, ChaCha20-Poly1305
- **密钥交换** - ECDHE（椭圆曲线 Diffie-Hellman）
- **签名算法** - ECDSA（椭圆曲线数字签名算法）

### HTTPS 最佳实践

1. **强制 HTTPS** - 在生产环境中，配置反向代理（如 Nginx）强制 HTTPS
2. **HSTS 头** - 启用 HSTS（HTTP Strict Transport Security）
3. **证书固定** - 考虑证书固定以防止中间人攻击
4. **安全重定向** - 将 HTTP 重定向到 HTTPS

## 性能考虑

### TLS 性能优化

1. **会话复用** - rustls 自动支持 TLS 会话复用
2. **OCSP 装订** - 提前获取证书状态
3. **HTTP/2** - TLS 1.3 支持 HTTP/2（未来特性）
4. **硬件加速** - 使用支持的硬件加密加速

### 性能基准

| 操作 | HTTP | HTTPS | 差异 |
|------|------|--------|------|
| 连接建立 | 快 | 慢 (+10-50ms) | TLS 握手 |
| 数据传输 | 快 | 慢 (+2-5%) | 加密开销 |
| CPU 使用 | 低 | 高 (+10-20%) | 加密计算 |
| 内存使用 | 低 | 中 (+10MB) | TLS 会话 |

## 故障排除

### 常见问题

#### 1. "Permission denied" 错误

```bash
# 检查文件权限
ls -l cert.pem key.pem

# 修复权限
chmod 600 key.pem    # 仅所有者可读
chmod 644 cert.pem   # 所有人可读
```

#### 2. "Address already in use" 错误

```bash
# 检查端口占用
sudo netstat -tulpn | grep :443
sudo lsof -i :443

# 杀死占用端口的进程
sudo kill -9 <PID>
```

#### 3. "Handshake failed" 错误

可能原因：
- 证书格式错误（PEM vs DER）
- 证书和密钥不匹配
- 证书已过期
- 加密算法不兼容

检查方法：
```bash
# 验证证书格式
openssl x509 -in cert.pem -text -noout

# 验证密钥匹配
openssl x509 -in cert.pem -noout -modulus | openssl md5
openssl rsa -in key.pem -noout -modulus | openssl md5
```

#### 4. 浏览器显示 "不安全" 警告

原因：
- 使用自签名证书
- 证书域名不匹配
- 证书已过期

解决方案：
- 使用受信任的 CA 签发证书
- 添加例外到浏览器
- 将证书添加到操作系统信任存储

## 调试

### 启用 TLS 调试日志

```toml
log_level = "debug"
```

### 使用 OpenSSL 调试

```bash
# 测试 TLS 连接
openssl s_client -connect localhost:443 -cert cert.pem -key key.pem

# 显示握手信息
openssl s_client -connect localhost:443 -showcerts
```

### 日志分析

查找关键日志：

- `TLS handshake failed` - 握手失败
- `Certificate file not found` - 证书文件缺失
- `Failed to build TLS config` - 配置错误

## 部署建议

### 生产部署

1. **使用权威证书** - Let's Encrypt 免费且受信任
2. **自动化续期** - 配置 certbot 自动续期
3. **反向代理** - 在服务器前使用 Nginx/Apache
4. **监控** - 监控证书过期时间和 TLS 错误

### 开发部署

1. **自签名证书** - 便于本地开发
2. **禁用浏览器验证** - 开发工具中设置安全例外
3. **快速重启** - 开发环境频繁重启，证书缓存有帮助

## 相关资源

- [Rustls 文档](https://docs.rs/rustls/)
- [Let's Encrypt](https://letsencrypt.org/)
- [SSL Labs SSL 测试](https://www.ssllabs.com/ssltest/)
- [Mozilla SSL 配置生成器](https://ssl-config.mozilla.org/)
- [OpenSSL 文档](https://www.openssl.org/docs/)

## 示例配置文件

### 基础 HTTPS 配置

```toml
# config.toml - Basic HTTPS configuration
port = 443
root = "./public"
enable_indexing = true
enable_compression = true
enable_tls = true
tls_cert = "./cert.pem"
tls_key = "./key.pem"
log_level = "info"
connection_timeout_secs = 30
max_connections = 1000
```

### Let's Encrypt 集成

```toml
# config.toml - Let's Encrypt integration
port = 443
root = "/var/www/public"
enable_indexing = true
enable_compression = true
enable_tls = true
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
tls_key = "/etc/letsencrypt/live/example.com/privkey.pem"
log_level = "info"
```

### 开发环境配置

```toml
# config.toml - Development with self-signed cert
port = 8443                          # 避免需要 root 权限
root = "./public"
enable_indexing = true
enable_compression = true
enable_tls = true
tls_cert = "./dev-cert.pem"
tls_key = "./dev-key.pem"
log_level = "debug"                    # 详细调试日志
connection_timeout_secs = 60
max_connections = 100
```
