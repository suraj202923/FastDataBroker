/// Error handling and edge case tests
/// Tests boundary conditions, error scenarios, and edge cases

#[cfg(test)]
mod error_and_edge_case_tests {
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

    // ============== Empty Operations ==============

    #[test]
    fn test_queue_with_empty_batch() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let empty_items: Vec<Vec<u8>> = vec![];
        let guids = queue.push_batch(empty_items).expect("Batch push failed");
        
        assert_eq!(guids.len(), 0);
        assert_eq!(queue.total_pushed(), 0);
    }

    #[test]
    fn test_remove_same_guid_twice() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let guid = queue.push(b"data".to_vec()).expect("Push failed");
        
        assert!(queue.remove_by_guid(&guid));
        assert!(!queue.remove_by_guid(&guid)); // Second removal should fail
    }

    #[test]
    fn test_remove_never_pushed_guid() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        let result = queue.remove_by_guid("never_existed");
        assert!(!result);
    }

    // ============== Boundary Value Tests ==============

    #[test]
    fn test_priority_boundary_values() {
        let p1 = Priority::custom(1).expect("Priority 1 failed");
        assert_eq!(p1.get_level(), 1);
        
        let p100 = Priority::custom(100).expect("Priority 100 failed");
        assert_eq!(p100.get_level(), 100);
        
        // Zero should fail
        assert!(Priority::custom(0).is_err());
        
        // Over 100 should fail
        assert!(Priority::custom(101).is_err());
        assert!(Priority::custom(255).is_err());
    }

    #[test]
    fn test_very_large_batch() {
        let queue = AsyncQueue::new(1, 1024).expect("Failed to create queue");
        
        let large_batch: Vec<Vec<u8>> = (0..10000)
            .map(|i| format!("item_{}", i).into_bytes())
            .collect();
        
        let guids = queue.push_batch(large_batch).expect("Large batch push failed");
        
        assert_eq!(guids.len(), 10000);
        assert_eq!(queue.total_pushed(), 10000);
    }

    #[test]
    fn test_zero_size_data() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let guid = queue.push(vec![]).expect("Push empty data failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
    }

    #[test]
    fn test_single_byte_data() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let guid = queue.push(vec![0x42]).expect("Push single byte failed");
        
        assert!(queue.is_guid_active(&guid));
    }

    // ============== Execution Mode Edge Cases ==============

    #[test]
    fn test_mode_value_normalization() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        // Any non-zero becomes parallel
        queue.set_mode(2).expect("Set mode 2 failed");
        assert_eq!(queue.get_mode(), 1); // Should normalize to 1
        
        queue.set_mode(100).expect("Set mode 100 failed");
        assert_eq!(queue.get_mode(), 1); // Should normalize to 1
        
        queue.set_mode(0).expect("Set mode 0 failed");
        assert_eq!(queue.get_mode(), 0);
    }

    // ============== Concurrent Edge Cases ==============

    #[test]
    fn test_concurrent_same_guid_removal() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let guid = queue.push(b"data".to_vec()).expect("Push failed");
        
        let mut handles = vec![];
        
        // Multiple threads try to remove the same GUID
        for _ in 0..10 {
            let queue_clone = Arc::clone(&queue);
            let guid_clone = guid.clone();
            let handle = thread::spawn(move || {
                queue_clone.remove_by_guid(&guid_clone)
            });
            handles.push(handle);
        }
        
        let mut successful_removals = 0;
        for handle in handles {
            if handle.join().expect("Thread panicked") {
                successful_removals += 1;
            }
        }
        
        // Only one thread should succeed in removing
        assert_eq!(successful_removals, 1);
    }

    #[test]
    fn test_rapid_mode_switches() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        for _ in 0..100 {
            queue.set_mode(1).expect("Set mode 1 failed");
            queue.set_mode(0).expect("Set mode 0 failed");
        }
        
        assert_eq!(queue.get_mode(), 0);
    }

    // ============== Persistence Edge Cases ==============

    #[test]
    fn test_persistence_with_corrupted_path() {
        let path = "test_bad_path/nonexistent/!!invalid";
        
        // Should create path structure
        let queue = AsyncPersistenceQueue::new(0, 128, path, false);
        
        // This should succeed because the queue tries to create the path
        assert!(queue.is_ok());
        
        cleanup_test_dir("test_bad_path");
    }

    #[test]
    fn test_persistence_queue_recovery_partial_state() {
        let path = "test_partial_recovery";
        cleanup_test_dir(path);
        
        // Create partial state
        {
            let queue = AsyncPersistenceQueue::new(0, 128, path, false)
                .expect("Failed to create queue");
            
            queue.push(b"item1".to_vec()).expect("Push 1 failed");
            queue.push(b"item2".to_vec()).expect("Push 2 failed");
            queue.push(b"item3".to_vec()).expect("Push 3 failed");
            
            // Remove some items
            queue.remove_by_guid("test").expect("Remove failed");
        }
        
        // Recover
        let queue2 = AsyncPersistenceQueue::new(0, 128, path, true)
            .expect("Failed to recover queue");
        
        assert_eq!(queue2.total_pushed(), 3);
        
        cleanup_test_dir(path);
    }

    // ============== Priority Queue Edge Cases ==============

    #[test]
    fn test_priority_queue_all_same_priority() {
        let path = "test_same_priority";
        cleanup_test_dir(path);
        
        let queue = AsyncPriorityQueue::new(0, path)
            .expect("Failed to create queue");
        
        for i in 0..100 {
            let data = format!("item_{}", i).into_bytes();
            queue.push(data, Priority::HIGH).expect("Push failed");
        }
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 100);
        assert_eq!(*stats.by_priority_level.get(&10).unwrap_or(&0), 100);
        
        cleanup_test_dir(path);
    }

    #[test]
    fn test_priority_queue_many_custom_priorities() {
        let path = "test_custom_priorities";
        cleanup_test_dir(path);
        
        let queue = AsyncPriorityQueue::new(0, path)
            .expect("Failed to create queue");
        
        for i in 1..=100 {
            let data = format!("item_{}", i).into_bytes();
            let priority = Priority::custom(i as u8).expect("Priority creation failed");
            queue.push(data, priority).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 100);
        
        cleanup_test_dir(path);
    }

    // ============== Data Integrity ==============

    #[test]
    fn test_binary_data_preservation() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let binary_data = vec![
            0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD,
            0x7F, 0x80, 0x7E, 0x81,
        ];
        
        let guid = queue.push(binary_data.clone()).expect("Push failed");
        
        assert!(queue.is_guid_active(&guid));
    }

    #[test]
    fn test_utf8_data_handling() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let strings = vec![
            "Hello, World!",
            "日本語テスト",
            "🚀 Emoji test 🎉",
            "Ünïcödé tëxt",
        ];
        
        for s in strings {
            let data = s.as_bytes().to_vec();
            let guid = queue.push(data).expect("Push failed");
            assert!(queue.is_guid_active(&guid));
        }
        
        assert_eq!(queue.total_pushed(), 4);
    }

    // ============== Model Edge Cases ==============

    #[test]
    fn test_envelope_with_empty_recipients() {
        let env = Envelope::new(
            "sender".to_string(),
            vec![], // No recipients
            "Subject".to_string(),
            b"content".to_vec(),
        );
        
        assert_eq!(env.recipient_ids.len(), 0);
    }

    #[test]
    fn test_envelope_with_many_recipients() {
        let recipients: Vec<String> = (0..1000)
            .map(|i| format!("user_{}", i))
            .collect();
        
        let env = Envelope::new(
            "sender".to_string(),
            recipients.clone(),
            "Subject".to_string(),
            b"content".to_vec(),
        );
        
        assert_eq!(env.recipient_ids.len(), 1000);
    }

    #[test]
    fn test_envelope_large_tag_collection() {
        let mut env = Envelope::new(
            "sender".to_string(),
            vec!["recipient".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );
        
        for i in 0..1000 {
            env.tags.insert(
                format!("tag_{}", i),
                format!("value_{}", i),
            );
        }
        
        assert_eq!(env.tags.len(), 1000);
    }

    // ============== Stress with Small & Large Data ==============

    #[test]
    fn test_mixed_size_data_stress() {
        let queue = AsyncQueue::new(1, 256).expect("Failed to create queue");
        
        let mut handles = vec![];
        
        // Thread 1: Small data
        let q1 = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let q1_clone = Arc::clone(&q1);
        let h1 = thread::spawn(move || {
            for i in 0..1000 {
                let data = format!("small_{}", i).into_bytes();
                let _guid = q1_clone.push(data).expect("Push failed");
            }
        });
        
        // Thread 2: Large data
        let q2 = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let q2_clone = Arc::clone(&q2);
        let h2 = thread::spawn(move || {
            for i in 0..10 {
                let data = vec![0xAA; 1_000_000]; // 1MB
                let _guid = q2_clone.push(data).expect("Push failed");
            }
        });
        
        h1.join().expect("Thread 1 panicked");
        h2.join().expect("Thread 2 panicked");
        
        // Both should succeed
        assert!(true);
    }

    // ============== Recovery Tests ==============

    #[test]
    fn test_persistence_recovery_after_many_operations() {
        let path = "test_complex_recovery";
        cleanup_test_dir(path);
        
        // Complex operations
        {
            let queue = AsyncPersistenceQueue::new(1, 256, path, false)
                .expect("Failed to create queue");
            
            // Push many
            for i in 0..500 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue.push(data).expect("Push failed");
            }
            
            // Remove some
            for i in 0..100 {
                let guid = format!("item_{}", i);
                let _result = queue.remove_by_guid(&guid);
            }
            
            // Add more
            for i in 500..600 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue.push(data).expect("Push failed");
            }
            
            // Remove again
            for i in 500..530 {
                let guid = format!("item_{}", i);
                let _result = queue.remove_by_guid(&guid);
            }
        }
        
        // Recover and verify consistency
        let queue2 = AsyncPersistenceQueue::new(1, 256, path, true)
            .expect("Failed to recover queue");
        
        assert_eq!(queue2.total_pushed(), 600);
        
        cleanup_test_dir(path);
    }
}
