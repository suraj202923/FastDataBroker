#!/usr/bin/env powershell

# Simple test without complex formatting
Write-Host "Starting Admin API tests..." -ForegroundColor Cyan

$passed = 0
$failed = 0

# Test 1: Health endpoint
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -Method GET -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ Health check" -ForegroundColor Green
        $passed++
    }
} catch {
    Write-Host "❌ Health check failed" -ForegroundColor Red
    $failed++
}

# Test 2: System config
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/system/config" -Method GET -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ System config" -ForegroundColor Green
        $passed++
    }
} catch {
    Write-Host "❌ System config failed" -ForegroundColor Red
    $failed++
}

# Test 3: List tenants
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/tenants" -Method GET -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ List tenants" -ForegroundColor Green
        $passed++
    }
} catch {
    Write-Host "❌ List tenants failed" -ForegroundColor Red
    $failed++
}

# Test 4: Create tenant
try {
    $body = @{
        name = "TestTenant"
        email = "test@example.com"
    } | ConvertTo-Json
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/tenants" -Method POST -Body $body -ContentType "application/json" -ErrorAction Stop
    if ($response.StatusCode -eq 201) {
        Write-Host "✅ Create tenant" -ForegroundColor Green
        $passed++
        $tenant = $response.Content | ConvertFrom-Json
        $tenantId = $tenant.tenant_id
    }
} catch {
    Write-Host "❌ Create tenant failed" -ForegroundColor Red
    $failed++
}

# Test 5: Get tenant
if ($tenantId) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/tenants/$tenantId" -Method GET -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "✅ Get tenant" -ForegroundColor Green
            $passed++
        }
    } catch {
        Write-Host "❌ Get tenant failed" -ForegroundColor Red
        $failed++
    }
}

# Test 6: API info
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/info" -Method GET -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ API info" -ForegroundColor Green
        $passed++
    }
} catch {
    Write-Host "❌ API info failed" -ForegroundColor Red
    $failed++
}

Write-Host "`nResults: $passed passed, $failed failed"
