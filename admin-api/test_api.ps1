$baseUrl = "http://127.0.0.1:8080"
$passed = 0
$failed = 0
$tests = @()

function Test-Endpoint {
    param($Method, $Endpoint, $Body, $ExpectedStatus = 200, $Name)
    
    $url = "$baseUrl$Endpoint"
    try {
        if ($Body) {
            $response = Invoke-WebRequest -Uri $url -Method $Method -Body ($Body | ConvertTo-Json) -ContentType "application/json" -ErrorAction SilentlyContinue
        }
        else {
            $response = Invoke-WebRequest -Uri $url -Method $Method -ErrorAction SilentlyContinue
        }
        
        if ($response.StatusCode -eq $ExpectedStatus) {
            Write-Host "[PASS] $Name (HTTP $($response.StatusCode))" -ForegroundColor Green
            $global:passed++
            return $response.Content | ConvertFrom-Json
        }
        else {
            Write-Host "[FAIL] $Name (Expected $ExpectedStatus, got $($response.StatusCode))" -ForegroundColor Red
            $global:failed++
        }
    }
    catch {
        Write-Host "[ERROR] $Name - $($_.Exception.Message)" -ForegroundColor Red
        $global:failed++
    }
    return $null
}

Write-Host "`n========== ADMIN API TEST SUITE ==========" -ForegroundColor Cyan
Write-Host "Server: $baseUrl`n" -ForegroundColor Gray

Write-Host "GROUP 1: HEALTH ENDPOINTS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/health" -Name "Health Check"
Test-Endpoint -Method GET -Endpoint "/health/detailed" -Name "Health Detailed"

Write-Host "`nGROUP 2: SYSTEM ENDPOINTS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/system/config" -Name "Get System Config"

Write-Host "`nGROUP 3: CLUSTER ENDPOINTS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/cluster/environments" -Name "List Cluster Environments"

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

Write-Host "`nGROUP 4: TENANT ENDPOINTS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/tenants" -Name "List Tenants"

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
    
    Write-Host "`nGROUP 5: SECRET ENDPOINTS" -ForegroundColor Yellow
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/secrets" -Name "List Secrets"
    
    $secretBody = @{
        secret_key = "testkey"
        secret_value = "testvalue"
    }
    
    Test-Endpoint -Method POST -Endpoint "/api/v1/tenants/$tenantId/secrets" -Body $secretBody -ExpectedStatus 201 -Name "Create Secret"
    
    Write-Host "`nGROUP 6: USAGE AND LIMITS ENDPOINTS" -ForegroundColor Yellow
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/usage" -Name "Get Tenant Usage"
    Test-Endpoint -Method GET -Endpoint "/api/v1/tenants/$tenantId/limits" -Name "Get Tenant Limits"
}

Write-Host "`nGROUP 7: NOTIFICATION ENDPOINTS" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/notifications/settings" -Name "List Notification Settings"
Test-Endpoint -Method GET -Endpoint "/api/v1/notifications/events" -Name "List Notification Events"

Write-Host "`nGROUP 8: API INFO ENDPOINT" -ForegroundColor Yellow
Test-Endpoint -Method GET -Endpoint "/api/v1/info" -Name "Get API Info"

Write-Host "`n========== TEST RESULTS ===========" -ForegroundColor Cyan
Write-Host "Passed: $passed" -ForegroundColor Green
Write-Host "Failed: $failed" -ForegroundColor $(if ($failed -gt 0) {"Red"} else {"Green"})
Write-Host "Total:  $($passed + $failed)" -ForegroundColor White
Write-Host "Success Rate: $([math]::Round(($passed / ($passed + $failed)) * 100, 1))%" -ForegroundColor Cyan
Write-Host "====================================`n" -ForegroundColor Cyan
