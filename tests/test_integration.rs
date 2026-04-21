/// Integration tests for FastDataBroker
/// Tests interactions between multiple modules and complex scenarios

#[cfg(test)]
mod integration_tests {
    use fastdatabroker::queue::AsyncQueue;
    use fastdatabroker::persistent_queue::AsyncPersistenceQueue;
    use fastdatabroker::priority_queue::{AsyncPriorityQueue, Priority};
    use fastdatabroker::models::Envelope;
    use std::sync::Arc;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    fn cleanup_test_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    // ============== Multi-Queue Workflow ==============

    #[test]
    fn test_queue_workflow_sequential() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // Push items
        for i in 0..100 {
            let data = format!("item_{}", i).into_bytes();
            queue.push(data).expect("Push failed");
        }

        assert_eq!(queue.total_pushed(), 100);

        // Simulate processing by removing items
        for i in 0..50 {
            let guid = queue.push(format!("remove_{}", i).into_bytes())
                .expect("Push failed");
            let _ = queue.remove_by_guid(&guid);
        }

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 150);
        assert_eq!(stats.total_removed, 50);
    }

    #[test]
    fn test_queue_workflow_parallel() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));

        // Multiple producer threads
        let mut handles = vec![];

        for producer_id in 0..3 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let data = format!("producer_{}_item_{}", producer_id, i).into_bytes();
                    queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        // Consumer thread that removes some items
        let queue_clone = Arc::clone(&queue);
        let consumer_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            for i in 0..150 {
                let guid = queue_clone.push(format!("remove_{}", i).into_bytes())
                    .expect("Push failed");
                let _ = queue_clone.remove_by_guid(&guid);
            }
        });

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Producer thread panicked");
        }
        consumer_handle.join().expect("Consumer thread panicked");

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 450); // 300 + 150 removes
        assert_eq!(stats.total_removed, 150);
    }

    // ============== Persistence + Mode Switching ==============

    #[test]
    fn test_persistence_with_mode_switching() {
        let path = "test_persistence_mode_switch";
        cleanup_test_dir(path);

        // Phase 1: Sequential mode
        {
            let queue = AsyncPersistenceQueue::new(0, 128, path, false)
                .expect("Failed to create queue");

            for i in 0..50 {
                queue.push(format!("seq_{}", i).into_bytes())
                    .expect("Push failed");
            }
        }

        // Phase 2: Switch to parallel, add more
        {
            let queue = AsyncPersistenceQueue::new(1, 128, path, true)
                .expect("Failed to create queue");

            assert_eq!(queue.total_pushed(), 50);

            for i in 0..50 {
                queue.push(format!("par_{}", i).into_bytes())
                    .expect("Push failed");
            }
        }

        // Phase 3: Verify persistence
        {
            let queue = AsyncPersistenceQueue::new(1, 128, path, true)
                .expect("Failed to create queue");

            assert_eq!(queue.total_pushed(), 100);
        }

        cleanup_test_dir(path);
    }

    // ============== Priority Queue Ordering ==============

    #[test]
    fn test_priority_ordering_single_thread() {
        let path = "test_priority_ordering";
        cleanup_test_dir(path);

        let queue = AsyncPriorityQueue::new(0, path)
            .expect("Failed to create queue");

        // Add items with different priorities
        queue.push(b"item_5_low".to_vec(), Priority::LOW)
            .expect("Push failed");
        queue.push(b"item_2_critical".to_vec(), Priority::CRITICAL)
            .expect("Push failed");
        queue.push(b"item_3_medium".to_vec(), Priority::MEDIUM)
            .expect("Push failed");
        queue.push(b"item_1_critical".to_vec(), Priority::CRITICAL)
            .expect("Push failed");
        queue.push(b"item_4_high".to_vec(), Priority::HIGH)
            .expect("Push failed");

        assert_eq!(queue.total_pushed(), 5);

        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 5);

        // Priority distribution should be:
        // 2 CRITICAL, 1 HIGH, 1 MEDIUM, 1 LOW
        assert_eq!(*stats.by_priority_level.get(&20).unwrap_or(&0), 2);
        assert_eq!(*stats.by_priority_level.get(&10).unwrap_or(&0), 1);
        assert_eq!(*stats.by_priority_level.get(&5).unwrap_or(&0), 1);
        assert_eq!(*stats.by_priority_level.get(&1).unwrap_or(&0), 1);

        cleanup_test_dir(path);
    }

    // ============== Models + Queue Integration ==============

    #[test]
    fn test_envelope_in_queue() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");

        // Create envelope
        let mut envelope = Envelope::new(
            "producer@system".to_string(),
            vec!["user1@app.com".to_string(), "user2@app.com".to_string()],
            "Test Notification".to_string(),
            b"Important message for users".to_vec(),
        );

        envelope = envelope.with_priority(150).with_ttl(3600);
        envelope.require_confirmation = true;

        // Serialize and push to queue
        let serialized = serde_json::to_vec(&envelope)
            .expect("Serialization failed");

        let guid = queue.push(serialized).expect("Push failed");

        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
    }

    #[test]
    fn test_envelope_batch_in_queue() {
        let queue = AsyncQueue::new(1, 256).expect("Failed to create queue");

        let mut envelopes = vec![];

        for i in 0..10 {
            let env = Envelope::new(
                "sender@system".to_string(),
                vec![format!("recipient_{}", i)],
                format!("Message {}", i),
                format!("Content for message {}", i).into_bytes(),
            );
            envelopes.push(env);
        }

        // Serialize all envelopes
        let serialized_items: Vec<Vec<u8>> = envelopes
            .iter()
            .map(|env| serde_json::to_vec(env).expect("Serialization failed"))
            .collect();

        let guids = queue.push_batch(serialized_items)
            .expect("Batch push failed");

        assert_eq!(guids.len(), 10);
        assert_eq!(queue.total_pushed(), 10);
    }

    // ============== Stress Test - Multiple Operations ==============

    #[test]
    fn test_stress_test_high_throughput() {
        let queue = Arc::new(AsyncQueue::new(1, 1024)
            .expect("Failed to create queue"));

        let item_count = 1000;
        let thread_count = 8;

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

        assert_eq!(queue.total_pushed(), item_count as u64);
    }

    #[test]
    fn test_stress_test_mixed_operations() {
        let path = "test_stress_mixed";
        cleanup_test_dir(path);

        let queue = Arc::new(AsyncPersistenceQueue::new(1, 512, path, false)
            .expect("Failed to create queue"));

        let queue_clone1 = Arc::clone(&queue);
        let push_handle = thread::spawn(move || {
            for i in 0..500 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone1.push(data).expect("Push failed");
                if i % 100 == 0 {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });

        thread::sleep(Duration::from_millis(50));

        let queue_clone2 = Arc::clone(&queue);
        let remove_handle = thread::spawn(move || {
            for i in 0..200 {
                let guid = format!("item_{}", i);
                let _result = queue_clone2.remove_by_guid(&guid);
                if i % 50 == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        });

        push_handle.join().expect("Push thread panicked");
        remove_handle.join().expect("Remove thread panicked");

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 500);
        assert!(stats.total_removed <= stats.total_pushed);

        cleanup_test_dir(path);
    }

    // ============== Mode Switching Performance ==============

    #[test]
    fn test_sequential_to_parallel_transition() {
        let queue = AsyncQueue::new(0, 128)
            .expect("Failed to create queue");

        // Sequential phase
        assert_eq!(queue.get_mode(), 0);
        for i in 0..100 {
            queue.push(format!("seq_{}", i).into_bytes())
                .expect("Push failed");
        }

        let sequential_count = queue.total_pushed();

        // Switch to parallel
        queue.set_mode(1).expect("Set mode failed");
        assert_eq!(queue.get_mode(), 1);

        for i in 0..100 {
            queue.push(format!("par_{}", i).into_bytes())
                .expect("Push failed");
        }

        let total_count = queue.total_pushed();

        assert_eq!(sequential_count, 100);
        assert_eq!(total_count, 200);
    }

    // ============== Data Consistency ==============

    #[test]
    fn test_data_consistency_concurrent_access() {
        let queue = Arc::new(AsyncQueue::new(1, 256)
            .expect("Failed to create queue"));

        let mut handlers = vec![];

        // Thread 1: Push items
        let q1 = Arc::clone(&queue);
        let h1 = thread::spawn(move || {
            for i in 0..100 {
                q1.push(format!("push_{}", i).into_bytes())
                    .expect("Push failed");
            }
        });
        handlers.push(h1);

        // Thread 2: Remove items
        let q2 = Arc::clone(&queue);
        let h2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            for i in 0..50 {
                let guid = format!("push_{}", i);
                let _result = q2.remove_by_guid(&guid);
            }
        });
        handlers.push(h2);

        // Thread 3: Monitor stats
        let q3 = Arc::clone(&queue);
        let h3 = thread::spawn(move || {
            for _ in 0..10 {
                let _stats = q3.get_stats();
                thread::sleep(Duration::from_millis(50));
            }
        });
        handlers.push(h3);

        for h in handlers {
            h.join().expect("Thread panicked");
        }

        let final_stats = queue.get_stats();
        assert_eq!(final_stats.total_pushed, 100);
        assert!(final_stats.total_removed <= final_stats.total_pushed);
    }

    // ============== Error Recovery ==============

    #[test]
    fn test_recovery_after_queue_recreation() {
        let path = "test_recovery";
        cleanup_test_dir(path);

        // Initial population
        {
            let queue = AsyncPersistenceQueue::new(0, 128, path, false)
                .expect("Failed to create queue");
            
            for i in 0..100 {
                queue.push(format!("item_{}", i).into_bytes())
                    .expect("Push failed");
            }

            // Remove some
            for i in 0..20 {
                let guid = format!("item_{}", i);
                let _ = queue.remove_by_guid(&guid);
            }
        }

        // Recreate and verify
        {
            let queue = AsyncPersistenceQueue::new(0, 128, path, true)
                .expect("Failed to create queue");

            let stats = queue.get_stats();
            assert_eq!(stats.total_pushed, 100);
            assert!(stats.total_removed <= stats.total_pushed);
        }

        cleanup_test_dir(path);
    }
}
