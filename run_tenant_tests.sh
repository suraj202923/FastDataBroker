#!/bin/bash

# FastDataBroker SDK Tenant Tests Runner
# Usage: ./run_tenant_tests.sh

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║   FastDataBroker - All SDK Tenant-Specific Tests          ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

start_time=$(date +%s)
SDKS_DIR="sdks"
declare -A test_results

# ============================================================
# PYTHON SDK TENANT TESTS
# ============================================================
echo "🐍 PYTHON SDK - TENANT TESTS"
echo "————————————————————————————————————————————————————————————"
echo ""

if [ -f "$SDKS_DIR/python/test_sdk.py" ]; then
    (
        cd "$SDKS_DIR/python" || exit 1
        if python -m pytest test_sdk.py -v --tb=short 2>&1; then
            echo "✅ Python SDK - All tenant tests PASSED"
            test_results["python"]=1
        else
            echo "❌ Python SDK - Some tests FAILED"
            test_results["python"]=0
        fi
    )
else
    echo "⚠️  Python SDK test file not found"
    test_results["python"]=0
fi

echo ""

# ============================================================
# GO SDK TENANT TESTS
# ============================================================
echo "🐹 GO SDK - TENANT TESTS"
echo "————————————————————————————————————————————————————————————"
echo ""

if [ -d "$SDKS_DIR/go" ]; then
    (
        cd "$SDKS_DIR/go" || exit 1
        
        if command -v go &> /dev/null; then
            echo "Running Go tenant-specific tests..."
            if go test -v -run "Tenant" ./... 2>&1; then
                echo "✅ Go SDK - All tenant tests PASSED"
                test_results["go"]=1
            else
                echo "❌ Go SDK - Some tests FAILED"
                test_results["go"]=0
            fi
        else
            echo "⚠️  Go compiler not installed - skipping"
            test_results["go"]=0
        fi
    )
else
    echo "⚠️  Go SDK directory not found"
    test_results["go"]=0
fi

echo ""

# ============================================================
# JAVA SDK TENANT TESTS
# ============================================================
echo "☕ JAVA SDK - TENANT TESTS"
echo "————————————————————————————————————————————————————————————"
echo ""

if [ -d "$SDKS_DIR/java" ]; then
    (
        cd "$SDKS_DIR/java" || exit 1
        
        if command -v mvn &> /dev/null; then
            echo "Running Java tenant-specific tests..."
            if mvn test -Dtest="*MultiTenant*" -DfailIfNoTests=false 2>&1 | grep -q "BUILD SUCCESS"; then
                echo "✅ Java SDK - All tenant tests PASSED"
                test_results["java"]=1
            else
                echo "❌ Java SDK - Some tests FAILED"
                test_results["java"]=0
            fi
        else
            echo "⚠️  Maven not installed - skipping"
            test_results["java"]=0
        fi
    )
else
    echo "⚠️  Java SDK directory not found"
    test_results["java"]=0
fi

echo ""

# ============================================================
# C# SDK TENANT TESTS
# ============================================================
echo "🔵 C# SDK - TENANT TESTS"
echo "————————————————————————————————————————————————————————————"
echo ""

if [ -d "$SDKS_DIR/csharp" ]; then
    (
        cd "$SDKS_DIR/csharp" || exit 1
        
        if command -v dotnet &> /dev/null; then
            echo "Running C# tenant-specific tests..."
            if dotnet test --filter "MultiTenant" -v normal 2>&1; then
                echo "✅ C# SDK - All tenant tests PASSED"
                test_results["csharp"]=1
            else
                echo "❌ C# SDK - Some tests FAILED"
                test_results["csharp"]=0
            fi
        else
            echo "⚠️  .NET SDK not installed - skipping"
            test_results["csharp"]=0
        fi
    )
else
    echo "⚠️  C# SDK directory not found"
    test_results["csharp"]=0
fi

echo ""

# ============================================================
# JAVASCRIPT SDK TENANT TESTS
# ============================================================
echo "📜 JAVASCRIPT SDK - TENANT TESTS"
echo "————————————————————————————————————————————————————————————"
echo ""

if [ -d "$SDKS_DIR/javascript" ]; then
    (
        cd "$SDKS_DIR/javascript" || exit 1
        
        if command -v npm &> /dev/null; then
            # Install dependencies if needed
            if [ ! -d "node_modules" ]; then
                echo "Installing npm dependencies..."
                npm install > /dev/null 2>&1
            fi
            
            echo "Running JavaScript tenant-specific tests..."
            if npm test -- --testNamePattern="MultiTenant|Tenant" 2>&1; then
                echo "✅ JavaScript SDK - All tenant tests PASSED"
                test_results["javascript"]=1
            else
                echo "❌ JavaScript SDK - Some tests FAILED"
                test_results["javascript"]=0
            fi
        else
            echo "⚠️  Node.js/npm not installed - skipping"
            test_results["javascript"]=0
        fi
    )
else
    echo "⚠️  JavaScript SDK directory not found"
    test_results["javascript"]=0
fi

echo ""

# ============================================================
# SUMMARY
# ============================================================
echo "════════════════════════════════════════════════════════════"
echo "📊 TEST EXECUTION SUMMARY"
echo "════════════════════════════════════════════════════════════"
echo ""

passed_count=0
total_count=0

for sdk in "${!test_results[@]}"; do
    if [ "${test_results[$sdk]}" -eq 1 ]; then
        echo "  ${sdk^^}: ✅ PASSED"
        ((passed_count++))
    else
        echo "  ${sdk^^}: ❌ FAILED/SKIPPED"
    fi
    ((total_count++))
done

echo ""
end_time=$(date +%s)
duration=$((end_time - start_time))

echo "Results: $passed_count/$total_count SDKs passed tenant-specific tests"
echo "Duration: $duration seconds"
echo ""

if [ "$passed_count" -eq "$total_count" ]; then
    echo "✅ ALL TENANT TESTS PASSED!"
    exit 0
else
    echo "⚠️  Some SDKs need attention"
    exit 1
fi
