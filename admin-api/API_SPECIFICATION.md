# FastDataBroker Admin API - Specification

## API Version: 1.0.0

### Base URL
```
http://localhost:8080/api/v1
```

### Response Format

All responses are in JSON format with the following structure:

#### Success Response (2xx)
```json
{
  "data": {},
  "timestamp": "2026-04-12T10:30:00Z"
}
```

#### Error Response (4xx, 5xx)
```json
{
  "error": "ERROR_CODE",
  "code": 400,
  "message": "Human readable error message",
  "timestamp": "2026-04-12T10:30:00Z"
}
```

## 1. Health & Status Endpoints

### 1.1 Basic Health Check
```
GET /health
```

**Response (200 OK)**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "broker_connected": true,
  "database_healthy": true,
  "timestamp": "2026-04-12T10:30:00Z"
}
```

**Status Codes:**
- `200 OK` - System is healthy
- `503 Service Unavailable` - System is unhealthy

### 1.2 Detailed Health Check
```
GET /health/detailed
```

**Response (200 OK)**
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

**Status Codes:**
- `200 OK` - Detailed health retrieved successfully

## 2. System Configuration Endpoints

### 2.1 Get System Configuration
```
GET /system/config
```

**Response (200 OK)**
```json
{
  "id": "default",
  "broker_url": "http://localhost:6000",
  "max_brokers": 3,
  "replication_factor": 3,
  "log_level": "info",
  "created_at": "2026-04-01T00:00:00Z",
  "updated_at": "2026-04-12T10:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Configuration retrieved successfully
- `404 Not Found` - No configuration found
- `500 Internal Server Error` - Database error

### 2.2 Update System Configuration
```
PUT /system/config
Content-Type: application/json
```

**Request Body**
```json
{
  "broker_url": "http://new-broker:6000",
  "max_brokers": 5,
  "replication_factor": 3,
  "log_level": "debug"
}
```

**Response (200 OK)**
```json
{
  "id": "default",
  "broker_url": "http://new-broker:6000",
  "max_brokers": 5,
  "replication_factor": 3,
  "log_level": "debug",
  "created_at": "2026-04-01T00:00:00Z",
  "updated_at": "2026-04-12T10:30:00Z"
}
```

**Status Codes:**
- `200 OK` - Configuration updated successfully
- `400 Bad Request` - Invalid parameters
- `500 Internal Server Error` - Database error

## 3. Cluster Environment Endpoints

### 3.1 List Cluster Environments
```
GET /cluster/environments
```

**Query Parameters:**
- `limit` (optional): Maximum results to return (default: 100)
- `offset` (optional): Pagination offset (default: 0)

**Response (200 OK)**
```json
[
  {
    "id": "cluster-1",
    "name": "production-us-east",
    "description": "Production cluster in US East",
    "region": "us-east-1",
    "broker_addresses": [
      "broker1.example.com:6000",
      "broker2.example.com:6000",
      "broker3.example.com:6000"
    ],
    "replication_factor": 3,
    "status": "active",
    "created_at": "2026-03-01T00:00:00Z",
    "updated_at": "2026-04-12T10:00:00Z"
  }
]
```

### 3.2 Create Cluster Environment
```
POST /cluster/environments
Content-Type: application/json
```

**Request Body**
```json
{
  "name": "production-us-west",
  "description": "Production cluster in US West",
  "region": "us-west-2",
  "broker_addresses": [
    "broker1.us-west.example.com:6000",
    "broker2.us-west.example.com:6000",
    "broker3.us-west.example.com:6000"
  ],
  "replication_factor": 3
}
```

**Response (201 Created)**
```json
{
  "id": "cluster-2",
  "name": "production-us-west",
  "description": "Production cluster in US West",
  "region": "us-west-2",
  "broker_addresses": [
    "broker1.us-west.example.com:6000",
    "broker2.us-west.example.com:6000",
    "broker3.us-west.example.com:6000"
  ],
  "replication_factor": 3,
  "status": "active",
  "created_at": "2026-04-12T10:30:00Z",
  "updated_at": "2026-04-12T10:30:00Z"
}
```

**Status Codes:**
- `201 Created` - Cluster environment created successfully
- `400 Bad Request` - Invalid parameters
- `409 Conflict` - Cluster name already exists

### 3.3 Get Specific Cluster Environment
```
GET /cluster/environments/{id}
```

**Path Parameters:**
- `id` (required): Cluster environment ID

**Response (200 OK)**
```json
{
  "id": "cluster-1",
  "name": "production-us-east",
  ...
}
```

**Status Codes:**
- `200 OK` - Cluster retrieved successfully
- `404 Not Found` - Cluster not found

### 3.4 Update Cluster Environment
```
PUT /cluster/environments/{id}
Content-Type: application/json
```

**Path Parameters:**
- `id` (required): Cluster environment ID

**Request Body**
```json
{
  "name": "production-us-east-v2",
  "description": "Updated description",
  "broker_addresses": [
    "broker1.example.com:6000",
    "broker2.example.com:6000",
    "broker3.example.com:6000",
    "broker4.example.com:6000"
  ]
}
```

**Response (200 OK)**
```json
{
  "id": "cluster-1",
  "name": "production-us-east-v2",
  "updated_at": "2026-04-12T11:00:00Z",
  ...
}
```

### 3.5 Delete Cluster Environment
```
DELETE /cluster/environments/{id}
```

**Response (204 No Content)**

**Status Codes:**
- `204 No Content` - Cluster deleted successfully
- `404 Not Found` - Cluster not found

## 4. Tenant Management Endpoints

### 4.1 List Tenants
```
GET /tenants
```

**Query Parameters:**
- `status` (optional): Filter by status (active, suspended, deleted)
- `limit` (optional): Maximum results (default: 100)

**Response (200 OK)**
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

### 4.2 Create Tenant
```
POST /tenants
Content-Type: application/json
```

**Request Body**
```json
{
  "name": "ACME Corp",
  "email": "admin@acme.com"
}
```

**Response (201 Created)**
```json
{
  "tenant_id": "tenant_abc123",
  "name": "ACME Corp",
  "email": "admin@acme.com",
  "status": "active",
  "limits": {
    "max_message_size": 10485760,
    "rate_limit_rps": 1000,
    "max_connections": 100,
    "retention_days": 30
  },
  "created_at": "2026-04-12T10:30:00Z",
  "updated_at": "2026-04-12T10:30:00Z"
}
```

**Status Codes:**
- `201 Created` - Tenant created successfully
- `400 Bad Request` - Invalid parameters
- `409 Conflict` - Email already exists

### 4.3 Get Tenant
```
GET /tenants/{tenant_id}
```

**Response (200 OK)**
```json
{
  "tenant_id": "tenant_abc123",
  "name": "ACME Corp",
  "email": "admin@acme.com",
  "status": "active",
  "limits": {
    "max_message_size": 10485760,
    "rate_limit_rps": 1000,
    "max_connections": 100,
    "retention_days": 30
  },
  "created_at": "2026-04-12T10:30:00Z",
  "updated_at": "2026-04-12T10:30:00Z"
}
```

### 4.4 Update Tenant
```
PUT /tenants/{tenant_id}
Content-Type: application/json
```

**Request Body**
```json
{
  "name": "ACME Corp Updated",
  "email": "newemail@acme.com",
  "status": "active"
}
```

**Response (200 OK)**
```json
{
  "tenant_id": "tenant_abc123",
  "name": "ACME Corp Updated",
  "updated_at": "2026-04-12T11:00:00Z",
  ...
}
```

### 4.5 Delete Tenant
```
DELETE /tenants/{tenant_id}
```

**Response (204 No Content)**

## 5. Tenant Secrets Endpoints

### 5.1 Get Tenant Secrets
```
GET /tenants/{tenant_id}/secrets
```

**Response (200 OK)**
```json
[
  {
    "secret_id": "secret_xyz789",
    "secret_key": "api_token",
    "created_at": "2026-04-01T00:00:00Z"
  }
]
```

### 5.2 Create Tenant Secret
```
POST /tenants/{tenant_id}/secrets
Content-Type: application/json
```

**Request Body**
```json
{
  "secret_key": "db_password",
  "secret_value": "secure_password_123"
}
```

**Response (201 Created)**
```json
{
  "secret_id": "secret_xyz789",
  "secret_key": "db_password",
  "created_at": "2026-04-12T10:30:00Z"
}
```

**Status Codes:**
- `201 Created` - Secret created
- `409 Conflict` - Secret key already exists

### 5.3 Update Tenant Secret
```
PUT /tenants/{tenant_id}/secrets
Content-Type: application/json
```

**Request Body**
```json
{
  "secret_key": "db_password",
  "secret_value": "new_secure_password_456"
}
```

**Response (200 OK)**
```json
{
  "success": true,
  "message": "Secret updated"
}
```

### 5.4 Delete Tenant Secret
```
DELETE /tenants/{tenant_id}/secrets/{secret_id}
```

**Response (204 No Content)**

## 6. Tenant Usage & Limits Endpoints

### 6.1 Get Tenant Usage
```
GET /tenants/{tenant_id}/usage
```

**Response (200 OK)**
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

### 6.2 Get Tenant Limits
```
GET /tenants/{tenant_id}/limits
```

**Response (200 OK)**
```json
{
  "max_message_size": 10485760,
  "rate_limit_rps": 1000,
  "max_connections": 100,
  "retention_days": 30
}
```

### 6.3 Update Tenant Limits
```
PUT /tenants/{tenant_id}/limits
Content-Type: application/json
```

**Request Body**
```json
{
  "max_message_size": 52428800,
  "rate_limit_rps": 5000,
  "max_connections": 200,
  "retention_days": 60
}
```

**Response (200 OK)**
```json
{
  "max_message_size": 52428800,
  "rate_limit_rps": 5000,
  "max_connections": 200,
  "retention_days": 60
}
```

### 6.4 Reset Tenant Limits
```
POST /tenants/{tenant_id}/limits/reset
```

**Response (200 OK)**
```json
{
  "max_message_size": 10485760,
  "rate_limit_rps": 1000,
  "max_connections": 100,
  "retention_days": 30
}
```

## 7. Notification Endpoints

### 7.1 Get SMTP Configuration
```
GET /notifications/smtp
```

**Response (200 OK)**
```json
{
  "host": "smtp.gmail.com",
  "port": 587,
  "from_email": "notifications@example.com",
  "use_tls": true,
  "created_at": "2026-03-01T00:00:00Z",
  "updated_at": "2026-04-01T00:00:00Z"
}
```

### 7.2 Update SMTP Configuration
```
PUT /notifications/smtp
Content-Type: application/json
```

**Request Body**
```json
{
  "host": "smtp.gmail.com",
  "port": 587,
  "username": "admin@example.com",
  "password": "app_password",
  "from_email": "notifications@example.com",
  "use_tls": true
}
```

**Response (200 OK)**
```json
{
  "host": "smtp.gmail.com",
  "port": 587,
  "from_email": "notifications@example.com",
  "use_tls": true,
  "updated_at": "2026-04-12T10:30:00Z"
}
```

### 7.3 Test SMTP Connection
```
POST /notifications/smtp/test
Content-Type: application/json
```

**Request Body**
```json
{
  "recipient_email": "test@example.com"
}
```

**Response (200 OK)**
```json
{
  "success": true,
  "message": "Test email sent successfully",
  "recipient": "test@example.com"
}
```

### 7.4 Get Notification Settings
```
GET /notifications/settings
```

**Response (200 OK)**
```json
[
  {
    "id": "notif-1",
    "event_type": "tenant_limit_exceeded",
    "enabled": true,
    "recipient_email": "admin@example.com",
    "notification_channels": ["email", "web"],
    "created_at": "2026-04-01T00:00:00Z"
  }
]
```

### 7.5 Update Notification Settings
```
PUT /notifications/settings
Content-Type: application/json
```

**Request Body**
```json
{
  "event_type": "broker_failure",
  "enabled": true,
  "recipient_email": "ops@example.com",
  "notification_channels": ["email", "sms"]
}
```

**Response (200 OK)**
```json
{
  "id": "notif-2",
  "event_type": "broker_failure",
  "enabled": true,
  "recipient_email": "ops@example.com",
  "notification_channels": ["email", "sms"],
  "created_at": "2026-04-12T10:30:00Z"
}
```

### 7.6 List Notification Events
```
GET /notifications/events
```

**Query Parameters:**
- `limit` (optional): Maximum results (default: 100)

**Response (200 OK)**
```json
[
  {
    "event_id": "event-1",
    "event_type": "tenant_limit_exceeded",
    "title": "Tenant Storage Limit Exceeded",
    "description": "ACME Corp has exceeded storage limit",
    "severity": "warning",
    "created_at": "2026-04-12T10:00:00Z"
  }
]
```

### 7.7 Get Specific Notification Event
```
GET /notifications/events/{event_id}
```

**Response (200 OK)**
```json
{
  "event_id": "event-1",
  "event_type": "tenant_limit_exceeded",
  "title": "Tenant Storage Limit Exceeded",
  "description": "ACME Corp has exceeded storage limit",
  "severity": "warning",
  "created_at": "2026-04-12T10:00:00Z"
}
```

## 8. API Information Endpoint

### 8.1 Get API Information
```
GET /info
```

**Response (200 OK)**
```json
{
  "name": "FastDataBroker Admin API",
  "version": "0.1.0",
  "description": "Lightweight REST API for managing FastDataBroker...",
  "endpoints": [
    {
      "path": "/health",
      "method": "GET",
      "description": "Basic health check",
      "authentication": false
    }
  ],
  "documentation": "https://github.com/suraj202923/fastdatabroker/docs"
}
```

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 400 | BAD_REQUEST | Invalid request parameters |
| 404 | NOT_FOUND | Resource not found |
| 409 | CONFLICT | Resource conflict (e.g., duplicate name) |
| 500 | INTERNAL_ERROR | Internal server error |
| 502 | BROKER_ERROR | Broker communication error |
| 503 | SERVICE_UNAVAILABLE | Service temporarily unavailable |

## Rate Limiting

Currently not enforced, but recommended for production:
- Limit: 1000 requests per minute per IP
- Burst: 10 requests per second

## Authentication

Currently no authentication required. For production, implement:
- JWT token validation
- API Key authentication
- OAuth2 integration
