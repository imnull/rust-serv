# Expert Guide

**English** | [中文](../expert-guide.md)

---

> Production deployment, performance tuning, and troubleshooting

## 🎯 Learning Objectives

- ✅ Production environment deployment
- ✅ Kubernetes integration
- ✅ Performance tuning
- ✅ High availability architecture
- ✅ Troubleshooting
- ✅ Security hardening

---

## 1. Production Deployment

### System Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 1 Core | 2+ Cores |
| Memory | 512 MB | 2+ GB |
| Disk | 1 GB | 10+ GB |
| Network | 10 Mbps | 100+ Mbps |

### Systemd Service

Create `/etc/systemd/system/rust-serv.service`:

```ini
[Unit]
Description=Rust HTTP Static Server
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/rust-serv
ExecStart=/usr/local/bin/rust-serv /etc/rust-serv/config.toml
Restart=on-failure
RestartSec=5s

LimitNOFILE=65535
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Manage service:
```bash
sudo systemctl start rust-serv
sudo systemctl enable rust-serv
sudo systemctl status rust-serv
```

---

## 2. Kubernetes Deployment

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-serv
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-serv
  template:
    spec:
      containers:
      - name: rust-serv
        image: ghcr.io/imnull/rust-serv:0.3.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 1000m
            memory: 1Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
```

---

## 3. Performance Tuning

### OS Optimization

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
fs.file-max = 2097152
```

Apply:
```bash
sudo sysctl -p
```

### Application Tuning

```toml
max_connections = 10000
connection_timeout_secs = 60

[memory_cache]
enabled = true
max_entries = 100000
max_size_mb = 500
```

### Load Testing

```bash
# Using wrk
wrk -t12 -c400 -d30s http://localhost:8080/

# Using ab
ab -n 10000 -c 100 http://localhost:8080/
```

---

## 4. Monitoring & Alerting

### Prometheus Integration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'rust-serv'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: /metrics
```

### Alert Rules

```yaml
groups:
  - name: rust-serv
    rules:
      - alert: HighErrorRate
        expr: rate(rust_serv_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
```

---

## 5. Troubleshooting

### Log Analysis

```bash
# Check error logs
grep ERROR /var/log/rust-serv/app.log

# Monitor in real-time
tail -f /var/log/rust-serv/app.log | grep --color=auto "ERROR\|WARN"

# Status code statistics
cat access.log | awk '{print $9}' | sort | uniq -c | sort -nr
```

### Performance Analysis

```bash
# CPU analysis
perf top -p $(pgrep rust-serv)

# Connection count
ss -s

# Port usage
lsof -i :8080
```

---

## 6. Security Hardening

### Firewall

```bash
# UFW
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### Rate Limiting

```toml
[throttle]
enabled = true
global_limit_bps = 10485760
per_ip_limit_bps = 1048576

[security]
enable_rate_limit = true
rate_limit_max_requests = 100
rate_limit_window_secs = 60
```

### IP Access Control

```toml
[security]
ip_allowlist = ["192.168.1.0/24"]
ip_blocklist = ["10.0.0.100"]
```

---

## 💡 Best Practices

1. **Monitoring First**: Configure monitoring before deployment
2. **Gradual Rollout**: Canary deployment, gradually increase traffic
3. **Regular Drills**: Fault simulation, validate recovery procedures
4. **Documentation**: Record all configurations and changes
5. **Automation**: CI/CD, auto-scaling, automated backups

---

## 📚 References

- [Performance Tuning Best Practices](https://rust-lang-nursery.github.io/cli-wg/in-production/docs.html)
- [Kubernetes Official Docs](https://kubernetes.io/docs/)
- [Prometheus Monitoring Guide](https://prometheus.io/docs/)
