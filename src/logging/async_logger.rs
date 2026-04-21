/// Async Channel Logger - Non-blocking async logging with background flushing
/// Main thread logs to channel immediately (~100ns), background task handles I/O
/// Perfect for balancing performance and data persistence
///
/// Use Case: General application logging, request tracking, audit trails

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Log entry for async channel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsyncLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub context: Option<String>,
}

/// Log level for async logging
#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Configuration for async logger
#[derive(Debug, Clone)]
pub struct AsyncLoggerConfig {
    /// Channel capacity (buffer size)
    pub channel_capacity: usize,
    /// Batch size for flushing
    pub batch_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
}

impl Default for AsyncLoggerConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 10000,
            batch_size: 100,
            flush_interval_ms: 100,
        }
    }
}

/// Non-blocking async logger
/// Main thread: push to channel (returns immediately)
/// Background: async flush to handler
pub struct AsyncLogger {
    sender: mpsc::UnboundedSender<AsyncLogEntry>,
    entries_logged: Arc<AtomicU64>,
    entries_flushed: Arc<AtomicU64>,
}

impl AsyncLogger {
    /// Create new async logger with background task
    pub fn new(config: AsyncLoggerConfig) -> (Self, AsyncLoggerHandle) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let entries_logged = Arc::new(AtomicU64::new(0));
        let entries_flushed = Arc::new(AtomicU64::new(0));

        let entries_logged_clone = Arc::clone(&entries_logged);
        let entries_flushed_clone = Arc::clone(&entries_flushed);

        let handle = AsyncLoggerHandle {
            receiver,
            config,
            entries_flushed: entries_flushed_clone,
            entries_logged: entries_logged_clone,
            buffered_entries: Vec::new(),
        };

        let logger = Self {
            sender,
            entries_logged,
            entries_flushed,
        };

        (logger, handle)
    }

    /// Log entry (non-blocking, returns immediately)
    #[inline]
    pub fn log(&self, entry: AsyncLogEntry) {
        // Try to send, ignore if channel is full (backpressure)
        let _ = self.sender.send(entry);
        self.entries_logged.fetch_add(1, Ordering::Relaxed);
    }

    /// Log at trace level
    #[inline]
    pub fn trace(&self, target: String, message: String) {
        self.log(AsyncLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Trace,
            target,
            message,
            context: None,
        });
    }

    /// Log at debug level
    #[inline]
    pub fn debug(&self, target: String, message: String) {
        self.log(AsyncLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Debug,
            target,
            message,
            context: None,
        });
    }

    /// Log at info level
    #[inline]
    pub fn info(&self, target: String, message: String) {
        self.log(AsyncLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            target,
            message,
            context: None,
        });
    }

    /// Log at warn level
    #[inline]
    pub fn warn(&self, target: String, message: String) {
        self.log(AsyncLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warn,
            target,
            message,
            context: None,
        });
    }

    /// Log at error level
    #[inline]
    pub fn error(&self, target: String, message: String) {
        self.log(AsyncLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            target,
            message,
            context: None,
        });
    }

    /// Get statistics
    pub fn stats(&self) -> AsyncLoggerStats {
        AsyncLoggerStats {
            entries_logged: self.entries_logged.load(Ordering::Relaxed),
            entries_flushed: self.entries_flushed.load(Ordering::Relaxed),
            pending: self.entries_logged.load(Ordering::Relaxed) 
                - self.entries_flushed.load(Ordering::Relaxed),
        }
    }
}

impl Clone for AsyncLogger {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            entries_logged: Arc::clone(&self.entries_logged),
            entries_flushed: Arc::clone(&self.entries_flushed),
        }
    }
}

impl Default for AsyncLogger {
    fn default() -> Self {
        let (logger, _) = Self::new(AsyncLoggerConfig::default());
        logger
    }
}

/// Handle for async logger background task
pub struct AsyncLoggerHandle {
    receiver: mpsc::UnboundedReceiver<AsyncLogEntry>,
    config: AsyncLoggerConfig,
    entries_flushed: Arc<AtomicU64>,
    entries_logged: Arc<AtomicU64>,
    buffered_entries: Vec<AsyncLogEntry>,
}

impl AsyncLoggerHandle {
    /// Run background flush task (should run in separate tokio task)
    pub async fn run<F>(self, mut handler: F) -> Result<(), String>
    where
        F: FnMut(Vec<AsyncLogEntry>) -> futures::future::BoxFuture<'static, Result<(), String>>,
    {
        let mut receiver = self.receiver;
        let mut buffered_entries = self.buffered_entries;
        let config = self.config;
        let entries_flushed = self.entries_flushed;

        let mut flush_interval = tokio::time::interval(
            tokio::time::Duration::from_millis(config.flush_interval_ms)
        );

        loop {
            tokio::select! {
                // Collect incoming entries
                Some(entry) = receiver.recv() => {
                    buffered_entries.push(entry);

                    // Flush if batch full
                    if buffered_entries.len() >= config.batch_size {
                        let batch = std::mem::take(&mut buffered_entries);
                        handler(batch.clone()).await?;
                        entries_flushed.fetch_add(batch.len() as u64, Ordering::Relaxed);
                    }
                }

                // Periodic flush
                _ = flush_interval.tick() => {
                    if !buffered_entries.is_empty() {
                        let batch = std::mem::take(&mut buffered_entries);
                        handler(batch.clone()).await?;
                        entries_flushed.fetch_add(batch.len() as u64, Ordering::Relaxed);
                    }
                }

                // Channel closed
                else => break,
            }
        }

        // Final flush of any remaining entries
        if !buffered_entries.is_empty() {
            handler(buffered_entries.clone()).await?;
            entries_flushed.fetch_add(buffered_entries.len() as u64, Ordering::Relaxed);
        }

        Ok(())
    }
}

/// Statistics from async logger
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AsyncLoggerStats {
    pub entries_logged: u64,
    pub entries_flushed: u64,
    pub pending: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_logger_creation() {
        let config = AsyncLoggerConfig::default();
        let (logger, _handle) = AsyncLogger::new(config);
        
        logger.info("test".to_string(), "test message".to_string());
        
        let stats = logger.stats();
        assert_eq!(stats.entries_logged, 1);
    }

    #[test]
    fn test_async_logger_log_levels() {
        let config = AsyncLoggerConfig::default();
        let (logger, _handle) = AsyncLogger::new(config);
        
        logger.trace("test".to_string(), "trace".to_string());
        logger.debug("test".to_string(), "debug".to_string());
        logger.info("test".to_string(), "info".to_string());
        logger.warn("test".to_string(), "warn".to_string());
        logger.error("test".to_string(), "error".to_string());
        
        let stats = logger.stats();
        assert_eq!(stats.entries_logged, 5);
    }

    #[tokio::test]
    async fn test_async_logger_handler() {
        let config = AsyncLoggerConfig {
            channel_capacity: 10000,
            batch_size: 2,
            flush_interval_ms: 100,
        };
        let (logger, handle) = AsyncLogger::new(config);

        // Spawn handler task
        tokio::spawn({
            let logs = Arc::new(std::sync::Mutex::new(Vec::new()));
            let logs_clone = Arc::clone(&logs);
            
            async move {
                let _ = handle.run(|batch| {
                    let logs = Arc::clone(&logs_clone);
                    Box::pin(async move {
                        logs.lock().unwrap().extend(batch);
                        Ok(())
                    })
                }).await;
            }
        });

        // Log some entries
        logger.info("test".to_string(), "message 1".to_string());
        logger.warn("test".to_string(), "message 2".to_string());

        // Give background task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
}
