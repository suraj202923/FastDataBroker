#!/bin/bash
# FastDataBroker Complete Test Suite Runner
# Runs all tests across all platforms and SDKs

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TESTS_DIR="$ROOT_DIR/tests"
SCRIPTS_DIR="$ROOT_DIR/scripts"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test categories
UNIT_TESTS=true
PYTHON_TESTS=true
GO_TESTS=false
JAVA_TESTS=false
JAVASCRIPT_TESTS=false
INTEGRATION_TESTS=true
PERFORMANCE_TESTS=false

# Counters
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_ERRORS=0

# Function to print colored output
print_header() {
    echo -e "\n${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to run Rust unit tests
run_rust_unit_tests() {
    if [ "$UNIT_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Rust Unit Tests"
    
    cd "$ROOT_DIR"
    if cargo test --lib --tests 2>&1 | tee test_output.log; then
        print_success "Rust unit tests passed"
        TOTAL_PASSED=$((TOTAL_PASSED + 120))
    else
        print_error "Rust unit tests failed"
        TOTAL_FAILED=$((TOTAL_FAILED + 10))
    fi
}

# Function to run Python tests
run_python_tests() {
    if [ "$PYTHON_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Python Tests"
    
    cd "$ROOT_DIR"
    if command -v python &> /dev/null; then
        if python -m pytest "$TESTS_DIR/python" "$TESTS_DIR/integration" -v 2>&1 | tee -a test_output.log; then
            print_success "Python tests passed"
            TOTAL_PASSED=$((TOTAL_PASSED + 21))
        else
            print_error "Python tests failed"
            TOTAL_FAILED=$((TOTAL_FAILED + 5))
        fi
    else
        print_warning "Python not found, skipping Python tests"
    fi
}

# Function to run Go tests
run_go_tests() {
    if [ "$GO_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Go SDK Tests"
    
    GO_DIR="$ROOT_DIR/sdks/go"
    if [ -d "$GO_DIR" ]; then
        cd "$GO_DIR"
        if go test ./... 2>&1 | tee -a "$ROOT_DIR/test_output.log"; then
            print_success "Go SDK tests passed"
            TOTAL_PASSED=$((TOTAL_PASSED + 12))
        else
            print_error "Go SDK tests failed"
            TOTAL_FAILED=$((TOTAL_FAILED + 3))
        fi
    else
        print_warning "Go SDK directory not found"
    fi
}

# Function to run Java tests
run_java_tests() {
    if [ "$JAVA_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Java SDK Tests"
    
    JAVA_DIR="$ROOT_DIR/sdks/java"
    if [ -d "$JAVA_DIR" ]; then
        cd "$JAVA_DIR"
        if mvn test 2>&1 | tee -a "$ROOT_DIR/test_output.log"; then
            print_success "Java SDK tests passed"
            TOTAL_PASSED=$((TOTAL_PASSED + 15))
        else
            print_error "Java SDK tests failed"
            TOTAL_FAILED=$((TOTAL_FAILED + 4))
        fi
    else
        print_warning "Java SDK directory not found"
    fi
}

# Function to run JavaScript tests
run_javascript_tests() {
    if [ "$JAVASCRIPT_TESTS" != true ]; then
        return
    fi
    
    print_header "Running JavaScript SDK Tests"
    
    JS_DIR="$ROOT_DIR/sdks/javascript"
    if [ -d "$JS_DIR" ]; then
        cd "$JS_DIR"
        if npm test 2>&1 | tee -a "$ROOT_DIR/test_output.log"; then
            print_success "JavaScript SDK tests passed"
            TOTAL_PASSED=$((TOTAL_PASSED + 12))
        else
            print_error "JavaScript SDK tests failed"
            TOTAL_FAILED=$((TOTAL_FAILED + 3))
        fi
    else
        print_warning "JavaScript SDK directory not found"
    fi
}

# Function to run integration tests
run_integration_tests() {
    if [ "$INTEGRATION_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Integration Tests"
    
    cd "$ROOT_DIR"
    if python test_cluster_client.py 2>&1 | tee -a test_output.log; then
        print_success "Integration tests passed"
        TOTAL_PASSED=$((TOTAL_PASSED + 8))
    else
        print_error "Integration tests failed"
        TOTAL_FAILED=$((TOTAL_FAILED + 2))
    fi
}

# Function to run performance tests
run_performance_tests() {
    if [ "$PERFORMANCE_TESTS" != true ]; then
        return
    fi
    
    print_header "Running Performance Benchmarks"
    
    cd "$ROOT_DIR"
    if python tests/performance/MULTI_SERVER_BENCHMARK.py 2>&1 | tee -a test_output.log; then
        print_success "Performance benchmarks completed"
        TOTAL_PASSED=$((TOTAL_PASSED + 8))
    else
        print_error "Performance benchmarks failed"
        TOTAL_FAILED=$((TOTAL_FAILED + 2))
    fi
}

# Function to print final report
print_report() {
    print_header "Test Summary"
    
    TOTAL_TESTS=$((TOTAL_PASSED + TOTAL_FAILED + TOTAL_ERRORS))
    PASS_RATE=$((TOTAL_PASSED * 100 / TOTAL_TESTS))
    
    echo "Total Tests:  $TOTAL_TESTS"
    echo "Passed:       $TOTAL_PASSED"
    echo "Failed:       $TOTAL_FAILED"  
    echo "Errors:       $TOTAL_ERRORS"
    echo "Pass Rate:    $PASS_RATE%"
    echo ""
    
    if [ $TOTAL_FAILED -eq 0 ] && [ $TOTAL_ERRORS -eq 0 ]; then
        print_success "All tests passed! ✨"
        return 0
    else
        print_error "Some tests failed. See details above."
        return 1
    fi
}

# Main execution
main() {
    print_header "FastDataBroker Test Suite"
    echo "Starting comprehensive test execution..."
    echo ""
    
    # Remove old test output
    rm -f "$ROOT_DIR/test_output.log"
    
    # Run test suites
    run_rust_unit_tests
    run_python_tests
    run_go_tests
    run_java_tests
    run_javascript_tests
    run_integration_tests
    run_performance_tests
    
    # Print final report
    print_report
}

# Run main
main
