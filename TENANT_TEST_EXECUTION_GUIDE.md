# FastDataBroker - Tenant-Specific Test Execution Guide

## Overview

This guide explains how to run tenant-specific tests across all FastDataBroker SDKs (Python, Go, Java, C#, JavaScript) to validate multi-tenant functionality.

## Quick Start

### On Windows (PowerShell)
```powershell
.\RUN_TENANT_TESTS.ps1
```

### On macOS/Linux (Bash)
```bash
chmod +x run_tenant_tests.sh
./run_tenant_tests.sh
```

### Using Python (Cross-Platform)
```bash
python run_tenant_tests.py
```

## Test Runner Scripts

### 1. **RUN_TENANT_TESTS.ps1** - Windows PowerShell Script
- **Location**: `FastDataBroker/RUN_TENANT_TESTS.ps1`
- **Usage**: `.\RUN_TENANT_TESTS.ps1`
- **Features**:
  - Colorized console output for easy reading
  - Automatic dependency checking for each SDK tool
  - Graceful fallback if tools are missing
  - Real-time test progress tracking
  - Summary report with pass/fail count
  - Exit code 0 if all tests pass, 1 otherwise

### 2. **run_tenant_tests.sh** - macOS/Linux Bash Script
- **Location**: `FastDataBroker/run_tenant_tests.sh`
- **Usage**: `./run_tenant_tests.sh`
- **Features**:
  - POSIX-compliant for maximum compatibility
  - Subprocess isolation for each SDK test
  - Tool availability checking
  - Formatted output with emoji indicators
  - Summary with execution duration
  - Exit code 0 if all tests pass, 1 otherwise

### 3. **run_tenant_tests.py** - Cross-Platform Python Script
- **Location**: `FastDataBroker/run_tenant_tests.py`
- **Usage**: `python run_tenant_tests.py` or `python3 run_tenant_tests.py`
- **Features**:
  - Works on Windows, macOS, and Linux
  - No external dependencies beyond standard library
  - Detailed error reporting
  - Individual SDK failure isolation
  - Comprehensive test result tracking
  - Can be imported as module for integration testing

## Test Coverage by SDK

### Python SDK
- **Test File**: `sdks/python/test_sdk.py`
- **Framework**: pytest
- **Command**: `python -m pytest sdks/python/test_sdk.py -v --tb=short`
- **Focus**: SDK core functionality and tenant isolation

### Go SDK
- **Test Files**: 
  - `sdks/go/fastdatabroker_multitenancy_test.go` (primary)
  - `sdks/go/fastdatabroker_test.go`
  - `sdks/go/fastdatabroker_comprehensive_test.go`
- **Framework**: go test
- **Command**: `go test -v -run "Tenant" ./...` (from sdks/go directory)
- **Focus**: Tenant configuration creation, isolation, and validation

### Java SDK
- **Test Files**: 
  - `sdks/java/src/test/java/MultiTenantConfigurationTest.java` (primary)
  - `sdks/java/src/test/java/TestSDK.java`
- **Framework**: Maven/JUnit
- **Command**: `mvn test -Dtest="*MultiTenant*"` (from sdks/java directory)
- **Focus**: Tenant configuration management and multi-tenant operations

### C# SDK
- **Test Files**:
  - `sdks/csharp/Tests/MultiTenantTests.cs` (primary)
  - `sdks/csharp/Tests/TestSDK.cs`
- **Framework**: .NET/xUnit
- **Command**: `dotnet test --filter "MultiTenant"` (from sdks/csharp directory)
- **Focus**: Tenant isolation and configuration validation

### JavaScript SDK
- **Test Files**:
  - `sdks/javascript/tests/multitenancy.test.ts` (primary)
  - `sdks/javascript/tests/fastdatabroker_comprehensive.test.ts`
  - `sdks/javascript/tests/fastdatabroker.test.ts`
- **Framework**: Jest
- **Command**: `npm test -- --testNamePattern="MultiTenant|Tenant"` (from sdks/javascript directory)
- **Focus**: TenantConfig creation, validation, and multi-tenant scenarios

## Prerequisites by SDK

### Python
```bash
pip install pytest
```

### Go
```bash
# Install from https://golang.org/dl/
go version  # Verify installation
```

### Java
```bash
# Install Maven from https://maven.apache.org/
mvn --version  # Verify installation
```

### C#
```bash
# Install .NET SDK from https://dotnet.microsoft.com/
dotnet --version  # Verify installation
```

### JavaScript
```bash
# Install Node.js from https://nodejs.org/
npm --version  # Verify installation
npm install  # Install dependencies from sdks/javascript directory
```

## Understanding Test Output

### Success Indicators
```
✅ Python SDK - All tenant tests PASSED
✅ Go SDK - All tenant tests PASSED
✅ Java SDK - All tenant tests PASSED
✅ C# SDK - All tenant tests PASSED
✅ JavaScript SDK - All tenant tests PASSED

✅ ALL TENANT TESTS PASSED!
```

### Failure/Skip Indicators
```
❌ Python SDK - Some tests FAILED
⚠️  Go SDK - Go compiler not installed - skipping
❌ Java SDK - Some tests FAILED
⚠️  JavaScript SDK - Node.js/npm not installed - skipping
```

### Summary Report
```
Results: 3/5 SDKs passed tenant-specific tests
Duration: 45.3 seconds
⚠️  Some SDKs need attention
```

## Troubleshooting

### Python Tests Fail
```bash
# Check pytest installation
python -m pytest --version

# Run with verbose output
python -m pytest sdks/python/test_sdk.py -vv --tb=long
```

### Go Tests Fail
```bash
# Check Go installation and version
go version

# Run tests with more output
cd sdks/go
go test -v -run "Tenant" -vet=all ./...
```

### Java Tests Fail
```bash
# Check Maven installation
mvn --version

# Run tests with debug output
cd sdks/java
mvn test -Dtest="*MultiTenant*" -X
```

### C# Tests Fail
```bash
# Check .NET installation
dotnet --version

# List available tests
cd sdks/csharp
dotnet test --no-build -v detailed
```

### JavaScript Tests Fail
```bash
# Check Node.js/npm installation
npm --version

# Clear npm cache and reinstall
cd sdks/javascript
rm -rf node_modules package-lock.json
npm install

# Run tests with verbose output
npm test -- --verbose
```

## Integration with CI/CD

### GitHub Actions
```yaml
- name: Run Tenant Tests (PowerShell)
  if: runner.os == 'Windows'
  run: .\RUN_TENANT_TESTS.ps1
  shell: powershell

- name: Run Tenant Tests (Bash)
  if: runner.os != 'Windows'
  run: |
    chmod +x run_tenant_tests.sh
    ./run_tenant_tests.sh
```

### Jenkins Pipeline
```groovy
stage('Tenant Tests') {
    steps {
        script {
            if (isUnix()) {
                sh 'chmod +x run_tenant_tests.sh && ./run_tenant_tests.sh'
            } else {
                powershell '.\RUN_TENANT_TESTS.ps1'
            }
        }
    }
}
```

## Interpreting Test Results

### What is Tested
Each SDK's tenant tests validate:

1. **Tenant Creation**: Can create TenantConfig objects with unique IDs
2. **Tenant Isolation**: Data in one tenant doesn't leak to others
3. **Configuration Validation**: Tenant settings are properly stored and retrieved
4. **Multi-Tenant Operations**: Operations work correctly with multiple tenants
5. **Tenant Cleanup**: Resources are properly freed when tenants are removed

### Test Success Criteria
- All test cases pass without errors
- No test timeouts or hangs
- No resource leaks detected
- No cross-tenant data contamination

### Sample Test Names
- `test_tenant_config_creation`
- `test_tenant_isolation`
- `test_multiple_tenants`
- `TestMultiTenantConfiguration`
- `MultiTenantTests`
- `describe('MultiTenant Config')`

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tenant tests passed |
| 1 | One or more SDKs failed tenant tests or were skipped |
| 2 | Script error or missing dependencies |

## Advanced Usage

### Run Tests for Single SDK
```bash
# Python only
cd sdks/python && python -m pytest test_sdk.py -v

# Go only
cd sdks/go && go test -v -run "Tenant" ./...

# Java only
cd sdks/java && mvn test -Dtest="*MultiTenant*"

# C# only
cd sdks/csharp && dotnet test --filter "MultiTenant"

# JavaScript only
cd sdks/javascript && npm test -- --testNamePattern="MultiTenant"
```

### Run with Custom Filters
```bash
# Python: Run specific test
python -m pytest sdks/python/test_sdk.py::test_tenant_isolation -v

# Go: Run with timeout
cd sdks/go && go test -timeout 30s -run "TenantConfig" ./...

# Java: Run with output
mvn test -Dtest="*MultiTenant*" -e

# C# Run with verbosity
dotnet test sdks/csharp --filter "MultiTenant" -v detailed

# JavaScript: Run with coverage
npm test -- --coverage --testNamePattern="Tenant"
```

## Next Steps

After running tenant tests:

1. **If All Pass**: Repository is ready for deployment ✅
2. **If Some Fail**: Review failure output and address SDK-specific issues
3. **If Some Skip**: Install missing tools for complete coverage
4. **Document Results**: Update test results in deployment documentation

## Support

For issues with specific SDKs:
- **Python**: See `sdks/python/README.md`
- **Go**: See `sdks/go/README.md`
- **Java**: See `sdks/java/README.md`
- **C#**: See `sdks/csharp/README.md`
- **JavaScript**: See `sdks/javascript/README.md`

---

**FastDataBroker v3.1 - Multi-Tenant Test Suite**
Last Updated: 2026-04-12
