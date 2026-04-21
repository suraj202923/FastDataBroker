# FastDataBroker SDK Tenant Tests Runner
# Usage: .\RUN_TENANT_TESTS.ps1

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   FastDataBroker - All SDK Tenant-Specific Tests          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$startTime = Get-Date
$sdksDir = Join-Path (Get-Location) "sdks"
$testResults = @{}

# ============================================================
# PYTHON SDK TENANT TESTS
# ============================================================
Write-Host "🐍 PYTHON SDK - TENANT TESTS" -ForegroundColor Yellow
Write-Host "————————————————————————————————————————————————————————————" -ForegroundColor Gray
Write-Host ""

if (Test-Path (Join-Path $sdksDir "python\test_sdk.py")) {
    try {
        Push-Location (Join-Path $sdksDir "python")
        $pythonResult = python -m pytest test_sdk.py -v --tb=short 2>&1
        $pythonPassed = $LASTEXITCODE -eq 0
        
        if ($pythonPassed) {
            Write-Host "✅ Python SDK - All tenant tests PASSED" -ForegroundColor Green
        } else {
            Write-Host "❌ Python SDK - Some tests FAILED" -ForegroundColor Red
            Write-Host ($pythonResult | Select-Object -Last 10 | Out-String)
        }
        
        $testResults["python"] = $pythonPassed
        Pop-Location
    } catch {
        Write-Host "⚠️  Error running Python tests: $_" -ForegroundColor Yellow
        $testResults["python"] = $false
    }
} else {
    Write-Host "⚠️  Python SDK test file not found" -ForegroundColor Yellow
    $testResults["python"] = $false
}

Write-Host ""

# ============================================================
# GO SDK TENANT TESTS
# ============================================================
Write-Host "🐹 GO SDK - TENANT TESTS" -ForegroundColor Yellow
Write-Host "————————————————————————————————————————————————————————————" -ForegroundColor Gray
Write-Host ""

if (Test-Path (Join-Path $sdksDir "go")) {
    try {
        Push-Location (Join-Path $sdksDir "go")
        
        # Check if Go is installed
        $goVersion = go version 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Running Go tenant-specific tests..." -ForegroundColor Cyan
            $goResult = go test -v -run "Tenant" ./... 2>&1
            $goPassed = $LASTEXITCODE -eq 0
            
            if ($goPassed) {
                Write-Host "✅ Go SDK - All tenant tests PASSED" -ForegroundColor Green
            } else {
                Write-Host "❌ Go SDK - Some tests FAILED" -ForegroundColor Red
                Write-Host ($goResult | Select-Object -Last 10 | Out-String)
            }
        } else {
            Write-Host "⚠️  Go compiler not installed - skipping" -ForegroundColor Yellow
            $goPassed = $false
        }
        
        $testResults["go"] = $goPassed
        Pop-Location
    } catch {
        Write-Host "⚠️  Error running Go tests: $_" -ForegroundColor Yellow
        $testResults["go"] = $false
    }
} else {
    Write-Host "⚠️  Go SDK directory not found" -ForegroundColor Yellow
    $testResults["go"] = $false
}

Write-Host ""

# ============================================================
# JAVA SDK TENANT TESTS  
# ============================================================
Write-Host "☕ JAVA SDK - TENANT TESTS" -ForegroundColor Yellow
Write-Host "————————————————————————————————————————————————————————————" -ForegroundColor Gray
Write-Host ""

if (Test-Path (Join-Path $sdksDir "java")) {
    try {
        Push-Location (Join-Path $sdksDir "java")
        
        # Check if Maven is installed
        $mvnVersion = mvn -version 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Running Java tenant-specific tests..." -ForegroundColor Cyan
            $javaResult = mvn test -Dtest="*MultiTenant*" -DfailIfNoTests=false 2>&1
            $javaPassed = ($javaResult | Select-String "BUILD SUCCESS").Count -gt 0
            
            if ($javaPassed) {
                Write-Host "✅ Java SDK - All tenant tests PASSED" -ForegroundColor Green
            } else {
                Write-Host "❌ Java SDK - Some tests FAILED" -ForegroundColor Red
                Write-Host ($javaResult | Select-Object -Last 10 | Out-String)
            }
        } else {
            Write-Host "⚠️  Maven not installed - skipping" -ForegroundColor Yellow
            $javaPassed = $false
        }
        
        $testResults["java"] = $javaPassed
        Pop-Location
    } catch {
        Write-Host "⚠️  Error running Java tests: $_" -ForegroundColor Yellow
        $testResults["java"] = $false
    }
} else {
    Write-Host "⚠️  Java SDK directory not found" -ForegroundColor Yellow
    $testResults["java"] = $false
}

Write-Host ""

# ============================================================
# C# SDK TENANT TESTS
# ============================================================
Write-Host "🔵 C# SDK - TENANT TESTS" -ForegroundColor Yellow
Write-Host "————————————————————————————————————————————————————————————" -ForegroundColor Gray
Write-Host ""

if (Test-Path (Join-Path $sdksDir "csharp")) {
    try {
        Push-Location (Join-Path $sdksDir "csharp")
        
        # Check if .NET is installed
        $dotnetVersion = dotnet --version 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Running C# tenant-specific tests..." -ForegroundColor Cyan
            $csharpResult = dotnet test --filter "MultiTenant" -v normal 2>&1
            $csharpPassed = $LASTEXITCODE -eq 0
            
            if ($csharpPassed) {
                Write-Host "✅ C# SDK - All tenant tests PASSED" -ForegroundColor Green
            } else {
                Write-Host "❌ C# SDK - Some tests FAILED" -ForegroundColor Red
                Write-Host ($csharpResult | Select-Object -Last 10 | Out-String)
            }
        } else {
            Write-Host "⚠️  .NET SDK not installed - skipping" -ForegroundColor Yellow
            $csharpPassed = $false
        }
        
        $testResults["csharp"] = $csharpPassed
        Pop-Location
    } catch {
        Write-Host "⚠️  Error running C# tests: $_" -ForegroundColor Yellow
        $testResults["csharp"] = $false
    }
} else {
    Write-Host "⚠️  C# SDK directory not found" -ForegroundColor Yellow
    $testResults["csharp"] = $false
}

Write-Host ""

# ============================================================
# JAVASCRIPT SDK TENANT TESTS
# ============================================================
Write-Host "📜 JAVASCRIPT SDK - TENANT TESTS" -ForegroundColor Yellow
Write-Host "————————————————————————————————————————————————————————————" -ForegroundColor Gray
Write-Host ""

if (Test-Path (Join-Path $sdksDir "javascript")) {
    try {
        Push-Location (Join-Path $sdksDir "javascript")
        
        # Check if npm is installed
        $npmVersion = npm --version 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            # Install dependencies if needed
            $nodeModules = Join-Path (Get-Location) "node_modules"
            if (-not (Test-Path $nodeModules)) {
                Write-Host "Installing npm dependencies..." -ForegroundColor Cyan
                npm install | Out-Null
            }
            
            Write-Host "Running JavaScript tenant-specific tests..." -ForegroundColor Cyan
            # Use npm test with Jest pattern filter
            $jsResult = npm test -- --testNamePattern="MultiTenant|Tenant" 2>&1
            $jsPassed = $LASTEXITCODE -eq 0
            
            if ($jsPassed) {
                Write-Host "✅ JavaScript SDK - All tenant tests PASSED" -ForegroundColor Green
            } else {
                Write-Host "❌ JavaScript SDK - Some tests FAILED" -ForegroundColor Red
                Write-Host ($jsResult | Select-Object -Last 10 | Out-String)
            }
        } else {
            Write-Host "⚠️  Node.js/npm not installed - skipping" -ForegroundColor Yellow
            $jsPassed = $false
        }
        
        $testResults["javascript"] = $jsPassed
        Pop-Location
    } catch {
        Write-Host "⚠️  Error running JavaScript tests: $_" -ForegroundColor Yellow
        $testResults["javascript"] = $false
    }
} else {
    Write-Host "⚠️  JavaScript SDK directory not found" -ForegroundColor Yellow
    $testResults["javascript"] = $false
}

Write-Host ""

# ============================================================
# SUMMARY
# ============================================================
Write-Host "════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "📊 TEST EXECUTION SUMMARY" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

$passedCount = 0
$totalCount = 0

foreach ($sdk in $testResults.Keys) {
    $status = if ($testResults[$sdk]) { "✅ PASSED" } else { "❌ FAILED/SKIPPED" }
    Write-Host "  $('{0,-15}' -f $sdk.ToUpper()): $status" -ForegroundColor $(if ($testResults[$sdk]) { "Green" } else { "Red" })
    
    if ($testResults[$sdk]) { $passedCount++ }
    $totalCount++
}

Write-Host ""
$endTime = Get-Date
$duration = ($endTime - $startTime).TotalSeconds

Write-Host "Results: $passedCount/$totalCount SDKs passed tenant-specific tests" -ForegroundColor $(if ($passedCount -eq $totalCount) { "Green" } else { "Yellow" })
Write-Host "Duration: $([math]::Round($duration, 1)) seconds" -ForegroundColor Cyan
Write-Host ""

if ($passedCount -eq $totalCount) {
    Write-Host "✅ ALL TENANT TESTS PASSED!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "⚠️  Some SDKs need attention" -ForegroundColor Yellow
    exit 1
}
