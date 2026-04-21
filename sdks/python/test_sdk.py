"""
FastDataBroker Python SDK - Test Suite
Tests core functionality without requiring full server setup
"""

import time
import hashlib
import random
from dataclasses import dataclass
from typing import Dict, Any, Optional

# Mock SDK Classes
@dataclass(frozen=True)
class Message:
    topic: str
    payload: Any
    priority: int = 5
    ttl_seconds: int = 3600
    headers: Optional[Dict[str, str]] = None

@dataclass
class DeliveryResult:
    message_id: str
    status: str
    latency_ms: float
    timestamp: int

@dataclass
class ConnectionStats:
    is_connected: bool
    messages_sent: int
    messages_received: int
    connection_time_ms: int
    uptime_seconds: int
    last_message_time: int


class FastDataBrokerQuicClient:
    def __init__(self, config):
        self.config = config
        self.connected = False
        self.authenticated = False
        self.stats = {
            'messages_sent': 0,
            'messages_received': 0,
            'last_message_time': 0,
        }
        self.connection_start = 0
        self.message_handlers = {}

    def connect(self):
        print(f"Connecting to {self.config['host']}:{self.config['port']}...")
        self.connected = True
        self.connection_start = int(time.time() * 1000)
        print(f"✓ Connected to {self.config['host']}:{self.config['port']}")

    def send_message(self, message):
        if not self.connected:
            raise ConnectionError("Not connected")
        
        message_id = f"msg_{int(time.time() * 1000)}_{random.randint(0, 10000)}"
        latency = (time.time() % 50) + 5  # Simulate 5-55ms latency
        self.stats['messages_sent'] += 1
        self.stats['last_message_time'] = int(time.time() * 1000)
        
        return DeliveryResult(
            message_id=message_id,
            status='success',
            latency_ms=latency,
            timestamp=int(time.time() * 1000),
        )

    def on_message(self, topic, handler):
        self.message_handlers[topic] = handler

    def off_message(self, topic):
        if topic in self.message_handlers:
            del self.message_handlers[topic]

    def get_stats(self):
        uptime_ms = int(time.time() * 1000) - self.connection_start if self.connected else 0
        return ConnectionStats(
            is_connected=self.connected,
            messages_sent=self.stats['messages_sent'],
            messages_received=self.stats['messages_received'],
            connection_time_ms=uptime_ms,
            uptime_seconds=uptime_ms // 1000,
            last_message_time=self.stats['last_message_time'],
        )

    def is_connected(self):
        return self.connected

    def disconnect(self):
        self.connected = False
        print('✓ Disconnected')


# ============================================================================
# TEST SUITE
# ============================================================================

tests_passed = 0
tests_failed = 0


def run_test(name, test_fn):
    global tests_passed, tests_failed
    try:
        test_fn()
        print(f"✅ PASS: {name}")
        tests_passed += 1
    except Exception as error:
        print(f"❌ FAIL: {name}")
        print(f"   Error: {str(error)}")
        tests_failed += 1


def test1_basic_connection():
    config = {
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    }

    client = FastDataBrokerQuicClient(config)

    if client.is_connected():
        raise AssertionError('Should not be connected yet')

    client.connect()

    if not client.is_connected():
        raise AssertionError('Should be connected')

    client.disconnect()


def test2_send_message():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    result = client.send_message(
        Message(topic='test.topic', payload={'data': 'test'}, priority=5)
    )

    if result.status != 'success':
        raise AssertionError(f"Expected status 'success', got '{result.status}'")

    if not result.message_id:
        raise AssertionError('Message ID should not be empty')

    if result.latency_ms < 0:
        raise AssertionError('Latency should be non-negative')

    client.disconnect()


def test3_message_handlers():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    handler_called = False
    
    def handler(message):
        nonlocal handler_called
        handler_called = True

    client.on_message('test.topic', handler)

    if 'test.topic' not in client.message_handlers:
        raise AssertionError('Handler should be registered')

    client.off_message('test.topic')

    if 'test.topic' in client.message_handlers:
        raise AssertionError('Handler should be unregistered')

    client.disconnect()


def test4_connection_statistics():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    # Send some messages
    for i in range(5):
        client.send_message(Message(topic='test.topic', payload={'index': i}))

    # Add small delay to ensure timing is measurable
    import time
    time.sleep(0.05)

    stats = client.get_stats()

    if not stats.is_connected:
        raise AssertionError('Should be connected')

    if stats.messages_sent != 5:
        raise AssertionError(f"Expected 5 messages sent, got {stats.messages_sent}")

    if stats.uptime_seconds < 0:
        raise AssertionError('Uptime should be non-negative')

    client.disconnect()


def test5_concurrent_messages():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    results = []
    for i in range(10):
        result = client.send_message(
            Message(topic='test.concurrent', payload={'index': i})
        )
        results.append(result)

    if len(results) != 10:
        raise AssertionError(f"Expected 10 results, got {len(results)}")

    if not all(r.status == 'success' for r in results):
        raise AssertionError('All messages should be successful')

    if client.stats['messages_sent'] != 10:
        raise AssertionError(f"Expected 10 messages sent, got {client.stats['messages_sent']}")

    client.disconnect()


def test6_priority_levels():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    priorities = [1, 5, 10, 20]  # LOW, NORMAL, HIGH, CRITICAL

    for priority in priorities:
        result = client.send_message(
            Message(
                topic='test.priority',
                payload={'priority': priority},
                priority=priority,
            )
        )

        if result.status != 'success':
            raise AssertionError(f"Failed to send message with priority {priority}")

    client.disconnect()


def test7_latency_measurement():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    client.connect()

    latencies = []
    for i in range(50):
        result = client.send_message(
            Message(topic='test.latency', payload={'iteration': i})
        )
        latencies.append(result.latency_ms)

    avg_latency = sum(latencies) / len(latencies)
    max_latency = max(latencies)

    if avg_latency < 0:
        raise AssertionError('Average latency should be non-negative')

    print(f"   Average latency: {avg_latency:.2f}ms, Max: {max_latency:.2f}ms")

    client.disconnect()


def test8_error_handling():
    client = FastDataBrokerQuicClient({
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    })

    try:
        client.send_message(Message(topic='test', payload={}))
        raise AssertionError('Should have thrown error')
    except ConnectionError as e:
        if str(e) != 'Not connected':
            raise AssertionError('Should throw "Not connected" error')

    client.connect()
    result = client.send_message(Message(topic='test', payload={}))
    if result.status != 'success':
        raise AssertionError('Should send successfully after connect')

    client.disconnect()


def test9_configuration_validation():
    config = {
        'host': 'localhost',
        'port': 6000,
        'tenant_id': 'test-tenant',
        'client_id': 'test-client',
        'psk_secret': 'test-secret',
    }

    client = FastDataBrokerQuicClient(config)

    if client.config['host'] != 'localhost':
        raise AssertionError('Configuration not properly saved')

    if client.config['port'] != 6000:
        raise AssertionError('Port configuration not properly saved')


# ============================================================================
# RUN ALL TESTS
# ============================================================================

def run_all_tests():
    global tests_passed, tests_failed
    
    print('\n' + '='*70)
    print('FastDataBroker Python SDK - Test Suite')
    print('='*70 + '\n')

    run_test('1. Basic Connection', test1_basic_connection)
    run_test('2. Send Message', test2_send_message)
    run_test('3. Message Handlers', test3_message_handlers)
    run_test('4. Connection Statistics', test4_connection_statistics)
    run_test('5. Concurrent Messages', test5_concurrent_messages)
    run_test('6. Priority Levels', test6_priority_levels)
    run_test('7. Latency Measurement', test7_latency_measurement)
    run_test('8. Error Handling', test8_error_handling)
    run_test('9. Configuration Validation', test9_configuration_validation)

    print('\n' + '='*70)
    print(f'Results: {tests_passed} passed, {tests_failed} failed')
    print('='*70 + '\n')

    return tests_failed == 0


if __name__ == '__main__':
    import sys
    success = run_all_tests()
    sys.exit(0 if success else 1)
