# Shadowcat Deployment Guide

## Deployment Patterns

### 1. Development Environment (Forward Proxy)

The forward proxy mode is ideal for development and debugging:

```bash
# Local development
shadowcat forward stdio -- npx @modelcontextprotocol/server-everything

# With recording for debugging
shadowcat record --output debug.tape -- forward stdio -- your-mcp-server

# Docker development
docker run -it shadowcat:latest forward stdio -- server-command
```

### 2. Production Environment (Reverse Proxy)

The reverse proxy mode provides authentication and security for production:

```bash
# Basic reverse proxy
shadowcat reverse \
  --bind 0.0.0.0:8080 \
  --upstream http://mcp-server:3000 \
  --auth-config auth.yaml

# With TLS
shadowcat reverse \
  --bind 0.0.0.0:443 \
  --upstream http://mcp-server:3000 \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem \
  --auth-config auth.yaml
```

### 3. Gateway Deployment (Multiple Upstreams)

For high availability with multiple backend servers:

```bash
shadowcat gateway --config gateway.toml
```

## Docker Deployment

### Building the Docker Image

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shadowcat /usr/local/bin/
COPY --from=builder /app/config /etc/shadowcat/

EXPOSE 8080
ENTRYPOINT ["shadowcat"]
```

Build and run:
```bash
docker build -t shadowcat:latest .
docker run -p 8080:8080 shadowcat:latest reverse --config /etc/shadowcat/config.toml
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  shadowcat:
    image: shadowcat:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=shadowcat=info
      - SHADOWCAT_CONFIG=/config/shadowcat.toml
    volumes:
      - ./config:/config
      - ./data:/data
    depends_on:
      - mcp-server
    command: reverse --config /config/shadowcat.toml

  mcp-server:
    image: mcp-server:latest
    expose:
      - "3000"
    environment:
      - NODE_ENV=production
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
# shadowcat-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: shadowcat
  namespace: mcp-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: shadowcat
  template:
    metadata:
      labels:
        app: shadowcat
    spec:
      containers:
      - name: shadowcat
        image: shadowcat:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: RUST_LOG
          value: "shadowcat=info"
        - name: SHADOWCAT_MODE
          value: "reverse"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /config
        - name: data
          mountPath: /data
      volumes:
      - name: config
        configMap:
          name: shadowcat-config
      - name: data
        persistentVolumeClaim:
          claimName: shadowcat-data
```

### Service

```yaml
# shadowcat-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: shadowcat
  namespace: mcp-system
spec:
  selector:
    app: shadowcat
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  type: LoadBalancer
```

### ConfigMap

```yaml
# shadowcat-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: shadowcat-config
  namespace: mcp-system
data:
  shadowcat.toml: |
    [proxy]
    mode = "reverse"
    bind = "0.0.0.0:8080"
    
    [[upstreams]]
    name = "primary"
    url = "http://mcp-server:3000"
    weight = 10
    
    [[upstreams]]
    name = "secondary"
    url = "http://mcp-server-backup:3000"
    weight = 5
    
    [auth]
    enabled = true
    provider = "oauth2"
    
    [rate_limit]
    global = 10000
    per_user = 100
```

## Systemd Service

For Linux deployments:

```ini
# /etc/systemd/system/shadowcat.service
[Unit]
Description=Shadowcat MCP Proxy
After=network.target

[Service]
Type=simple
User=shadowcat
Group=shadowcat
WorkingDirectory=/opt/shadowcat
ExecStart=/opt/shadowcat/bin/shadowcat reverse --config /etc/shadowcat/config.toml
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/shadowcat

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=shadowcat

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable shadowcat
sudo systemctl start shadowcat
sudo systemctl status shadowcat
```

## Configuration Management

### Environment Variables

All configuration can be overridden via environment variables:

```bash
# Proxy configuration
export SHADOWCAT_MODE=reverse
export SHADOWCAT_BIND=0.0.0.0:8080
export SHADOWCAT_UPSTREAM=http://mcp-server:3000

# Authentication
export SHADOWCAT_AUTH_ENABLED=true
export SHADOWCAT_AUTH_PROVIDER=oauth2
export SHADOWCAT_AUTH_CLIENT_ID=your-client-id
export SHADOWCAT_AUTH_CLIENT_SECRET=your-secret

# Telemetry
export SHADOWCAT_TELEMETRY_ENABLED=true
export SHADOWCAT_TELEMETRY_ENDPOINT=http://otel-collector:4317

# Logging
export RUST_LOG=shadowcat=info,tower_http=debug
```

### Configuration File

Complete configuration example:

```toml
# /etc/shadowcat/config.toml

[proxy]
mode = "reverse"
bind = "0.0.0.0:8080"
worker_threads = 4

[[upstreams]]
name = "primary"
url = "http://mcp-server-1:3000"
weight = 10
max_connections = 100
timeout = 30

[[upstreams]]
name = "secondary"
url = "http://mcp-server-2:3000"
weight = 5
max_connections = 50
timeout = 30

[session]
store = "sqlite"
database_path = "/var/lib/shadowcat/sessions.db"
timeout = 300
max_sessions = 10000
cleanup_interval = 60

[auth]
enabled = true
provider = "oauth2"
client_id = "${OAUTH_CLIENT_ID}"
client_secret = "${OAUTH_CLIENT_SECRET}"
auth_url = "https://auth.example.com/oauth2/authorize"
token_url = "https://auth.example.com/oauth2/token"
jwks_url = "https://auth.example.com/.well-known/jwks.json"
redirect_uri = "https://proxy.example.com/callback"
scopes = ["read", "write"]
pkce_required = true

[rate_limit]
enabled = true
global_limit = 10000
global_window = 60
per_user_limit = 100
per_user_window = 60
burst_size = 20

[telemetry]
enabled = true
service_name = "shadowcat"
otlp_endpoint = "http://otel-collector:4317"
sampling_ratio = 0.1

[tls]
cert_path = "/etc/shadowcat/tls/cert.pem"
key_path = "/etc/shadowcat/tls/key.pem"
client_ca_path = "/etc/shadowcat/tls/ca.pem"
require_client_cert = false

[health_check]
enabled = true
path = "/health"
interval = 10
timeout = 5
unhealthy_threshold = 3
healthy_threshold = 2

[circuit_breaker]
enabled = true
failure_threshold = 5
recovery_timeout = 60
failure_ratio = 0.5
```

## Monitoring & Observability

### Health Checks

Shadowcat provides health check endpoints:

```bash
# Liveness check
curl http://localhost:8080/health

# Readiness check
curl http://localhost:8080/ready

# Metrics (Prometheus format)
curl http://localhost:8080/metrics
```

### Logging

Configure structured logging:

```toml
[logging]
level = "info"
format = "json"
output = "stdout"

# Or use environment variables
RUST_LOG=shadowcat=debug,tower_http=trace
```

### Metrics

Shadowcat exports metrics in Prometheus format:

```prometheus
# Session metrics
shadowcat_sessions_active{transport="http"} 42
shadowcat_sessions_total{transport="http"} 1234
shadowcat_session_duration_seconds{quantile="0.99"} 300

# Proxy metrics
shadowcat_proxy_requests_total{upstream="primary"} 10000
shadowcat_proxy_request_duration_seconds{upstream="primary",quantile="0.99"} 0.150
shadowcat_proxy_errors_total{upstream="primary",error="timeout"} 5

# Rate limit metrics
shadowcat_rate_limit_exceeded_total{key="global"} 10
shadowcat_rate_limit_remaining{key="user:alice"} 45
```

### Distributed Tracing

Configure OpenTelemetry:

```toml
[telemetry]
enabled = true
service_name = "shadowcat"
otlp_endpoint = "http://jaeger:4317"
sampling_ratio = 0.1

[telemetry.resource_attributes]
environment = "production"
region = "us-west-2"
version = "0.2.0"
```

## Security Hardening

### Network Security

```bash
# Bind to localhost only (development)
shadowcat reverse --bind 127.0.0.1:8080

# Use TLS in production
shadowcat reverse \
  --bind 0.0.0.0:443 \
  --tls-cert cert.pem \
  --tls-key key.pem \
  --tls-min-version 1.3
```

### File Permissions

```bash
# Create dedicated user
sudo useradd -r -s /bin/false shadowcat

# Set permissions
sudo chown -R shadowcat:shadowcat /opt/shadowcat
sudo chmod 750 /opt/shadowcat
sudo chmod 640 /etc/shadowcat/config.toml
sudo chmod 400 /etc/shadowcat/tls/key.pem
```

### Resource Limits

```yaml
# Kubernetes resource limits
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "1000m"
```

```ini
# Systemd resource limits
[Service]
MemoryLimit=512M
CPUQuota=100%
TasksMax=100
```

## Backup & Recovery

### Session Backup

```bash
# Backup SQLite sessions
sqlite3 /var/lib/shadowcat/sessions.db ".backup /backup/sessions-$(date +%Y%m%d).db"

# Backup tape recordings
tar -czf /backup/tapes-$(date +%Y%m%d).tar.gz /var/lib/shadowcat/tapes/
```

### Configuration Backup

```bash
# Backup configuration
cp -r /etc/shadowcat /backup/config-$(date +%Y%m%d)/

# Include in version control
git add config/
git commit -m "backup: configuration snapshot"
git push
```

## Performance Tuning

### System Tuning

```bash
# Increase file descriptors
ulimit -n 65536

# TCP tuning
sysctl -w net.core.somaxconn=65535
sysctl -w net.ipv4.tcp_max_syn_backlog=65535
sysctl -w net.ipv4.ip_local_port_range="1024 65535"
```

### Proxy Tuning

```toml
[performance]
worker_threads = 8
max_connections = 10000
connection_timeout = 30
keep_alive_timeout = 60
buffer_size = 65536

[pool]
min_idle = 10
max_size = 100
idle_timeout = 300
```

## Troubleshooting

### Common Issues

#### High Memory Usage
```bash
# Check memory usage
ps aux | grep shadowcat
top -p $(pgrep shadowcat)

# Analyze heap
heaptrack shadowcat
```

#### Connection Issues
```bash
# Check open connections
ss -tulpn | grep shadowcat
netstat -an | grep 8080

# Test connectivity
curl -v http://localhost:8080/health
```

#### Performance Issues
```bash
# Enable debug logging
RUST_LOG=shadowcat=trace shadowcat reverse

# Profile with flamegraph
cargo flamegraph --release --bin shadowcat
```

### Logs Location

- **Systemd**: `journalctl -u shadowcat -f`
- **Docker**: `docker logs shadowcat`
- **Kubernetes**: `kubectl logs -n mcp-system deployment/shadowcat`

## Maintenance

### Rolling Updates

```bash
# Kubernetes rolling update
kubectl set image deployment/shadowcat shadowcat=shadowcat:v0.3.0 -n mcp-system
kubectl rollout status deployment/shadowcat -n mcp-system

# Docker Swarm
docker service update --image shadowcat:v0.3.0 shadowcat
```

### Database Maintenance

```bash
# Vacuum SQLite database
sqlite3 /var/lib/shadowcat/sessions.db "VACUUM;"

# Clean old sessions
shadowcat admin cleanup-sessions --older-than 7d
```

## Disaster Recovery

### Backup Strategy

1. **Configuration**: Version controlled in Git
2. **Sessions**: Daily SQLite backups
3. **Tapes**: Weekly archives to object storage
4. **Secrets**: Encrypted backup to secure storage

### Recovery Procedure

1. Deploy new instance
2. Restore configuration
3. Restore session database
4. Restore tape recordings
5. Update DNS/load balancer
6. Verify health checks