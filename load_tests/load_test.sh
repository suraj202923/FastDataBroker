#!/bin/bash
# Load testing script for FastDataBroker
# Usage: ./load_test.sh [target] [duration] [rate]

TARGET="${1:-http://localhost:6380}"
DURATION="${2:-300}"  # 5 minutes default
RATE="${3:-1000}"     # 1000 req/s default

echo "FastDataBroker Load Testing Suite"
echo "=============================="
echo "Target: $TARGET"
echo "Duration: ${DURATION}s"
echo "Target Rate: ${RATE} req/s"
echo ""

# Install Apache Bench if not present
if ! command -v ab &> /dev/null; then
    echo "Installing Apache Bench..."
    sudo apt-get install -y apache2-utils
fi

# Test 1: Baseline throughput
echo "[Test 1/5] Baseline Throughput Test"
echo "Running 10,000 requests..."
ab -n 10000 -c 100 -t 60 "$TARGET/health" 2>&1 | grep -E "Requests per second|Time per request|Failed requests"

# Test 2: Sustained load
echo ""
echo "[Test 2/5] Sustained Load Test (${RATE} req/s for ${DURATION}s)"
timeout $DURATION sh -c "while true; do \
    curl -s '$TARGET/api/send' \
      -H 'Content-Type: application/json' \
      -d '{\"sender\":\"load-test\",\"recipients\":[\"user@test.com\"],\"subject\":\"Test\",\"content\":\"Load test message\"}' \
    & sleep 0.001
done" 2>&1 | head -20

# Test 3: Spike test (sudden load increase)
echo ""
echo "[Test 3/5] Spike Test (1000 concurrent requests)"
ab -n 1000 -c 1000 "$TARGET/health"

# Test 4: Stress test
echo ""
echo "[Test 4/5] Stress Test (increasing load)"
for i in 10 50 100 200 500; do
    echo "  Testing with $i concurrent connections..."
    ab -n 1000 -c $i "$TARGET/health" 2>&1 | grep -E "Requests per second|Failed requests" | head -2
done

# Test 5: Message sending load
echo ""
echo "[Test 5/5] Message Sending Load Test"
for i in {1..100}; do
    curl -s "$TARGET/api/send" \
      -H 'Content-Type: application/json' \
      -d "{\"sender\":\"load-test-$i\",\"recipients\":[\"user$i@test.com\"],\"subject\":\"Test $i\",\"content\":\"Message $i\"}" &
done
wait

echo ""
echo "Load testing complete!"
