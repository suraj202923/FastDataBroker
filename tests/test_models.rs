/// Comprehensive unit tests for Models
/// Tests Envelope, Mailbox, and related structures

#[cfg(test)]
mod model_tests {
    use fastdatabroker::models::{Envelope, Mailbox, MessageId, RecipientId, SenderId};
    use std::collections::HashMap;

    // ============== Envelope Creation ==============

    #[test]
    fn test_envelope_creation() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Test Subject".to_string(),
            b"Test content".to_vec(),
        );

        assert_eq!(env.sender_id, "sender1");
        assert_eq!(env.recipient_ids.len(), 1);
        assert_eq!(env.recipient_ids[0], "recipient1");
        assert_eq!(env.subject, "Test Subject");
        assert_eq!(env.content, b"Test content");
        assert_eq!(env.priority, 100);
        assert_eq!(env.require_confirmation, false);
        assert!(env.ttl_seconds.is_none());
    }

    #[test]
    fn test_envelope_with_priority() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        ).with_priority(200);

        assert_eq!(env.priority, 200);
    }

    #[test]
    fn test_envelope_with_ttl() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        ).with_ttl(3600);

        assert_eq!(env.ttl_seconds, Some(3600));
    }

    #[test]
    fn test_envelope_with_priority_and_ttl() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        )
        .with_priority(150)
        .with_ttl(7200);

        assert_eq!(env.priority, 150);
        assert_eq!(env.ttl_seconds, Some(7200));
    }

    // ============== Multiple Recipients ==============

    #[test]
    fn test_envelope_multiple_recipients() {
        let recipients = vec![
            "recipient1".to_string(),
            "recipient2".to_string(),
            "recipient3".to_string(),
        ];

        let env = Envelope::new(
            "sender1".to_string(),
            recipients.clone(),
            "Subject".to_string(),
            b"content".to_vec(),
        );

        assert_eq!(env.recipient_ids.len(), 3);
        assert_eq!(env.recipient_ids, recipients);
    }

    // ============== Envelope Tags ==============

    #[test]
    fn test_envelope_with_tags() {
        let mut env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        env.tags.insert("category".to_string(), "notification".to_string());
        env.tags.insert("source".to_string(), "api".to_string());

        assert_eq!(env.tags.len(), 2);
        assert_eq!(env.tags.get("category").unwrap(), "notification");
        assert_eq!(env.tags.get("source").unwrap(), "api");
    }

    // ============== Envelope ID/UUID ==============

    #[test]
    fn test_envelope_id_uniqueness() {
        let env1 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        let env2 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        assert_ne!(env1.id, env2.id);
    }

    #[test]
    fn test_envelope_timestamp() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        assert!(env.timestamp > 0);
    }

    #[test]
    fn test_envelope_timestamp_monotonic() {
        let env1 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        std::thread::sleep(std::time::Duration::from_millis(10));

        let env2 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        assert!(env2.timestamp >= env1.timestamp);
    }

    // ============== Empty Content ==============

    #[test]
    fn test_envelope_empty_content() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            vec![],
        );

        assert_eq!(env.content.len(), 0);
    }

    #[test]
    fn test_envelope_empty_subject() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "".to_string(),
            b"content".to_vec(),
        );

        assert_eq!(env.subject, "");
    }

    // ============== Large Content ==============

    #[test]
    fn test_envelope_large_content() {
        let large_content = vec![0xFFu8; 10 * 1024 * 1024]; // 10MB
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            large_content.clone(),
        );

        assert_eq!(env.content.len(), 10 * 1024 * 1024);
        assert_eq!(env.content, large_content);
    }

    // ============== Mailbox ==============

    #[test]
    fn test_mailbox_creation() {
        let mailbox = Mailbox::new("user123".to_string());

        assert_eq!(mailbox.recipient_id, "user123");
        assert_eq!(mailbox.inbox.len(), 0);
        assert_eq!(mailbox.archive.len(), 0);
        assert_eq!(mailbox.spam.len(), 0);
    }

    #[test]
    fn test_mailbox_add_to_inbox() {
        let mut mailbox = Mailbox::new("user123".to_string());
        let message_id = uuid::Uuid::new_v4();

        mailbox.inbox.push(message_id);

        assert_eq!(mailbox.inbox.len(), 1);
        assert_eq!(mailbox.inbox[0], message_id);
    }

    #[test]
    fn test_mailbox_add_multiple_messages() {
        let mut mailbox = Mailbox::new("user123".to_string());
        
        for _ in 0..5 {
            mailbox.inbox.push(uuid::Uuid::new_v4());
        }
        for _ in 0..3 {
            mailbox.archive.push(uuid::Uuid::new_v4());
        }
        for _ in 0..2 {
            mailbox.spam.push(uuid::Uuid::new_v4());
        }

        assert_eq!(mailbox.inbox.len(), 5);
        assert_eq!(mailbox.archive.len(), 3);
        assert_eq!(mailbox.spam.len(), 2);
    }

    #[test]
    fn test_mailbox_message_movement() {
        let mut mailbox = Mailbox::new("user123".to_string());
        let msg_id = uuid::Uuid::new_v4();

        // Add to inbox
        mailbox.inbox.push(msg_id);
        assert_eq!(mailbox.inbox.len(), 1);

        // Move to archive
        mailbox.inbox.retain(|&id| id != msg_id);
        mailbox.archive.push(msg_id);

        assert_eq!(mailbox.inbox.len(), 0);
        assert_eq!(mailbox.archive.len(), 1);
    }

    #[test]
    fn test_mailbox_mark_as_spam() {
        let mut mailbox = Mailbox::new("user123".to_string());
        let msg_id = uuid::Uuid::new_v4();

        mailbox.inbox.push(msg_id);
        mailbox.inbox.retain(|&id| id != msg_id);
        mailbox.spam.push(msg_id);

        assert_eq!(mailbox.inbox.len(), 0);
        assert_eq!(mailbox.spam.len(), 1);
    }

    // ============== Serialization/Deserialization ==============

    #[test]
    fn test_envelope_serialization() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string(), "recipient2".to_string()],
            "Test Subject".to_string(),
            b"Test content".to_vec(),
        );

        let serialized = serde_json::to_string(&env).expect("Serialization failed");
        let deserialized: Envelope = serde_json::from_str(&serialized)
            .expect("Deserialization failed");

        assert_eq!(deserialized.sender_id, env.sender_id);
        assert_eq!(deserialized.recipient_ids, env.recipient_ids);
        assert_eq!(deserialized.subject, env.subject);
        assert_eq!(deserialized.content, env.content);
        assert_eq!(deserialized.id, env.id);
    }

    #[test]
    fn test_envelope_bincode_serialization() {
        let env = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        let encoded = bincode::serialize(&env).expect("Encoding failed");
        let decoded: Envelope = bincode::deserialize(&encoded)
            .expect("Decoding failed");

        assert_eq!(decoded.sender_id, env.sender_id);
        assert_eq!(decoded.recipient_ids, env.recipient_ids);
    }

    // ============== Complex Envelope ==============

    #[test]
    fn test_complex_envelope() {
        let mut env = Envelope::new(
            "system@app.com".to_string(),
            vec![
                "user1@app.com".to_string(),
                "user2@app.com".to_string(),
                "user3@app.com".to_string(),
            ],
            "Important Notification".to_string(),
            b"This is a critical message for all users".to_vec(),
        );

        env = env.with_priority(250).with_ttl(86400);
        env.require_confirmation = true;
        env.tags.insert("severity".to_string(), "high".to_string());
        env.tags.insert("type".to_string(), "system-alert".to_string());
        env.tags.insert("version".to_string(), "1.0".to_string());

        assert_eq!(env.sender_id, "system@app.com");
        assert_eq!(env.recipient_ids.len(), 3);
        assert_eq!(env.priority, 250);
        assert_eq!(env.ttl_seconds, Some(86400));
        assert_eq!(env.require_confirmation, true);
        assert_eq!(env.tags.len(), 3);
    }

    // ============== Envelope Cloning ==============

    #[test]
    fn test_envelope_clone() {
        let env1 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        let env2 = env1.clone();

        assert_eq!(env1.sender_id, env2.sender_id);
        assert_eq!(env1.id, env2.id);
        assert_eq!(env1.subject, env2.subject);
        assert_eq!(env1.content, env2.content);
    }

    // ============== PartialEq Implementation ==============

    #[test]
    fn test_envelope_equality() {
        let env1 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        let env2 = env1.clone();

        assert_eq!(env1, env2);
    }

    #[test]
    fn test_envelope_inequality() {
        let env1 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        let env2 = Envelope::new(
            "sender1".to_string(),
            vec!["recipient1".to_string()],
            "Subject".to_string(),
            b"content".to_vec(),
        );

        assert_ne!(env1.id, env2.id);
    }

    // ============== Type Aliases ==============

    #[test]
    fn test_type_aliases() {
        let sender: SenderId = "sender123".to_string();
        let recipient: RecipientId = "recipient456".to_string();
        
        assert_eq!(sender, "sender123");
        assert_eq!(recipient, "recipient456");
    }
}
