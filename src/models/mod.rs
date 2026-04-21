//! Data models for Post Office Architecture
//!
//! Defines core types like Envelope, Message, Mailbox, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

pub type MessageId = Uuid;
pub type RecipientId = String;
pub type SenderId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envelope {
    pub id: MessageId,
    pub sender_id: SenderId,
    pub recipient_ids: Vec<RecipientId>,
    pub subject: String,
    pub content: Vec<u8>,
    pub priority: u8,
    pub ttl_seconds: Option<u64>,
    pub require_confirmation: bool,
    pub tags: HashMap<String, String>,
    pub timestamp: u64,
}

impl Envelope {
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
            priority: 100,
            ttl_seconds: None,
            require_confirmation: false,
            tags: HashMap::new(),
            timestamp: now,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Mailbox {
    pub recipient_id: RecipientId,
    pub inbox: Vec<MessageId>,
    pub archive: Vec<MessageId>,
    pub spam: Vec<MessageId>,
}

impl Mailbox {
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
