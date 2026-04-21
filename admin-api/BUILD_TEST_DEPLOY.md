# Admin API - Build, Test & Deployment Manual

## Prerequisites

### Required Software
- Rust 1.70+
- Cargo
- SQLite 3.x
- curl or similar HTTP client
- FastDataBroker Broker running on port 6000

### Optional Tools
- Docker & Docker Compose (for containerized deployment)
- Kubernetes cluster (for K8s deployment)
- Nginx/HAProxy (for load balancing)
- PostgreSQL (for future production databases)

### System Requirements
- **Minimum**: 256MB RAM, 100MB disk space
- **Recommended**: 512MB RAM, 1GB disk space
- **Production**: 1-2GB RAM, 10GB disk space (with backups)

## Build Instructions

### 1. Clone and Navigate
```bash
cd FastDataBroker/admin-api
```

### 2. Verify Dependencies
```bash
# Check Rust version
rustc --version  # Should be 1.70+
cargo --version

# List dependencies
cat Cargo.toml

# Update dependencies
cargo update
```

### 3. Build Debug Version
```bash
# Development build (slower, with debug info)
cargo build

# Output: target/debug/admin-api
```

### 4. Build Release Version
```bash
# Production build (optimized, smaller)
cargo build --release

# Output: target/release/admin-api

# Size comparison
ls -lh target/debug/admin-api target/release/admin-api
```

### 5. Build Verification
```bash
# Check build succeeded
cargo check

# Also works without full compilation:
cargo test --lib --no-run
```

### 6. Documentation Build
```bash
# Generate Rust docs
cargo doc --no-deps --open

# Creates: target/doc/admin_api/index.html
```

## Testing Instructions

### Unit Test Suite
```bash
# Run all unit tests
cargo test

# Run specific test
cargo test test_create_tenant

# Run with output
cargo test -- --nocapture

# With parallel disabled
cargo test -- --test-threads=1
```

### Integration Tests

#### Prerequisite: Start Broker
```bash
# Terminal 1: Start broker
cd FastDataBroker
cargo run --release

# Wait for: "Broker listening on 0.0.0.0:6000"
```

#### Run Admin API
```bash
# Terminal 2: Start admin-api
cd FastDataBroker/admin-api
cargo run --release

# Wait for: "Starting Admin API server on 0.0.0.0:8080"
```

#### Run Integration Tests
```bash
# Terminal 3: Run tests
cd FastDataBroker/admin-api

# Full integration test suite
bash INTEGRATION_TESTING_GUIDE.md

# Or run individual tests
bash test_admin_api.sh
```

### Manual Testing

#### 1. Test Health Endpoint
```bash
# Basic test
curl http://localhost:8080/health

# Detailed test
curl http://localhost:8080/health/detailed | jq .

# Expected: Status 200 OK with health data
```

#### 2. Test System Configuration
```bash
# Create test data
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "Test", "email": "test@example.com"}'

# Should get 201 Created response
```

#### 3. Test Error Handling
```bash
# Try invalid request (missing email)
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "Test"}'
# Expected: 400 Bad Request

# Try duplicate email
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "Test2", "email": "test@example.com"}'
# Expected: 409 Conflict
```

### Load Testing

#### Using wrk
```bash
# Install wrk
apt-get install wrk  # Linux
brew install wrk     # macOS

# Basic load test (12 threads, 400 connections, 30 seconds)
wrk -t12 -c400 -d30s http://localhost:8080/health

# Output includes: Requests/sec, latency percentiles, errors
```

#### Using Apache Bench
```bash
# Install ab
apt-get install apache2-utils

# Test health endpoint
ab -n 10000 -c 100 http://localhost:8080/health

# Test POST endpoint
ab -n 1000 -c 50 -p test_payload.json -T "application/json" \
  http://localhost:8080/api/v1/tenants
```

#### Using custom Lua script with wrk
```lua
-- wrk_test.lua
request = function()
  return wrk.format("POST", "/api/v1/tenants",
    '{"name":"Test'..math.random()...'","email":"test'..math.random()..'@example.com"}',
    {["Content-Type"] = "application/json"}
  )
end

response = function(status, headers, body)
  if status >= 400 then
    io.write("ERROR: " .. status .. "\n")
  end
end
```

Run:
```bash
wrk -t4 -c100 -d30s -s wrk_test.lua http://localhost:8080
```

### Performance Testing

#### Memory Usage
```bash
# Monitor memory during testing
watch -n 1 'ps aux | grep admin-api | grep -v grep'

# Long-running memory test
cargo test stress_test -- --nocapture --test-threads=1
```

#### Response Time Profiling
```bash
# Measure individual endpoint response time
time curl -w "\nTime: %{time_total}s\n" http://localhost:8080/health

# Batch test with timing
for i in {1..100}; do
  curl -s -w "%{time_total}\n" -o /dev/null \
    http://localhost:8080/health
done | awk '{sum+=$1; count++} END {print "Avg:", sum/count}'
```

### Database Testing

#### SQLite Verification
```bash
# List all tables
sqlite3 admin.db ".tables"

# Verify schema
sqlite3 admin.db ".schema"

# Check data integrity
sqlite3 admin.db "PRAGMA integrity_check"

# Get row counts
sqlite3 admin.db "SELECT name, COUNT(*) as rows FROM \
  (SELECT name FROM sqlite_master WHERE type='table') \
  GROUP BY name;"
```

## Deployment Instructions

### Development Deployment (Local)

#### Quick Start
```bash
# Terminal 1: Broker
cd FastDataBroker && cargo run --release

# Terminal 2: Admin API
cd admin-api && cargo run --release

# Terminal 3: Test
curl http://localhost:8080/health
```

#### With Custom Configuration
```bash
export ADMIN_API_ADDR="127.0.0.1:9000"
export BROKER_URL="http://localhost:6000"
export LOG_LEVEL="debug"

cargo run --release

# Test
curl http://localhost:9000/health
```

### Docker Deployment

#### Build Docker Image
```bash
# From admin-api directory
docker build -t fastdatabroker/admin-api:latest .

# Verify build
docker images | grep admin-api
```

#### Run in Docker
```bash
# Create volume for persistence
docker volume create admin-api-data

# Run container
docker run -d \
  --name admin-api \
  --network host \
  -v admin-api-data:/data \
  -e BROKER_URL="http://localhost:6000" \
  -e ADMIN_DB_PATH="/data/admin.db" \
  -e LOG_LEVEL="info" \
  -p 8080:8080 \
  fastdatabroker/admin-api:latest

# Verify running
docker ps | grep admin-api

# View logs
docker logs -f admin-api

# Health check
curl http://localhost:8080/health

# Stop
docker stop admin-api
docker rm admin-api
```

### Docker Compose Deployment

#### Simple Stack
```bash
# Start entire stack
docker-compose -f docker-compose.yml up -d

# Verify services
docker-compose ps

# View logs
docker-compose logs -f admin-api

# Stop services
docker-compose down

# Clean up (remove volumes)
docker-compose down -v
```

#### With Production Settings
```yaml
# .env file for docker-compose
ADMIN_API_ADDR=0.0.0.0:8080
BROKER_URL=http://broker:6000
ADMIN_DB_PATH=/data/admin.db
LOG_LEVEL=info
```

Run:
```bash
docker-compose --env-file .env.production up -d
```

### Kubernetes Deployment

#### Prepare Manifests
```bash
# All manifests are in kubernetes/ directory
ls kubernetes/
# 01-namespace-config.yaml
# 02-statefulset-service.yaml
# 03-rbac-network.yaml
# 04-autoscaling-monitoring.yaml
```

#### Deploy to Cluster
```bash
# Create namespace and config
kubectl apply -f kubernetes/01-namespace-config.yaml

# Deploy admin-api
kubectl apply -f kubernetes/02-statefulset-service.yaml

# Apply RBAC and network policies
kubectl apply -f kubernetes/03-rbac-network.yaml

# Apply autoscaling
kubectl apply -f kubernetes/04-autoscaling-monitoring.yaml

# Verify deployment
kubectl get deployments -n fastdatabroker
kubectl get pods -n fastdatabroker
kubectl get svc -n fastdatabroker

# View logs
kubectl logs -f -n fastdatabroker deployment/admin-api

# Test service
kubectl port-forward -n fastdatabroker svc/admin-api 8080:8080 &
curl http://localhost:8080/health
```

### Systemd Service Deployment

#### Create Service File
```bash
# Create config file
sudo vi /etc/fastdatabroker/admin-api.conf

# Add environment variables:
ADMIN_API_ADDR=0.0.0.0:8080
BROKER_URL=http://localhost:6000
ADMIN_DB_PATH=/var/lib/fastdatabroker/admin.db
LOG_LEVEL=info
```

#### Create Systemd Unit
```bash
sudo vi /etc/systemd/system/admin-api.service

# Add content:
[Unit]
Description=FastDataBroker Admin API
After=network.target broker.service
Wants=broker.service

[Service]
Type=simple
User=admin-api
Group=admin-api
WorkingDirectory=/opt/fastdatabroker/admin-api
EnvironmentFile=/etc/fastdatabroker/admin-api.conf
ExecStart=/opt/fastdatabroker/admin-api/target/release/admin-api
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=admin-api

[Install]
WantedBy=multi-user.target
```

#### Deploy Service
```bash
# Create user and directories
sudo useradd -r -s /bin/false admin-api
sudo mkdir -p /opt/fastdatabroker
sudo mkdir -p /var/lib/fastdatabroker
sudo chown admin-api:admin-api /var/lib/fastdatabroker

# Copy binary
sudo cp target/release/admin-api /opt/fastdatabroker/admin-api/

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable admin-api
sudo systemctl start admin-api

# Monitor
sudo systemctl status admin-api
sudo journalctl -f -u admin-api
```

### Nginx Reverse Proxy

#### Configuration
```nginx
# /etc/nginx/sites-available/admin-api

upstream admin_api_backend {
    server localhost:8080;
    server localhost:8081;  # If multiple instances
    keepalive 32;
}

server {
    listen 80;
    server_name admin-api.example.com;

    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name admin-api.example.com;

    ssl_certificate /etc/letsencrypt/live/admin-api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/admin-api.example.com/privkey.pem;

    location / {
        proxy_pass http://admin_api_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # Health check endpoint (no caching)
    location /health {
        proxy_pass http://admin_api_backend;
        proxy_no_cache 1;
    }
}
```

Enable:
```bash
sudo ln -s /etc/nginx/sites-available/admin-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

## Monitoring & Logging

### Application Logs
```bash
# View logs
tail -f /var/log/admin-api.log

# Filter by level
grep "ERROR" /var/log/admin-api.log
grep "WARN" /var/log/admin-api.log

# Rotate logs
logrotate /etc/logrotate.d/admin-api
```

### System Metrics
```bash
# Monitor process
top -p $(pgrep -f admin-api)

# Network connections
ss -tnap | grep admin-api

# Database file size
du -h /var/lib/fastdatabroker/admin.db

# Disk usage
df -h /var/lib/fastdatabroker/
```

### Health Monitoring
```bash
# Continuous health check
watch -n 5 'curl -s http://localhost:8080/health | jq .'

# Detailed metrics
curl -s http://localhost:8080/health/detailed | jq .

# Parse specific fields
curl -s http://localhost:8080/health | jq '.broker_connected, .database_healthy'
```

## Backup & Recovery

### Automated Backups
```bash
#!/bin/bash
# backup-admin-api.sh

BACKUP_DIR="/backups/admin-api"
DATE=$(date +%Y%m%d_%H%M%S)
DB_PATH="/var/lib/fastdatabroker/admin.db"

mkdir -p $BACKUP_DIR

# Backup database
cp $DB_PATH $BACKUP_DIR/admin.db.$DATE

# Backup configuration
cp /etc/fastdatabroker/admin-api.conf $BACKUP_DIR/config.$DATE

# Archive and compress
tar -czf $BACKUP_DIR/admin-api-backup-$DATE.tar.gz \
  $BACKUP_DIR/admin.db.$DATE \
  $BACKUP_DIR/config.$DATE

# Cleanup old backups (keep 30 days)
find $BACKUP_DIR -name "*.tar.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/admin-api-backup-$DATE.tar.gz"
```

Schedule with cron:
```bash
# Backup every day at 2am
0 2 * * * /opt/fastdatabroker/scripts/backup-admin-api.sh
```

### Recovery Procedure
```bash
# Restore from backup
tar -xzf admin-api-backup-YYYYMMDD_HHMMSS.tar.gz -C /var/lib/fastdatabroker/

# Restart service
sudo systemctl restart admin-api

# Verify
curl http://localhost:8080/health
```

## Upgrade Procedure

### In-Place Upgrade
```bash
# 1. Backup current version
cp target/release/admin-api target/release/admin-api.backup

# 2. Build new version
cargo build --release

# 3. Stop service
sudo systemctl stop admin-api

# 4. Copy new binary
sudo cp target/release/admin-api /opt/fastdatabroker/admin-api/

# 5. Start service
sudo systemctl start admin-api

# 6. Verify
curl http://localhost:8080/health

# 7. If issues, rollback
sudo cp /opt/fastdatabroker/admin-api.backup /opt/fastdatabroker/admin-api
sudo systemctl restart admin-api
```

### Blue-Green Deployment
```bash
# 1. Start new instance on different port
ADMIN_API_ADDR="0.0.0.0:8081" cargo run --release &

# 2. Test new instance
curl http://localhost:8081/health

# 3. Switch traffic (via Nginx or load balancer)
# Update upstream in Nginx to point to 8081

# 4. Monitor for issues
tail -f /var/log/admin-api.log

# 5. If stable, shut down old instance
pkill -f "0.0.0.0:8080"

# 6. Update to use port 8080
# Reconfigure and restart
```

## Rollback Procedure

### From Failed Deployment
```bash
# 1. Identify previous working version
ls -lh target/release/admin-api*

# 2. Restore previous binary
sudo cp /opt/fastdatabroker/admin-api.backup /opt/fastdatabroker/admin-api

# 3. Restart service
sudo systemctl restart admin-api

# 4. Verify functionality
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/tenants

# 5. Investigate failure
# Review logs, database integrity
sqlite3 admin.db "PRAGMA integrity_check;"
```

## Performance Benchmark Results

### Expected Performance Metrics
```
Health Check Endpoint (/health):
  - Throughput: 10,000+ req/sec
  - Latency p50: 1-2ms
  - Latency p95: 5-10ms
  - Latency p99: 20-30ms

Tenant CRUD Endpoints:
  - Throughput: 2,000-5,000 req/sec
  - Latency p50: 5-10ms
  - Latency p95: 20-50ms
  - Latency p99: 50-100ms

Memory Usage:
  - Idle: 20-30MB
  - Under load: 30-100MB
  - Peak: <200MB
```

## Success Checklist

- [ ] Code compiles without warnings
- [ ] All unit tests pass
- [ ] Integration tests successful
- [ ] Health endpoint responds (< 5ms)
- [ ] Tenant CRUD operations work
- [ ] Database persists data correctly
- [ ] Load test shows < 100ms p99 latency
- [ ] Memory usage within limits
- [ ] Documentation complete and accurate
- [ ] Docker image builds successfully
- [ ] Kubernetes manifests valid
- [ ] Systemd service runs correctly
- [ ] Monitoring/logging configured
- [ ] Backup procedures tested
- [ ] Rollback plan verified
