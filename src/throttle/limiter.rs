//! Throttle limiter

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::config::ThrottleConfig;
use super::token_bucket::TokenBucket;

/// Result of a throttle check
#[derive(Debug, Clone, PartialEq)]
pub enum ThrottleResult {
    /// Allowed, with remaining bandwidth
    Allowed { remaining: u64 },
    /// Throttled, need to wait
    Throttled { wait_ms: u64 },
    /// No limit configured
    Unlimited,
}

impl ThrottleResult {
    /// Check if the request is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, ThrottleResult::Allowed { .. } | ThrottleResult::Unlimited)
    }
}

/// Throttle limiter manages bandwidth limits
#[derive(Debug)]
pub struct ThrottleLimiter {
    config: ThrottleConfig,
    global_bucket: Arc<RwLock<TokenBucket>>,
    ip_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

impl ThrottleLimiter {
    /// Create a new throttle limiter
    pub fn new(config: ThrottleConfig) -> Self {
        let global_bucket = if config.has_global_limit() {
            TokenBucket::new(
                config.bucket_capacity,
                config.global_limit,
            )
        } else {
            TokenBucket::new(0, 0)
        };
        
        Self {
            config,
            global_bucket: Arc::new(RwLock::new(global_bucket)),
            ip_buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ThrottleConfig {
        &self.config
    }

    /// Check if a request should be throttled
    pub async fn check(&self, ip: &str, bytes: u64) -> ThrottleResult {
        if !self.config.is_active() {
            return ThrottleResult::Unlimited;
        }
        
        // Check global limit first
        if self.config.has_global_limit() {
            let mut bucket = self.global_bucket.write().await;
            if !bucket.try_consume(bytes) {
                let wait = bucket.wait_time(bytes);
                return ThrottleResult::Throttled {
                    wait_ms: wait.as_millis() as u64,
                };
            }
        }
        
        // Check per-IP limit
        if self.config.has_per_ip_limit() {
            let mut buckets = self.ip_buckets.write().await;
            let bucket = buckets.entry(ip.to_string()).or_insert_with(|| {
                TokenBucket::new(
                    self.config.bucket_capacity,
                    self.config.per_ip_limit,
                )
            });
            
            if !bucket.try_consume(bytes) {
                let wait = bucket.wait_time(bytes);
                return ThrottleResult::Throttled {
                    wait_ms: wait.as_millis() as u64,
                };
            }
        }
        
        ThrottleResult::Allowed { remaining: 0 }
    }

    /// Consume bandwidth tokens
    /// Returns the number of bytes actually consumed
    pub async fn consume(&self, ip: &str, bytes: u64) -> u64 {
        if !self.config.is_active() {
            return bytes;
        }
        
        let mut total_consumed = bytes;
        
        // Consume from global bucket
        if self.config.has_global_limit() {
            let mut bucket = self.global_bucket.write().await;
            total_consumed = total_consumed.min(bucket.consume(bytes));
        }
        
        // Consume from per-IP bucket
        if self.config.has_per_ip_limit() && total_consumed > 0 {
            let mut buckets = self.ip_buckets.write().await;
            let bucket = buckets.entry(ip.to_string()).or_insert_with(|| {
                TokenBucket::new(
                    self.config.bucket_capacity,
                    self.config.per_ip_limit,
                )
            });
            total_consumed = total_consumed.min(bucket.consume(bytes));
        }
        
        total_consumed
    }

    /// Get wait time for a request
    pub async fn wait_time(&self, ip: &str, bytes: u64) -> Duration {
        if !self.config.is_active() {
            return Duration::ZERO;
        }
        
        let mut max_wait = Duration::ZERO;
        
        if self.config.has_global_limit() {
            let mut bucket = self.global_bucket.write().await;
            max_wait = max_wait.max(bucket.wait_time(bytes));
        }
        
        if self.config.has_per_ip_limit() {
            let mut buckets = self.ip_buckets.write().await;
            if let Some(bucket) = buckets.get(ip) {
                let mut bucket = bucket.clone();
                max_wait = max_wait.max(bucket.wait_time(bytes));
            }
        }
        
        max_wait
    }

    /// Reset all buckets
    pub async fn reset(&self) {
        if self.config.has_global_limit() {
            let mut bucket = self.global_bucket.write().await;
            bucket.reset();
        }
        
        let mut buckets = self.ip_buckets.write().await;
        for bucket in buckets.values_mut() {
            bucket.reset();
        }
    }

    /// Clear per-IP buckets
    pub async fn clear_ip_buckets(&self) {
        let mut buckets = self.ip_buckets.write().await;
        buckets.clear();
    }

    /// Get number of tracked IPs
    pub async fn tracked_ip_count(&self) -> usize {
        let buckets = self.ip_buckets.read().await;
        buckets.len()
    }

    /// Remove an IP from tracking
    pub async fn remove_ip(&self, ip: &str) -> bool {
        let mut buckets = self.ip_buckets.write().await;
        buckets.remove(ip).is_some()
    }

    /// Update configuration (creates new buckets)
    pub fn update_config(&mut self, config: ThrottleConfig) {
        self.config = config;
        
        // Recreate global bucket
        let global_bucket = if self.config.has_global_limit() {
            TokenBucket::new(
                self.config.bucket_capacity,
                self.config.global_limit,
            )
        } else {
            TokenBucket::new(0, 0)
        };
        
        self.global_bucket = Arc::new(RwLock::new(global_bucket));
    }

    /// Get global bucket tokens
    pub async fn global_tokens(&self) -> u64 {
        if !self.config.has_global_limit() {
            return u64::MAX;
        }
        let mut bucket = self.global_bucket.write().await;
        bucket.tokens()
    }

    /// Get per-IP bucket tokens
    pub async fn ip_tokens(&self, ip: &str) -> u64 {
        if !self.config.has_per_ip_limit() {
            return u64::MAX;
        }
        let buckets = self.ip_buckets.read().await;
        if let Some(bucket) = buckets.get(ip) {
            let mut bucket = bucket.clone();
            bucket.tokens()
        } else {
            self.config.bucket_capacity
        }
    }
}

impl Clone for ThrottleLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            global_bucket: Arc::clone(&self.global_bucket),
            ip_buckets: Arc::clone(&self.ip_buckets),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_creation() {
        let config = ThrottleConfig::new();
        let limiter = ThrottleLimiter::new(config);
        
        assert!(!limiter.config().is_active());
    }

    #[tokio::test]
    async fn test_check_unlimited() {
        let config = ThrottleConfig::new(); // Not enabled
        let limiter = ThrottleLimiter::new(config);
        
        let result = limiter.check("127.0.0.1", 1000).await;
        assert_eq!(result, ThrottleResult::Unlimited);
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_global_limit() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        // First request should be allowed
        let result = limiter.check("127.0.0.1", 500).await;
        assert!(result.is_allowed());
        
        // Second request should be allowed
        let result = limiter.check("127.0.0.1", 500).await;
        assert!(result.is_allowed());
        
        // Third request should be throttled
        let result = limiter.check("127.0.0.1", 500).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_per_ip_limit() {
        let config = ThrottleConfig::new()
            .enable()
            .with_per_ip_limit(500)
            .with_bucket_capacity(500);
        
        let limiter = ThrottleLimiter::new(config);
        
        // IP1 should be limited
        let result = limiter.check("127.0.0.1", 300).await;
        assert!(result.is_allowed());
        
        let result = limiter.check("127.0.0.1", 300).await;
        assert!(!result.is_allowed());
        
        // IP2 should still have tokens
        let result = limiter.check("127.0.0.2", 300).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_consume_unlimited() {
        let config = ThrottleConfig::new();
        let limiter = ThrottleLimiter::new(config);
        
        let consumed = limiter.consume("127.0.0.1", 1000).await;
        assert_eq!(consumed, 1000);
    }

    #[tokio::test]
    async fn test_consume_partial() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(500)
            .with_bucket_capacity(500);
        
        let limiter = ThrottleLimiter::new(config);
        
        let consumed = limiter.consume("127.0.0.1", 300).await;
        assert_eq!(consumed, 300);
        
        let consumed = limiter.consume("127.0.0.1", 300).await;
        assert_eq!(consumed, 200); // Only 200 left
    }

    #[tokio::test]
    async fn test_wait_time_zero() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        let wait = limiter.wait_time("127.0.0.1", 500).await;
        assert_eq!(wait, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_reset() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        limiter.check("127.0.0.1", 1000).await;
        
        limiter.reset().await;
        
        let tokens = limiter.global_tokens().await;
        assert_eq!(tokens, 1000);
    }

    #[tokio::test]
    async fn test_clear_ip_buckets() {
        let config = ThrottleConfig::new()
            .enable()
            .with_per_ip_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        limiter.check("127.0.0.1", 500).await;
        limiter.check("127.0.0.2", 500).await;
        
        assert_eq!(limiter.tracked_ip_count().await, 2);
        
        limiter.clear_ip_buckets().await;
        
        assert_eq!(limiter.tracked_ip_count().await, 0);
    }

    #[tokio::test]
    async fn test_remove_ip() {
        let config = ThrottleConfig::new()
            .enable()
            .with_per_ip_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        limiter.check("127.0.0.1", 500).await;
        limiter.check("127.0.0.2", 500).await;
        
        assert!(limiter.remove_ip("127.0.0.1").await);
        assert_eq!(limiter.tracked_ip_count().await, 1);
        
        assert!(!limiter.remove_ip("127.0.0.1").await); // Already removed
    }

    #[tokio::test]
    async fn test_update_config() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let mut limiter = ThrottleLimiter::new(config);
        
        limiter.check("127.0.0.1", 500).await;
        
        let new_config = ThrottleConfig::new()
            .enable()
            .with_global_limit(2000)
            .with_bucket_capacity(2000);
        
        limiter.update_config(new_config);
        
        // New bucket should be full
        let tokens = limiter.global_tokens().await;
        assert_eq!(tokens, 2000);
    }

    #[tokio::test]
    async fn test_global_tokens() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        assert_eq!(limiter.global_tokens().await, 1000);
        
        limiter.consume("127.0.0.1", 300).await;
        
        assert_eq!(limiter.global_tokens().await, 700);
    }

    #[tokio::test]
    async fn test_ip_tokens() {
        let config = ThrottleConfig::new()
            .enable()
            .with_per_ip_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        
        // New IP should have full bucket
        assert_eq!(limiter.ip_tokens("127.0.0.1").await, 1000);
        
        limiter.consume("127.0.0.1", 300).await;
        
        assert_eq!(limiter.ip_tokens("127.0.0.1").await, 700);
    }

    #[tokio::test]
    async fn test_clone() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1000)
            .with_bucket_capacity(1000);
        
        let limiter = ThrottleLimiter::new(config);
        let cloned = limiter.clone();
        
        // Both should share the same buckets
        limiter.consume("127.0.0.1", 500).await;
        assert_eq!(cloned.global_tokens().await, 500);
    }

    #[test]
    fn test_throttle_result_is_allowed() {
        assert!(ThrottleResult::Allowed { remaining: 100 }.is_allowed());
        assert!(ThrottleResult::Unlimited.is_allowed());
        assert!(!ThrottleResult::Throttled { wait_ms: 100 }.is_allowed());
    }
}
