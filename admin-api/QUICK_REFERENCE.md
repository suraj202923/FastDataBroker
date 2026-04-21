# Admin API - Quick Reference Guide

## Common Operations

### Admin API Startup

#### Development Mode
```bash
cd admin-api
cargo run
```

Output should show:
```
Starting Admin API server on 0.0.0.0:8080
Database initialized at admin.db
Connected to broker at http://localhost:6000
Ready to handle requests
```

#### Production Mode
```bash
cd admin-api
cargo build --release
./target/release/admin-api
```

Or with environment variables:
```bash
ADMIN_API_ADDR="0.0.0.0:8080" \
BROKER_URL="http://broker:6000" \
ADMIN_DB_PATH="/data/admin.db" \
LOG_LEVEL="info" \
cargo run --release
```

## API Endpoints Reference

### Quick URL Map

```
🏥 HEALTH
  GET  /health                          → Basic health check
  GET  /health/detailed                 → Detailed health metrics

⚙️  SYSTEM
  GET  /api/v1/system/config            → Get system config
  PUT  /api/v1/system/config            → Update config

🖥️  CLUSTER
  GET  /api/v1/cluster/environments     → List clusters
  POST /api/v1/cluster/environments     → Create cluster
  GET  /api/v1/cluster/environments/{id}→ Get cluster
  PUT  /api/v1/cluster/environments/{id}→ Update cluster
  DEL  /api/v1/cluster/environments/{id}→ Delete cluster

👥 TENANT
  GET  /api/v1/tenants                  → List tenants
  POST /api/v1/tenants                  → Create tenant
  GET  /api/v1/tenants/{id}             → Get tenant
  PUT  /api/v1/tenants/{id}             → Update tenant
  DEL  /api/v1/tenants/{id}             → Delete tenant

🔑 SECRETS
  GET  /api/v1/tenants/{id}/secrets     → List secrets
  POST /api/v1/tenants/{id}/secrets     → Create secret
  PUT  /api/v1/tenants/{id}/secrets     → Update secret
  DEL  /api/v1/tenants/{id}/secrets/{sid}→ Delete secret

📊 USAGE & LIMITS
  GET  /api/v1/tenants/{id}/usage       → Get usage stats
  GET  /api/v1/tenants/{id}/limits      → Get limits
  PUT  /api/v1/tenants/{id}/limits      → Update limits
  POST /api/v1/tenants/{id}/limits/reset→ Reset to defaults

🔔 NOTIFICATIONS
  GET  /api/v1/notifications/smtp       → Get SMTP config
  PUT  /api/v1/notifications/smtp       → Update SMTP
  POST /api/v1/notifications/smtp/test  → Test SMTP
  GET  /api/v1/notifications/settings   → Get notification settings
  PUT  /api/v1/notifications/settings   → Update settings
  GET  /api/v1/notifications/events     → List events
  GET  /api/v1/notifications/events/{id}→ Get event

ℹ️  INFO
  GET  /api/v1/info                     → API documentation
```

## Common Tasks

### 1. Onboard a New Tenant

```bash
# Create tenant
TENANT=$(curl -s -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MyCompany",
    "email": "admin@mycompany.com"
  }')

TENANT_ID=$(echo $TENANT | jq -r '.tenant_id')
echo "Created tenant: $TENANT_ID"

# Add secret/credentials
curl -X POST http://localhost:8080/api/v1/tenants/$TENANT_ID/secrets \
  -H "Content-Type: application/json" \
  -d '{
    "secret_key": "api_key",
    "secret_value": "generated_api_key_value"
  }'

# Configure limits
curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID/limits \
  -H "Content-Type: application/json" \
  -d '{
    "max_message_size": 10485760,
    "rate_limit_rps": 1000,
    "max_connections": 100,
    "retention_days": 30
  }'

echo "Tenant $TENANT_ID onboarded successfully"
```

### 2. Scale Message Size Limits

```bash
# Increase message size limit to 50MB
TENANT_ID="tenant_abc123"

curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID/limits \
  -H "Content-Type: application/json" \
  -d '{
    "max_message_size": 52428800
  }'

echo "✓ Updated tenant $TENANT_ID message size limit to 50MB"
```

### 3. Set Up Multi-Region Clusters

```bash
# US East Cluster
curl -X POST http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-us-east",
    "region": "us-east-1",
    "broker_addresses": [
      "broker1.us-east.internal:6000",
      "broker2.us-east.internal:6000",
      "broker3.us-east.internal:6000"
    ],
    "replication_factor": 3
  }'

# US West Cluster
curl -X POST http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-us-west",
    "region": "us-west-2",
    "broker_addresses": [
      "broker1.us-west.internal:6000",
      "broker2.us-west.internal:6000",
      "broker3.us-west.internal:6000"
    ],
    "replication_factor": 3
  }'

# Europe Cluster
curl -X POST http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-eu-west",
    "region": "eu-west-1",
    "broker_addresses": [
      "broker1.eu-west.internal:6000",
      "broker2.eu-west.internal:6000",
      "broker3.eu-west.internal:6000"
    ],
    "replication_factor": 3
  }'

echo "✓ Multi-region clusters configured"
```

### 4. Configure Email Notifications

```bash
# Set SMTP configuration
curl -X PUT http://localhost:8080/api/v1/notifications/smtp \
  -H "Content-Type: application/json" \
  -d '{
    "host": "smtp.gmail.com",
    "port": 587,
    "username": "alerts@company.com",
    "password": "app_specific_password",
    "from_email": "fdb-alerts@company.com",
    "use_tls": true
  }'

# Test SMTP connection
curl -X POST http://localhost:8080/api/v1/notifications/smtp/test \
  -H "Content-Type: application/json" \
  -d '{
    "recipient_email": "ops@company.com"
  }'

# Subscribe to limit exceeded alerts
curl -X PUT http://localhost:8080/api/v1/notifications/settings \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "tenant_limit_exceeded",
    "enabled": true,
    "recipient_email": "ops@company.com",
    "notification_channels": ["email"]
  }'

# Subscribe to broker failure alerts
curl -X PUT http://localhost:8080/api/v1/notifications/settings \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "broker_failure",
    "enabled": true,
    "recipient_email": "critical-team@company.com",
    "notification_channels": ["email"]
  }'

echo "✓ Email notifications configured"
```

### 5. Monitor System Health

```bash
# Check basic status
curl -s http://localhost:8080/health | jq .

# Get detailed metrics
curl -s http://localhost:8080/health/detailed | jq '{
  status: .status,
  broker: .broker,
  database: .database,
  system: .system
}'

# Monitor in real-time (requires watch or similar)
watch -n 5 'curl -s http://localhost:8080/health/detailed | jq .'
```

### 6. Get Tenant Usage Reports

```bash
# Get all tenants
TENANTS=$(curl -s http://localhost:8080/api/v1/tenants | jq -r '.[].tenant_id')

for TENANT in $TENANTS; do
  echo "=== $TENANT ==="
  curl -s http://localhost:8080/api/v1/tenants/$TENANT/usage | jq '{
    tenant_id: .tenant_id,
    messages_sent: .messages_sent,
    storage_used_mb: .storage_used_mb,
    bandwidth_used_gb: .bandwidth_used_gb
  }'
  echo ""
done
```

### 7. Backup and Export Configuration

```bash
# Export all tenants
curl -s http://localhost:8080/api/v1/tenants | jq . > backup_tenants.json

# Export all clusters
curl -s http://localhost:8080/api/v1/cluster/environments | jq . > backup_clusters.json

# Export system config
curl -s http://localhost:8080/api/v1/system/config | jq . > backup_system_config.json

# Create complete backup
tar -czf fdb_admin_backup_$(date +%Y%m%d_%H%M%S).tar.gz \
  backup_tenants.json \
  backup_clusters.json \
  backup_system_config.json \
  admin.db

echo "✓ Backup created"
```

### 8. Migrate Tenant to New Cluster

```bash
TENANT_ID="tenant_abc123"
NEW_CLUSTER_ID="cluster_xyz789"

# Get current tenant config
TENANT_CONFIG=$(curl -s http://localhost:8080/api/v1/tenants/$TENANT_ID)

# Update tenant with new cluster reference (custom field, if implemented)
curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'$(echo $TENANT_CONFIG | jq -r '.name')'",
    "email": "'$(echo $TENANT_CONFIG | jq -r '.email')'"
  }'

echo "✓ Tenant $TENANT_ID migrated to cluster $NEW_CLUSTER_ID"
```

## Error Handling

### Common Errors and Solutions

```
400 Bad Request
→ Check JSON formatting
→ Verify all required fields are present
→ Example: 'email' field required for tenant creation

404 Not Found
→ Check if resource ID exists
→ curl http://localhost:8080/api/v1/tenants to list valid IDs
→ Verify URL path is correct

409 Conflict
→ Resource with same name/email already exists
→ Use different name or delete existing resource first
→ Example: Tenant email must be unique

500 Internal Server Error
→ Check server logs: look for database/broker errors
→ Verify database file permissions
→ Verify broker is accessible

502 Bad Gateway
→ Broker is not responding
→ Check: curl http://localhost:6000/health
→ Verify BROKER_URL environment variable
```

## Environment Variables

```bash
# API Server Address (default: 127.0.0.1:8080)
ADMIN_API_ADDR=0.0.0.0:8080

# Broker Connection (default: http://localhost:6000)
BROKER_URL=http://broker-prod:6000

# Database Path (default: admin.db)
ADMIN_DB_PATH=/data/admin.db

# Logging Level (default: info)
LOG_LEVEL=debug|info|warn|error

# Tokio Worker Threads (optional)
TOKIO_WORKER_THREADS=4

# SQLite Debug Mode (optional)
SQLITE_DEBUG=1
```

## Database Management

### Backup Database
```bash
# Automatic backup
cp admin.db admin.db.backup_$(date +%Y%m%d_%H%M%S)

# With compression
tar -czf admin_db_backup.tar.gz admin.db
```

### Reset Database
```bash
# Remove current database (will recreate on startup)
rm admin.db

# Restart Admin API
systemctl restart admin-api
# or
cargo run --release
```

### Query Database Directly
```bash
# Install sqlite3
apt-get install sqlite3

# Open database
sqlite3 admin.db

# List tables
> .tables

# Query tenants
> SELECT tenant_id, name, status FROM tenants;

# Query configuration
> SELECT * FROM system_config;

# Exit
> .quit
```

## Performance Tips

1. **Connection Pool**: Increase for high-concurrency scenarios
   ```bash
   # In config.rs (adjust pool_size)
   max_connections: 50  # Increase if needed
   ```

2. **Database Optimization**:
   ```bash
   # Enable WAL mode (faster writes)
   sqlite3 admin.db "PRAGMA journal_mode=WAL;"
   ```

3. **Caching**: For frequently accessed data
   ```bash
   # Add Redis caching layer in front
   # Implement in-memory caching for health checks
   ```

4. **Async Operations**: All I/O is already async
   ```bash
   # Verify with high concurrent requests
   wrk -t16 -c1000 -d60s http://localhost:8080/health
   ```

## Deployment Checklist

- [ ] Port 8080 is open and not blocked
- [ ] Broker is accessible at configured BROKER_URL
- [ ] Database directory has write permissions
- [ ] Environment variables are set correctly
- [ ] TLS/HTTPS is configured (if required)
- [ ] Backups are scheduled
- [ ] Monitoring/logging is enabled
- [ ] Documentation is accessible
- [ ] Health check passes (curl /health)
- [ ] All endpoints tested (run INTEGRATION_TESTING_GUIDE.md tests)

## Support & Troubleshooting

For issues:
1. Check Admin API logs: `tail -f admin-api.log`
2. Check database: `sqlite3 admin.db` → `.tables`
3. Check broker connection: `curl http://localhost:6000/health`
4. Verify configuration: `curl http://localhost:8080/api/v1/system/config`
5. Review [README.md](README.md) and [DEPLOYMENT.md](DEPLOYMENT.md)
