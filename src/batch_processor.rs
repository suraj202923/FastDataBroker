/// Batch Processing Optimization (Phase 3)
///
/// Implements efficient batch processing for queue operations.
/// Combines multiple items into batches to reduce per-item overhead.
/// Particularly effective for high-throughput workloads with many small items.

use std::sync::Arc;
use crossbeam::queue::SegQueue;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum items per batch
    pub max_batch_size: usize,
    /// Maximum time to wait before processing a partial batch (milliseconds)
    pub batch_timeout_ms: u64,
    /// Enable adaptive batching (adjust size based on throughput)
    pub adaptive: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            max_batch_size: 64,
            batch_timeout_ms: 10,
            adaptive: true,
        }
    }
}

/// Generic batch processor
pub struct BatchProcessor<T: Clone + Send + Sync> {
    /// Pending items queue
    pending: Arc<SegQueue<T>>,
    /// Configuration
    config: BatchConfig,
    /// Statistics
    processed_batches: Arc<AtomicU64>,
    /// Last batch creation time
    last_batch_time: Arc<parking_lot::Mutex<Instant>>,
}

impl<T: Clone + Send + Sync + 'static> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(config: BatchConfig) -> Self {
        BatchProcessor {
            pending: Arc::new(SegQueue::new()),
            config,
            processed_batches: Arc::new(AtomicU64::new(0)),
            last_batch_time: Arc::new(parking_lot::Mutex::new(Instant::now())),
        }
    }

    /// Add item to batch
    pub fn add(&self, item: T) {
        self.pending.push(item);
    }

    /// Add multiple items
    pub fn add_batch(&self, items: Vec<T>) {
        for item in items {
            self.pending.push(item);
        }
    }

    /// Get next batch for processing
    pub fn get_batch(&self) -> Vec<T> {
        let mut batch = Vec::with_capacity(self.config.max_batch_size);

        // Get items from pending queue
        while batch.len() < self.config.max_batch_size {
            match self.pending.pop() {
                Some(item) => batch.push(item),
                None => break,
            }
        }

        if !batch.is_empty() {
            self.processed_batches.fetch_add(1, Ordering::Relaxed);
            *self.last_batch_time.lock() = Instant::now();
        }
        batch
    }

    /// Check if there are pending items (approximate)
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Check if batch should be processed (size or timeout)
    pub fn should_process(&self) -> bool {
        // Check size of pending queue
        if self.pending.len() >= self.config.max_batch_size {
            return true;
        }

        // Check timeout
        let last_time = self.last_batch_time.lock();
        let elapsed = last_time.elapsed();

        elapsed >= Duration::from_millis(self.config.batch_timeout_ms)
    }

    /// Get statistics
    pub fn stats(&self) -> BatchStats {
        BatchStats {
            processed_batches: self.processed_batches.load(Ordering::Relaxed),
            pending_items: self.pending.len(),
        }
    }

    /// Update batch timeout (for adaptive batching)
    pub fn update_timeout(&mut self, timeout_ms: u64) {
        self.config.batch_timeout_ms = timeout_ms;
    }
}

/// Batch statistics
#[derive(Debug, Clone)]
pub struct BatchStats {
    /// Number of batches processed
    pub processed_batches: u64,
    /// Current pending items
    pub pending_items: usize,
}

/// Batch processing result
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Number of items processed
    pub items_processed: usize,
    /// Number of successful operations
    pub successes: usize,
    /// Number of failed operations
    pub failures: usize,
    /// Processing time in microseconds
    pub duration_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_add() {
        let processor = BatchProcessor::new(BatchConfig::default());

        processor.add(1);
        processor.add(2);
        processor.add(3);

        let batch = processor.get_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch, vec![1, 2, 3]);
    }

    #[test]
    fn test_batch_processor_max_size() {
        let config = BatchConfig {
            max_batch_size: 5,
            batch_timeout_ms: 1000,
            adaptive: false,
        };
        let processor = BatchProcessor::new(config);

        for i in 0..10 {
            processor.add(i);
        }

        let batch1 = processor.get_batch();
        assert_eq!(batch1.len(), 5);

        let batch2 = processor.get_batch();
        assert_eq!(batch2.len(), 5);
    }

    #[test]
    fn test_batch_processor_statistics() {
        let processor = BatchProcessor::new(BatchConfig::default());

        for i in 0..10 {
            processor.add(i);
        }

        let _ = processor.get_batch();
        let stats = processor.stats();

        assert_eq!(stats.processed_batches, 1);
    }

    #[test]
    fn test_batch_has_pending() {
        let processor = BatchProcessor::new(BatchConfig::default());

        assert!(!processor.has_pending());

        processor.add(1);
        assert!(processor.has_pending());

        let _ = processor.get_batch();
        assert!(!processor.has_pending());
    }
}
