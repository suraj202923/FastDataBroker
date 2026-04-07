//! Data models for Post Office Architecture
//!
//! Defines core types like Envelope, Message, Mailbox, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique message identifier
pub type MessageId = Uuid;

/// Recipient queue identifier
pub type RecipientId = String;

/// Sender identifier (producer)
pub type SenderId = String;

/// Represents a message envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envelope {
    /// Unique message ID
    pub id: MessageId,

    /// Producer/sender identifier
    pub sender_id: SenderId,

    /// List of recipient queue IDs
    pub recipient_ids: Vec<RecipientId>,

    /// Message subject/metadata
    pub subject: String,

    /// Binary message payload
    pub content: Vec<u8>,

    /// Priority level (0-255, higher = more urgent)
    pub priority: u8,

    /// Time to live in seconds (optional)
    pub ttl_seconds: Option<u64>,

    /// Whether delivery confirmation is required
    pub require_confirmation: bool,

    /// Custom metadata tags
    pub tags: HashMap<String, String>,

    /// Server timestamp when message was received
    pub timestamp: u64,
}

impl Envelope {
    /// Create a new envelope
    pub fn new(
        sender_id: String,
        recipient_ids: Vec<String>,
        subject: String,
        content: Vec<u8>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Envelope {
            id: Uuid::new_v4(),
            sender_id,
            recipient_ids,
            subject,
            content,
            priority: 100,  // Default: normal priority
            ttl_seconds: None,
            require_confirmation: false,
            tags: HashMap::new(),
            timestamp: now,
        }
    }

    /// Set the priority level
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set time-to-live
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
}

/// Mailbox for a recipient
#[derive(Debug, Clone)]
pub struct Mailbox {
    /// Recipient ID
    pub recipient_id: RecipientId,

    /// Messages in inbox
    pub inbox: Vec<MessageId>,

    /// Read/archived messages
    pub archive: Vec<MessageId>,

    /// Spam/filtered messages
    pub spam: Vec<MessageId>,
}

impl Mailbox {
    /// Create a new mailbox for a recipient
    pub fn new(recipient_id: RecipientId) -> Self {
        Mailbox {
            recipient_id,
            inbox: Vec::new(),
            archive: Vec::new(),
            spam: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_creation() {
        let env = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test Message".to_string(),
            b"Hello, World!".to_vec(),
        );

        assert_eq!(env.sender_id, "producer-1");
        assert_eq!(env.priority, 100);
        assert_eq!(env.content, b"Hello, World!");
    }
}
