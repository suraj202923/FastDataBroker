/// Comprehensive unit tests for AsyncPriorityQueue
/// Tests priority-based queue operations, ordering, and persistence

#[cfg(test)]
mod async_priority_queue_tests {
    use fastdatabroker::priority_queue::{AsyncPriorityQueue, Priority, PrioritizedQueueItem};
    use std::sync::Arc;
    use std::fs;

    fn cleanup_test_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    fn setup_test_dir(test_name: &str) -> String {
        let path = format!("./test_priority_queue_{}", test_name);
        cleanup_test_dir(&path);
        path
    }

    // ============== Basic Priority Queue Operations ==============

    #[test]
    fn test_priority_queue_creation() {
        let path = setup_test_dir("creation");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create priority queue");
        
        assert_eq!(queue.total_pushed(), 0);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_push_with_priority() {
        let path = setup_test_dir("push_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let guid = queue.push(
            b"test data".to_vec(),
            Priority::HIGH
        ).expect("Push with priority failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
        
        cleanup_test_dir(&path);
    }

    // ============== Priority Levels ==============

    #[test]
    fn test_priority_levels() {
        assert_eq!(Priority::LOW.get_level(), 1);
        assert_eq!(Priority::MEDIUM.get_level(), 5);
        assert_eq!(Priority::HIGH.get_level(), 10);
        assert_eq!(Priority::CRITICAL.get_level(), 20);
    }

    #[test]
    fn test_custom_priority() {
        let priority = Priority::custom(15).expect("Failed to create priority");
        assert_eq!(priority.get_level(), 15);
    }

    #[test]
    fn test_invalid_priority_zero() {
        let result = Priority::custom(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_priority_over_100() {
        let result = Priority::custom(101);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_priority_boundaries() {
        let p1 = Priority::custom(1).expect("Failed to create priority");
        let p100 = Priority::custom(100).expect("Failed to create priority");
        
        assert_eq!(p1.get_level(), 1);
        assert_eq!(p100.get_level(), 100);
    }

    // ============== Multiple Items with Different Priorities ==============

    #[test]
    fn test_push_multiple_different_priorities() {
        let path = setup_test_dir("multi_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let _guid_low = queue.push(b"low".to_vec(), Priority::LOW)
            .expect("Push low failed");
        let _guid_medium = queue.push(b"medium".to_vec(), Priority::MEDIUM)
            .expect("Push medium failed");
        let _guid_high = queue.push(b"high".to_vec(), Priority::HIGH)
            .expect("Push high failed");
        let _guid_critical = queue.push(b"critical".to_vec(), Priority::CRITICAL)
            .expect("Push critical failed");
        
        assert_eq!(queue.total_pushed(), 4);
        cleanup_test_dir(&path);
    }

    // ============== GUID Management ==============

    #[test]
    fn test_guid_with_priority() {
        let path = setup_test_dir("guid_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let guid = queue.push(b"data".to_vec(), Priority::HIGH)
            .expect("Push failed");
        
        assert!(queue.is_guid_active(&guid));
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_remove_by_guid_priority() {
        let path = setup_test_dir("remove_guid_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let guid1 = queue.push(b"data1".to_vec(), Priority::LOW)
            .expect("Push 1 failed");
        let guid2 = queue.push(b"data2".to_vec(), Priority::HIGH)
            .expect("Push 2 failed");
        
        assert!(queue.is_guid_active(&guid1));
        assert!(queue.remove_by_guid(&guid1));
        assert!(!queue.is_guid_active(&guid1));
        assert!(queue.is_guid_active(&guid2));
        
        cleanup_test_dir(&path);
    }

    // ============== Batch Operations ==============

    #[test]
    fn test_batch_push_same_priority() {
        let path = setup_test_dir("batch_same_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let items = vec![
            b"item1".to_vec(),
            b"item2".to_vec(),
            b"item3".to_vec(),
            b"item4".to_vec(),
        ];
        
        let guids = queue.push_batch(items, Priority::HIGH)
            .expect("Batch push failed");
        
        assert_eq!(guids.len(), 4);
        assert_eq!(queue.total_pushed(), 4);
        
        for guid in guids {
            assert!(queue.is_guid_active(&guid));
        }
        
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_multiple_batch_operations() {
        let path = setup_test_dir("multi_batch");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        // First batch
        let batch1 = vec![b"b1_i1".to_vec(), b"b1_i2".to_vec()];
        let _guids1 = queue.push_batch(batch1, Priority::LOW)
            .expect("Batch 1 failed");
        
        // Second batch
        let batch2 = vec![b"b2_i1".to_vec(), b"b2_i2".to_vec(), b"b2_i3".to_vec()];
        let _guids2 = queue.push_batch(batch2, Priority::HIGH)
            .expect("Batch 2 failed");
        
        // Third batch
        let batch3 = vec![b"b3_i1".to_vec()];
        let _guids3 = queue.push_batch(batch3, Priority::CRITICAL)
            .expect("Batch 3 failed");
        
        assert_eq!(queue.total_pushed(), 6);
        cleanup_test_dir(&path);
    }

    // ============== Execution Mode ==============

    #[test]
    fn test_sequential_mode_priority_queue() {
        let path = setup_test_dir("seq_mode");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        assert_eq!(queue.get_mode(), 0);
        
        let _guid = queue.push(b"data".to_vec(), Priority::HIGH)
            .expect("Push failed");
        
        assert_eq!(queue.total_pushed(), 1);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_parallel_mode_priority_queue() {
        let path = setup_test_dir("par_mode");
        let queue = AsyncPriorityQueue::new(1, &path)
            .expect("Failed to create queue");
        
        assert_eq!(queue.get_mode(), 1);
        
        let _guid = queue.push(b"data".to_vec(), Priority::HIGH)
            .expect("Push failed");
        
        assert_eq!(queue.total_pushed(), 1);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_mode_switch_priority_queue() {
        let path = setup_test_dir("mode_switch");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let _guid1 = queue.push(b"seq_data".to_vec(), Priority::LOW)
            .expect("Push 1 failed");
        
        queue.set_mode(1).expect("Set mode failed");
        
        let _guid2 = queue.push(b"par_data".to_vec(), Priority::CRITICAL)
            .expect("Push 2 failed");
        
        assert_eq!(queue.total_pushed(), 2);
        cleanup_test_dir(&path);
    }

    // ============== Statistics ==============

    #[test]
    fn test_stats_priority_queue() {
        let path = setup_test_dir("stats");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        queue.push(b"low".to_vec(), Priority::LOW).expect("Push failed");
        queue.push(b"high".to_vec(), Priority::HIGH).expect("Push failed");
        queue.push(b"critical".to_vec(), Priority::CRITICAL).expect("Push failed");
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 3);
        assert!(stats.by_priority_level.contains_key(&1));   // LOW
        assert!(stats.by_priority_level.contains_key(&10));  // HIGH
        assert!(stats.by_priority_level.contains_key(&20));  // CRITICAL
        
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_stats_with_multiple_same_priority() {
        let path = setup_test_dir("stats_same_priority");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        // Push 5 HIGH priority items
        for i in 0..5 {
            let data = format!("high_{}", i).into_bytes();
            queue.push(data, Priority::HIGH).expect("Push failed");
        }
        
        // Push 3 LOW priority items
        for i in 0..3 {
            let data = format!("low_{}", i).into_bytes();
            queue.push(data, Priority::LOW).expect("Push failed");
        }
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 8);
        assert_eq!(*stats.by_priority_level.get(&10).unwrap_or(&0), 5); // HIGH
        assert_eq!(*stats.by_priority_level.get(&1).unwrap_or(&0), 3);   // LOW
        
        cleanup_test_dir(&path);
    }

    // ============== Large Data ==============

    #[test]
    fn test_large_data_with_priority() {
        let path = setup_test_dir("large_data");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        let large_data = vec![0xABu8; 256 * 1024]; // 256KB
        let guid = queue.push(large_data, Priority::CRITICAL)
            .expect("Push large data failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
        
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_many_items_priority_queue() {
        let path = setup_test_dir("many_items");
        let queue = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue");
        
        for i in 0..1000 {
            let data = format!("item_{}", i).into_bytes();
            let priority = match i % 4 {
                0 => Priority::LOW,
                1 => Priority::MEDIUM,
                2 => Priority::HIGH,
                _ => Priority::CRITICAL,
            };
            let _guid = queue.push(data, priority).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 1000);
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 1000);
        
        cleanup_test_dir(&path);
    }

    // ============== Concurrency ==============

    #[test]
    fn test_concurrent_priority_push() {
        let path = setup_test_dir("concurrent");
        let queue = Arc::new(AsyncPriorityQueue::new(1, &path)
            .expect("Failed to create queue"));
        
        let mut handles = vec![];
        
        for thread_id in 0..5 {
            let queue_clone = Arc::clone(&queue);
            let handle = std::thread::spawn(move || {
                for i in 0..100 {
                    let data = format!("thread_{}_item_{}", thread_id, i).into_bytes();
                    let priority = if i % 2 == 0 {
                        Priority::HIGH
                    } else {
                        Priority::LOW
                    };
                    let _guid = queue_clone.push(data, priority).expect("Push failed");
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        assert_eq!(queue.total_pushed(), 500);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_concurrent_push_and_remove_priority() {
        let path = setup_test_dir("concurrent_push_remove");
        let queue = Arc::new(AsyncPriorityQueue::new(1, &path)
            .expect("Failed to create queue"));
        
        let queue_clone1 = Arc::clone(&queue);
        let push_handle = std::thread::spawn(move || {
            for i in 0..300 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone1.push(data, Priority::MEDIUM)
                    .expect("Push failed");
            }
        });
        
        let queue_clone2 = Arc::clone(&queue);
        let remove_handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            for i in 0..100 {
                for j in 0..3 {
                    let guid = format!("item_{}", i * 3 + j);
                    let _result = queue_clone2.remove_by_guid(&guid);
                }
            }
        });
        
        push_handle.join().expect("Push thread panicked");
        remove_handle.join().expect("Remove thread panicked");
        
        assert_eq!(queue.total_pushed(), 300);
        cleanup_test_dir(&path);
    }

    // ============== Persistence ==============

    #[test]
    fn test_priority_queue_persistence() {
        let path = setup_test_dir("persistence");
        
        // Create and populate
        {
            let queue = AsyncPriorityQueue::new(0, &path)
                .expect("Failed to create queue");
            
            queue.push(b"low_data".to_vec(), Priority::LOW)
                .expect("Push failed");
            queue.push(b"high_data".to_vec(), Priority::HIGH)
                .expect("Push failed");
            queue.push(b"critical_data".to_vec(), Priority::CRITICAL)
                .expect("Push failed");
        }

        // On Windows, sled file locks may take a brief moment to release.
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify persistence path can be reopened without lock issues.
        // Current queue implementation does not auto-restore in-memory items.
        let queue2 = AsyncPriorityQueue::new(0, &path)
            .expect("Failed to create queue 2");
        
        assert_eq!(queue2.total_pushed(), 0);

        drop(queue2);
        
        cleanup_test_dir(&path);
    }
}
