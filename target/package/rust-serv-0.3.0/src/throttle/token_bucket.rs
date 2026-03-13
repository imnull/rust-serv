//! Token bucket implementation for rate limiting

use std::time::{Duration, Instant};

/// Token bucket for rate limiting
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum tokens in bucket
    capacity: u64,
    /// Current tokens
    tokens: f64,
    /// Tokens added per second
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate: refill_rate as f64,
            last_refill: Instant::now(),
        }
    }

    /// Get bucket capacity
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Get refill rate (tokens per second)
    pub fn refill_rate(&self) -> u64 {
        self.refill_rate as u64
    }

    /// Get current token count
    pub fn tokens(&mut self) -> u64 {
        self.refill();
        self.tokens as u64
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f64() * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
        self.last_refill = now;
    }

    /// Try to consume tokens
    /// Returns the number of tokens actually consumed
    pub fn consume(&mut self, requested: u64) -> u64 {
        self.refill();
        
        if self.tokens >= requested as f64 {
            self.tokens -= requested as f64;
            requested
        } else {
            let consumed = self.tokens as u64;
            self.tokens = 0.0;
            consumed
        }
    }

    /// Try to consume exactly the requested amount
    /// Returns true if successful, false if not enough tokens
    pub fn try_consume(&mut self, amount: u64) -> bool {
        self.refill();
        
        if self.tokens >= amount as f64 {
            self.tokens -= amount as f64;
            true
        } else {
            false
        }
    }

    /// Wait until enough tokens are available (simulated)
    /// Returns the duration to wait
    pub fn wait_time(&mut self, amount: u64) -> Duration {
        self.refill();
        
        if self.tokens >= amount as f64 {
            return Duration::ZERO;
        }
        
        let tokens_needed = amount as f64 - self.tokens;
        let wait_secs = tokens_needed / self.refill_rate;
        
        Duration::from_secs_f64(wait_secs)
    }

    /// Reset bucket to full capacity
    pub fn reset(&mut self) {
        self.tokens = self.capacity as f64;
        self.last_refill = Instant::now();
    }

    /// Update refill rate
    pub fn set_refill_rate(&mut self, rate: u64) {
        self.refill_rate = rate as f64;
    }

    /// Update capacity
    pub fn set_capacity(&mut self, capacity: u64) {
        self.capacity = capacity;
        self.tokens = self.tokens.min(capacity as f64);
    }

    /// Check if bucket has enough tokens
    pub fn has_tokens(&mut self, amount: u64) -> bool {
        self.refill();
        self.tokens >= amount as f64
    }

    /// Get fill percentage (0.0 to 1.0)
    pub fn fill_level(&mut self) -> f64 {
        self.refill();
        self.tokens / self.capacity as f64
    }
}

impl Clone for TokenBucket {
    fn clone(&self) -> Self {
        Self {
            capacity: self.capacity,
            tokens: self.tokens,
            refill_rate: self.refill_rate,
            last_refill: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_creation() {
        let bucket = TokenBucket::new(1000, 100);
        assert_eq!(bucket.capacity(), 1000);
        assert_eq!(bucket.refill_rate(), 100);
    }

    #[test]
    fn test_bucket_starts_full() {
        let mut bucket = TokenBucket::new(1000, 100);
        assert_eq!(bucket.tokens(), 1000);
    }

    #[test]
    fn test_consume_success() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        let consumed = bucket.consume(500);
        assert_eq!(consumed, 500);
        assert_eq!(bucket.tokens(), 500);
    }

    #[test]
    fn test_consume_partial() {
        let mut bucket = TokenBucket::new(100, 10);
        
        // Try to consume more than available
        let consumed = bucket.consume(150);
        assert_eq!(consumed, 100);
        assert_eq!(bucket.tokens(), 0);
    }

    #[test]
    fn test_consume_empty_bucket() {
        let mut bucket = TokenBucket::new(100, 10);
        
        bucket.consume(100);
        assert_eq!(bucket.tokens(), 0);
        
        let consumed = bucket.consume(50);
        assert_eq!(consumed, 0);
    }

    #[test]
    fn test_try_consume_success() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        assert!(bucket.try_consume(500));
        assert_eq!(bucket.tokens(), 500);
    }

    #[test]
    fn test_try_consume_fail() {
        let mut bucket = TokenBucket::new(100, 10);
        
        assert!(!bucket.try_consume(150));
        // Should not have consumed anything
        assert_eq!(bucket.tokens(), 100);
    }

    #[test]
    fn test_refill() {
        let mut bucket = TokenBucket::new(1000, 1000); // 1000 tokens/sec
        
        bucket.consume(500);
        assert_eq!(bucket.tokens(), 500);
        
        // Wait and check refill
        std::thread::sleep(Duration::from_millis(100));
        
        // Should have refilled approximately 100 tokens (±50% tolerance for timing)
        let tokens = bucket.tokens();
        assert!(tokens > 500 && tokens < 800, "Expected ~600 tokens, got {}", tokens);
    }

    #[test]
    fn test_wait_time_zero() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        let wait = bucket.wait_time(500);
        assert_eq!(wait, Duration::ZERO);
    }

    #[test]
    fn test_wait_time_needed() {
        let mut bucket = TokenBucket::new(100, 100); // 100 tokens/sec
        
        // Consume all tokens
        bucket.consume(100);
        
        // Wait time for 100 tokens = 1 second
        let wait = bucket.wait_time(100);
        assert!(wait >= Duration::from_millis(900) && wait <= Duration::from_millis(1100));
    }

    #[test]
    fn test_reset() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        bucket.consume(500);
        assert_eq!(bucket.tokens(), 500);
        
        bucket.reset();
        assert_eq!(bucket.tokens(), 1000);
    }

    #[test]
    fn test_set_refill_rate() {
        let mut bucket = TokenBucket::new(1000, 100);
        bucket.set_refill_rate(200);
        
        assert_eq!(bucket.refill_rate(), 200);
    }

    #[test]
    fn test_set_capacity() {
        let mut bucket = TokenBucket::new(1000, 100);
        bucket.consume(500);
        
        bucket.set_capacity(300);
        
        assert_eq!(bucket.capacity(), 300);
        // Tokens should be capped at new capacity
        assert_eq!(bucket.tokens(), 300);
    }

    #[test]
    fn test_has_tokens() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        assert!(bucket.has_tokens(500));
        assert!(bucket.has_tokens(1000));
        assert!(!bucket.has_tokens(1500));
    }

    #[test]
    fn test_fill_level() {
        let mut bucket = TokenBucket::new(1000, 100);
        
        let level = bucket.fill_level();
        assert!(level > 0.99 && level <= 1.0, "Expected ~1.0, got {}", level);
        
        bucket.consume(500);
        let level = bucket.fill_level();
        assert!(level > 0.49 && level < 0.51, "Expected ~0.5, got {}", level);
        
        bucket.consume(500);
        let level = bucket.fill_level();
        assert!(level >= 0.0 && level < 0.01, "Expected ~0.0, got {}", level);
    }

    #[test]
    fn test_clone() {
        let bucket = TokenBucket::new(1000, 100);
        let cloned = bucket.clone();
        
        assert_eq!(cloned.capacity(), 1000);
        assert_eq!(cloned.refill_rate(), 100);
    }

    #[test]
    fn test_no_overflow() {
        let mut bucket = TokenBucket::new(1000, 1000);
        
        // Wait for potential refill
        std::thread::sleep(Duration::from_millis(50));
        
        // Tokens should not exceed capacity
        let tokens = bucket.tokens();
        assert!(tokens <= 1000);
    }
}
