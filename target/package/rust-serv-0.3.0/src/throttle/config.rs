//! Throttle configuration

use std::time::Duration;

/// Bandwidth throttle configuration
#[derive(Debug, Clone)]
pub struct ThrottleConfig {
    /// Global bandwidth limit in bytes per second (0 = unlimited)
    pub global_limit: u64,
    /// Per-IP bandwidth limit in bytes per second (0 = unlimited)
    pub per_ip_limit: u64,
    /// Bucket capacity (burst size) in bytes
    pub bucket_capacity: u64,
    /// Refill interval
    pub refill_interval: Duration,
    /// Enabled flag
    pub enabled: bool,
}

impl ThrottleConfig {
    /// Create a new throttle config
    pub fn new() -> Self {
        Self {
            global_limit: 0,
            per_ip_limit: 0,
            bucket_capacity: 64 * 1024, // 64KB burst
            refill_interval: Duration::from_millis(100),
            enabled: false,
        }
    }

    /// Set global bandwidth limit (bytes/sec)
    pub fn with_global_limit(mut self, limit: u64) -> Self {
        self.global_limit = limit;
        self
    }

    /// Set per-IP bandwidth limit (bytes/sec)
    pub fn with_per_ip_limit(mut self, limit: u64) -> Self {
        self.per_ip_limit = limit;
        self
    }

    /// Set bucket capacity
    pub fn with_bucket_capacity(mut self, capacity: u64) -> Self {
        self.bucket_capacity = capacity;
        self
    }

    /// Set refill interval
    pub fn with_refill_interval(mut self, interval: Duration) -> Self {
        self.refill_interval = interval;
        self
    }

    /// Enable throttling
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable throttling
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if global limit is set
    pub fn has_global_limit(&self) -> bool {
        self.enabled && self.global_limit > 0
    }

    /// Check if per-IP limit is set
    pub fn has_per_ip_limit(&self) -> bool {
        self.enabled && self.per_ip_limit > 0
    }

    /// Check if any limit is set
    pub fn is_active(&self) -> bool {
        self.enabled && (self.global_limit > 0 || self.per_ip_limit > 0)
    }

    /// Calculate tokens to add per interval
    pub fn tokens_per_interval(&self, limit: u64) -> u64 {
        let intervals_per_sec = 1000 / self.refill_interval.as_millis() as u64;
        limit / intervals_per_sec
    }

    /// Create a human-readable limit string
    pub fn format_limit(bytes_per_sec: u64) -> String {
        if bytes_per_sec == 0 {
            return "unlimited".to_string();
        }
        
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;
        
        if bytes_per_sec >= GB {
            format!("{:.1} GB/s", bytes_per_sec as f64 / GB as f64)
        } else if bytes_per_sec >= MB {
            format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
        } else if bytes_per_sec >= KB {
            format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
        } else {
            format!("{} B/s", bytes_per_sec)
        }
    }
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ThrottleConfig::new();
        assert_eq!(config.global_limit, 0);
        assert_eq!(config.per_ip_limit, 0);
        assert!(!config.enabled);
    }

    #[test]
    fn test_config_with_global_limit() {
        let config = ThrottleConfig::new()
            .with_global_limit(1024 * 1024); // 1 MB/s
        
        assert_eq!(config.global_limit, 1024 * 1024);
    }

    #[test]
    fn test_config_with_per_ip_limit() {
        let config = ThrottleConfig::new()
            .with_per_ip_limit(512 * 1024); // 512 KB/s
        
        assert_eq!(config.per_ip_limit, 512 * 1024);
    }

    #[test]
    fn test_config_with_bucket_capacity() {
        let config = ThrottleConfig::new()
            .with_bucket_capacity(128 * 1024);
        
        assert_eq!(config.bucket_capacity, 128 * 1024);
    }

    #[test]
    fn test_config_with_refill_interval() {
        let config = ThrottleConfig::new()
            .with_refill_interval(Duration::from_millis(50));
        
        assert_eq!(config.refill_interval, Duration::from_millis(50));
    }

    #[test]
    fn test_config_enable() {
        let config = ThrottleConfig::new().enable();
        assert!(config.enabled);
    }

    #[test]
    fn test_config_disable() {
        let config = ThrottleConfig::new().enable().disable();
        assert!(!config.enabled);
    }

    #[test]
    fn test_has_global_limit() {
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1024);
        
        assert!(config.has_global_limit());
        assert!(!config.has_per_ip_limit());
    }

    #[test]
    fn test_has_per_ip_limit() {
        let config = ThrottleConfig::new()
            .enable()
            .with_per_ip_limit(1024);
        
        assert!(!config.has_global_limit());
        assert!(config.has_per_ip_limit());
    }

    #[test]
    fn test_has_limit_disabled() {
        let config = ThrottleConfig::new()
            .with_global_limit(1024); // Not enabled
        
        assert!(!config.has_global_limit());
    }

    #[test]
    fn test_is_active() {
        // Not enabled
        let config = ThrottleConfig::new()
            .with_global_limit(1024);
        assert!(!config.is_active());
        
        // Enabled with limit
        let config = ThrottleConfig::new()
            .enable()
            .with_global_limit(1024);
        assert!(config.is_active());
        
        // Enabled but no limit
        let config = ThrottleConfig::new().enable();
        assert!(!config.is_active());
    }

    #[test]
    fn test_tokens_per_interval() {
        let config = ThrottleConfig::new()
            .with_refill_interval(Duration::from_millis(100));
        
        // 1000 bytes/sec with 100ms interval = 100 tokens per interval
        assert_eq!(config.tokens_per_interval(1000), 100);
        
        // 10 MB/sec with 100ms interval = 1 MB per interval
        assert_eq!(config.tokens_per_interval(10 * 1024 * 1024), 1024 * 1024);
    }

    #[test]
    fn test_format_limit_unlimited() {
        assert_eq!(ThrottleConfig::format_limit(0), "unlimited");
    }

    #[test]
    fn test_format_limit_bytes() {
        assert_eq!(ThrottleConfig::format_limit(500), "500 B/s");
    }

    #[test]
    fn test_format_limit_kb() {
        let result = ThrottleConfig::format_limit(1024);
        assert!(result.contains("KB/s"));
    }

    #[test]
    fn test_format_limit_mb() {
        let result = ThrottleConfig::format_limit(1024 * 1024);
        assert!(result.contains("MB/s"));
    }

    #[test]
    fn test_format_limit_gb() {
        let result = ThrottleConfig::format_limit(1024 * 1024 * 1024);
        assert!(result.contains("GB/s"));
    }

    #[test]
    fn test_default() {
        let config = ThrottleConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.global_limit, 0);
    }
}
