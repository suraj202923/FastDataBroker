"""
FastDataBroker Python SDK - Performance Benchmarking Suite
Comprehensive performance metrics for QUIC handshake and message queue operations
"""

import time
import statistics
import json
import psutil
import os
from datetime import datetime
from typing import List, Dict, Any
from fastdatabroker_sdk import TenantQuicClient, TenantConfig, Message, TenantRole

# ============================================================================
# BENCHMARK CONFIGURATION
# ============================================================================

BENCHMARK_CONFIG = {
    'handshake_iterations': 50,
    'message_batch_size': 100,
    'concurrent_tenants': 10,
    'message_sizes': [100, 1000, 10000],
    'duration_seconds': 5,
}

# ============================================================================
# METRICS COLLECTION
# ============================================================================

class BenchmarkMetrics:
    """Collect and analyze benchmark metrics"""
    
    def __init__(self, name: str):
        self.name = name
        self.measurements = []
        self.start_time = None
        self.end_time = None
        self.start_memory = None
        self.end_memory = None
        self.start_cpu = None
        self.end_cpu = None
        
    def start(self):
        """Start benchmark timer"""
        self.start_time = time.time() * 1000  # milliseconds
        self.start_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)  # MB
        self.start_cpu = psutil.cpu_percent(interval=None)
        
    def end(self):
        """End benchmark timer"""
        self.end_time = time.time() * 1000
        self.end_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
        self.end_cpu = psutil.cpu_percent(interval=None)
        
    def add_measurement(self, value: float):
        """Add individual measurement"""
        self.measurements.append(value)
        
    def get_stats(self) -> Dict[str, Any]:
        """Calculate statistics"""
        if not self.measurements:
            return {}
            
        return {
            'name': self.name,
            'count': len(self.measurements),
            'min': min(self.measurements),
            'max': max(self.measurements),
            'mean': statistics.mean(self.measurements),
            'median': statistics.median(self.measurements),
            'stdev': statistics.stdev(self.measurements) if len(self.measurements) > 1 else 0,
            'total_time_ms': self.end_time - self.start_time if self.end_time else 0,
            'memory_delta_mb': self.end_memory - self.start_memory if self.end_memory else 0,
            'cpu_usage': self.end_cpu if self.end_cpu else 0,
        }

# ============================================================================
# HANDSHAKE BENCHMARKS
# ============================================================================

def benchmark_handshake_duration():
    """Benchmark QUIC handshake duration"""
    print("\n[BENCHMARK 1] QUIC Handshake Duration")
    print("-" * 70)
    
    metrics = BenchmarkMetrics("QUIC Handshake")
    metrics.start()
    
    for i in range(BENCHMARK_CONFIG['handshake_iterations']):
        config = TenantConfig(
            tenant_id=f'benchmark-tenant-{i}',
            psk_secret=f'secret-{i}',
            client_id=f'client-{i}',
            api_key=f'api_key_{i}'
        )
        
        client = TenantQuicClient('localhost', 6000, config)
        handshake_start = time.time() * 1000
        
        client.connect()
        
        handshake_time = (time.time() * 1000) - handshake_start
        metrics.add_measurement(handshake_time)
        
        client.disconnect()
    
    metrics.end()
    stats = metrics.get_stats()
    
    print(f"Total Handshakes: {stats['count']}")
    print(f"Min Time: {stats['min']:.2f}ms")
    print(f"Max Time: {stats['max']:.2f}ms")
    print(f"Mean Time: {stats['mean']:.2f}ms")
    print(f"Median Time: {stats['median']:.2f}ms")
    print(f"Std Dev: {stats['stdev']:.2f}ms")
    print(f"Total Duration: {stats['total_time_ms']:.2f}ms")
    print(f"Memory Delta: {stats['memory_delta_mb']:.2f}MB")
    
    return stats

# ============================================================================
# MESSAGE THROUGHPUT BENCHMARKS
# ============================================================================

def benchmark_message_throughput():
    """Benchmark message send throughput"""
    print("\n[BENCHMARK 2] Message Throughput")
    print("-" * 70)
    
    config = TenantConfig(
        tenant_id='throughput-tenant',
        psk_secret='throughput-secret',
        client_id='throughput-client',
        api_key='throughput_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    client.connect()
    
    metrics = BenchmarkMetrics("Message Send Throughput")
    metrics.start()
    
    # Send messages in batches
    total_messages = 0
    for batch in range(3):
        for i in range(BENCHMARK_CONFIG['message_batch_size']):
            msg = Message(
                topic='benchmark.topic',
                payload={'batch': batch, 'index': i, 'data': 'x' * 100}
            )
            
            msg_start = time.time() * 1000
            result = client.send_message(msg)
            msg_time = (time.time() * 1000) - msg_start
            
            metrics.add_measurement(msg_time)
            total_messages += 1
    
    metrics.end()
    stats = metrics.get_stats()
    
    throughput = (total_messages / (stats['total_time_ms'] / 1000))  # messages/sec
    
    print(f"Total Messages Sent: {total_messages}")
    print(f"Throughput: {throughput:.2f} messages/second")
    print(f"Min Latency: {stats['min']:.2f}ms")
    print(f"Max Latency: {stats['max']:.2f}ms")
    print(f"Mean Latency: {stats['mean']:.2f}ms")
    print(f"Median Latency: {stats['median']:.2f}ms")
    print(f"Total Duration: {stats['total_time_ms']:.2f}ms")
    print(f"Memory Delta: {stats['memory_delta_mb']:.2f}MB")
    
    client.disconnect()
    return stats

# ============================================================================
# CONCURRENT CONNECTIONS BENCHMARK
# ============================================================================

def benchmark_concurrent_connections():
    """Benchmark concurrent tenant connections"""
    print("\n[BENCHMARK 3] Concurrent Connections")
    print("-" * 70)
    
    metrics = BenchmarkMetrics("Concurrent Connections")
    metrics.start()
    
    clients = []
    connection_times = []
    
    for i in range(BENCHMARK_CONFIG['concurrent_tenants']):
        config = TenantConfig(
            tenant_id=f'concurrent-tenant-{i}',
            psk_secret=f'secret-{i}',
            client_id=f'client-{i}',
            api_key=f'api_key_{i}'
        )
        
        client = TenantQuicClient('localhost', 6000, config)
        conn_start = time.time() * 1000
        
        client.connect()
        
        conn_time = (time.time() * 1000) - conn_start
        connection_times.append(conn_time)
        metrics.add_measurement(conn_time)
        clients.append(client)
    
    # All connected - send messages concurrently
    print(f"\nAll {len(clients)} tenants connected. Sending messages concurrently...")
    
    concurrent_metrics = BenchmarkMetrics("Concurrent Message Send")
    concurrent_metrics.start()
    
    for round_num in range(5):
        for client in clients:
            msg = Message(
                topic='concurrent.topic',
                payload={'round': round_num, 'data': 'test'}
            )
            send_start = time.time() * 1000
            client.send_message(msg)
            send_time = (time.time() * 1000) - send_start
            concurrent_metrics.add_measurement(send_time)
    
    concurrent_metrics.end()
    
    # Disconnect all
    for client in clients:
        client.disconnect()
    
    metrics.end()
    stats = metrics.get_stats()
    concurrent_stats = concurrent_metrics.get_stats()
    
    print(f"Concurrent Connections: {len(clients)}")
    print(f"Connection Time (per client):")
    print(f"  Min: {min(connection_times):.2f}ms")
    print(f"  Max: {max(connection_times):.2f}ms")
    print(f"  Mean: {statistics.mean(connection_times):.2f}ms")
    print(f"Concurrent Message Send:")
    print(f"  Mean Latency: {concurrent_stats['mean']:.2f}ms")
    print(f"  Total Duration: {stats['total_time_ms']:.2f}ms")
    
    return stats

# ============================================================================
# MESSAGE SIZE BENCHMARK
# ============================================================================

def benchmark_message_sizes():
    """Benchmark different message sizes"""
    print("\n[BENCHMARK 4] Message Size Performance")
    print("-" * 70)
    
    config = TenantConfig(
        tenant_id='size-benchmark-tenant',
        psk_secret='size-secret',
        client_id='size-client',
        api_key='size_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    client.connect()
    
    results = {}
    
    for size in BENCHMARK_CONFIG['message_sizes']:
        print(f"\nTesting {size} byte payload...")
        metrics = BenchmarkMetrics(f"Message Size {size}")
        metrics.start()
        
        payload_data = 'x' * size
        
        for i in range(20):
            msg = Message(
                topic='size.benchmark',
                payload={'size': size, 'data': payload_data}
            )
            
            msg_start = time.time() * 1000
            client.send_message(msg)
            msg_time = (time.time() * 1000) - msg_start
            metrics.add_measurement(msg_time)
        
        metrics.end()
        stats = metrics.get_stats()
        results[size] = stats
        
        print(f"  Mean Latency: {stats['mean']:.2f}ms")
        print(f"  Std Dev: {stats['stdev']:.2f}ms")
    
    client.disconnect()
    return results

# ============================================================================
# TENANT ISOLATION BENCHMARK
# ============================================================================

def benchmark_tenant_isolation():
    """Benchmark performance with multiple isolated tenants"""
    print("\n[BENCHMARK 5] Tenant Isolation Performance")
    print("-" * 70)
    
    num_tenants = 5
    messages_per_tenant = 20
    
    metrics = BenchmarkMetrics("Tenant Isolation")
    metrics.start()
    
    # Create and connect all tenants
    clients = []
    for i in range(num_tenants):
        config = TenantConfig(
            tenant_id=f'isolation-tenant-{i}',
            psk_secret=f'secret-{i}',
            client_id=f'client-{i}',
            api_key=f'api_key_{i}',
            rate_limit_rps=1000
        )
        
        client = TenantQuicClient('localhost', 6000, config)
        client.connect()
        clients.append(client)
    
    # Send messages from each tenant
    cross_tenant_sends = 0
    for tenant_idx, client in enumerate(clients):
        for msg_idx in range(messages_per_tenant):
            msg = Message(
                topic='isolation.test',
                payload={'tenant': tenant_idx, 'msg': msg_idx}
            )
            
            send_start = time.time() * 1000
            result = client.send_message(msg)
            send_time = (time.time() * 1000) - send_start
            
            metrics.add_measurement(send_time)
            cross_tenant_sends += 1
            
            # Verify tenant isolation
            assert result.tenant_id == f'isolation-tenant-{tenant_idx}'
    
    # Disconnect all
    for client in clients:
        client.disconnect()
    
    metrics.end()
    stats = metrics.get_stats()
    
    print(f"Tenants: {num_tenants}")
    print(f"Messages per Tenant: {messages_per_tenant}")
    print(f"Total Cross-Tenant Operations: {cross_tenant_sends}")
    print(f"Mean Latency: {stats['mean']:.2f}ms")
    print(f"Total Duration: {stats['total_time_ms']:.2f}ms")
    print(f"✓ All tenant isolation checks passed")
    
    return stats

# ============================================================================
# MEMORY USAGE BENCHMARK
# ============================================================================

def benchmark_memory_usage():
    """Benchmark memory usage"""
    print("\n[BENCHMARK 6] Memory Usage Analysis")
    print("-" * 70)
    
    import gc
    gc.collect()
    
    initial_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
    print(f"Initial Memory: {initial_memory:.2f}MB")
    
    # Create multiple connections
    clients = []
    config_start_memory = initial_memory
    
    for i in range(20):
        config = TenantConfig(
            tenant_id=f'memory-tenant-{i}',
            psk_secret=f'secret-{i}',
            client_id=f'client-{i}',
            api_key=f'api_key_{i}'
        )
        clients.append(config)
    
    config_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
    print(f"After 20 Configs: {config_memory:.2f}MB (+{config_memory - config_start_memory:.2f}MB)")
    
    # Connect all
    client_objects = []
    conn_start_memory = config_memory
    
    for config in clients:
        client = TenantQuicClient('localhost', 6000, config)
        client.connect()
        client_objects.append(client)
    
    connected_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
    print(f"After 20 Connections: {connected_memory:.2f}MB (+{connected_memory - conn_start_memory:.2f}MB)")
    
    # Average per connection
    avg_per_connection = (connected_memory - initial_memory) / 20
    print(f"Average per Connection: {avg_per_connection:.2f}MB")
    
    # Disconnect all
    for client in client_objects:
        client.disconnect()
    
    gc.collect()
    final_memory = psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
    print(f"After Disconnection: {final_memory:.2f}MB")
    
    return {
        'initial_mb': initial_memory,
        'peak_mb': connected_memory,
        'final_mb': final_memory,
        'avg_per_connection_mb': avg_per_connection
    }

# ============================================================================
# CONNECTION STABILITY BENCHMARK
# ============================================================================

def benchmark_connection_stability():
    """Benchmark connection stability over time"""
    print("\n[BENCHMARK 7] Connection Stability & Recovery")
    print("-" * 70)
    
    config = TenantConfig(
        tenant_id='stability-tenant',
        psk_secret='stability-secret',
        client_id='stability-client',
        api_key='stability_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    client.connect()
    
    print("Testing connection stability over 30 operations...")
    
    failures = 0
    recovery_times = []
    
    for i in range(30):
        try:
            msg = Message(topic='stability.test', payload={'index': i})
            start = time.time() * 1000
            result = client.send_message(msg)
            end = time.time() * 1000
            
            if result.status != 'success':
                failures += 1
                recovery_start = time.time() * 1000
                # Try to recover
                if client.connect():
                    recovery_time = time.time() * 1000 - recovery_start
                    recovery_times.append(recovery_time)
        except Exception as e:
            failures += 1
    
    client.disconnect()
    
    uptime = 100 * (1 - (failures / 30))
    print(f"Uptime: {uptime:.1f}%")
    print(f"Failures: {failures}/30")
    if recovery_times:
        print(f"Average Recovery Time: {statistics.mean(recovery_times):.2f}ms")
    
    return {
        'uptime_percent': uptime,
        'failures': failures,
        'avg_recovery_ms': statistics.mean(recovery_times) if recovery_times else 0
    }

# ============================================================================
# SUMMARY REPORT
# ============================================================================

def generate_benchmark_report(results: Dict[str, Any]):
    """Generate comprehensive benchmark report"""
    
    report = {
        'timestamp': datetime.now().isoformat(),
        'system_info': {
            'cpu_count': psutil.cpu_count(),
            'total_memory_gb': psutil.virtual_memory().total / (1024**3),
            'python_version': os.sys.version,
        },
        'benchmarks': results
    }
    
    return report

# ============================================================================
# MAIN EXECUTION
# ============================================================================

def run_all_benchmarks():
    """Run complete benchmark suite"""
    
    print("\n" + "="*70)
    print("FastDataBroker Python SDK - Performance Benchmarking Suite")
    print("="*70)
    print(f"Start Time: {datetime.now()}")
    print(f"Configuration: {BENCHMARK_CONFIG}")
    
    results = {}
    
    try:
        # Run all benchmarks
        results['handshake'] = benchmark_handshake_duration()
        results['throughput'] = benchmark_message_throughput()
        results['concurrent'] = benchmark_concurrent_connections()
        results['message_sizes'] = benchmark_message_sizes()
        results['isolation'] = benchmark_tenant_isolation()
        results['memory'] = benchmark_memory_usage()
        results['stability'] = benchmark_connection_stability()
        
    except Exception as e:
        print(f"\n❌ Benchmark Error: {e}")
        import traceback
        traceback.print_exc()
        return False
    
    # Generate and display report
    print("\n" + "="*70)
    print("BENCHMARK SUMMARY")
    print("="*70)
    
    report = generate_benchmark_report(results)
    
    print(f"\n[OK] All benchmarks completed at {datetime.now()}")
    print(f"\nKey Metrics:")
    print(f"  - Handshake: {results['handshake']['mean']:.2f}ms ±{results['handshake']['stdev']:.2f}ms")
    print(f"  - Message Throughput: {results['throughput']['count']} messages")
    print(f"  - Concurrent Connections: {BENCHMARK_CONFIG['concurrent_tenants']} tenants")
    print(f"  - Peak Memory: {results['memory']['peak_mb']:.2f}MB")
    print(f"  - Connection Stability: {results['stability']['uptime_percent']:.1f}%")
    
    # Save report
    report_file = 'benchmark_results.json'
    with open(report_file, 'w') as f:
        # Convert non-serializable objects
        json.dump({
            'timestamp': report['timestamp'],
            'handshake_ms': results['handshake']['mean'],
            'throughput_count': results['throughput']['count'],
            'concurrent_tenants': BENCHMARK_CONFIG['concurrent_tenants'],
            'peak_memory_mb': results['memory']['peak_mb'],
            'stability_percent': results['stability']['uptime_percent'],
        }, f, indent=2)
    
    print(f"\n[REPORT] Report saved to: {report_file}")
    
    print("\n" + "="*70 + "\n")
    
    return True


if __name__ == '__main__':
    import sys
    success = run_all_benchmarks()
    sys.exit(0 if success else 1)
