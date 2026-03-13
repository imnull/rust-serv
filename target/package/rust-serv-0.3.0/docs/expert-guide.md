# 高级指南

[English](./en/expert-guide.md) | **中文**

---

> 生产环境部署、性能优化、故障排查

## 📖 前置知识

- 已完成[进阶指南](./advanced-guide.md)
- 熟悉 Linux 系统管理
- 了解容器化和编排
- 掌握性能调优方法

---

## 🎯 本章目标

- ✅ 生产环境部署
- ✅ Kubernetes 集成
- ✅ 性能调优
- ✅ 高可用架构
- ✅ 故障排查
- ✅ 安全加固

---

## 1. 生产环境部署

### 1.1 系统要求

| 资源 | 最低 | 推荐 |
|------|------|------|
| CPU | 1 核 | 2+ 核 |
| 内存 | 512 MB | 2+ GB |
| 磁盘 | 1 GB | 10+ GB |
| 网络 | 10 Mbps | 100+ Mbps |

### 1.2 Systemd 服务

创建 `/etc/systemd/system/rust-serv.service`：

```ini
[Unit]
Description=Rust HTTP Static Server
Documentation=https://github.com/imnull/rust-serv
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/rust-serv
ExecStart=/usr/local/bin/rust-serv /etc/rust-serv/config.toml
Restart=on-failure
RestartSec=5s

# 资源限制
LimitNOFILE=65535
LimitNPROC=4096

# 安全加固
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/rust-serv/logs /opt/rust-serv/certs

# 环境变量
Environment=RUST_LOG=info
Environment=ROCKET_ENV=production

[Install]
WantedBy=multi-user.target
```

管理服务：
```bash
# 启动
sudo systemctl start rust-serv

# 停止
sudo systemctl stop rust-serv

# 重启
sudo systemctl restart rust-serv

# 开机自启
sudo systemctl enable rust-serv

# 查看状态
sudo systemctl status rust-serv

# 查看日志
sudo journalctl -u rust-serv -f
```

### 1.3 Nginx 反向代理

```nginx
# /etc/nginx/conf.d/rust-serv.conf
upstream rust_serv {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name example.com;
    
    # 重定向到 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name example.com;
    
    # SSL 配置
    ssl_certificate /etc/letsencrypt/live/example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # 安全头
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # 静态文件（高性能）
    location /static/ {
        proxy_pass http://rust_serv;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_cache static_cache;
        proxy_cache_valid 200 1d;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
    
    # API 代理
    location /api/ {
        proxy_pass http://rust_serv;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # 健康检查
    location /health {
        proxy_pass http://rust_serv;
        access_log off;
    }
}
```

---

## 2. Kubernetes 部署

### 2.1 Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-serv
  labels:
    app: rust-serv
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-serv
  template:
    metadata:
      labels:
        app: rust-serv
    spec:
      containers:
      - name: rust-serv
        image: ghcr.io/imnull/rust-serv:0.3.0
        ports:
        - containerPort: 8080
        
        # 资源限制
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 1000m
            memory: 1Gi
        
        # 健康检查
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        
        # 环境变量
        env:
        - name: RUST_LOG
          value: "info"
        
        # 挂载配置
        volumeMounts:
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
        - name: static-files
          mountPath: /app/public
      
      volumes:
      - name: config
        configMap:
          name: rust-serv-config
      - name: static-files
        persistentVolumeClaim:
          claimName: static-files-pvc
```

### 2.2 Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: rust-serv
spec:
  selector:
    app: rust-serv
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### 2.3 Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rust-serv
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - example.com
    secretName: rust-serv-tls
  rules:
  - host: example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: rust-serv
            port:
              number: 80
```

### 2.4 ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rust-serv-config
data:
  config.toml: |
    port = 8080
    root = "/app/public"
    enable_compression = true
    
    [management]
    enabled = true
    
    [metrics]
    enabled = true
    
    [memory_cache]
    enabled = true
    max_entries = 10000
```

### 2.5 HorizontalPodAutoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-serv-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-serv
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 3. 性能调优

### 3.1 操作系统优化

```bash
# /etc/sysctl.conf

# 网络优化
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 3
net.ipv4.tcp_keepalive_intvl = 30

# 文件描述符
fs.file-max = 2097152
fs.nr_open = 2097152

# 内存
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
```

应用配置：
```bash
sudo sysctl -p
```

### 3.2 应用层优化

```toml
# config.toml

# 连接管理
max_connections = 10000
connection_timeout_secs = 60

# 缓存优化
[memory_cache]
enabled = true
max_entries = 100000        # 增大缓存
max_size_mb = 500
ttl_secs = 3600

# 压缩
enable_compression = true

# HTTP/2
enable_http2 = true

# 带宽
[throttle]
enabled = false  # 高性能场景可关闭
```

### 3.3 压力测试

使用 wrk 测试：
```bash
# 安装 wrk
git clone https://github.com/wg/wrk.git
cd wrk && make

# 测试
wrk -t12 -c400 -d30s http://localhost:8080/

# 结果示例
Running 30s test @ http://localhost:8080/
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    15.32ms   10.21ms 150.00ms   75.23%
    Req/Sec    2.15k     321.45     3.20k    68.50%
  772334 requests in 30.01s, 1.23GB read
Requests/sec:  25735.21
Transfer/sec:     42.00MB
```

使用 ab 测试：
```bash
ab -n 10000 -c 100 http://localhost:8080/

# 关键指标
Requests per second:    25432.21 [#/sec] (mean)
Time per request:       3.932 [ms] (mean)
```

---

## 4. 监控与告警

### 4.1 Prometheus 集成

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'rust-serv'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: /metrics
```

### 4.2 Grafana Dashboard

导入 Dashboard：
```json
{
  "title": "rust-serv Dashboard",
  "panels": [
    {
      "title": "QPS",
      "targets": [{
        "expr": "rate(rust_serv_requests_total[1m])"
      }]
    },
    {
      "title": "Latency P99",
      "targets": [{
        "expr": "histogram_quantile(0.99, rust_serv_request_duration_seconds_bucket)"
      }]
    },
    {
      "title": "Cache Hit Rate",
      "targets": [{
        "expr": "rust_serv_cache_hit_rate"
      }]
    }
  ]
}
```

### 4.3 告警规则

```yaml
# alerts.yml
groups:
  - name: rust-serv
    rules:
      - alert: HighErrorRate
        expr: rate(rust_serv_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate"
          
      - alert: HighLatency
        expr: histogram_quantile(0.99, rust_serv_request_duration_seconds_bucket) > 1
        for: 5m
        annotations:
          summary: "High latency P99"
```

---

## 5. 故障排查

### 5.1 日志分析

```bash
# 查看错误日志
grep ERROR /var/log/rust-serv/app.log

# 实时监控
tail -f /var/log/rust-serv/app.log | grep --color=auto "ERROR\|WARN"

# 统计状态码
cat access.log | awk '{print $9}' | sort | uniq -c | sort -nr
```

### 5.2 性能分析

```bash
# CPU 分析
perf top -p $(pgrep rust-serv)

# 内存分析
valgrind --tool=massif ./rust-serv

# 火焰图
perf record -g -p $(pgrep rust-serv) -- sleep 60
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### 5.3 网络诊断

```bash
# 连接数
ss -s

# 端口占用
lsof -i :8080

# 抓包
tcpdump -i any port 8080 -w capture.pcap
```

---

## 6. 安全加固

### 6.1 防火墙

```bash
# UFW
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# iptables
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT
iptables -A INPUT -j DROP
```

### 6.2 速率限制

```toml
[throttle]
enabled = true
global_limit_bps = 10485760    # 10 MB/s
per_ip_limit_bps = 1048576     # 1 MB/s

[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60
```

### 6.3 IP 黑白名单

```toml
[security]
ip_allowlist = ["192.168.1.0/24"]
ip_blocklist = ["10.0.0.100"]
```

### 6.4 安全头

自动添加：
```http
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
```

---

## 7. 备份与恢复

### 7.1 配置备份

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d)
tar -czf rust-serv-backup-$DATE.tar.gz \
  /etc/rust-serv/ \
  /opt/rust-serv/certs/
  
# 保留 7 天
find /backups -name "*.tar.gz" -mtime +7 -delete
```

### 7.2 自动备份

```bash
# crontab -e
0 2 * * * /opt/scripts/backup.sh
```

---

## 📚 参考资料

- [性能调优最佳实践](https://rust-lang-nursery.github.io/cli-wg/in-production/docs.html)
- [Kubernetes 官方文档](https://kubernetes.io/docs/)
- [Prometheus 监控指南](https://prometheus.io/docs/)

---

## 💡 最佳实践

1. **监控先行**：部署前先配置监控
2. **渐进式上线**：灰度发布，逐步放开流量
3. **定期演练**：故障模拟，验证恢复流程
4. **文档完善**：记录所有配置和变更
5. **自动化**：CI/CD、自动扩缩容、自动备份
