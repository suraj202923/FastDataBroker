/// Concurrency and thread safety tests
/// Tests concurrent access, race conditions, and synchronization

#[cfg(test)]
mod concurrency_tests {
    use fastdatabroker::queue::AsyncQueue;
    use fastdatabroker::persistent_queue::AsyncPersistenceQueue;
    use fastdatabroker::priority_queue::{AsyncPriorityQueue, Priority};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;
    use std::fs;

    fn cleanup_test_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    // ============== Basic Concurrent Access ==============

    #[test]
    fn test_concurrent_push_no_data_loss() {
        let queue = Arc::new(AsyncQueue::new(1, 512).expect("Failed to create queue"));
        let thread_count = 16;
        let items_per_thread = 1000;

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

        let expected_total = (thread_count * items_per_thread) as u64;
        assert_eq!(queue.total_pushed(), expected_total);
    }

    #[test]
    fn test_concurrent_guid_uniqueness() {
        let queue = Arc::new(AsyncQueue::new(1, 512).expect("Failed to create queue"));
        let guids = Arc::new(std::sync::Mutex::new(Vec::new()));

        let thread_count = 8;
        let items_per_thread = 100;

        let mut handles = vec![];

        for thread_id in 0..thread_count {
            let queue_clone = Arc::clone(&queue);
            let guids_clone = Arc::clone(&guids);

            let handle = thread::spawn(move || {
                for i in 0..items_per_thread {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let guid = queue_clone.push(data).expect("Push failed");
                    
                    let mut g = guids_clone.lock().unwrap();
                    g.push(guid);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let collected_guids = guids.lock().unwrap();
        
        // Check all GUIDs are unique
        let mut unique_guids = std::collections::HashSet::new();
        for guid in collected_guids.iter() {
            assert!(unique_guids.insert(guid.clone()), "Duplicate GUID found: {}", guid);
        }

        assert_eq!(collected_guids.len(), thread_count * items_per_thread);
    }

    // ============== Race Conditions ==============

    #[test]
    fn test_push_remove_race() {
        let queue = Arc::new(AsyncQueue::new(1, 512).expect("Failed to create queue"));
        let finished = Arc::new(AtomicBool::new(false));

        let queue_clone1 = Arc::clone(&queue);
        let finished_clone1 = Arc::clone(&finished);
        
        let push_handle = thread::spawn(move || {
            for i in 0..500 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_clone1.push(data).expect("Push failed");
                
                if i % 100 == 0 {
                    thread::yield_now();
                }
            }
            finished_clone1.store(true, Ordering::Release);
        });

        thread::sleep(Duration::from_millis(10));

        let queue_clone2 = Arc::clone(&queue);
        let remove_handle = thread::spawn(move || {
            let mut removed_count = 0;
            
            loop {
                for i in 0..250 {
                    let guid = format!("item_{}", i);
                    if queue_clone2.remove_by_guid(&guid) {
                        removed_count += 1;
                    }
                }
                
                if finished.load(Ordering::Acquire) {
                    break;
                }
                
                thread::sleep(Duration::from_millis(1));
            }
            
            removed_count
        });

        push_handle.join().expect("Push thread panicked");
        let removed = remove_handle.join().expect("Remove thread panicked");

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 500);
        assert!(removed > 0);
    }

    // ============== Atomic Counter Testing ==============

    #[test]
    fn test_stats_consistency_concurrent() {
        let queue = Arc::new(AsyncQueue::new(1, 512).expect("Failed to create queue"));

        let mut handles = vec![];

        // Multiple threads pushing
        for thread_id in 0..4 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..250 {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        // Monitor thread checking stats
        let queue_clone = Arc::clone(&queue);
        let stats_handle = thread::spawn(move || {
            for _ in 0..50 {
                let stats = queue_clone.get_stats();
                assert!(stats.total_pushed <= 1000);
                assert_eq!(stats.total_processed, 0);
                thread::sleep(Duration::from_millis(1));
            }
        });

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        stats_handle.join().expect("Stats thread panicked");

        let final_stats = queue.get_stats();
        assert_eq!(final_stats.total_pushed, 1000);
    }

    // ============== Persistence + Concurrent ==============

    #[test]
    fn test_concurrent_persistent_push() {
        let path = "test_concurrent_persist";
        cleanup_test_dir(path);

        let queue = Arc::new(AsyncPersistenceQueue::new(1, 512, path, false)
            .expect("Failed to create queue"));

        let thread_count = 8;
        let items_per_thread = 500;

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

        let expected = (thread_count * items_per_thread) as u64;
        assert_eq!(queue.total_pushed(), expected);

        cleanup_test_dir(path);
    }

    #[test]
    fn test_persistence_concurrent_push_remove() {
        let path = "test_persist_concurrent_ops";
        cleanup_test_dir(path);

        let queue = Arc::new(AsyncPersistenceQueue::new(1, 512, path, false)
            .expect("Failed to create queue"));

        let queue_push = Arc::clone(&queue);
        let push_handle = thread::spawn(move || {
            for i in 0..1000 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_push.push(data).expect("Push failed");
            }
        });

        thread::sleep(Duration::from_millis(100));

        let queue_remove = Arc::clone(&queue);
        let remove_handle = thread::spawn(move || {
            for i in 0..500 {
                let guid = format!("item_{}", i);
                let _result = queue_remove.remove_by_guid(&guid);
            }
        });

        push_handle.join().expect("Push thread panicked");
        remove_handle.join().expect("Remove thread panicked");

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 1000);
        assert!(stats.total_removed > 0);

        cleanup_test_dir(path);
    }

    // ============== Priority Queue Concurrent ==============

    #[test]
    fn test_priority_queue_concurrent_push_different_priorities() {
        let path = "test_prio_concurrent";
        cleanup_test_dir(path);

        let queue = Arc::new(AsyncPriorityQueue::new(1, path)
            .expect("Failed to create queue"));

        let mut handles = vec![];

        for thread_id in 0..4 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..250 {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let priority = match thread_id % 4 {
                        0 => Priority::LOW,
                        1 => Priority::MEDIUM,
                        2 => Priority::HIGH,
                        _ => Priority::CRITICAL,
                    };
                    let _guid = queue_clone.push(data, priority).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let stats = queue.get_stats();
        assert_eq!(stats.total_items, 1000);

        cleanup_test_dir(path);
    }

    // ============== Mode Switching Under Concurrent Load ==============

    #[test]
    fn test_mode_switch_during_concurrent_ops() {
        let queue = Arc::new(AsyncQueue::new(0, 512).expect("Failed to create queue"));

        let queue_push = Arc::clone(&queue);
        let push_handle = thread::spawn(move || {
            for i in 0..1000 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_push.push(data).expect("Push failed");
                
                if i == 333 {
                    thread::yield_now();
                }
            }
        });

        thread::sleep(Duration::from_millis(100));

        let queue_mode = Arc::clone(&queue);
        let mode_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            queue_mode.set_mode(1).expect("Set mode failed");
            thread::sleep(Duration::from_millis(100));
            queue_mode.set_mode(0).expect("Set mode failed");
        });

        push_handle.join().expect("Push thread panicked");
        mode_handle.join().expect("Mode thread panicked");

        assert_eq!(queue.total_pushed(), 1000);
    }

    // ============== Multiple Queues Concurrent ==============

    #[test]
    fn test_multiple_queues_concurrent() {
        let queues: Vec<Arc<AsyncQueue>> = (0..4)
            .map(|_| Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue")))
            .collect();

        let mut handles = vec![];

        for (queue_idx, queue) in queues.iter().enumerate() {
            let queue_clone = Arc::clone(queue);
            let handle = thread::spawn(move || {
                for i in 0..250 {
                    let data = format!("q{}_i{}", queue_idx, i).into_bytes();
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        for queue in queues {
            assert_eq!(queue.total_pushed(), 250);
        }
    }

    // ============== Stress Test with Rapid Operations ==============

    #[test]
    fn test_rapid_push_remove_stress() {
        let queue = Arc::new(AsyncQueue::new(1, 1024).expect("Failed to create queue"));

        let queue_push = Arc::clone(&queue);
        let push_handle = thread::spawn(move || {
            for i in 0..5000 {
                let data = format!("item_{}", i).into_bytes();
                let _guid = queue_push.push(data).expect("Push failed");
            }
        });

        thread::sleep(Duration::from_millis(50));

        let mut remove_handles = vec![];
        for _ in 0..3 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..1000 {
                    let guid = format!("item_{}", i);
                    let _result = queue_clone.remove_by_guid(&guid);
                }
            });
            remove_handles.push(handle);
        }

        push_handle.join().expect("Push thread panicked");
        for handle in remove_handles {
            handle.join().expect("Remove thread panicked");
        }

        let stats = queue.get_stats();
        assert_eq!(stats.total_pushed, 5000);
        assert!(stats.total_removed > 0);
    }

    // ============== Deadlock Prevention Tests ==============

    #[test]
    fn test_no_deadlock_high_contention() {
        let queue = Arc::new(AsyncQueue::new(1, 512).expect("Failed to create queue"));
        let timeout_secs = 10;
        let start = std::time::Instant::now();

        let mut handles = vec![];

        for thread_id in 0..8 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..500 {
                    let data = format!("t{}_i{}", thread_id, i).into_bytes();
                    let guid = queue_clone.push(data).expect("Push failed");
                    
                    // Quickly try to remove
                    if i % 10 == 0 {
                        let _result = queue_clone.remove_by_guid(&guid);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.join();
            
            // Check timeout hasn't been exceeded
            assert!(start.elapsed().as_secs() < timeout_secs, "Deadlock suspected!");
            assert!(result.is_ok(), "Thread panicked");
        }
    }

    // ============== Memory Safety Concurrent ==============

    #[test]
    fn test_concurrent_large_data_access() {
        let queue = Arc::new(AsyncQueue::new(1, 256).expect("Failed to create queue"));

        let mut handles = vec![];

        for thread_id in 0..4 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..50 {
                    // Each thread pushes 1MB of data
                    let data = vec![thread_id as u8; 1_000_000];
                    let _guid = queue_clone.push(data).expect("Push failed");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        assert_eq!(queue.total_pushed(), 200); // 4 threads * 50 items
    }
}
