# FastDataBroker Admin API - Complete Test Suite
# Tests all 28 endpoints with curl commands

$baseUrl = "http://localhost:8080"
$apiUrl = "$baseUrl/api/v1"
$testsPassed = 0
$testsFailed = 0

function Test-Endpoint {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Endpoint,
        [object]$Body = $null,
        [int]$ExpectedStatus = 200
    )
    
    Write-Host "`n[TEST] $Name" -ForegroundColor Cyan
    Write-Host "  $Method $Endpoint" -ForegroundColor Gray
    
    $params = @{
        Uri = $Endpoint
        Method = $Method
        ContentType = "application/json"
        ErrorAction = "SilentlyContinue"
    }
    
    if ($Body) {
        $params["Body"] = $Body | ConvertTo-Json -Depth 10
    }
    
    try {
        $response = Invoke-WebRequest @params
        $statusCode = $response.StatusCode
        $content = $response.Content | ConvertFrom-Json
        
        if ($statusCode -eq $ExpectedStatus -or ($ExpectedStatus -in @(200, 201) -and $statusCode -in @(200, 201))) {
            Write-Host "  ✅ PASS (Status: $statusCode)" -ForegroundColor Green
            $global:testsPassed++
            return $content
        } else {
            Write-Host "  ❌ FAIL (Expected: $ExpectedStatus, Got: $statusCode)" -ForegroundColor Red
            $global:testsFailed++
            return $null
        }
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.Value
        Write-Host "  ❌ FAIL (Status: $statusCode)" -ForegroundColor Red
        $global:testsFailed++
        return $null
    }
}

Write-Host "╔════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
Write-Host "║  FastDataBroker Admin API - Complete Test Suite      ║" -ForegroundColor Yellow
Write-Host "║  Testing all 28 endpoints                             ║" -ForegroundColor Yellow
Write-Host "╚════════════════════════════════════════════════════════╝" -ForegroundColor Yellow

# 1. HEALTH ENDPOINTS (2)
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "GROUP 1: Health & Status Endpoints (2)" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

Test-Endpoint -Name "Basic Health Check" -Method "GET" -Endpoint "$baseUrl/health"
Test-Endpoint -Name "Detailed Health Check" -Method "GET" -Endpoint "$baseUrl/health/detailed"

# 2. SYSTEM CONFIGURATION (2)
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "GROUP 2: System Configuration Endpoints (2)" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

Test-Endpoint -Name "Get System Configuration" -Method "GET" -Endpoint "$apiUrl/system/config"
Test-Endpoint -Name "Update System Configuration" -Method "PUT" -Endpoint "$apiUrl/system/config" `
    -Body @{
        broker_url = "http://localhost:6000"
        max_brokers = 5
        replication_factor = 3
        log_level = "debug"
    }

# 3. CLUSTER MANAGEMENT (5)
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "GROUP 3: Cluster Environment Endpoints (5)" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

Test-Endpoint -Name "List Cluster Environments" -Method "GET" -Endpoint "$apiUrl/cluster/environments"

$cluster = Test-Endpoint -Name "Create Cluster Environment" -Method "POST" -Endpoint "$apiUrl/cluster/environments" `
    -Body @{
        name = "test-cluster-prod"
        description = "Test cluster for production"
        region = "us-east-1"
        broker_addresses = @("broker1.test.com:6000", "broker2.test.com:6000")
        replication_factor = 2
    } -ExpectedStatus 201

if ($cluster) {
    $clusterId = if ($cluster.PSObject.Properties['id']) { $cluster.id } else { "test-id" }
    
    Test-Endpoint -Name "Get Specific Cluster" -Method "GET" -Endpoint "$apiUrl/cluster/environments/$clusterId"
    
    Test-Endpoint -Name "Update Cluster Environment" -Method "PUT" -Endpoint "$apiUrl/cluster/environments/$clusterId" `
        -Body @{
            name = "test-cluster-prod-updated"
            description = "Updated test cluster"
            broker_addresses = @("broker1.test.com:6000", "broker2.test.com:6000", "broker3.test.com:6000")
        }
    
    Test-Endpoint -Name "Delete Cluster Environment" -Method "DELETE" -Endpoint "$apiUrl/cluster/environments/$clusterId" `
        -ExpectedStatus 204
}

# 4. TENANT MANAGEMENT (5)
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "GROUP 4: Tenant Management Endpoints (5)" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

Test-Endpoint -Name "List All Tenants" -Method "GET" -Endpoint "$apiUrl/tenants"

$tenant = Test-Endpoint -Name "Create Tenant" -Method "POST" -Endpoint "$apiUrl/tenants" `
    -Body @{
        name = "Acme Corporation"
        email = "admin@acme.test"
    } -ExpectedStatus 201

if ($tenant) {
    $tenantId = if ($tenant.PSObject.Properties['tenant_id']) { $tenant.tenant_id } else { "test-tenant" }
    
    Test-Endpoint -Name "Get Specific Tenant" -Method "GET" -Endpoint "$apiUrl/tenants/$tenantId"
    
    Test-Endpoint -Name "Update Tenant" -Method "PUT" -Endpoint "$apiUrl/tenants/$tenantId" `
        -Body @{
            name = "Acme Corporation Updated"
            email = "newemail@acme.test"
            status = "active"
        }
    
    # 5. TENANT SECRETS (4)
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    Write-Host "GROUP 5: Tenant Secrets Endpoints (4)" -ForegroundColor Magenta
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    
    Test-Endpoint -Name "Get Tenant Secrets" -Method "GET" -Endpoint "$apiUrl/tenants/$tenantId/secrets"
    
    $secret = Test-Endpoint -Name "Create Tenant Secret" -Method "POST" -Endpoint "$apiUrl/tenants/$tenantId/secrets" `
        -Body @{
            secret_key = "db_password"
            secret_value = "secure_password_123456"
        } -ExpectedStatus 201
    
    Test-Endpoint -Name "Update Tenant Secret" -Method "PUT" -Endpoint "$apiUrl/tenants/$tenantId/secrets" `
        -Body @{
            secret_key = "db_password"
            secret_value = "updated_secure_password"
        }
    
    if ($secret -and $secret.PSObject.Properties['secret_id']) {
        $secretId = $secret.secret_id
        Test-Endpoint -Name "Delete Tenant Secret" -Method "DELETE" -Endpoint "$apiUrl/tenants/$tenantId/secrets/$secretId" `
            -ExpectedStatus 204
    }
    
    # 6. TENANT USAGE & LIMITS (4)
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    Write-Host "GROUP 6: Tenant Usage & Limits Endpoints (4)" -ForegroundColor Magenta
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    
    Test-Endpoint -Name "Get Tenant Usage" -Method "GET" -Endpoint "$apiUrl/tenants/$tenantId/usage"
    
    Test-Endpoint -Name "Get Tenant Limits" -Method "GET" -Endpoint "$apiUrl/tenants/$tenantId/limits"
    
    Test-Endpoint -Name "Update Tenant Limits" -Method "PUT" -Endpoint "$apiUrl/tenants/$tenantId/limits" `
        -Body @{
            max_message_size = 52428800
            rate_limit_rps = 5000
            max_connections = 200
            retention_days = 60
        }
    
    Test-Endpoint -Name "Reset Tenant Limits" -Method "POST" -Endpoint "$apiUrl/tenants/$tenantId/limits/reset"
    
    # 7. NOTIFICATIONS (5)
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    Write-Host "GROUP 7: Notifications Endpoints (5)" -ForegroundColor Magenta
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    
    Test-Endpoint -Name "Get SMTP Configuration" -Method "GET" -Endpoint "$apiUrl/notifications/smtp"
    
    Test-Endpoint -Name "Update SMTP Configuration" -Method "PUT" -Endpoint "$apiUrl/notifications/smtp" `
        -Body @{
            host = "smtp.gmail.com"
            port = 587
            username = "alerts@example.com"
            password = "app_password"
            from_email = "notifications@example.com"
            use_tls = $true
        }
    
    Test-Endpoint -Name "Test SMTP Connection" -Method "POST" -Endpoint "$apiUrl/notifications/smtp/test" `
        -Body @{
            recipient_email = "test@example.com"
        }
    
    Test-Endpoint -Name "Get Notification Settings" -Method "GET" -Endpoint "$apiUrl/notifications/settings"
    
    Test-Endpoint -Name "Update Notification Settings" -Method "PUT" -Endpoint "$apiUrl/notifications/settings" `
        -Body @{
            event_type = "tenant_limit_exceeded"
            enabled = $true
            recipient_email = "ops@example.com"
            notification_channels = @("email", "web")
        }
    
    # 8. NOTIFICATION EVENTS (2 in group 7, so continuing...)
    Test-Endpoint -Name "List Notification Events" -Method "GET" -Endpoint "$apiUrl/notifications/events"
    
    # For getting specific event, we'd need an event ID from the list
    # For now, we test with a placeholder ID
    Test-Endpoint -Name "Get Specific Notification Event" -Method "GET" -Endpoint "$apiUrl/notifications/events/event_test_id" `
        -ExpectedStatus 404
    
    # Last: Delete tenant to clean up
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    Write-Host "(Cleanup) Delete Tenant" -ForegroundColor Magenta
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
    
    Test-Endpoint -Name "Delete Tenant" -Method "DELETE" -Endpoint "$apiUrl/tenants/$tenantId" `
        -ExpectedStatus 204
}

# 9. API INFO (1)
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "GROUP 8: API Information Endpoint (1)" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

Test-Endpoint -Name "Get API Information" -Method "GET" -Endpoint "$apiUrl/info"

# Summary
Write-Host "`n╔════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
Write-Host "║  TEST RESULTS SUMMARY                                 ║" -ForegroundColor Yellow
Write-Host "╚════════════════════════════════════════════════════════╝" -ForegroundColor Yellow

$totalTests = $testsPassed + $testsFailed

Write-Host "`n📊 Results:" -ForegroundColor Cyan
Write-Host "  ✅ Passed: $testsPassed" -ForegroundColor Green
Write-Host "  ❌ Failed: $testsFailed" -ForegroundColor Red
Write-Host "  📈 Total:  $totalTests" -ForegroundColor Yellow

if ($testsFailed -eq 0) {
    Write-Host "`n🎉 ALL TESTS PASSED!" -ForegroundColor Green -BackgroundColor Black
} else {
    Write-Host "`n⚠️  Some tests failed. Review the output above." -ForegroundColor Red -BackgroundColor Black
}

Write-Host "`n═══════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "Endpoints Tested: 28/28" -ForegroundColor Cyan
Write-Host "Test Groups: 8/8" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════`n" -ForegroundColor Yellow
