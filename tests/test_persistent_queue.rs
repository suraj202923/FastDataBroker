/// Comprehensive unit tests for AsyncPersistenceQueue
/// Tests persistent storage, recovery, and durability guarantees

#[cfg(test)]
mod async_persistence_queue_tests {
    use fastdatabroker::persistent_queue::AsyncPersistenceQueue;
    use std::sync::Arc;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn cleanup_test_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    fn setup_test_dir(test_name: &str) -> String {
        let path = format!("./test_persistence_queue_{}", test_name);
        cleanup_test_dir(&path);
        path
    }

    // ============== Basic Persistence Operations ==============

    #[test]
    fn test_persistence_queue_creation() {
        let path = setup_test_dir("creation");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create persistence queue");
        
        assert_eq!(queue.total_pushed(), 0);
        assert_eq!(queue.total_processed(), 0);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_push_with_persistence() {
        let path = setup_test_dir("push_persist");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        let guid = queue.push(b"persistent data".to_vec())
            .expect("Push failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
        
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_persistence_directory_creation() {
        let path = setup_test_dir("dir_creation");
        let _queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        assert!(Path::new(&path).exists());
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_multiple_push_with_persistence() {
        let path = setup_test_dir("multiple_push");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        for i in 0..100 {
            let data = format!("persistent_item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 100);
        cleanup_test_dir(&path);
    }

    // ============== Data Durability Tests ==============

    #[test]
    fn test_persistence_survives_queue_recreate() {
        let path = setup_test_dir("durability");
        
        // First queue instance - push data
        {
            let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
                .expect("Failed to create queue");
            queue.push(b"data1".to_vec()).expect("Push 1 failed");
            queue.push(b"data2".to_vec()).expect("Push 2 failed");
            queue.push(b"data3".to_vec()).expect("Push 3 failed");
        } // queue dropped
        
        // Second queue instance - should see the data persisted
        let queue2 = AsyncPersistenceQueue::new(0, 128, &path, true)
            .expect("Failed to create queue 2");
        
        assert_eq!(queue2.total_pushed(), 3);
        
        cleanup_test_dir(&path);
    }

    // ============== Batch Operations with Persistence ==============

    #[test]
    fn test_batch_push_persistence() {
        let path = setup_test_dir("batch_persist");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        let items = vec![
            b"batch_item_1".to_vec(),
            b"batch_item_2".to_vec(),
            b"batch_item_3".to_vec(),
            b"batch_item_4".to_vec(),
        ];
        
        let guids = queue.push_batch(items).expect("Batch push failed");
        
        assert_eq!(guids.len(), 4);
        assert_eq!(queue.total_pushed(), 4);
        
        for guid in guids {
            assert!(queue.is_guid_active(&guid));
        }
        
        cleanup_test_dir(&path);
    }

    // ============== GUID Management with Persistence ==============

    #[test]
    fn test_remove_guid_persistence() {
        let path = setup_test_dir("remove_persist");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        let guid1 = queue.push(b"data1".to_vec()).expect("Push 1 failed");
        let guid2 = queue.push(b"data2".to_vec()).expect("Push 2 failed");
        let guid3 = queue.push(b"data3".to_vec()).expect("Push 3 failed");
        
        queue.remove_by_guid(&guid1).expect("Remove failed");
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_removed, 1);
        assert!(!queue.is_guid_active(&guid1));
        assert!(queue.is_guid_active(&guid2));
        assert!(queue.is_guid_active(&guid3));
        
        cleanup_test_dir(&path);
    }

    // ============== Execution Mode with Persistence ==============

    #[test]
    fn test_sequential_mode_persistence() {
        let path = setup_test_dir("seq_mode");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        assert_eq!(queue.get_mode(), 0);
        
        for i in 0..50 {
            let data = format!("seq_item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 50);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_parallel_mode_persistence() {
        let path = setup_test_dir("par_mode");
        let queue = AsyncPersistenceQueue::new(1, 128, &path, false)
            .expect("Failed to create queue");
        
        assert_eq!(queue.get_mode(), 1);
        
        for i in 0..50 {
            let data = format!("par_item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        assert_eq!(queue.total_pushed(), 50);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_mode_switch_persistence() {
        let path = setup_test_dir("mode_switch");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        assert_eq!(queue.get_mode(), 0);
        
        let guid1 = queue.push(b"seq_data".to_vec()).expect("Push 1 failed");
        
        queue.set_mode(1).expect("Set mode failed");
        assert_eq!(queue.get_mode(), 1);
        
        let guid2 = queue.push(b"par_data".to_vec()).expect("Push 2 failed");
        
        assert_eq!(queue.total_pushed(), 2);
        assert!(queue.is_guid_active(&guid1));
        assert!(queue.is_guid_active(&guid2));
        
        cleanup_test_dir(&path);
    }

    // ============== Large Data Persistence ==============

    #[test]
    fn test_large_data_persistence() {
        let path = setup_test_dir("large_data");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        let large_data = vec![0xFFu8; 5 * 1024 * 1024]; // 5MB
        let guid = queue.push(large_data).expect("Push large data failed");
        
        assert_eq!(queue.total_pushed(), 1);
        assert!(queue.is_guid_active(&guid));
        
        cleanup_test_dir(&path);
    }

    // ============== Storage Path Handling ==============

    #[test]
    fn test_nested_storage_path() {
        let path = setup_test_dir("nested/storage/path/queue");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        queue.push(b"test".to_vec()).expect("Push failed");
        assert_eq!(queue.total_pushed(), 1);
        
        cleanup_test_dir("./test_persistence_queue_nested");
    }

    // ============== Concurrent Persistence Tests ==============

    #[test]
    fn test_concurrent_persistent_push() {
        let path = setup_test_dir("concurrent_persist");
        let queue = Arc::new(AsyncPersistenceQueue::new(1, 256, &path, false)
            .expect("Failed to create queue"));
        
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let queue_clone = Arc::clone(&queue);
            let handle = std::thread::spawn(move || {
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
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_concurrent_push_and_remove_persist() {
        let path = setup_test_dir("concurrent_push_remove");
        let queue = Arc::new(AsyncPersistenceQueue::new(1, 256, &path, false)
            .expect("Failed to create queue"));
        
        let queue_clone1 = Arc::clone(&queue);
        let push_handle = std::thread::spawn(move || {
            for i in 0..500 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone1.push(data).expect("Push failed");
            }
        });
        
        let queue_clone2 = Arc::clone(&queue);
        let remove_handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            for i in 0..100 {
                let guid = format!("item_{}", i);
                let _result = queue_clone2.remove_by_guid(&guid);
            }
        });
        
        push_handle.join().expect("Push thread panicked");
        remove_handle.join().expect("Remove thread panicked");
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 500);
        assert!(stats.total_removed > 0);
        
        cleanup_test_dir(&path);
    }

    // ============== Statistics with Persistence ==============

    #[test]
    fn test_stats_with_persistence() {
        let path = setup_test_dir("stats_persist");
        let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        for i in 0..100 {
            let data = format!("item_{}", i).into_bytes();
            let _guid = queue.push(data).expect("Push failed");
        }
        
        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 100);
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.total_errors, 0);
        
        cleanup_test_dir(&path);
    }

    // ============== Auto Restore Tests ==============

    #[test]
    fn test_auto_restore_empty_queue() {
        let path = setup_test_dir("auto_restore_empty");
        
        // Create empty queue
        let _queue = AsyncPersistenceQueue::new(0, 128, &path, false)
            .expect("Failed to create queue");
        
        // Recreate with auto_restore
        let queue2 = AsyncPersistenceQueue::new(0, 128, &path, true)
            .expect("Failed to create queue 2");
        
        assert_eq!(queue2.total_pushed(), 0);
        cleanup_test_dir(&path);
    }

    #[test]
    fn test_auto_restore_with_data() {
        let path = setup_test_dir("auto_restore_data");
        
        // Create and populate queue
        {
            let queue = AsyncPersistenceQueue::new(0, 128, &path, false)
                .expect("Failed to create queue");
            
            for i in 0..50 {
                let data = format!("persisted_{}", i).into_bytes();
                let _guid = queue.push(data).expect("Push failed");
            }
        } // dropped
        
        // Recreate with auto_restore
        let queue2 = AsyncPersistenceQueue::new(0, 128, &path, true)
            .expect("Failed to create queue 2");
        
        assert_eq!(queue2.total_pushed(), 50);
        cleanup_test_dir(&path);
    }
}
