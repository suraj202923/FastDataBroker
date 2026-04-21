# Admin API - Integration Testing Guide

## Setup

### Prerequisites
```bash
# 1. Ensure broker is running on port 6000
# 2. Admin API compiled and ready to run
# 3. curl installed (for testing)
# 4. jq installed (for JSON parsing - optional but helpful)
```

### Environment Setup
```bash
cd admin-api

# Export environment variables
export ADMIN_API_ADDR="127.0.0.1:8080"
export BROKER_URL="http://localhost:6000"
export ADMIN_DB_PATH="./admin.db"
export LOG_LEVEL="debug"

# Start the Admin API
cargo run --release
```

The API should start and bind to http://127.0.0.1:8080

## Test Suite

### 1. Health Check Tests

#### 1.1 Basic Health Check
```bash
# Test basic health
curl -X GET http://localhost:8080/health \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "status": "healthy",
  "uptime_seconds": 15,
  "broker_connected": true,
  "database_healthy": true,
  "timestamp": "2026-04-12T10:30:00Z"
}
```

#### 1.2 Detailed Health Check
```bash
curl -X GET http://localhost:8080/health/detailed \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "status": "healthy",
  "broker": {
    "connected": true,
    "url": "http://localhost:6000",
    "response_time_ms": 5,
    "active_connections": 0
  },
  "database": {
    "connected": true,
    "query_time_ms": 2,
    "pool_size": 10
  },
  "system": {
    "cpu_usage_percent": 15.5,
    "memory_usage_mb": 256,
    "active_tenants": 0,
    "total_message_volume": 0
  },
  "timestamp": "2026-04-12T10:30:00Z"
}
```

### 2. System Configuration Tests

#### 2.1 Get Current Configuration
```bash
curl -X GET http://localhost:8080/api/v1/system/config \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "id": "default",
  "broker_url": "http://localhost:6000",
  "max_brokers": 3,
  "replication_factor": 3,
  "log_level": "info",
  "created_at": "2026-04-12T10:00:00Z",
  "updated_at": "2026-04-12T10:00:00Z"
}
```

#### 2.2 Update Configuration
```bash
curl -X PUT http://localhost:8080/api/v1/system/config \
  -H "Content-Type: application/json" \
  -d '{
    "broker_url": "http://localhost:6000",
    "max_brokers": 5,
    "replication_factor": 3,
    "log_level": "debug"
  }'

# Expected Response (200 OK)
{
  "id": "default",
  "broker_url": "http://localhost:6000",
  "max_brokers": 5,
  "replication_factor": 3,
  "log_level": "debug",
  "updated_at": "2026-04-12T10:31:00Z"
}
```

### 3. Cluster Environment Tests

#### 3.1 List All Clusters
```bash
curl -X GET http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json"

# Expected Response (200 OK - Empty initially)
[]
```

#### 3.2 Create Cluster Environment
```bash
curl -X POST http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production-us-east",
    "description": "Production cluster in US East",
    "region": "us-east-1",
    "broker_addresses": [
      "broker1.example.com:6000",
      "broker2.example.com:6000",
      "broker3.example.com:6000"
    ],
    "replication_factor": 3
  }'

# Expected Response (201 Created)
# Save the returned cluster ID for later tests
export CLUSTER_ID="cluster_<uuid>"
```

#### 3.3 Get Specific Cluster
```bash
curl -X GET http://localhost:8080/api/v1/cluster/environments/$CLUSTER_ID \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "id": "cluster_<uuid>",
  "name": "production-us-east",
  "description": "Production cluster in US East",
  "region": "us-east-1",
  "broker_addresses": [...],
  "replication_factor": 3,
  "status": "active",
  "created_at": "2026-04-12T10:30:00Z"
}
```

#### 3.4 Update Cluster
```bash
curl -X PUT http://localhost:8080/api/v1/cluster/environments/$CLUSTER_ID \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production-us-east-v2",
    "description": "Updated description",
    "broker_addresses": [
      "broker1.example.com:6000",
      "broker2.example.com:6000",
      "broker3.example.com:6000",
      "broker4.example.com:6000"
    ]
  }'

# Expected Response (200 OK)
```

#### 3.5 Delete Cluster
```bash
curl -X DELETE http://localhost:8080/api/v1/cluster/environments/$CLUSTER_ID \
  -H "Content-Type: application/json"

# Expected Response (204 No Content)
```

### 4. Tenant Management Tests

#### 4.1 Create Tenant
```bash
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ACME Corp",
    "email": "admin@acme.com"
  }'

# Expected Response (201 Created)
# Save the returned tenant_id for later tests
export TENANT_ID="tenant_<uuid>"

# Response includes:
{
  "tenant_id": "tenant_<uuid>",
  "name": "ACME Corp",
  "email": "admin@acme.com",
  "status": "active",
  "limits": {
    "max_message_size": 10485760,
    "rate_limit_rps": 1000,
    "max_connections": 100,
    "retention_days": 30
  },
  "created_at": "2026-04-12T10:30:00Z"
}
```

#### 4.2 List All Tenants
```bash
curl -X GET http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
[
  {
    "tenant_id": "tenant_<uuid>",
    "name": "ACME Corp",
    "status": "active",
    "usage": {
      "messages_sent": 0,
      "active_connections": 0,
      "storage_used_mb": 0
    }
  }
]
```

#### 4.3 Get Specific Tenant
```bash
curl -X GET http://localhost:8080/api/v1/tenants/$TENANT_ID \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "tenant_id": "tenant_<uuid>",
  "name": "ACME Corp",
  "email": "admin@acme.com",
  "status": "active",
  "limits": {...},
  "created_at": "2026-04-12T10:30:00Z"
}
```

#### 4.4 Update Tenant
```bash
curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ACME Corp Updated",
    "email": "newemail@acme.com",
    "status": "active"
  }'

# Expected Response (200 OK)
```

### 5. Tenant Secrets Tests

#### 5.1 Create Secret
```bash
curl -X POST http://localhost:8080/api/v1/tenants/$TENANT_ID/secrets \
  -H "Content-Type: application/json" \
  -d '{
    "secret_key": "db_password",
    "secret_value": "secure_password_123"
  }'

# Expected Response (201 Created)
{
  "secret_id": "secret_<uuid>",
  "secret_key": "db_password",
  "created_at": "2026-04-12T10:30:00Z"
}
```

#### 5.2 Get Tenant Secrets
```bash
curl -X GET http://localhost:8080/api/v1/tenants/$TENANT_ID/secrets \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
[
  {
    "secret_id": "secret_<uuid>",
    "secret_key": "db_password",
    "created_at": "2026-04-12T10:30:00Z"
  }
]
```

#### 5.3 Update Secret
```bash
curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID/secrets \
  -H "Content-Type: application/json" \
  -d '{
    "secret_key": "db_password",
    "secret_value": "new_secure_password_456"
  }'

# Expected Response (200 OK)
{
  "success": true,
  "message": "Secret updated"
}
```

### 6. Tenant Usage & Limits Tests

#### 6.1 Get Tenant Usage
```bash
curl -X GET http://localhost:8080/api/v1/tenants/$TENANT_ID/usage \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "tenant_id": "tenant_<uuid>",
  "messages_sent": 0,
  "messages_received": 0,
  "storage_used_mb": 0,
  "bandwidth_used_gb": 0,
  "active_connections": 0,
  "last_activity": "2026-04-12T10:30:00Z"
}
```

#### 6.2 Get Tenant Limits
```bash
curl -X GET http://localhost:8080/api/v1/tenants/$TENANT_ID/limits \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "max_message_size": 10485760,
  "rate_limit_rps": 1000,
  "max_connections": 100,
  "retention_days": 30
}
```

#### 6.3 Update Tenant Limits
```bash
curl -X PUT http://localhost:8080/api/v1/tenants/$TENANT_ID/limits \
  -H "Content-Type: application/json" \
  -d '{
    "max_message_size": 52428800,
    "rate_limit_rps": 5000,
    "max_connections": 200,
    "retention_days": 60
  }'

# Expected Response (200 OK)
{
  "max_message_size": 52428800,
  "rate_limit_rps": 5000,
  "max_connections": 200,
  "retention_days": 60
}
```

#### 6.4 Reset Tenant Limits
```bash
curl -X POST http://localhost:8080/api/v1/tenants/$TENANT_ID/limits/reset \
  -H "Content-Type: application/json"

# Expected Response (200 OK) - Returns to defaults
{
  "max_message_size": 10485760,
  "rate_limit_rps": 1000,
  "max_connections": 100,
  "retention_days": 30
}
```

### 7. Notification Tests

#### 7.1 Get SMTP Configuration
```bash
curl -X GET http://localhost:8080/api/v1/notifications/smtp \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "host": "smtp.example.com",
  "port": 587,
  "from_email": "notifications@example.com",
  "use_tls": true,
  "created_at": "2026-04-12T10:00:00Z"
}
```

#### 7.2 Update SMTP Configuration
```bash
curl -X PUT http://localhost:8080/api/v1/notifications/smtp \
  -H "Content-Type: application/json" \
  -d '{
    "host": "smtp.gmail.com",
    "port": 587,
    "username": "admin@example.com",
    "password": "app_password",
    "from_email": "notifications@example.com",
    "use_tls": true
  }'

# Expected Response (200 OK)
```

#### 7.3 Test SMTP Connection
```bash
curl -X POST http://localhost:8080/api/v1/notifications/smtp/test \
  -H "Content-Type: application/json" \
  -d '{
    "recipient_email": "test@example.com"
  }'

# Expected Response (200 OK)
{
  "success": true,
  "message": "Test email sent successfully",
  "recipient": "test@example.com"
}
```

#### 7.4 Get Notification Settings
```bash
curl -X GET http://localhost:8080/api/v1/notifications/settings \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
[]
```

#### 7.5 Update Notification Settings
```bash
curl -X PUT http://localhost:8080/api/v1/notifications/settings \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "tenant_limit_exceeded",
    "enabled": true,
    "recipient_email": "admin@example.com",
    "notification_channels": ["email", "web"]
  }'

# Expected Response (200 OK or 201 Created)
{
  "id": "notif_<uuid>",
  "event_type": "tenant_limit_exceeded",
  "enabled": true,
  "recipient_email": "admin@example.com",
  "notification_channels": ["email", "web"],
  "created_at": "2026-04-12T10:30:00Z"
}
```

#### 7.6 List Notification Events
```bash
curl -X GET http://localhost:8080/api/v1/notifications/events \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
[]
```

### 8. API Information Test

#### 8.1 Get API Info
```bash
curl -X GET http://localhost:8080/api/v1/info \
  -H "Content-Type: application/json"

# Expected Response (200 OK)
{
  "name": "FastDataBroker Admin API",
  "version": "0.1.0",
  "description": "Lightweight REST API for managing FastDataBroker configurations...",
  "endpoints": [
    {
      "path": "/health",
      "method": "GET",
      "description": "Basic health check",
      "authentication": false
    },
    ...
  ],
  "documentation": "https://github.com/suraj202923/fastdatabroker/docs"
}
```

## Automated Test Script

Save as `test_admin_api.sh`:

```bash
#!/bin/bash

set -e

BASE_URL="http://localhost:8080"
API_URL="$BASE_URL/api/v1"

echo "=== FastDataBroker Admin API Tests ==="
echo ""

# Test 1: Health Check
echo "[1/8] Testing Health Check..."
curl -s -X GET $BASE_URL/health | jq .
echo ""

# Test 2: System Configuration
echo "[2/8] Testing System Configuration..."
curl -s -X GET $API_URL/system/config | jq .
echo ""

# Test 3: Create Cluster
echo "[3/8] Creating Cluster Environment..."
CLUSTER=$(curl -s -X POST $API_URL/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-cluster",
    "description": "Test cluster",
    "region": "us-east-1",
    "broker_addresses": ["broker1:6000"],
    "replication_factor": 1
  }')
CLUSTER_ID=$(echo $CLUSTER | jq -r '.id')
echo $CLUSTER | jq .
echo ""

# Test 4: Create Tenant
echo "[4/8] Creating Tenant..."
TENANT=$(curl -s -X POST $API_URL/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Tenant",
    "email": "test@example.com"
  }')
TENANT_ID=$(echo $TENANT | jq -r '.tenant_id')
echo $TENANT | jq .
echo ""

# Test 5: Create Secret
echo "[5/8] Creating Tenant Secret..."
curl -s -X POST $API_URL/tenants/$TENANT_ID/secrets \
  -H "Content-Type: application/json" \
  -d '{
    "secret_key": "api_key",
    "secret_value": "secret_value_123"
  }' | jq .
echo ""

# Test 6: Get Tenant Usage
echo "[6/8] Getting Tenant Usage..."
curl -s -X GET $API_URL/tenants/$TENANT_ID/usage | jq .
echo ""

# Test 7: Update Notification Settings
echo "[7/8] Updating Notification Settings..."
curl -s -X PUT $API_URL/notifications/settings \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "broker_failure",
    "enabled": true,
    "recipient_email": "admin@example.com",
    "notification_channels": ["email"]
  }' | jq .
echo ""

# Test 8: Get API Info
echo "[8/8] Getting API Information..."
curl -s -X GET $API_URL/info | jq .
echo ""

echo "=== All Tests Completed ==="
```

Run with:
```bash
chmod +x test_admin_api.sh
./test_admin_api.sh
```

## Performance Testing

Use `wrk` for load testing:

```bash
# Install wrk (macOS)
brew install wrk

# Install wrk (Linux)
apt-get install wrk

# Run load test
wrk -t12 -c400 -d30s http://localhost:8080/health
```

## Monitoring

Monitor resource usage during testing:

```bash
# Terminal 1: Watch memory and CPU
watch -n 1 'ps aux | grep admin-api'

# Terminal 2: Check database file size
watch -n 1 'ls -lh admin.db'

# Terminal 3: Monitor logs
tail -f /var/log/admin-api.log
```

## Success Criteria

✅ All health checks return 200 OK
✅ All CRUD operations succeed
✅ Error responses have correct HTTP status codes
✅ Database operations maintain data integrity
✅ Response times < 50ms (99th percentile)
✅ Memory usage stays within 5-10MB
✅ Concurrent requests handled smoothly (1000+)
