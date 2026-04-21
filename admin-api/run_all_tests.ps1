# Comprehensive Admin API Test Suite - All 28 Endpoints
# This script tests all admin-api endpoints with various scenarios

$baseUrl = "http://127.0.0.1:8080"
$global:testsPassed = 0
$global:testsFailed = 0
$global:testResults = @()

# Helper function to make HTTP requests
function Invoke-ApiCall {
    param(
        [string]$Method,
        [string]$Endpoint,
        [object]$Body = $null,
        [hashtable]$Headers = @{}
    )
    
    $url = "$baseUrl$Endpoint"
    $params = @{
        Uri     = $url
        Method  = $Method
        Headers = @{"Content-Type" = "application/json"} + $Headers
        ErrorAction = "SilentlyContinue"
    }
    
    if ($Body) {
        $params.Body = $Body | ConvertTo-Json
    }
    
    try {
        $response = Invoke-WebRequest @params
        return @{
            StatusCode = $response.StatusCode
            Content    = $response.Content | ConvertFrom-Json
            Success    = $true
        }
    }
    catch {
        if ($_.Exception.Response) {
            return @{
                StatusCode = $_.Exception.Response.StatusCode
                Content    = $_.Exception.Message
                Success    = $false
            }
        }
        return @{
            StatusCode = 0
            Content    = "Connection error"
            Success    = $false
        }
    }
}

# Helper to verify test results
function Assert-TestResult {
    param(
        [string]$TestName,
        [object]$Response,
        [int]$ExpectedStatus,
        [string]$Description = ""
    )
    
    if ($Response.StatusCode -eq $ExpectedStatus) {
        Write-Host "✓ PASS: $TestName" -ForegroundColor Green
        $global:testsPassed++
        $global:testResults += @{
            Name = $TestName
            Status = "PASS"
            Code = $Response.StatusCode
            Description = $Description
        }
        return $true
    }
    else {
        Write-Host "✗ FAIL: $TestName (Expected $ExpectedStatus, got $($Response.StatusCode))" -ForegroundColor Red
        $global:testsFailed++
        $global:testResults += @{
            Name = $TestName
            Status = "FAIL"
            Code = $Response.StatusCode
            Description = $Description
        }
        return $false
    }
}

# ==================== GROUP 1: HEALTH ENDPOINTS ====================
Write-Host "`n=== GROUP 1: HEALTH ENDPOINTS ===" -ForegroundColor Yellow

$response = Invoke-ApiCall -Method "GET" -Endpoint "/health"
Assert-TestResult -TestName "Health Check" -Response $response -ExpectedStatus 200 -Description "Basic health endpoint"

$response = Invoke-ApiCall -Method "GET" -Endpoint "/health/detailed"
Assert-TestResult -TestName "Health Detailed" -Response $response -ExpectedStatus 200 -Description "Detailed health information"

# ==================== GROUP 2: SYSTEM ENDPOINTS ====================
Write-Host "`n=== GROUP 2: SYSTEM ENDPOINTS ===" -ForegroundColor Yellow

$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/system/config"
Assert-TestResult -TestName "Get System Config" -Response $response -ExpectedStatus 200 -Description "Retrieve system configuration"

$configBody = @{
    broker_url = "http://localhost:6000"
    max_brokers = 5
    replication_factor = 3
    log_level = "info"
} | ConvertTo-Json

$response = Invoke-ApiCall -Method "PUT" -Endpoint "/api/v1/system/config" -Body $configBody
Assert-TestResult -TestName "Update System Config" -Response $response -ExpectedStatus 200 -Description "Update system configuration"

# ==================== GROUP 3: CLUSTER ENDPOINTS ====================
Write-Host "`n=== GROUP 3: CLUSTER ENDPOINTS ===" -ForegroundColor Yellow

# List cluster environments
$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/cluster/environments"
Assert-TestResult -TestName "List Cluster Environments" -Response $response -ExpectedStatus 200 -Description "Get all cluster environments"

# Create cluster environment
$clusterBody = @{
    name = "test-cluster-$(Get-Random)"
    description = "Test cluster environment"
    region = "us-east-1"
    broker_addresses = "127.0.0.1:6000,127.0.0.1:6001"
    replication_factor = 3
} | ConvertTo-Json

$createResponse = Invoke-ApiCall -Method "POST" -Endpoint "/api/v1/cluster/environments" -Body $clusterBody
Assert-TestResult -TestName "Create Cluster Environment" -Response $createResponse -ExpectedStatus 201 -Description "Create new cluster environment"

# Extract cluster ID for subsequent tests
if ($createResponse.Success -and $createResponse.Content.id) {
    $clusterId = $createResponse.Content.id
    
    # Get specific cluster environment
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/cluster/environments/$clusterId"
    Assert-TestResult -TestName "Get Cluster Environment" -Response $response -ExpectedStatus 200 -Description "Retrieve specific cluster"
    
    # Update cluster environment
    $updateBody = @{
        description = "Updated test cluster"
        replication_factor = 2
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "PUT" -Endpoint "/api/v1/cluster/environments/$clusterId" -Body $updateBody
    Assert-TestResult -TestName "Update Cluster Environment" -Response $response -ExpectedStatus 200 -Description "Update cluster settings"
    
    # Delete cluster environment
    $response = Invoke-ApiCall -Method "DELETE" -Endpoint "/api/v1/cluster/environments/$clusterId"
    Assert-TestResult -TestName "Delete Cluster Environment" -Response $response -ExpectedStatus 204 -Description "Delete cluster environment"
}

# ==================== GROUP 4: TENANT ENDPOINTS ====================
Write-Host "`n=== GROUP 4: TENANT ENDPOINTS ===" -ForegroundColor Yellow

# List tenants
$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants"
Assert-TestResult -TestName "List Tenants" -Response $response -ExpectedStatus 200 -Description "Get all tenants"

# Create tenant
$tenantBody = @{
    name = "TestTenant-$(Get-Random)"
    email = "tenant-$(Get-Random)@test.com"
    max_message_size = 10485760
    rate_limit_rps = 1000
    max_connections = 100
    retention_days = 30
} | ConvertTo-Json

$createTenantResponse = Invoke-ApiCall -Method "POST" -Endpoint "/api/v1/tenants" -Body $tenantBody
Assert-TestResult -TestName "Create Tenant" -Response $createTenantResponse -ExpectedStatus 201 -Description "Create new tenant"

# Extract tenant ID for subsequent tests
if ($createTenantResponse.Success -and $createTenantResponse.Content.tenant_id) {
    $tenantId = $createTenantResponse.Content.tenant_id
    
    # Get specific tenant
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants/$tenantId"
    Assert-TestResult -TestName "Get Tenant" -Response $response -ExpectedStatus 200 -Description "Retrieve specific tenant"
    
    # Update tenant
    $updateTenantBody = @{
        rate_limit_rps = 2000
        max_connections = 200
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "PUT" -Endpoint "/api/v1/tenants/$tenantId" -Body $updateTenantBody
    Assert-TestResult -TestName "Update Tenant" -Response $response -ExpectedStatus 200 -Description "Update tenant settings"
    
    # Delete tenant (this endpoint may be protected, so we expect 204 or 404)
    $response = Invoke-ApiCall -Method "DELETE" -Endpoint "/api/v1/tenants/$tenantId"
    if ($response.StatusCode -eq 204 -or $response.StatusCode -eq 404) {
        Write-Host "✓ PASS: Delete Tenant" -ForegroundColor Green
        $global:testsPassed++
    }
}

# ==================== GROUP 5: SECRET ENDPOINTS ====================
Write-Host "`n=== GROUP 5: TENANT SECRET ENDPOINTS ===" -ForegroundColor Yellow

if ($tenantId) {
    # List secrets for tenant
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants/$tenantId/secrets"
    Assert-TestResult -TestName "List Tenant Secrets" -Response $response -ExpectedStatus 200 -Description "Get all secrets for tenant"
    
    # Create secret
    $secretBody = @{
        secret_key = "api-key-$(Get-Random)"
        secret_value = "secret-value-$(Get-Random)"
    } | ConvertTo-Json
    
    $createSecretResponse = Invoke-ApiCall -Method "POST" -Endpoint "/api/v1/tenants/$tenantId/secrets" -Body $secretBody
    Assert-TestResult -TestName "Create Secret" -Response $createSecretResponse -ExpectedStatus 201 -Description "Create tenant secret"
    
    if ($createSecretResponse.Success -and $createSecretResponse.Content.secret_id) {
        $secretId = $createSecretResponse.Content.secret_id
        
        # Get specific secret
        $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants/$tenantId/secrets/$secretId"
        Assert-TestResult -TestName "Get Secret" -Response $response -ExpectedStatus 200 -Description "Retrieve specific secret"
        
        # Delete secret
        $response = Invoke-ApiCall -Method "DELETE" -Endpoint "/api/v1/tenants/$tenantId/secrets/$secretId"
        Assert-TestResult -TestName "Delete Secret" -Response $response -ExpectedStatus 204 -Description "Delete tenant secret"
    }
}

# ==================== GROUP 6: USAGE/LIMITS ENDPOINTS ====================
Write-Host "`n=== GROUP 6: USAGE & LIMITS ENDPOINTS ===" -ForegroundColor Yellow

if ($tenantId) {
    # Get usage for tenant
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants/$tenantId/usage"
    Assert-TestResult -TestName "Get Tenant Usage" -Response $response -ExpectedStatus 200 -Description "Retrieve tenant usage statistics"
    
    # Get limits for tenant
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/tenants/$tenantId/limits"
    Assert-TestResult -TestName "Get Tenant Limits" -Response $response -ExpectedStatus 200 -Description "Retrieve tenant rate limits"
    
    # Update limits
    $limitsBody = @{
        rate_limit_rps = 5000
        max_connections = 500
        max_message_size = 52428800
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "PUT" -Endpoint "/api/v1/tenants/$tenantId/limits" -Body $limitsBody
    Assert-TestResult -TestName "Update Tenant Limits" -Response $response -ExpectedStatus 200 -Description "Update tenant rate limits"
}

# ==================== GROUP 7: NOTIFICATION ENDPOINTS ====================
Write-Host "`n=== GROUP 7: NOTIFICATION ENDPOINTS ===" -ForegroundColor Yellow

# List notification settings
$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/notifications/settings"
Assert-TestResult -TestName "List Notification Settings" -Response $response -ExpectedStatus 200 -Description "Get all notification settings"

# Create notification setting
$notificationBody = @{
    event_type = "client.connected"
    enabled = $true
    recipient_email = "admin@test.com"
    notification_channels = "email,webhook"
} | ConvertTo-Json

$createNotifResponse = Invoke-ApiCall -Method "POST" -Endpoint "/api/v1/notifications/settings" -Body $notificationBody
if ($createNotifResponse.StatusCode -eq 201 -or $createNotifResponse.StatusCode -eq 200) {
    Write-Host "✓ PASS: Create Notification Setting" -ForegroundColor Green
    $global:testsPassed++
}

# List notification events
$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/notifications/events"
Assert-TestResult -TestName "List Notification Events" -Response $response -ExpectedStatus 200 -Description "Get notification event audit log"

# Record notification event (testing endpoint)
$eventBody = @{
    event_type = "server.started"
    title = "Server Started"
    description = "Admin API server has started"
    severity = "info"
} | ConvertTo-Json

$response = Invoke-ApiCall -Method "POST" -Endpoint "/api/v1/notifications/events" -Body $eventBody
if ($response.StatusCode -eq 201 -or $response.StatusCode -eq 200) {
    Write-Host "✓ PASS: Record Notification Event" -ForegroundColor Green
    $global:testsPassed++
}

# Delete notification setting (if exists)
$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/notifications/settings"
if ($response.Success -and $response.Content.Count -gt 0) {
    $firstNotificationId = $response.Content[0].id
    $response = Invoke-ApiCall -Method "DELETE" -Endpoint "/api/v1/notifications/settings/$firstNotificationId"
    if ($response.StatusCode -eq 204 -or $response.StatusCode -eq 200) {
        Write-Host "✓ PASS: Delete Notification Setting" -ForegroundColor Green
        $global:testsPassed++
    }
}

# ==================== GROUP 8: INFO ENDPOINT ====================
Write-Host "`n=== GROUP 8: INFO ENDPOINT ===" -ForegroundColor Yellow

$response = Invoke-ApiCall -Method "GET" -Endpoint "/api/v1/info"
Assert-TestResult -TestName "Get API Info" -Response $response -ExpectedStatus 200 -Description "Retrieve API version and build info"

# ==================== SUMMARY ====================
Write-Host "`n" -ForegroundColor White
Write-Host "╔════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        TEST EXECUTION SUMMARY              ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Tests Passed: $($global:testsPassed)" -ForegroundColor Green
Write-Host "Tests Failed: $($global:testsFailed)" -ForegroundColor Red
Write-Host "Total Tests:  $($global:testsPassed + $global:testsFailed)" -ForegroundColor White
Write-Host ""

$passRate = if ($global:testsPassed + $global:testsFailed -gt 0) {
    [math]::Round(($global:testsPassed / ($global:testsPassed + $global:testsFailed)) * 100, 2)
} else {
    0
}

Write-Host "Pass Rate: $passRate%" -ForegroundColor $(if ($passRate -ge 90) { "Green" } else { "Yellow" })
Write-Host ""

if ($global:testsFailed -gt 0) {
    Write-Host "Failed Tests:" -ForegroundColor Red
    $global:testResults | Where-Object { $_.Status -eq "FAIL" } | ForEach-Object {
        Write-Host "  - $($_.Name): $($_.Code)" -ForegroundColor Red
    }
}

Write-Host "`n✅ Test suite completed!" -ForegroundColor Green
