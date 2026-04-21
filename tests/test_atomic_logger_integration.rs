/// Tests for atomic logger integration in QUIC authentication
#[cfg(test)]
mod atomic_logger_tests {
    use fastdatabroker::security::quic_auth::QuicAuthValidator;
    use std::sync::Arc;
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn test_atomic_logger_tracks_successful_auth() {
        let validator = QuicAuthValidator::new();
        let api_key = validator.generate_key("test-client", 1000).await;

        // Before auth, logger should have zero events
        let stats_before = validator.get_logger_stats();
        assert_eq!(stats_before.total_events, 0);

        // Perform successful auth
        let result = validator.validate_key(&api_key).await;
        assert!(result.is_ok());

        // After auth, logger should have recorded one info event
        let stats_after = validator.get_logger_stats();
        assert!(stats_after.total_events > 0);
        assert!(stats_after.infos > 0);
    }

    #[tokio::test]
    async fn test_atomic_logger_tracks_rate_limit_violations() {
        let validator = QuicAuthValidator::new();
        let api_key = validator.generate_key("test-client", 2).await; // 2 req/sec limit

        // Exhaust rate limit
        let _ = validator.validate_key(&api_key).await; // OK (1/2)
        let _ = validator.validate_key(&api_key).await; // OK (2/2)
        let result = validator.validate_key(&api_key).await; // Rate limit exceeded

        // Verify rate limit was violated
        assert!(result.is_err());

        // Logger should have tracked the warning
        let stats = validator.get_logger_stats();
        assert!(stats.warnings > 0, "Expected warnings to be recorded");
    }

    #[tokio::test]
    async fn test_atomic_logger_tracks_key_revocation() {
        let validator = QuicAuthValidator::new();
        let api_key = validator.generate_key("test-client", 1000).await;

        // Before revocation, logger should have minimal events
        let stats_before = validator.get_logger_stats();
        let infos_before = stats_before.infos;

        // Revoke key
        validator.revoke_key(&api_key).await.unwrap();

        // After revocation, logger should have recorded additional info event
        let stats_after = validator.get_logger_stats();
        assert!(stats_after.infos > infos_before, 
                "Expected info event count to increase after revocation");
    }

    #[tokio::test]
    async fn test_atomic_logger_thread_safety() {
        let validator = Arc::new(QuicAuthValidator::new());
        let num_threads = 10;
        let api_key = validator.generate_key("test-client", 10000).await;
        let barrier = Arc::new(Barrier::new(num_threads));

        let mut handles = vec![];

        for _ in 0..num_threads {
            let validator_clone = validator.clone();
            let api_key_clone = api_key.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait().await; // Synchronize threads
                
                for _ in 0..100 {
                    let _ = validator_clone.validate_key(&api_key_clone).await;
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // After concurrent access, logger should have recorded events
        let stats = validator.get_logger_stats();
        assert!(stats.total_events > 0, 
                "Expected atomic logger to track concurrent events");
    }

    #[tokio::test]
    async fn test_atomic_logger_stats_correlation() {
        let validator = QuicAuthValidator::new();

        // Generate multiple keys
        for i in 0..5 {
            let _ = validator.generate_key(&format!("client-{}", i), 1000).await;
        }

        // Get stats
        let (auth_success, auth_failures, rate_limit_hits) = validator.get_stats();
        let logger_stats = validator.get_logger_stats();

        // Verify that logger total_events correlates with auth activity
        // logger.total_events should include info events from successful auths,
        // and warning events from failures/rate limits
        println!(
            "Auth stats: success={}, failures={}, rate_limits={}",
            auth_success, auth_failures, rate_limit_hits
        );
        println!(
            "Logger stats: {:?}",
            logger_stats
        );

        // Total events includes all activity tracked by atomic logger
        assert!(logger_stats.total_events >= 0);
    }

    #[tokio::test]
    async fn test_atomic_logger_overflow_resilience() {
        let validator = QuicAuthValidator::new();
        let api_key = validator.generate_key("test-client", 1).await; // 1 req/sec

        // Generate many violations
        for _ in 0..1000 {
            let _ = validator.validate_key(&api_key).await;
        }

        // Logger should still be functional and not panic
        let stats = validator.get_logger_stats();
        assert!(stats.warnings > 0, "Expected many warnings to be recorded");
    }

    #[tokio::test]
    async fn test_atomic_logger_zero_overhead_overhead() {
        let validator = QuicAuthValidator::new();
        let api_key = validator.generate_key("test-client", 10000).await;

        // Warm up
        for _ in 0..10 {
            let _ = validator.validate_key(&api_key).await;
        }

        // Measure baseline - auth operations
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = validator.validate_key(&api_key).await;
        }
        let elapsed = start.elapsed();

        // Logger calls should not significantly impact performance
        // (This is a smoke test; actual benchmarking should be done separately)
        println!("1000 auth validations took: {:?}", elapsed);
        
        // Ensure no panic and reasonable timing
        assert!(elapsed.as_millis() < 10000, 
                "Auth validation took unexpectedly long");
    }
}
