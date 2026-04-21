/// Performance and Benchmark tests for FastDataBroker
/// Tests throughput, latency, and scalability

#[cfg(test)]
mod benchmark_tests {
    use fastdatabroker::queue::AsyncQueue;
    use fastdatabroker::persistent_queue::AsyncPersistenceQueue;
    use fastdatabroker::priority_queue::{AsyncPriorityQueue, Priority};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Instant, Duration};
    use std::thread;
    use std::fs;

    fn cleanup_test_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    // ============== Throughput Benchmarks ==============

    #[test]
    fn bench_async_queue_sequential_throughput() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let item_count = 100_000;

        let start = Instant::now();

        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }

        let elapsed = start.elapsed();
        let throughput = item_count as f64 / elapsed.as_secs_f64();

        println!("Sequential Queue Push Throughput: {:.0} items/sec", throughput);
        println!("Time for {} items: {:?}", item_count, elapsed);

        assert_eq!(queue.total_pushed(), item_count as u64);
    }

    #[test]
    fn bench_async_queue_parallel_throughput() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let item_count = 100_000;
        let thread_count = 8;

        let start = Instant::now();

        let mut handles = vec![];

        for thread_id in 0..thread_count {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                let items_per_thread = item_count / thread_count;
                for i in 0..items_per_thread {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let elapsed = start.elapsed();
        let throughput = item_count as f64 / elapsed.as_secs_f64();

        println!("Parallel Queue Push Throughput: {:.0} items/sec", throughput);
        println!("Time for {} items across {} threads: {:?}", item_count, thread_count, elapsed);

        assert_eq!(queue.total_pushed(), item_count as u64);
    }

    #[test]
    fn bench_batch_push_performance() {
        let queue = AsyncQueue::new(1, 256).expect("Failed to create queue");
        let total_items = 100_000;
        let batch_size = 1000;

        let start = Instant::now();

        let mut total_pushed = 0;
        while total_pushed < total_items {
            let items: Vec<Vec<u8>> = (0..batch_size.min(total_items - total_pushed))
                .map(|i| format!("item_{}", total_pushed + i).into_bytes())
                .collect();

            let _guids = queue.push_batch(items).expect("Batch push failed");
            total_pushed += batch_size;
        }

        let elapsed = start.elapsed();
        let throughput = total_items as f64 / elapsed.as_secs_f64();

        println!("Batch Push Throughput ({} batch size): {:.0} items/sec", batch_size, throughput);
        println!("Time for {} items: {:?}", total_items, elapsed);

        assert_eq!(queue.total_pushed(), total_items as u64);
    }

    #[test]
    fn bench_persistence_queue_throughput() {
        let path = "bench_persistence_throughput";
        cleanup_test_dir(path);

        let queue = AsyncPersistenceQueue::new(1, 256, path, false)
            .expect("Failed to create queue");

        let item_count = 50_000;

        let start = Instant::now();

        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }

        let elapsed = start.elapsed();
        let throughput = item_count as f64 / elapsed.as_secs_f64();

        println!("Persistence Queue Push Throughput: {:.0} items/sec", throughput);
        println!("Time for {} items with persistence: {:?}", item_count, elapsed);

        assert_eq!(queue.total_pushed(), item_count as u64);

        cleanup_test_dir(path);
    }

    #[test]
    fn bench_priority_queue_throughput() {
        let path = "bench_priority_throughput";
        cleanup_test_dir(path);

        let queue = AsyncPriorityQueue::new(1, path)
            .expect("Failed to create queue");

        let item_count = 50_000;

        let start = Instant::now();

        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let priority = match i % 4 {
                0 => Priority::LOW,
                1 => Priority::MEDIUM,
                2 => Priority::HIGH,
                _ => Priority::CRITICAL,
            };
            let _guid = queue.push(data, priority).expect("Push failed");
        }

        let elapsed = start.elapsed();
        let throughput = item_count as f64 / elapsed.as_secs_f64();

        println!("Priority Queue Push Throughput: {:.0} items/sec", throughput);
        println!("Time for {} items: {:?}", item_count, elapsed);

        assert_eq!(queue.total_pushed(), item_count as u64);

        cleanup_test_dir(path);
    }

    // ============== Latency Benchmarks ==============

    #[test]
    fn bench_single_push_latency() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        let iterations = 10_000;

        let mut latencies = vec![];

        for i in 0..iterations {
            let data = format!("item_{}", i).into_bytes();
            
            let start = Instant::now();
            let _guid = queue.push(data).expect("Push failed");
            let elapsed = start.elapsed();
            
            if i > 100 { // Skip warmup
                latencies.push(elapsed);
            }
        }

        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let min_latency = latencies.iter().min().unwrap();
        let max_latency = latencies.iter().max().unwrap();

        println!("Single Push Latency - Avg: {:?}, Min: {:?}, Max: {:?}", 
                 avg_latency, min_latency, max_latency);

        // Check latency is reasonable
        assert!(avg_latency < Duration::from_micros(100));
    }

    #[test]
    fn bench_batch_push_latency() {
        let queue = AsyncQueue::new(1, 256).expect("Failed to create queue");
        let iterations = 1000;
        let batch_size = 100;

        let mut latencies = vec![];

        for batch_num in 0..iterations {
            let items: Vec<Vec<u8>> = (0..batch_size)
                .map(|i| format!("batch_{}_item_{}", batch_num, i).into_bytes())
                .collect();

            let start = Instant::now();
            let _guids = queue.push_batch(items).expect("Batch push failed");
            let elapsed = start.elapsed();

            if batch_num > 10 { // Skip warmup
                latencies.push(elapsed);
            }
        }

        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let min_latency = latencies.iter().min().unwrap();
        let max_latency = latencies.iter().max().unwrap();

        println!("Batch Push Latency ({} items) - Avg: {:?}, Min: {:?}, Max: {:?}",
                 batch_size, avg_latency, min_latency, max_latency);

        // Batch should be faster per item
        assert!(avg_latency / (batch_size as u32) < Duration::from_micros(50));
    }

    // ============== Concurrent Throughput ==============

    #[test]
    fn bench_concurrent_push_remove_throughput() {
        let queue = Arc::new(AsyncQueue::new(1, 512)
            .expect("Failed to create queue"));

        let total_items = 100_000;
        let thread_count = 4;

        let push_count = Arc::new(AtomicU64::new(0));
        let remove_count = Arc::new(AtomicU64::new(0));

        let start = Instant::now();

        // Push threads
        let mut handles = vec![];

        for thread_id in 0..thread_count {
            let queue_clone = Arc::clone(&queue);
            let push_count_clone = Arc::clone(&push_count);

            let handle = thread::spawn(move || {
                let items_per_thread = total_items / thread_count;
                for i in 0..items_per_thread {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                    push_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }

        // Remove thread
        let queue_clone = Arc::clone(&queue);
        let remove_count_clone = Arc::clone(&remove_count);
        let remove_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for i in 0..total_items / 2 {
                let guid = format!("remove_{}", i);
                if queue_clone.remove_by_guid(&guid) {
                    remove_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        remove_handle.join().expect("Remove thread panicked");

        let elapsed = start.elapsed();

        let push_throughput = total_items as f64 / elapsed.as_secs_f64();
        println!("Concurrent Push/Remove Throughput: {:.0} ops/sec", push_throughput);
        println!("Push count: {}, Remove attempts: {}", 
                 push_count.load(Ordering::Relaxed),
                 remove_count.load(Ordering::Relaxed));
    }

    // ============== Memory Usage Benchmarks ==============

    #[test]
    fn bench_memory_efficiency() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");

        let small_data_size = 100;
        let large_data_size = 1_000_000;
        let item_count = 10_000;

        // Small data
        let start = Instant::now();
        for i in 0..item_count {
            let data = vec![0u8; small_data_size];
            let _guid = queue.push(data).expect("Push failed");
        }
        let small_elapsed = start.elapsed();

        let queue2 = AsyncQueue::new(1, 128).expect("Failed to create queue 2");

        // Large data
        let start = Instant::now();
        for i in 0..(item_count / 10) {
            let data = vec![0u8; large_data_size];
            let _guid = queue2.push(data).expect("Push failed");
        }
        let large_elapsed = start.elapsed();

        println!("Small data ({} bytes, {} items): {:?}", small_data_size, item_count, small_elapsed);
        println!("Large data ({} bytes, {} items): {:?}", large_data_size, item_count / 10, large_elapsed);
    }

    // ============== Scalability Benchmarks ==============

    #[test]
    fn bench_scalability_thread_count() {
        let thread_counts = vec![1, 2, 4, 8, 16];

        for thread_count in thread_counts {
            let queue = Arc::new(AsyncQueue::new(1, 256)
                .expect("Failed to create queue"));

            let items_per_thread = 10_000;

            let start = Instant::now();

            let mut handles = vec![];
            for thread_id in 0..thread_count {
                let queue_clone = Arc::clone(&queue);
                let handle = thread::spawn(move || {
                    for i in 0..items_per_thread {
                        let data = format!("t{}_i{}", thread_id, i).into_bytes();
                        let _guid = queue_clone.push(data).expect("Push failed");
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().expect("Thread panicked");
            }

            let elapsed = start.elapsed();
            let total_items = thread_count * items_per_thread;
            let throughput = total_items as f64 / elapsed.as_secs_f64();

            println!("Threads: {}, Throughput: {:.0} items/sec", thread_count, throughput);
        }
    }

    // ============== Mode Performance Comparison ==============

    #[test]
    fn bench_sequential_vs_parallel() {
        let item_count = 50_000;

        // Sequential
        let queue_seq = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let start = Instant::now();
        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue_seq.push(data).expect("Push failed");
        }
        let seq_elapsed = start.elapsed();

        // Parallel
        let queue_par = Arc::new(AsyncQueue::new(1, 256)
            .expect("Failed to create queue"));
        let start = Instant::now();

        let mut handles = vec![];
        for thread_id in 0..4 {
            let queue_clone = Arc::clone(&queue_par);
            let handle = thread::spawn(move || {
                for i in 0..(item_count / 4) {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let par_elapsed = start.elapsed();

        let seq_throughput = item_count as f64 / seq_elapsed.as_secs_f64();
        let par_throughput = item_count as f64 / par_elapsed.as_secs_f64();

        println!("Sequential Mode: {:.0} items/sec", seq_throughput);
        println!("Parallel Mode: {:.0} items/sec", par_throughput);
        println!("Parallel speedup: {:.2}x", par_throughput / seq_throughput);
    }

    // ============== Persistence Overhead ==============

    #[test]
    fn bench_persistence_overhead() {
        let item_count = 10_000;

        // Without persistence
        let queue_mem = AsyncQueue::new(1, 256)
            .expect("Failed to create queue");

        let start = Instant::now();
        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue_mem.push(data).expect("Push failed");
        }
        let mem_elapsed = start.elapsed();

        // With persistence
        let path = "bench_persistence_overhead";
        cleanup_test_dir(path);

        let queue_disk = AsyncPersistenceQueue::new(1, 256, path, false)
            .expect("Failed to create queue");

        let start = Instant::now();
        for i in 0..item_count {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue_disk.push(data).expect("Push failed");
        }
        let disk_elapsed = start.elapsed();

        let mem_throughput = item_count as f64 / mem_elapsed.as_secs_f64();
        let disk_throughput = item_count as f64 / disk_elapsed.as_secs_f64();

        println!("Memory Queue: {:.0} items/sec", mem_throughput);
        println!("Persistent Queue: {:.0} items/sec", disk_throughput);
        println!("Persistence Overhead: {:.2}x", mem_throughput / disk_throughput);

        cleanup_test_dir(path);
    }
}
