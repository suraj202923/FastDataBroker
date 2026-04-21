# FastDataBroker Admin API - Deployment Guide

## Overview

This guide covers deploying the FastDataBroker Admin API in various environments.

## Quick Start

### Development (Local)

```bash
# Prerequisites
cd admin-api

# Build
cargo build --release

# Run
export ADMIN_API_ADDR=127.0.0.1:8080
export BROKER_URL=http://localhost:6000
export ADMIN_DB_PATH=./admin.db
./target/release/admin-api
```

### Docker Container

#### Build Image
```bash
# From admin-api directory
docker build -t fastdatabroker-admin-api:latest .

# Tag for registry
docker tag fastdatabroker-admin-api:latest your-registry.com/fastdatabroker-admin-api:latest
```

#### Run Container
```bash
docker run -d \
  --name admin-api \
  -p 8080:8080 \
  -e ADMIN_API_ADDR=0.0.0.0:8080 \
  -e BROKER_URL=http://broker:6000 \
  -e ADMIN_DB_PATH=/data/admin.db \
  -e LOG_LEVEL=info \
  -v admin-data:/data \
  fastdatabroker-admin-api:latest
```

### Docker Compose

```yaml
version: '3.8'

services:
  admin-api:
    image: fastdatabroker-admin-api:latest
    ports:
      - "8080:8080"
    environment:
      ADMIN_API_ADDR: 0.0.0.0:8080
      BROKER_URL: http://broker:6000
      ADMIN_DB_PATH: /data/admin.db
      LOG_LEVEL: info
    volumes:
      - admin-data:/data
    depends_on:
      - broker
    restart: unless-stopped
    
  broker:
    image: fastdatabroker:latest
    ports:
      - "6000:6000"
    volumes:
      - broker-data:/data
    restart: unless-stopped

volumes:
  admin-data:
  broker-data:
```

## Production Deployment

### Kubernetes

#### ConfigMap for Configuration
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: admin-api-config
  namespace: fastdatabroker
data:
  log_level: "info"
  broker_url: "http://broker-service:6000"
```

#### Deployment
```yaml
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
        image: fastdatabroker-admin-api:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: ADMIN_API_ADDR
          value: "0.0.0.0:8080"
        - name: BROKER_URL
          valueFrom:
            configMapKeyRef:
              name: admin-api-config
              key: broker_url
        - name: ADMIN_DB_PATH
          value: "/pvc/admin.db"
        - name: LOG_LEVEL
          valueFrom:
            configMapKeyRef:
              name: admin-api-config
              key: log_level
        resources:
          requests:
            cpu: "100m"
            memory: "128Mi"
          limits:
            cpu: "500m"
            memory: "512Mi"
        volumeMounts:
        - name: data
          mountPath: /pvc
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: admin-api-pvc
```

#### Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: admin-api-service
  namespace: fastdatabroker
spec:
  type: LoadBalancer
  selector:
    app: admin-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
    name: http
```

#### Persistent Volume Claim
```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: admin-api-pvc
  namespace: fastdatabroker
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
```

### Systemd Service (Linux)

#### Create service file: `/etc/systemd/system/admin-api.service`
```ini
[Unit]
Description=FastDataBroker Admin API
After=network.target

[Service]
Type=simple
User=fastdatabroker
WorkingDirectory=/opt/fastdatabroker/admin-api
ExecStart=/opt/fastdatabroker/admin-api/target/release/admin-api

Environment="ADMIN_API_ADDR=0.0.0.0:8080"
Environment="BROKER_URL=http://localhost:6000"
Environment="ADMIN_DB_PATH=/var/lib/fastdatabroker/admin.db"
Environment="LOG_LEVEL=info"
Environment="RUST_LOG=info"

Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### Enable and Start
```bash
sudo systemctl daemon-reload
sudo systemctl enable admin-api
sudo systemctl start admin-api
sudo systemctl status admin-api
```

### Reverse Proxy (Nginx)

```nginx
upstream admin_api {
    server localhost:8080;
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

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;

    location / {
        proxy_pass http://admin_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check endpoint (no logging)
    location /health {
        proxy_pass http://admin_api;
        access_log off;
    }
}
```

## High Availability Setup

### Multi-Instance with Load Balancer

```yaml
# HAProxy Configuration
global
    log stdout local0
    log stdout local1 notice
    chroot /var/lib/haproxy
    stats socket /run/haproxy/admin.sock mode 660 level admin
    stats timeout 30s
    daemon

defaults
    log     global
    mode    http
    option  httplog
    option  dontlognull
    timeout connect 5000
    timeout client  50000
    timeout server  50000

frontend admin_api_frontend
    bind *:8080
    default_backend admin_api_servers
    option forwardfor

backend admin_api_servers
    balance roundrobin
    server api1 localhost:8081 check
    server api2 localhost:8082 check
    server api3 localhost:8083 check

listen stats
    bind *:8404
    stats enable
    stats uri /stats
```

## Monitoring & Logging

### Prometheus Metrics Integration

```yaml
# prometheus.yml addition
scrape_configs:
  - job_name: 'admin-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Centralized Logging (ELK Stack)

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/fastdatabroker/admin-api.log

output.elasticsearch:
  hosts: ["elasticsearch:9200"]

processors:
  - add_kubernetes_metadata:
      in_cluster: true
```

### Log Rotation

```bash
# /etc/logrotate.d/admin-api
/var/log/fastdatabroker/admin-api.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 fastdatabroker fastdatabroker
    sharedscripts
    postrotate
        systemctl reload admin-api > /dev/null 2>&1 || true
    endscript
}
```

## Performance Tuning

### Environment Variables for Optimization
```bash
# Increase tokio worker threads (default: num_cpus)
export TOKIO_WORKER_THREADS=8

# Enable backtraces for debugging
export RUST_BACKTRACE=1

# Custom logging format
export RUST_LOG=admin_api=debug,actix_web=info
```

### Database Optimization
```sql
-- Enable write-ahead logging (WAL mode)
PRAGMA journal_mode = WAL;

-- Increase cache size (5000 = ~5MB)
PRAGMA cache_size = -5000;

-- Optimize for faster writing
PRAGMA synchronous = NORMAL;
```

## Backup & Recovery

### Automated Backup Script
```bash
#!/bin/bash
# backup-admin-api.sh

BACKUP_DIR="/backups/fastdatabroker"
DB_PATH="/var/lib/fastdatabroker/admin.db"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
mkdir -p $BACKUP_DIR
cp $DB_PATH $BACKUP_DIR/admin_$DATE.db
tar -czf $BACKUP_DIR/admin_$DATE.tar.gz $BACKUP_DIR/admin_$DATE.db

# Keep only last 30 days
find $BACKUP_DIR -name "admin_*.tar.gz" -mtime +30 -delete

echo "Backup completed: $BACKUP_DIR/admin_$DATE.tar.gz"
```

### Restore from Backup
```bash
# Stop the service
systemctl stop admin-api

# Restore database
cp /backups/fastdatabroker/admin_20260412_120000.db /var/lib/fastdatabroker/admin.db

# Restart service
systemctl start admin-api
```

## Verification

### Health Check
```bash
curl -v http://localhost:8080/health
curl -v http://localhost:8080/health/detailed
```

### Test Endpoints
```bash
# Get system config
curl http://localhost:8080/api/v1/system/config

# List tenants
curl http://localhost:8080/api/v1/tenants

# API info
curl http://localhost:8080/api/v1/info
```

## Troubleshooting

### Out of Memory
```bash
# Increase memory limit in systemd
# Edit service file
systemctl edit admin-api

# Add:
[Service]
MemoryLimit=1G

# Reload and restart
systemctl restart admin-api
```

### Database Lock Issues
```bash
# Reset database if corrupted
rm /var/lib/fastdatabroker/admin.db
systemctl restart admin-api
```

### High Latency
```bash
# Check broker connectivity
curl http://broker-host:6000/health

# Monitor system resources
htop
```

## Security Hardening

### Firewall Rules (UFW)
```bash
# Allow only from internal network
sudo ufw allow from 10.0.0.0/8 to any port 8080
sudo ufw allow from 172.16.0.0/12 to any port 8080

# Allow SSH
sudo ufw allow 22/tcp
sudo ufw enable
```

### File Permissions
```bash
# Restrict database access
chmod 600 /var/lib/fastdatabroker/admin.db
chown fastdatabroker:fastdatabroker /var/lib/fastdatabroker/admin.db
```
