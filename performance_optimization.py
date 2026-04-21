#!/usr/bin/env python3
"""
FastDataBroker Performance Optimization & Benchmarking Suite
Includes load testing, performance profiling, and bottleneck detection
"""

import requests
import time
import threading
import statistics
from datetime import datetime
from typing import Dict, List, Tuple
from concurrent.futures import ThreadPoolExecutor, as_completed
import json

class PerformanceOptimizer:
    """Performance optimization and benchmarking utilities"""
    
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.results = {
            'throughput': [],
            'latencies': [],
            'errors': 0,
            'total_requests': 0
        }
    
    def health_check(self) -> bool:
        """Check if server is healthy"""
        try:
            resp = requests.get(f"{self.base_url}/health", timeout=2)
            return resp.status_code == 200
        except:
            return False
    
    def benchmark_endpoint(self, method: str, endpoint: str, 
                          data: dict = None, runs: int = 100) -> Dict:
        """Benchmark a single endpoint"""
        latencies = []
        errors = 0
        
        print(f"\n  Benchmarking: {method:<6} {endpoint:<40}")
        print(f"  Runs: {runs}")
        
        for i in range(runs):
            try:
                start = time.time()
                
                headers = {"Content-Type": "application/json"}
                if method.upper() == "GET":
                    requests.get(f"{self.base_url}{endpoint}", headers=headers, timeout=5)
                elif method.upper() == "POST":
                    requests.post(f"{self.base_url}{endpoint}", headers=headers, json=data, timeout=5)
                
                latency = (time.time() - start) * 1000
                latencies.append(latency)
                
                if (i + 1) % 20 == 0:
                    print(f"    Progress: {i+1}/{runs}")
            except Exception as e:
                errors += 1
        
        if latencies:
            stats = {
                'min': min(latencies),
                'max': max(latencies),
                'mean': statistics.mean(latencies),
                'median': statistics.median(latencies),
                'stdev': statistics.stdev(latencies) if len(latencies) > 1 else 0,
                'p90': sorted(latencies)[int(len(latencies)*0.9)] if len(latencies) > 1 else latencies[0],
                'p99': sorted(latencies)[int(len(latencies)*0.99)] if len(latencies) > 1 else latencies[0],
                'errors': errors,
                'success_rate': ((runs - errors) / runs * 100)
            }
            return stats
        return None
    
    def load_test(self, endpoint_config: List[Tuple], 
                 duration_seconds: int = 30,
                 concurrent_threads: int = 10) -> Dict:
        """
        Run load test with multiple concurrent connections
        endpoint_config: List of (method, endpoint, data_payload)
        """
        print(f"\n{'='*70}")
        print(f"{'LOAD TEST'.center(70)}")
        print(f"{'='*70}")
        print(f"  Duration: {duration_seconds} seconds")
        print(f"  Concurrent Threads: {concurrent_threads}")
        print(f"  Total Endpoints: {len(endpoint_config)}\n")
        
        start_time = time.time()
        results = {
            'total_requests': 0,
            'successful_requests': 0,
            'failed_requests': 0,
            'latencies': [],
            'requests_per_second': [],
            'throughput_start': 0
        }
        
        lock = threading.Lock()
        
        def worker(endpoint_config_item):
            method, endpoint, data = endpoint_config_item
            try:
                start = time.time()
                headers = {"Content-Type": "application/json"}
                
                if method.upper() == "GET":
                    resp = requests.get(f"{self.base_url}{endpoint}", headers=headers, timeout=5)
                else:
                    resp = requests.post(f"{self.base_url}{endpoint}", headers=headers, json=data, timeout=5)
                
                latency = (time.time() - start) * 1000
                
                with lock:
                    results['latencies'].append(latency)
                    if resp.status_code < 400:
                        results['successful_requests'] += 1
                    else:
                        results['failed_requests'] += 1
                    results['total_requests'] += 1
                
                return True
            except Exception as e:
                with lock:
                    results['failed_requests'] += 1
                    results['total_requests'] += 1
                return False
        
        # Run load test
        with ThreadPoolExecutor(max_workers=concurrent_threads) as executor:
            while time.time() - start_time < duration_seconds:
                futures = [executor.submit(worker, config) for config in endpoint_config]
                
                # Track throughput every second
                current_throughput = results['total_requests'] / (time.time() - start_time)
                results['requests_per_second'].append(current_throughput)
                
                elapsed = int(time.time() - start_time)
                print(f"  [{elapsed:2d}s] RPS: {current_throughput:7.0f} | Total: {results['total_requests']:6d} | "
                      f"Success: {results['successful_requests']:6d} | Failed: {results['failed_requests']:6d}")
                
                for future in as_completed(futures):
                    try:
                        future.result()
                    except:
                        pass
                
                time.sleep(0.1)
        
        elapsed = time.time() - start_time
        
        # Calculate statistics
        stats = {
            'duration': elapsed,
            'total_requests': results['total_requests'],
            'successful': results['successful_requests'],
            'failed': results['failed_requests'],
            'success_rate': results['successful_requests'] / results['total_requests'] * 100,
            'throughput_avg': results['total_requests'] / elapsed,
            'throughput_peak': max(results['requests_per_second']) if results['requests_per_second'] else 0,
            'throughput_min': min(results['requests_per_second']) if results['requests_per_second'] else 0,
        }
        
        if results['latencies']:
            stats.update({
                'latency_min': min(results['latencies']),
                'latency_max': max(results['latencies']),
                'latency_avg': statistics.mean(results['latencies']),
                'latency_p50': sorted(results['latencies'])[len(results['latencies'])//2],
                'latency_p90': sorted(results['latencies'])[int(len(results['latencies'])*0.9)],
                'latency_p99': sorted(results['latencies'])[int(len(results['latencies'])*0.99)]
            })
        
        return stats
    
    def print_benchmark(self, endpoint: str, stats: Dict):
        """Print benchmark results"""
        print(f"\n  Latency Statistics (ms):")
        print(f"    Min:      {stats['min']:.3f}")
        print(f"    Max:      {stats['max']:.3f}")
        print(f"    Mean:     {stats['mean']:.3f}")
        print(f"    Median:   {stats['median']:.3f}")
        print(f"    StdDev:   {stats['stdev']:.3f}")
        print(f"    p90:      {stats['p90']:.3f}")
        print(f"    p99:      {stats['p99']:.3f}")
        print(f"  Success Rate: {stats['success_rate']:.2f}%")
        if stats['errors'] > 0:
            print(f"  Errors: {stats['errors']}")
    
    def print_load_test_results(self, stats: Dict):
        """Print load test results"""
        print(f"\n{'='*70}")
        print(f"{'LOAD TEST RESULTS'.center(70)}")
        print(f"{'='*70}\n")
        
        print(f"  Duration: {stats['duration']:.2f} seconds")
        print(f"  Total Requests: {stats['total_requests']}")
        print(f"  Successful: {stats['successful']} ({stats['success_rate']:.2f}%)")
        print(f"  Failed: {stats['failed']}")
        
        print(f"\n  Throughput (RPS):")
        print(f"    Average:   {stats['throughput_avg']:.0f}")
        print(f"    Peak:      {stats['throughput_peak']:.0f}")
        print(f"    Minimum:   {stats['throughput_min']:.0f}")
        
        if 'latency_min' in stats:
            print(f"\n  Latency (ms):")
            print(f"    Min:       {stats['latency_min']:.3f}")
            print(f"    Avg:       {stats['latency_avg']:.3f}")
            print(f"    p50:       {stats['latency_p50']:.3f}")
            print(f"    p90:       {stats['latency_p90']:.3f}")
            print(f"    p99:       {stats['latency_p99']:.3f}")
            print(f"    Max:       {stats['latency_max']:.3f}")
        
        print(f"\n{'='*70}\n")
    
    def detect_bottlenecks(self) -> Dict:
        """Detect performance bottlenecks"""
        print(f"\n{'='*70}")
        print(f"{'BOTTLENECK DETECTION'.center(70)}")
        print(f"{'='*70}\n")
        
        bottlenecks = {
            'cpu_intensive': [],
            'io_slow': [],
            'memory_heavy': [],
            'lock_contention': []
        }
        
        # Test endpoints and identify slow ones
        endpoints = [
            ('GET', '/api/auth/me'),
            ('GET', '/api/admin/metrics'),
            ('POST', '/api/auth/login', {'username': 'admin', 'password': 'password'}),
            ('GET', '/api/admin/tenants'),
        ]
        
        for endpoint_config in endpoints:
            if len(endpoint_config) == 2:
                method, endpoint = endpoint_config
                data = None
            else:
                method, endpoint, data = endpoint_config
            
            stats = self.benchmark_endpoint(method, endpoint, data, runs=50)
            if stats:
                if stats['mean'] > 5:
                    bottlenecks['io_slow'].append((endpoint, stats['mean']))
                if stats['stdev'] > stats['mean'] * 0.5:
                    bottlenecks['lock_contention'].append((endpoint, stats['stdev']))
        
        # Print bottleneck analysis
        if bottlenecks['io_slow']:
            print("  🔴 Slow I/O Operations (>5ms average):")
            for endpoint, latency in bottlenecks['io_slow']:
                print(f"    • {endpoint}: {latency:.2f}ms")
        
        if bottlenecks['lock_contention']:
            print("  🟡 Lock Contention (high stddev):")
            for endpoint, stdev in bottlenecks['lock_contention']:
                print(f"    • {endpoint}: {stdev:.2f}ms stddev")
        
        if not any(bottlenecks.values()):
            print("  ✅ No significant bottlenecks detected!")
        
        return bottlenecks
    
    def optimization_recommendations(self) -> List[str]:
        """Provide optimization recommendations"""
        print(f"\n{'='*70}")
        print(f"{'OPTIMIZATION RECOMMENDATIONS'.center(70)}")
        print(f"{'='*70}\n")
        
        recommendations = [
            "1. 🔒 Replace Arc<Mutex>> with DashMap for lock-free metrics (2-3x improvement)",
            "2. 💾 Implement object pooling for frequently allocated types (1.5-2x improvement)",
            "3. 📦 Use SmallVec for small collections instead of Vec (20-30% improvement)",
            "4. 🔄 Add connection pooling for outbound requests",
            "5. 📊 Implement batch processing for high-volume operations",
            "6. 🗜️  Consider bincode serialization for internal messages (2-5x faster than JSON)",
            "7. 🚀 Enable SIMD optimizations in Cargo.toml",
            "8. 📈 Add request coalescing to reduce redundant processing",
            "9. 🧠 Implement intelligent caching for frequently accessed data",
            "10. ⚡ Profile with flamegraph to identify CPU hotspots",
        ]
        
        for rec in recommendations:
            print(f"  {rec}")
        
        return recommendations

def main():
    print("\n" + "="*70)
    print("FastDataBroker Performance Optimization Suite".center(70))
    print("="*70)
    
    optimizer = PerformanceOptimizer("http://localhost:8080")
    
    # Check server health
    print("\n1️⃣  Checking server health...")
    if not optimizer.health_check():
        print("   ❌ Server is not running!")
        print("   Start the server first and try again")
        return
    print("   ✅ Server is healthy!")
    
    # Benchmark individual endpoints
    print("\n2️⃣  Benchmarking individual endpoints...")
    endpoints = [
        ('GET', '/api/auth/me'),
        ('GET', '/api/admin/metrics'),
        ('GET', '/api/admin/health'),
    ]
    
    for method, endpoint in endpoints:
        stats = optimizer.benchmark_endpoint(method, endpoint, runs=50)
        if stats:
            optimizer.print_benchmark(endpoint, stats)
    
    # Load testing
    print("\n3️⃣  Running load test...")
    load_config = [
        ('GET', '/api/auth/me', None),
        ('GET', '/api/admin/metrics', None),
        ('GET', '/api/admin/tenants', None),
    ]
    
    stats = optimizer.load_test(load_config, duration_seconds=30, concurrent_threads=10)
    optimizer.print_load_test_results(stats)
    
    # Detect bottlenecks
    print("\n4️⃣  Detecting bottlenecks...")
    optimizer.detect_bottlenecks()
    
    # Optimization recommendations
    print("\n5️⃣  Optimization recommendations...")
    optimizer.optimization_recommendations()
    
    print("\n✅ Performance analysis complete!")
    print("   Review the recommendations above and implement optimizations")

if __name__ == "__main__":
    main()
