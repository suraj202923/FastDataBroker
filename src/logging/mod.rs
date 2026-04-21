/// Optimized Logging Module
/// 
/// Four-tier logging system for maximum performance:
/// 1. Atomic Logger - Critical path, rate limiting (nanoseconds)
/// 2. Async Logger - General application logs (non-blocking)
/// 3. Ring Buffer - Audit trails with fixed memory (O(1) operations)
/// 4. Tracing - Complex structured logs (for non-critical paths)

pub mod atomic_logger;
pub mod async_logger;
pub mod ring_buffer;

pub use atomic_logger::{AtomicLogger, RateLimitCounter, AtomicStats, RateLimitStats};
pub use async_logger::{AsyncLogger, AsyncLoggerHandle, AsyncLogEntry, LogLevel, AsyncLoggerConfig, AsyncLoggerStats};
pub use ring_buffer::{RingBuffer, RingBufferEntry, LogSeverity};
