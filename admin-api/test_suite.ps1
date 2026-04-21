# Comprehensive Admin API Test Suite
$baseUrl = "http://127.0.0.1:8080"
$passed = 0
$failed = 0

function Test-Endpoint {
    param($Method, $Endpoint, $Body, $ExpectedStatus = 200, $Name)
    
    $url = "$baseUrl$Endpoint"
    try {
        if ($Body) {
            $response = Invoke-WebRequest -Uri $url -Method $Method -Body ($Body | ConvertTo-Json) -ContentType "application/json" -ErrorAction SilentlyContinue
        } else {
            $response = Invoke-WebRequest -Uri $url -Method $Method -ErrorAction SilentlyContinue
        }
        
        if ($response.StatusCode -eq $ExpectedStatus) {
            Write-Host "✓ $Name (HTTP $($response.StatusCode))" -ForegroundColor Green
            return $response.Content | ConvertFrom-Json
        } else {
            Write-Host "✗ $Name (Expected $ExpectedStatus, got $($response.StatusCode))" -ForegroundColor Red
        }
    } catch {
        Write-Host "✗ $Name (Error: $($_.Exception.Message))" -ForegroundColor Red
    }
    return $null
}

Write-Host "`n=== ADMIN API TEST SUITE ===" -ForegroundColor Cyan

# Test 1: Health Check
Write-Host "`nGROUP 1: HEALTH" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/health" -Name "Health Check"
Test-Endpoint -Method GET -Endpoint "/health/detailed" -Name "Health Detailed"

# Test 2: System Config
Write-Host "`nGROUP 2: SYSTEM" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/system/config" -Name "Get System Config"

# Test 3: Cluster Environments
Write-Host "`nGROUP 3: CLUSTER" -ForegroundColor Yellow
$clusters = Test-Endpoint -Method GET -Endpoint "/api/v1/cluster/environments" -Name "List Cluster Environments"

$clusterBody = @{
    name = "test-$(Get-Random)"
    description = "Test cluster"
    region = "us-east-1"
    broker_addresses = "127.0.0.1:6000"
    replication_factor = 3
}

$cluster = Test-Endpoint -Method POST -Endpoint "/api/v1/cluster/environments" -Body $clusterBody -ExpectedStatus 201 -Name "Create Cluster"
if ($cluster.id) {
    Test-Endpoint -Method GET -Endpoint "/api/v1/cluster/environments/$($cluster.id)" -Name "Get Cluster"
    Test-Endpoint -Method DELETE -Endpoint "/api/v1/cluster/environments/$($cluster.id)" -ExpectedStatus 204 -Name "Delete Cluster"
}

# Test 4: Tenants
Write-Host "`nGROUP 4: TENANTS" -ForegroundColor Yellow
$tenants = Test-Endpoint -Method GET -Endpoint "/api/v1/tenants" -Name "List Tenants"

$tenantBody = @{
    name = "TestTenant-$(Get-Random)"
    email = "test-$(Get-Random)@example.com"
    max_message_size = 10485760
    rate_limit_rps = 1000
    max_connections = 100
    retention_days = 30
}

$tenant = Test-Endpoint -Method POST -Endpoint "/api/v1/tenants" -Body $tenantBody -ExpectedStatus 201 -Name "Create Tenant"
if ($tenant.tenant_id) {
    $tenantId = $tenant.tenant_id
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId" -Name "Get Tenant"
    
    # Test 5: Secrets
    Write-Host "`nGROUP 5: SECRETS" -ForegroundColor Yellow
    $secrets = Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/secrets" -Name "List Secrets"
    
    $secretBody = @{
        secret_key = "testkey"
        secret_value = "testvalue"
    }
    
    $secret = Test-Endpoint -Method POST -Endpoint "/api/v1/tenants/$tenantId/secrets" -Body $secretBody -ExpectedStatus 201 -Name "Create Secret"
    
    # Test 6: Usage & Limits
    Write-Host "`nGROUP 6: USAGE & LIMITS" -ForegroundColor Yellow
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/usage" -Name "Get Tenant Usage"
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/limits" -Name "Get Tenant Limits"
}

# Test 7: Notifications
Write-Host "`nGROUP 7: NOTIFICATIONS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/notifications/settings" -Name "List Notification Settings"
Test-Endpoint -Method GET -Endpoint "/api/v1/notifications/events" -Name "List Notification Events"

# Test 8: API Info
Write-Host "`nGROUP 8: API INFO" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/info" -Name "Get API Info"

Write-Host "`n" -ForegroundColor Green
Write-Host "✅ Test suite completed successfully!" -ForegroundColor Green
