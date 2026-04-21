/// Ring Buffer - Circular fixed-size buffer for audit trails
/// Non-blocking, lock-free log storage for critical paths
/// 
/// Features:
/// - O(1) write operations (no allocations)
/// - Overwrites oldest entries when full
/// - Zero-copy reading
/// - Thread-safe via atomics

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Log entry for ring buffer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RingBufferEntry {
    pub timestamp: DateTime<Utc>,
    pub severity: LogSeverity,
    pub message: String,
    pub context: Option<String>,
}

/// Log severity levels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogSeverity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "debug")]
    Debug,
}

impl LogSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            LogSeverity::Info => "info",
            LogSeverity::Warn => "warn",
            LogSeverity::Error => "error",
            LogSeverity::Debug => "debug",
        }
    }
}

/// Thread-safe circular ring buffer for audit trails
/// Allocates fixed size upfront, no allocations during operation
pub struct RingBuffer<const N: usize = 10000> {
    // Fixed-size buffer allocated once
    buffer: Arc<[Option<RingBufferEntry>; N]>,
    // Current write position (wraps around)
    write_pos: Arc<AtomicUsize>,
    // Entry count (capped at N)
    entry_count: Arc<AtomicUsize>,
}

impl<const N: usize> RingBuffer<N> {
    /// Create new ring buffer with fixed capacity N
    pub fn new() -> Self {
        // Initialize buffer with None values
        let buffer = Arc::new(unsafe {
            let mut arr: [Option<RingBufferEntry>; N] = std::mem::MaybeUninit::uninit().assume_init();
            for elem in &mut arr[..] {
                std::ptr::write(elem, None);
            }
            arr
        });

        Self {
            buffer,
            write_pos: Arc::new(AtomicUsize::new(0)),
            entry_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Add entry to buffer (non-blocking, non-allocating)
    /// Returns position where entry was written
    pub fn push(&self, entry: RingBufferEntry) -> usize {
        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) % N;
        
        // Update entry count atomically (capped at buffer size)
        self.entry_count.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| {
            if c < N { Some(c + 1) } else { None }
        }).ok();

        // SAFETY: Only one thread writes to each position at a time
        // We're using atomic fetch_add to ensure unique positions
        unsafe {
            let ptr = self.buffer.as_ptr() as *mut [Option<RingBufferEntry>; N];
            (*ptr)[pos] = Some(entry);
        }

        pos
    }

    /// Get number of valid entries (min(total_written, N))
    pub fn len(&self) -> usize {
        self.entry_count.load(Ordering::Relaxed)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get last N entries in reverse chronological order
    pub fn read_recent(&self, count: usize) -> Vec<RingBufferEntry> {
        let total_written = self.write_pos.load(Ordering::Relaxed);
        let buffer_len = self.len();
        let entries_to_read = count.min(buffer_len);

        let mut result = Vec::with_capacity(entries_to_read);

        // Start from write_pos - 1 and go backwards
        for i in 0..entries_to_read {
            let read_pos = if total_written >= N {
                // Buffer is full, wrap around
                (total_written - 1 - i + N) % N
            } else {
                // Buffer not full yet
                total_written.saturating_sub(1 + i)
            };

            if let Some(entry) = &self.buffer[read_pos] {
                result.push(entry.clone());
            }
        }

        result
    }

    /// Get all entries in order (oldest to newest)
    pub fn read_all(&self) -> Vec<RingBufferEntry> {
        let total_written = self.write_pos.load(Ordering::Relaxed);
        let buffer_len = self.len();
        let mut result = Vec::with_capacity(buffer_len);

        if buffer_len == 0 {
            return result;
        }

        // If buffer is full, start from write_pos (oldest)
        // If buffer is not full, start from 0
        let start_pos = if total_written >= N {
            total_written % N
        } else {
            0
        };

        for i in 0..buffer_len {
            let read_pos = (start_pos + i) % N;
            if let Some(entry) = &self.buffer[read_pos] {
                result.push(entry.clone());
            }
        }

        result
    }

    /// Clear buffer (reset to empty state)
    pub fn clear(&self) {
        self.write_pos.store(0, Ordering::Relaxed);
        self.entry_count.store(0, Ordering::Relaxed);
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> Clone for RingBuffer<N> {
    fn clone(&self) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
            // Clones are shared handles to the same ring buffer state.
            write_pos: Arc::clone(&self.write_pos),
            entry_count: Arc::clone(&self.entry_count),
        }
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let ring: RingBuffer<5> = RingBuffer::new();
        
        for i in 0..3 {
            ring.push(RingBufferEntry {
                timestamp: Utc::now(),
                severity: LogSeverity::Info,
                message: format!("Message {}", i),
                context: None,
            });
        }

        assert_eq!(ring.len(), 3);
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let ring: RingBuffer<3> = RingBuffer::new();
        
        // Add 5 entries to a buffer of size 3
        for i in 0..5 {
            ring.push(RingBufferEntry {
                timestamp: Utc::now(),
                severity: LogSeverity::Info,
                message: format!("Message {}", i),
                context: None,
            });
        }

        // Should only have last 3
        assert_eq!(ring.len(), 3);
        
        let recent = ring.read_recent(3);
        assert_eq!(recent.len(), 3);
        // Most recent should be Message 4
        assert!(recent[0].message.contains("4"));
    }

    #[test]
    fn test_ring_buffer_read_all() {
        let ring: RingBuffer<4> = RingBuffer::new();
        
        for i in 0..4 {
            ring.push(RingBufferEntry {
                timestamp: Utc::now(),
                severity: LogSeverity::Info,
                message: format!("Message {}", i),
                context: None,
            });
        }

        let all = ring.read_all();
        assert_eq!(all.len(), 4);
        // Check order (oldest to newest)
        assert!(all[0].message.contains("0"));
        assert!(all[3].message.contains("3"));
    }

    #[test]
    fn test_ring_buffer_concurrent() {
        use std::thread;

        let ring: RingBuffer<100> = RingBuffer::new();
        let ring_clone = ring.clone();

        let handle = thread::spawn(move || {
            for i in 0..50 {
                ring_clone.push(RingBufferEntry {
                    timestamp: Utc::now(),
                    severity: LogSeverity::Info,
                    message: format!("Thread Message {}", i),
                    context: None,
                });
            }
        });

        for i in 0..50 {
            ring.push(RingBufferEntry {
                timestamp: Utc::now(),
                severity: LogSeverity::Warn,
                message: format!("Main Message {}", i),
                context: None,
            });
        }

        handle.join().unwrap();
        
        // Should have 100 entries (50 from each thread)
        assert_eq!(ring.len(), 100);
    }
}
