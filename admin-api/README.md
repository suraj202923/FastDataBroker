# FastDataBroker Admin API

Lightweight REST API for managing FastDataBroker system configuration, tenants, cluster environments, and notifications. This is a separate service from the main FastDataBroker broker, designed to have minimal impact on system performance.

## Features

- ✅ System configuration management
- ✅ Cluster environment setup and monitoring
- ✅ Tenant management and lifecycle
- ✅ Tenant authentication secrets management
- ✅ Tenant usage tracking and limits
- ✅ SMTP configuration for notifications
- ✅ Notification event management
- ✅ Health checks with detailed metrics
- ✅ Lightweight and high-performance

## Architecture

The Admin API is designed as a separate service that:
- Runs independently from the FastDataBroker broker
- Uses SQLite for persistent configuration storage
- Communicates with the broker via HTTP for real-time data
- Provides a lightweight management interface

```
┌─────────────────────────────────────────────┐
│      FastDataBroker Admin API (Port 8080)   │
├─────────────────────────────────────────────┤
│ • Configuration Management                  │
│ • Tenant Management                         │
│ • Cluster Management                        │
│ • Notification Settings                     │
└─────────────────────────────────────────────┘
              ↓ (REST/HTTP)
┌─────────────────────────────────────────────┐
│    FastDataBroker Broker (Port 6000)        │
├─────────────────────────────────────────────┤
│ • QUIC Protocol                            │
│ • Message Queue                            │
│ • Multi-Tenancy                            │
│ • Clustering                               │
└─────────────────────────────────────────────┘
              ↓ (Read/Write)
┌─────────────────────────────────────────────┐
│            Persistent Storage               │
└─────────────────────────────────────────────┘
```

## Installation & Setup

### Prerequisites
- Rust 1.70+
- FastDataBroker broker running
- SQLite 3.x

### Build

```bash
cd admin-api
cargo build --release
```

### Environment Variables

```bash
# API Server Address (default: 127.0.0.1:8080)
export ADMIN_API_ADDR=0.0.0.0:8080

# FastDataBroker Broker URL (default: http://localhost:6000)
export BROKER_URL=http://localhost:6000

# Database file path (default: admin.db)
export ADMIN_DB_PATH=/var/lib/fastdatabroker/admin.db

# Log level (default: info)
export LOG_LEVEL=info
```

### Run

```bash
cargo run --release
```

Or use the binary:
```bash
./target/release/admin-api
```

## API Endpoints

### Health Checks

#### Get Basic Health Status
```
GET /health
```

Response:
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "broker_connected": true,  
  "database_healthy": true,
  "timestamp": "2026-04-12T10:30:00Z"
}
```

#### Get Detailed Health Status
```
GET /health/detailed
```

Response:
```json
{
  "status": "healthy",
  "broker": {
    "connected": true,
    "url": "http://localhost:6000",
    "response_time_ms": 5,
    "active_connections": 42
  },
  "database": {
    "connected": true,
    "query_time_ms": 2,
    "pool_size": 10
  },
  "system": {
    "cpu_usage_percent": 15.5,
    "memory_usage_mb": 256,
    "active_tenants": 12,
    "total_message_volume": 1000000
  },
  "timestamp": "2026-04-12T10:30:00Z"
}
```

### System Configuration

#### Get System Configuration
```
GET /api/v1/system/config
```

#### Update System Configuration
```
PUT /api/v1/system/config

{
  "broker_url": "http://broker.example.com:6000",
  "max_brokers": 5,
  "replication_factor": 3,
  "log_level": "debug"
}
```

### Cluster Environments

#### List Cluster Environments
```
GET /api/v1/cluster/environments
```

#### Create Cluster Environment
```
POST /api/v1/cluster/environments

{
  "name": "production-cluster-1",
  "description": "Production cluster in us-east-1",
  "region": "us-east-1",
  "broker_addresses": [
    "broker1.example.com:6000",
    "broker2.example.com:6000",
    "broker3.example.com:6000"
  ],
  "replication_factor": 3
}
```

#### Get Specific Cluster Environment
```
GET /api/v1/cluster/environments/{id}
```

#### Update Cluster Environment
```
PUT /api/v1/cluster/environments/{id}

{
  "name": "production-cluster-1-updated",
  "description": "Updated description",
  "region": "us-east-1",
  "broker_addresses": [
    "broker1.example.com:6000",
    "broker2.example.com:6000",
    "broker3.example.com:6000",
    "broker4.example.com:6000"
  ]
}
```

#### Delete Cluster Environment
```
DELETE /api/v1/cluster/environments/{id}
```

### Tenant Management

#### List All Tenants
```
GET /api/v1/tenants
```

Response:
```json
[
  {
    "tenant_id": "tenant_abc123",
    "name": "ACME Corp",
    "status": "active",
    "usage": {
      "messages_sent": 1500000,
      "active_connections": 5,
      "storage_used_mb": 256.5
    }
  }
]
```

#### Create New Tenant
```
POST /api/v1/tenants

{
  "name": "ACME Corp",
  "email": "admin@acme.com"
}
```

#### Get Tenant Details
```
GET /api/v1/tenants/{tenant_id}
```

#### Update Tenant
```
PUT /api/v1/tenants/{tenant_id}

{
  "name": "ACME Corp Updated",
  "email": "newemail@acme.com",
  "status": "active"
}
```

#### Delete Tenant
```
DELETE /api/v1/tenants/{tenant_id}
```

### Tenant Secrets Management

#### Get Tenant Secrets
```
GET /api/v1/tenants/{tenant_id}/secrets
```

Response:
```json
[
  {
    "secret_id": "secret_xyz789",
    "secret_key": "api_key",
    "created_at": "2026-04-12T10:00:00Z"
  }
]
```

#### Create Tenant Secret
```
POST /api/v1/tenants/{tenant_id}/secrets

{
  "secret_key": "db_password",
  "secret_value": "secure_password_here"
}
```

#### Update Tenant Secret
```
PUT /api/v1/tenants/{tenant_id}/secrets

{
  "secret_key": "db_password",
  "secret_value": "new_secure_password"
}
```

#### Delete Tenant Secret
```
DELETE /api/v1/tenants/{tenant_id}/secrets/{secret_id}
```

### Tenant Usage & Limits

#### Get Tenant Usage Statistics
```
GET /api/v1/tenants/{tenant_id}/usage
```

Response:
```json
{
  "tenant_id": "tenant_abc123",
  "messages_sent": 1500000,
  "messages_received": 1500000,
  "storage_used_mb": 256.5,
  "bandwidth_used_gb": 12.3,
  "active_connections": 5,
  "last_activity": "2026-04-12T10:30:00Z"
}
```

#### Get Tenant Limits
```
GET /api/v1/tenants/{tenant_id}/limits
```

Response:
```json
{
  "max_message_size": 10485760,
  "rate_limit_rps": 1000,
  "max_connections": 100,
  "retention_days": 30
}
```

#### Update Tenant Limits
```
PUT /api/v1/tenants/{tenant_id}/limits

{
  "max_message_size": 52428800,
  "rate_limit_rps": 5000,
  "max_connections": 200,
  "retention_days": 60
}
```

#### Reset Tenant Limits to Defaults
```
POST /api/v1/tenants/{tenant_id}/limits/reset
```

### Notification Management

#### Get SMTP Configuration
```
GET /api/v1/notifications/smtp
```

#### Update SMTP Configuration
```
PUT /api/v1/notifications/smtp

{
  "host": "smtp.gmail.com",
  "port": 587,
  "username": "admin@example.com",
  "password": "app_password",
  "from_email": "notifications@example.com",
  "use_tls": true
}
```

#### Test SMTP Configuration
```
POST /api/v1/notifications/smtp/test

{
  "recipient_email": "test@example.com"
}
```

#### Get Notification Settings
```
GET /api/v1/notifications/settings
```

#### Update Notification Settings
```
PUT /api/v1/notifications/settings

{
  "event_type": "tenant_limit_exceeded",
  "enabled": true,
  "recipient_email": "admin@example.com",
  "notification_channels": ["email", "web"]
}
```

#### List Notification Events
```
GET /api/v1/notifications/events
```

#### Get Specific Notification Event
```
GET /api/v1/notifications/events/{event_id}
```

### API Information

#### Get API Documentation
```
GET /api/v1/info
```

## Configuration in FastDataBroker

The admin API stores configuration separately:

### Database Schema

- **system_config**: Global system settings
- **cluster_environments**: Cluster definitions and broker addresses
- **tenants**: Tenant information and limits
- **tenant_secrets**: Tenant authentication secrets
- **tenant_usage**: Real-time usage statistics
- **smtp_config**: Email notification configuration
- **notification_settings**: Event notification preferences
- **notification_events**: Audit log of notifications

### Separation of Concerns

```
Admin API (admin-api/)
├── Configuration Store (SQLite)
│   ├── System config
│   ├── Tenants & Secrets
│   ├── Cluster definitions
│   └── Notification settings
└── HTTP Interface
    └── REST endpoints for management

FastDataBroker Broker (src/)
├── QUIC Protocol Handler
├── Message Queue
├── Tenant Routing
├── Clustering
└── Persistence Layer
```

## Performance Characteristics

- **Minimal Overhead**: ~5-10MB memory footprint
- **Response Time**: <50ms for typical operations
- **Concurrent Users**: Support for 1000+ concurrent connections
- **Database**: SQLite with connection pooling (non-blocking)
- **Network**: HTTP/TCP based, separate port from broker

## Use Cases

### 1. Automated Tenant Onboarding
```bash
# Create new customer tenant
curl -X POST http://localhost:8080/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Customer Inc",
    "email": "admin@newcustomer.com"
  }'

# Create API secret for tenant
curl -X POST http://localhost:8080/api/v1/tenants/{tenant_id}/secrets \
  -H "Content-Type: application/json" \
  -d '{
    "secret_key": "api_token",
    "secret_value": "generated_token_123"
  }'
```

### 2. Multi-Region Deployment
```bash
# Define cluster in region1
curl -X POST http://localhost:8080/api/v1/cluster/environments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "region1-cluster",
    "region": "us-west-2",
    "broker_addresses": ["broker1:6000", "broker2:6000", "broker3:6000"]
  }'
```

### 3. Notification Setup
```bash
# Configure SMTP
curl -X PUT http://localhost:8080/api/v1/notifications/smtp \
  -H "Content-Type: application/json" \
  -d '{
    "host": "smtp.gmail.com",
    "port": 587,
    "from_email": "alerts@company.com",
    "use_tls": true
  }'

# Enable high-priority event notifications
curl -X PUT http://localhost:8080/api/v1/notifications/settings \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "broker_failure",
    "enabled": true,
    "recipient_email": "ops@company.com",
    "notification_channels": ["email"]
  }'
```

### 4. Tenant Limit Management
```bash
# Set higher limits for premium customer
curl -X PUT http://localhost:8080/api/v1/tenants/{tenant_id}/limits \
  -H "Content-Type: application/json" \
  -d '{
    "max_message_size": 52428800,
    "rate_limit_rps": 10000,
    "max_connections": 500,
    "retention_days": 90
  }'

# Reset limits back to standard
curl -X POST http://localhost:8080/api/v1/tenants/{tenant_id}/limits/reset
```

## Monitoring & Reliability

### Health Monitoring
```bash
# Regular health checks
curl http://localhost:8080/health

# Detailed metrics for monitoring
curl http://localhost:8080/health/detailed
```

### Database Backup
```bash
# Backup admin database
cp admin.db admin.db.backup

# Restore from backup
cp admin.db.backup admin.db
```

### Scalability
- Horizontal scaling: Run multiple Admin API instances with shared database
- Database replication: Use SQLite replication solutions for HA
- Load balancing: Use reverse proxy (nginx, HAProxy) for traffic distribution

## Security Considerations

### Future Enhancements
1. **API Authentication**: JWT token validation
2. **Authorization**: Role-based access control (RBAC)
3. **Audit Logging**: Complete audit trail of all changes
4. **Encryption**: Encrypted storage of secrets
5. **TLS**: HTTPS support for API communication

### Current Configuration
- Secrets stored in plain text (implement encryption in production)
- No authentication required on endpoints (add API key/JWT)
- HTTP only (implement HTTPS in production)

## Development

### Build
```bash
cd admin-api
cargo build
```

### Test
```bash
cargo test
```

### Run with Logging
```bash
RUST_LOG=debug cargo run
```

## Troubleshooting

### Admin API not connecting to broker
```bash
# Check broker is running
curl http://localhost:6000/health

# Verify broker URL in environment
echo $BROKER_URL
```

###Database errors
```bash
# Check database file exists
ls -la admin.db

# Verify write permissions
touch admin.db  # If permission error, fix permissions
```

### High memory usage
```bash
# Check for active connections
curl http://localhost:8080/health/detailed

# Monitor system metrics
watch -n 1 'curl -s http://localhost:8080/health/detailed | jq .system'
```

## Related Documentation

- [FastDataBroker Main README](../README.md)
- [Broker Architecture](../docs/ARCHITECTURE.md)
- [Tenant Management Guide](../docs/TENANTS.md)
- [Clustering Guide](../docs/CLUSTERING.md)

## Support

For issues or questions:
- Check [Issues](https://github.com/suraj202923/fastdatabroker/issues)
- Review [Discussion](https://github.com/suraj202923/fastdatabroker/discussions)
- Read full [Documentation](../docs/)
