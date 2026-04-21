# Admin API - Troubleshooting & Configuration Examples

## Troubleshooting Guide

### 1. Server Won't Start

#### Issue: Port Already in Use
```
Error: Address already in use (os error 98)
```

**Solution:**
```bash
# Find process using port 8080
lsof -i :8080
# or
netstat -tlnp | grep 8080

# Kill the process
kill -9 <PID>

# OR use different port
export ADMIN_API_ADDR="127.0.0.1:8081"
cargo run
```

#### Issue: Permission Denied
```
Error: Permission denied (os error 13)
```

**Solution:**
```bash
# Check file permissions
ls -la admin.db
ls -la .

# Fix permissions
chmod 755 .
chmod 644 admin.db

# Or run with proper user
sudo -u admin-user cargo run
```

#### Issue: Database Lock
```
Error: database is locked
```

**Solution:**
```bash
# Close other connections
sqlite3 admin.db ".quit"

# Remove lock file if exists
rm admin.db-wal
rm admin.db-shm

# Reset/backup database
mv admin.db admin.db.backup
# Server will recreate on startup
```

### 2. Broker Connection Issues

#### Issue: Cannot Connect to Broker
```
Error: Connection refused
Error: Failed to connect to broker
```

**Solution:**
```bash
# Verify broker is running
curl http://localhost:6000/health

# If broker not running, start it
# From FastDataBroker root:
cargo run --release

# Or check if using custom broker URL
export BROKER_URL="http://broker-prod:6000"

# Test broker connectivity
curl -v http://$BROKER_URL/health
```

#### Issue: Broker Timeout
```
Error: Broker request timeout
Error: Connection timed out
```

**Solution:**
```bash
# Test broker response time
time curl http://localhost:6000/health

# If slow, check broker performance:
# - Memory usage: ps aux | grep broker
# - Connection count: ss -tnap | grep broker

# Increase timeout in broker.rs (current: 5 seconds)
// In src/broker.rs, find:
let timeout = Duration::from_secs(5);
// Change to:
let timeout = Duration::from_secs(10);
```

### 3. Database Issues

#### Issue: Database Corruption
```
Error: database disk image is malformed
Error: SQL parsing error
```

**Solution:**
```bash
# Backup corrupted database
cp admin.db admin.db.corrupted

# Try repair
sqlite3 admin.db "PRAGMA integrity_check;"

# If not repairable, delete and let server recreate
rm admin.db
# Restart: Admin API will recreate schema

# Restore from backup if available
cp admin.db.backup admin.db
```

#### Issue: Out of Disk Space
```
Error: disk I/O error
Error: database or disk is full
```

**Solution:**
```bash
# Check disk space
df -h

# Find large files
du -sh *
du -sh ./*

# Delete old backups
ls -lh *.backup | head -5
rm oldest_backup.db

# Alternative: Move database to different disk
mv admin.db /mnt/larger-disk/admin.db
export ADMIN_DB_PATH="/mnt/larger-disk/admin.db"
```

### 4. API Endpoint Issues

#### Issue: 404 Not Found on Valid ID
```
GET /api/v1/tenants/tenant_abc123 → 404 Not Found
```

**Solution:**
```bash
# Verify tenant exists
curl http://localhost:8080/api/v1/tenants | jq '.[] | select(.tenant_id == "tenant_abc123")'

# Check for typos in tenant ID
curl http://localhost:8080/api/v1/tenants | jq '.[] | .tenant_id'

# If not found, tenant may have been deleted
# Recreate tenant:
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "New Name", "email": "new@example.com"}'
```

#### Issue: 409 Conflict on Create
```
POST /api/v1/tenants → 409 Conflict (duplicate email)
```

**Solution:**
```bash
# Check for existing tenant with same email
curl http://localhost:8080/api/v1/tenants | jq '.[] | select(.email == "test@example.com")'

# Use different email
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "Test", "email": "unique-email@example.com"}'

# OR delete existing first
# Get tenant_id with that email
TENANT_ID=$(curl -s http://localhost:8080/api/v1/tenants | \
  jq -r '.[] | select(.email == "test@example.com") | .tenant_id')
curl -X DELETE http://localhost:8080/api/v1/tenants/$TENANT_ID
```

### 5. Configuration Issues

#### Issue: Configuration Not Persisting
```
Update config → GET returns old values
```

**Solution:**
```bash
# Verify update was successful (200 response)
curl -X PUT http://localhost:8080/api/v1/system/config \
  -H "Content-Type: application/json" \
  -d '{...}' \
  -w "\nHTTP Status: %{http_code}\n"

# Check database directly
sqlite3 admin.db "SELECT * FROM system_config;"

# If not in DB, restart server:
# Updates might be cached in memory
```

### 6. Performance Issues

#### Issue: High Memory Usage
```
Admin API using > 100MB
```

**Solution:**
```bash
# Check memory usage
ps aux | grep admin-api | grep -v grep

# Identify memory leaks (monitor over time)
watch -n 1 'ps aux | grep admin-api'

# Reduce connection pool size in config.rs:
max_connections: 5  # Was 10

# Limit number of concurrent requests in main.rs:
.app_data(web::JsonConfig::default()
  .limit(4096)  // Reduce from 8192 if needed
)

# Restart with reduced footprint
cargo build --release
./target/release/admin-api
```

#### Issue: High Response Latency
```
Typical latency > 100ms instead of < 50ms
```

**Solution:**
```bash
# Profile response times
time curl http://localhost:8080/health

# Check broker latency
time curl http://localhost:6000/health

# Check database performance
sqlite3 admin.db
> .timer on
> SELECT COUNT(*) FROM tenants;

# Optimize queries: add indexes if needed
sqlite3 admin.db
> CREATE INDEX idx_tenant_email ON tenants(email);
> CREATE INDEX idx_cluster_name ON cluster_environments(name);
```

### 7. SSL/TLS Issues

#### Issue: Certificate Validation Failed
```
Error: TLS certificate problem
```

**Solution:**
```bash
# For development (disable cert verification):
curl -k https://localhost:8080/health

# For production, use valid certificate:
# Update in main.rs to enable SSL
// Add rustls dependency to Cargo.toml
actix-web-rustls = "0.6"

// In main.rs:
use actix_web_rustls::Acceptor;
let cert = load_ssl_certificate("cert.pem");
let key = load_ssl_private_key("key.pem");
```

### 8. Log Analysis

#### Issue: No Logs
```
Cannot find admin-api logs
```

**Solution:**
```bash
# Capture logs to file
RUST_LOG=debug cargo run > admin-api.log 2>&1

# Or configure logging in config.rs:
env_logger::Builder::from_env(
  env_logger::Env::default()
    .default_filter_or("debug")
).init();

# Check log output
tail -f admin-api.log

# Filter by level
grep "ERROR" admin-api.log
```

## Configuration Examples

### Development Configuration
```bash
# .env or shell export
ADMIN_API_ADDR="127.0.0.1:8080"
BROKER_URL="http://localhost:6000"
ADMIN_DB_PATH="./admin.db"
LOG_LEVEL="debug"

# Run
cargo run
```

### Production Configuration (Single Server)
```bash
# /etc/fastdatabroker/admin-api.conf
ADMIN_API_ADDR="0.0.0.0:8080"
BROKER_URL="http://broker-internal:6000"
ADMIN_DB_PATH="/var/lib/fastdatabroker/admin.db"
LOG_LEVEL="info"

# Systemd service
[Unit]
Description=FastDataBroker Admin API
After=network.target

[Service]
Type=simple
User=admin-api
WorkingDirectory=/opt/fastdatabroker/admin-api
EnvironmentFile=/etc/fastdatabroker/admin-api.conf
ExecStart=/opt/fastdatabroker/admin-api/target/release/admin-api
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Production Configuration (Kubernetes)
```yaml
# admin-api-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: admin-api-config
  namespace: fastdatabroker
data:
  ADMIN_API_ADDR: "0.0.0.0:8080"
  BROKER_URL: "http://broker-service:6000"
  ADMIN_DB_PATH: "/data/admin.db"
  LOG_LEVEL: "info"

---
apiVersion: v1
kind: Secret
metadata:
  name: admin-api-secrets
  namespace: fastdatabroker
type: Opaque
stringData:
  DB_PASSWORD: "your-secure-password"

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: admin-api-pvc
  namespace: fastdatabroker
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: admin-api
  namespace: fastdatabroker
spec:
  replicas: 2
  selector:
    matchLabels:
      app: admin-api
  template:
    metadata:
      labels:
        app: admin-api
    spec:
      containers:
      - name: admin-api
        image: fastdatabroker/admin-api:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
        envFrom:
        - configMapRef:
            name: admin-api-config
        - secretRef:
            name: admin-api-secrets
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: data
          mountPath: /data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: admin-api-pvc
```

### Production Configuration (Docker)
```dockerfile
# Dockerfile with environment configuration
FROM rust:latest as builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3
COPY --from=builder /build/target/release/admin-api /usr/local/bin/
RUN useradd -m -u 1000 admin-api
USER admin-api
WORKDIR /home/admin-api
EXPOSE 8080

# Environment variables
ENV ADMIN_API_ADDR="0.0.0.0:8080" \
    BROKER_URL="http://broker:6000" \
    ADMIN_DB_PATH="/data/admin.db" \
    LOG_LEVEL="info"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["admin-api"]

# Docker run:
# docker run -d \
#   --name admin-api \
#   -p 8080:8080 \
#   -v admin-data:/data \
#   -e BROKER_URL=http://broker:6000 \
#   admin-api:latest
```

### Production Configuration (Docker Compose)
```yaml
# docker-compose.yml
version: '3.8'

services:
  broker:
    image: fastdatabroker/broker:latest
    ports:
      - "6000:6000"
    volumes:
      - broker-data:/data
    environment:
      LOG_LEVEL: info

  admin-api:
    image: fastdatabroker/admin-api:latest
    ports:
      - "8080:8080"
    depends_on:
      - broker
    volumes:
      - admin-data:/data
    environment:
      ADMIN_API_ADDR: "0.0.0.0:8080"
      BROKER_URL: "http://broker:6000"
      ADMIN_DB_PATH: "/data/admin.db"
      LOG_LEVEL: "info"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s

volumes:
  broker-data:
  admin-data:
```

## Performance Tuning Examples

### For High Concurrency (1000+ requests/sec)

```bash
# Environment variables
export TOKIO_WORKER_THREADS=16
export TOKIO_NUM_WORKERS=16
export TOKIO_TASK_MAX_BLOCKING_THREADS=100

# SQLite tuning
sqlite3 admin.db << EOF
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 30000000;
EOF

# Run
cargo run --release
```

### For Limited Resources (IoT/Edge)

```bash
# Environment variables (minimal)
export ADMIN_API_ADDR="127.0.0.1:8080"
export LOG_LEVEL="warn"

# In config.rs, modify:
max_connections: 3  // Very low
connection_timeout: Duration::from_secs(5)

# SQLite minimal
sqlite3 admin.db << EOF
PRAGMA cache_size = -1000;
PRAGMA synchronous = OFF;
EOF

# Compile optimized
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release
```

## Health Check Monitoring Examples

### Prometheus Metrics (Future Addition)

```rust
// Example for adding metrics endpoint
use prometheus::{Counter, Gauge, Registry};

#[derive(Clone)]
pub struct Metrics {
    pub request_count: Counter,
    pub error_count: Counter,
    pub latency_gauge: Gauge,
}

pub async fn metrics_endpoint() -> impl Responder {
    // Returns Prometheus-formatted metrics
}
```

### Alert Configuration (Alertmanager)

```yaml
# alertmanager.yml section
alert_rules:
  - alert: AdminAPIDown
    expr: up{job="admin-api"} == 0
    for: 1m
    annotations:
      summary: "Admin API is down"
  
  - alert: AdminAPIHighMemory
    expr: container_memory_usage_bytes{pod="admin-api"} > 500000000
    for: 5m
    annotations:
      summary: "Admin API memory usage > 500MB"
  
  - alert: AdminAPIHighLatency
    expr: histogram_quantile(0.99, admin_api_duration_seconds) > 0.1
    for: 5m
    annotations:
      summary: "Admin API p99 latency > 100ms"
```

## Recovery Procedures

### Complete Restart
```bash
# Stop service
systemctl stop admin-api
# or Ctrl+C in terminal

# Delete data (if needed)
rm admin.db

# Start service
systemctl start admin-api
# or cargo run
```

### Backup and Restore
```bash
# Backup current state
tar -czf admin_backup_$(date +%Y%m%d_%H%M%S).tar.gz \
  admin.db \
  $(curl -s http://localhost:8080/api/v1/tenants) \
  $(curl -s http://localhost:8080/api/v1/cluster/environments)

# Restore from backup
tar -xzf admin_backup_*.tar.gz
systemctl restart admin-api
```
