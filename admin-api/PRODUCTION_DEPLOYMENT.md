# FastDataBroker Admin API - Production Deployment Guide

## Overview

This guide covers deploying the FastDataBroker Admin API with:
- ✅ API Key Authentication
- ✅ In-Memory Caching (LRU)
- ✅ Request Logging & Tracing
- ✅ JSON-based Storage
- ✅ Docker/Docker Compose
- ✅ Health Checks

## Architecture Components

### Core Features Implemented

1. **Authentication Middleware**
   - API Key header validation (`X-API-Key`)
   - Configurable allowed keys via `ADMIN_API_KEYS` env var
   - Health endpoints bypass auth
   - OpenAPI docs bypass auth

2. **Caching Layer**
   - In-memory LRU cache (1000 entries default)
   - Automatic eviction of least-recently-used entries
   - JSON data serialization/deserialization
   - Thread-safe with Arc<Mutex<>>

3. **Request Tracing**
   - Unique request ID per request
   - Structured logging with tracing
   - Request ID included in all logs
   - Response time tracking

4. **Storage**
   - JSON file-based (no database)
   - Directory: `tenants/`
   - Structure: `tenants/{tenant_id}.json`
   - Secrets: `tenants/{tenant_id}/.secrets/{secret_id}.json`

## Local Deployment (Development)

### Prerequisites

- Rust 1.75+
- Cargo
- Broker service (Kafka/compatible) on localhost:9092

### Build from Source

```bash
# Clone and navigate
cd admin-api

# Copy environment config
cp .env.example .env

# Build release binary
cargo build --release

# Binary location: target/release/admin-api.exe (Windows) or target/release/admin-api (Unix)
```

### Run Locally

```bash
# Set environment variables
set RUST_LOG=info
set SERVER_ADDR=127.0.0.1:8080
set BROKER_URL=http://localhost:9092
set ADMIN_API_KEYS=admin-key-default-change-me

# Run the binary
./target/release/admin-api

# Server runs on: http://127.0.0.1:8080
```

### Test Health Check

```bash
# No auth required
curl http://127.0.0.1:8080/health

# Response:
# {"status":"healthy","timestamp":"2026-04-13T04:30:00.000Z"}
```

### Test API Endpoint (with auth)

```bash
# Create tenant (requires X-API-Key header)
curl -X POST http://127.0.0.1:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -H "X-API-Key: admin-key-default-change-me" \
  -d '{
    "name": "Demo Tenant",
    "email": "admin@demo.com",
    "max_message_size": 10485760,
    "rate_limit_rps": 1000,
    "max_connections": 100,
    "retention_days": 30
  }'

# Response: HTTP 201 Created
```

## Docker Deployment

### Build Docker Image

```bash
# From admin-api directory
docker build -t fastdatabroker-admin-api:latest .

# Or use docker-compose to build
docker-compose build
```

### Run with Docker Compose

```bash
# Start all services (admin-api + broker + zookeeper)
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f admin-api

# Stop services
docker-compose down

# Clean up volumes
docker-compose down -v
```

### Environment Variables in Docker

Set in `docker-compose.yml` or via `.env` file:

```yaml
environment:
  RUST_LOG: info
  SERVER_ADDR: 0.0.0.0:8080
  BROKER_URL: http://broker:9092
  ADMIN_API_KEYS: "admin-key-1,admin-key-2"
```

### Production Configuration

For production, update the following:

```yaml
# docker-compose.yml
environment:
  RUST_LOG: warn  # Reduce verbosity in production
  ADMIN_API_KEYS: "strong-production-key-1,strong-production-key-2"
  # Add TLS support (future enhancement)
  
restart: always  # Auto-restart on failure
healthcheck:
  retries: 5
  timeout: 20s
```

## Kubernetes Deployment

Create `admin-api-k8s.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: admin-api
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
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: info
        - name: SERVER_ADDR
          value: 0.0.0.0:8080
        - name: BROKER_URL
          value: http://broker:9092
        - name: ADMIN_API_KEYS
          valueFrom:
            secretKeyRef:
              name: admin-api-secrets
              key: api-keys
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

Deploy with:

```bash
# Create secret
kubectl create secret generic admin-api-secrets \
  --from-literal=api-keys="admin-key-1,admin-key-2"

# Deploy
kubectl apply -f admin-api-k8s.yaml

# Check status
kubectl get pods -l app=admin-api
kubectl logs -f deployment/admin-api
```

## Production Checklist

- [ ] Update `ADMIN_API_KEYS` with strong, unique keys
- [ ] Set `RUST_LOG=warn` for production
- [ ] Enable TLS/HTTPS (future enhancement)
- [ ] Set up request rate limiting (future enhancement)
- [ ] Configure persistent storage for tenants volume
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy for `tenants/` directory
- [ ] Set up log aggregation (ELK, Splunk, etc.)
- [ ] Test failover and recovery procedures
- [ ] Document runbook for common operations

## Monitoring

### Health Endpoint

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health (requires query, future implementation)
curl http://localhost:8080/health/detailed
```

### Logs

Monitor logs for:
- Authorization failures (invalid API keys)
- 5xx errors (internal issues)
- High latency responses
- Cache statistics

### Metrics (Future)

Prometheus metrics endpoint (planned for v0.2.0):
- Request count by endpoint
- Response times (p50, p95, p99)
- Cache hit/miss rates
- Tenant count
- Storage usage

## Troubleshooting

### Issue: "Invalid API key" errors

Solution: Verify the `X-API-Key` header matches one in `ADMIN_API_KEYS`

```bash
# Debug: Check env var
echo $env:ADMIN_API_KEYS

# Update with new key
set ADMIN_API_KEYS=new-key-here
```

### Issue: Connection refused to broker

Solution: Ensure broker is running and `BROKER_URL` is correct

```bash
# Check broker connectivity
curl -v telnet://localhost:9092  # Change host:port as needed
```

### Issue: Storage directory permissions

Solution: Ensure `tenants/` directory is writable

```bash
# Check permissions
ls -la tenants/

# Fix permissions if needed
chmod -R 755 tenants/
```

## Performance Tuning

### Cache Configuration

Current default: 1000 LRU entries

To increase:
- Modify `src/cache.rs` `JsonCache::new(1000)` 
- Rebuild binary
- More cache = more memory but faster lookups

### Logging Level Optimization

- `trace` - Maximum detail (slowest)
- `debug` - Development debugging
- `info` - Recommended for production
- `warn` - Only warnings and errors
- `error` - Only errors (fastest)

### Database Caching Strategy (Future)

When scaling beyond JSON files:
1. Add SQLite with indexes
2. Implement automatic cache refresh
3. Add background sync to persistent store
4. Implement batch operations

## Rollback Procedure

If deployment fails:

```bash
# With Docker Compose
docker-compose down
docker-compose up -d  # Start previous version

# With Kubernetes
kubectl rollout undo deployment/admin-api
```

## Support & Debugging

Enable debug logging for troubleshooting:

```bash
export RUST_LOG=debug
./target/release/admin-api
```

Logs will show:
- Request IDs for tracing
- Cache hit/miss events
- Auth validation failures
- File I/O operations
- JSON serialization details

## Next Steps (Future Enhancements)

1. **v0.2.0**: Add TLS support
2. **v0.3.0**: Implement request validation middleware
3. **v0.4.0**: Add OpenAPI/Swagger UI
4. **v0.5.0**: Database caching layer (SQLite + Cache)
5. **v0.6.0**: Implement Prometheus metrics
6. **v0.7.0**: Multi-node clustering support
7. **v1.0.0**: Production hardening & performance optimization
