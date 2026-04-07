/// Comprehensive unit tests for AsyncQueue
/// Tests basic queue operations, concurrency, and execution modes

#[cfg(test)]
mod async_queue_tests {
    use fastdatabroker::queue::{AsyncQueue, ExecutionMode, QueueItem, ProcessedResult};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    // ============== Basic Queue Operations ==============

    #[test]
    fn test_queue_creation() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        assert_eq!(queue.total_pushed(), 0);
        assert_eq!(queue.total_processed(), 0);
        assert_eq!(queue.get_mode(), 0);
    }

    #[test]
    fn test_queue_creation_parallel_mode() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        assert_eq!(queue.get_mode(), 1);
    }

    #[test]
    fn test_push_single_item() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let data = b"test data".to_vec();
        
        let guid = queue.push(data).expect("Push failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(!guid.is_empty());
        assert!(queue.is_guid_active(&guid));
    }

    #[test]
    fn test_push_multiple_items() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        for i in 0..100 {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 100);
    }

    #[test]
    fn test_push_batch() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let items = vec![
            b"item1".to_vec(),
            b"item2".to_vec(),
            b"item3".to_vec(),
            b"item4".to_vec(),
            b"item5".to_vec(),
        ];
        
        let guids = queue.push_batch(items).expect("Batch push failed");
        
        assert_eq!(guids.len(), 5);
        assert_eq!(queue.total_pushed(), 5);
        
        for guid in guids {
            assert!(queue.is_guid_active(&guid));
        }
    }

    #[test]
    fn test_push_empty_data() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let data = vec![];
        
        let guid = queue.push(data).expect("Push failed");
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
    }

    // ============== GUID Management ==============

    #[test]
    fn test_remove_by_guid() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        let guid = queue.push(b"test".to_vec()).expect("Push failed");
        
        assert!(queue.is_guid_active(&guid));
        assert!(queue.remove_by_guid(&guid));
        assert!(!queue.is_guid_active(&guid));
    }

    #[test]
    fn test_remove_nonexistent_guid() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        let result = queue.remove_by_guid("nonexistent");
        assert!(!result);
    }

    #[test]
    fn test_guid_uniqueness() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let guid1 = queue.push(b"data1".to_vec()).expect("Push 1 failed");
        let guid2 = queue.push(b"data2".to_vec()).expect("Push 2 failed");
        let guid3 = queue.push(b"data3".to_vec()).expect("Push 3 failed");
        
        assert_ne!(guid1, guid2);
        assert_ne!(guid2, guid3);
        assert_ne!(guid1, guid3);
    }

    // ============== Execution Mode Tests ==============

    #[test]
    fn test_set_execution_mode() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        assert_eq!(queue.get_mode(), 0);
        
        queue.set_mode(1).expect("set_mode failed");
        assert_eq!(queue.get_mode(), 1);
        
        queue.set_mode(0).expect("set_mode failed");
        assert_eq!(queue.get_mode(), 0);
    }

    // ============== Statistics Tests ==============

    #[test]
    fn test_stats_tracking() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        // Push items
        for i in 0..10 {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 10);
        assert_eq!(stats.total_processed, 0); // Not processed yet
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.total_removed, 0);
    }

    #[test]
    fn test_stats_after_removal() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        let guid1 = queue.push(b"data1".to_vec()).expect("Push 1 failed");
        let guid2 = queue.push(b"data2".to_vec()).expect("Push 2 failed");
        let guid3 = queue.push(b"data3".to_vec()).expect("Push 3 failed");
        
        assert!(queue.remove_by_guid(&guid1));
        assert!(queue.remove_by_guid(&guid2));
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_removed, 2);
        assert_eq!(stats.total_pushed, 3);
    }

    // ============== Large Data Tests ==============

    #[test]
    fn test_push_large_data() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        // 10MB of data
        let large_data = vec![0u8; 10 * 1024 * 1024];
        let guid = queue.push(large_data).expect("Push large data failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
    }

    #[test]
    fn test_push_many_items() {
        let queue = Arc::new(AsyncQueue::new(1, 1024).expect("Failed to create queue"));
        
        let item_count = 10000;
        
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for i in 0..item_count {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone.push(data).expect("Push failed");
            }
        });
        
        handle.join().expect("Thread panicked");
        assert_eq!(queue.total_pushed(), item_count as u64);
    }

    // ============== Concurrency Tests ==============

    #[test]
    fn test_concurrent_push() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let data = format!("thread_{}_item_{}", thread_id, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        assert_eq!(queue.total_pushed(), 1000);
    }

    #[test]
    fn test_concurrent_push_and_remove() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));
        let remove_count = Arc::new(AtomicUsize::new(0));
        
        // Push thread
        let queue_clone1 = Arc::clone(&queue);
        let push_handle = thread::spawn(move || {
            for i in 0..500 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone1.push(data).expect("Push failed");
            }
        });
        
        // Remove thread
        let queue_clone2 = Arc::clone(&queue);
        let remove_count_clone = Arc::clone(&remove_count);
        let remove_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50)); // Let some items queue up
            for i in 0..100 {
                let guid = format!("item_{}", i);
                if queue_clone2.remove_by_guid(&guid) {
                    remove_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        
        push_handle.join().expect("Push thread panicked");
        remove_handle.join().expect("Remove thread panicked");
        
        assert_eq!(queue.total_pushed(), 500);
    }

    // ============== Edge Cases ==============

    #[test]
    fn test_queue_stats_consistency() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        let mut guids = vec![];
        for i in 0..50 {
            let guid = queue.push(format!("item_{}", i).into_bytes())
                .expect("Push failed");
            guids.push(guid);
        }
        
        // Remove half
        for i in 0..25 {
            assert!(queue.remove_by_guid(&guids[i]));
        }
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 50);
        assert_eq!(stats.total_removed, 25);
    }

    #[test]
    fn test_mode_conversion() {
        assert_eq!(ExecutionMode::Sequential.to_int(), 0);
        assert_eq!(ExecutionMode::Parallel.to_int(), 1);
        assert_eq!(ExecutionMode::from_int(0), ExecutionMode::Sequential);
        assert_eq!(ExecutionMode::from_int(1), ExecutionMode::Parallel);
        assert_eq!(ExecutionMode::from_int(2), ExecutionMode::Parallel); // Any non-zero
    }

    #[test]
    fn test_push_after_mode_change() {
        let queue = AsyncQueue::new(0, 128).expect("Failed to create queue");
        
        let guid1 = queue.push(b"data1".to_vec()).expect("Push 1 failed");
        queue.set_mode(1).expect("Set mode failed");
        let guid2 = queue.push(b"data2".to_vec()).expect("Push 2 failed");
        
        assert_eq!(queue.total_pushed(), 2);
        assert!(queue.is_guid_active(&guid1));
        assert!(queue.is_guid_active(&guid2));
    }

    // ============== Batch Operations Stress Test ==============

    #[test]
    fn test_multiple_batch_pushes() {
        let queue = AsyncQueue::new(1, 256).expect("Failed to create queue");
        
        for batch_num in 0..10 {
            let items = (0..100)
                .map(|i| format!("batch_{}_item_{}", batch_num, i).into_bytes())
                .collect();
            
            let _guids = queue.push_batch(items).expect("Batch push failed");
        }
        
        assert_eq!(queue.total_pushed(), 1000);
    }

    #[test]
    fn test_mixed_push_and_batch() {
        let queue = AsyncQueue::new(1, 128).expect("Failed to create queue");
        
        // Single push
        let guid1 = queue.push(b"single1".to_vec()).expect("Push failed");
        
        // Batch push
        let batch_items = vec![
            b"batch1".to_vec(),
            b"batch2".to_vec(),
            b"batch3".to_vec(),
        ];
        let batch_guids = queue.push_batch(batch_items).expect("Batch push failed");
        
        // Single push again
        let guid2 = queue.push(b"single2".to_vec()).expect("Push failed");
        
        assert_eq!(queue.total_pushed(), 5);
        assert!(queue.is_guid_active(&guid1));
        assert!(queue.is_guid_active(&guid2));
        for guid in batch_guids {
            assert!(queue.is_guid_active(&guid));
        }
    }
}
