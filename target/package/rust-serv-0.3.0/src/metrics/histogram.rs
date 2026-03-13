//! Histogram metric - distribution of values

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Histogram bucket configuration
#[derive(Debug, Clone)]
pub struct BucketConfig {
    /// Bucket boundaries (upper bounds)
    pub boundaries: Vec<f64>,
}

impl Default for BucketConfig {
    fn default() -> Self {
        // Default Prometheus-style buckets for response times in seconds
        Self {
            boundaries: vec![
                0.001,  // 1ms
                0.005,  // 5ms
                0.01,   // 10ms
                0.025,  // 25ms
                0.05,   // 50ms
                0.1,    // 100ms
                0.25,   // 250ms
                0.5,    // 500ms
                1.0,    // 1s
                2.5,    // 2.5s
                5.0,    // 5s
                10.0,   // 10s
            ],
        }
    }
}

impl BucketConfig {
    /// Create custom bucket configuration
    pub fn new(boundaries: Vec<f64>) -> Self {
        Self { boundaries }
    }
}

/// A histogram samples observations and counts them in configurable buckets
#[derive(Debug)]
pub struct Histogram {
    /// Bucket counts
    buckets: Vec<AtomicU64>,
    /// Bucket boundaries
    boundaries: Vec<f64>,
    /// Sum of all observed values
    sum: RwLock<f64>,
    /// Count of observations
    count: AtomicU64,
    /// Metric name
    name: String,
    /// Help text
    help: String,
}

impl Histogram {
    /// Create a new histogram with default buckets
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self::with_buckets(name, help, BucketConfig::default())
    }

    /// Create a new histogram with custom buckets
    pub fn with_buckets(
        name: impl Into<String>,
        help: impl Into<String>,
        config: BucketConfig,
    ) -> Self {
        let num_buckets = config.boundaries.len() + 1; // +1 for +Inf bucket
        Self {
            buckets: (0..num_buckets).map(|_| AtomicU64::new(0)).collect(),
            boundaries: config.boundaries,
            sum: RwLock::new(0.0),
            count: AtomicU64::new(0),
            name: name.into(),
            help: help.into(),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        // Find the appropriate bucket
        let bucket_idx = self.find_bucket(value);
        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        
        // Update sum and count
        self.count.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut sum) = self.sum.write() {
            *sum += value;
        }
    }

    /// Find the bucket index for a value
    fn find_bucket(&self, value: f64) -> usize {
        for (idx, &boundary) in self.boundaries.iter().enumerate() {
            if value <= boundary {
                return idx;
            }
        }
        // Value exceeds all boundaries, goes to +Inf bucket
        self.boundaries.len()
    }

    /// Get bucket counts
    pub fn bucket_counts(&self) -> Vec<u64> {
        self.buckets.iter().map(|b| b.load(Ordering::Relaxed)).collect()
    }

    /// Get bucket boundaries
    pub fn boundaries(&self) -> &[f64] {
        &self.boundaries
    }

    /// Get sum of all values
    pub fn sum(&self) -> f64 {
        self.sum.read().map(|s| *s).unwrap_or(0.0)
    }

    /// Get count of observations
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get histogram name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get help text
    pub fn help(&self) -> &str {
        &self.help
    }

    /// Calculate percentile (approximate)
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if p < 0.0 || p > 100.0 {
            return None;
        }

        let total = self.count();
        if total == 0 {
            return None;
        }

        let target = (p / 100.0) * total as f64;
        let mut cumulative = 0u64;

        for (idx, count) in self.bucket_counts().iter().enumerate() {
            cumulative += *count;
            if cumulative as f64 >= target {
                if idx < self.boundaries.len() {
                    return Some(self.boundaries[idx]);
                } else {
                    // +Inf bucket
                    return Some(f64::INFINITY);
                }
            }
        }

        None
    }

    /// Reset the histogram
    pub fn reset(&self) {
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        self.count.store(0, Ordering::Relaxed);
        if let Ok(mut sum) = self.sum.write() {
            *sum = 0.0;
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets.iter().map(|b| AtomicU64::new(b.load(Ordering::Relaxed))).collect(),
            boundaries: self.boundaries.clone(),
            sum: RwLock::new(*self.sum.read().unwrap()),
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
            name: self.name.clone(),
            help: self.help.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::EPSILON;

    #[test]
    fn test_histogram_creation() {
        let hist = Histogram::new("request_duration", "Request duration in seconds");
        assert_eq!(hist.count(), 0);
        assert_eq!(hist.sum(), 0.0);
        assert_eq!(hist.name(), "request_duration");
        assert!(!hist.boundaries().is_empty());
    }

    #[test]
    fn test_histogram_observe() {
        let hist = Histogram::new("test", "test histogram");
        hist.observe(0.1);
        hist.observe(0.2);
        
        assert_eq!(hist.count(), 2);
        assert!((hist.sum() - 0.3).abs() < EPSILON);
    }

    #[test]
    fn test_histogram_bucket_distribution() {
        let config = BucketConfig::new(vec![0.1, 0.5, 1.0]);
        let hist = Histogram::with_buckets("test", "test", config);
        
        // Observe values in different buckets
        hist.observe(0.05);  // <= 0.1
        hist.observe(0.3);   // <= 0.5
        hist.observe(0.7);   // <= 1.0
        hist.observe(2.0);   // > 1.0 (+Inf)
        
        let counts = hist.bucket_counts();
        assert_eq!(counts[0], 1); // <= 0.1
        assert_eq!(counts[1], 1); // <= 0.5
        assert_eq!(counts[2], 1); // <= 1.0
        assert_eq!(counts[3], 1); // +Inf
    }

    #[test]
    fn test_histogram_percentile() {
        let config = BucketConfig::new(vec![0.1, 0.5, 1.0]);
        let hist = Histogram::with_buckets("test", "test", config);
        
        // Add some values
        for _ in 0..10 {
            hist.observe(0.05);
        }
        for _ in 0..10 {
            hist.observe(0.3);
        }
        for _ in 0..10 {
            hist.observe(0.8);
        }
        
        // P50 should be around 0.3
        let p50 = hist.percentile(50.0).unwrap();
        assert!((p50 - 0.3).abs() < EPSILON || (p50 - 0.5).abs() < EPSILON);
        
        // P99 should be <= 1.0
        let p99 = hist.percentile(99.0).unwrap();
        assert!(p99 <= 1.0 || p99.is_infinite());
    }

    #[test]
    fn test_histogram_percentile_empty() {
        let hist = Histogram::new("test", "test");
        assert!(hist.percentile(50.0).is_none());
    }

    #[test]
    fn test_histogram_percentile_invalid() {
        let hist = Histogram::new("test", "test");
        hist.observe(0.1);
        
        assert!(hist.percentile(-1.0).is_none());
        assert!(hist.percentile(101.0).is_none());
    }

    #[test]
    fn test_histogram_reset() {
        let hist = Histogram::new("test", "test");
        hist.observe(0.1);
        hist.observe(0.2);
        
        assert_eq!(hist.count(), 2);
        
        hist.reset();
        
        assert_eq!(hist.count(), 0);
        assert_eq!(hist.sum(), 0.0);
        assert!(hist.bucket_counts().iter().all(|&c| c == 0));
    }

    #[test]
    fn test_histogram_clone() {
        let hist = Histogram::new("test", "test");
        hist.observe(0.5);
        
        let cloned = hist.clone();
        assert_eq!(cloned.count(), 1);
        assert!((cloned.sum() - 0.5).abs() < EPSILON);
        assert_eq!(cloned.name(), "test");
    }

    #[test]
    fn test_histogram_concurrent_observe() {
        use std::sync::Arc;
        use std::thread;

        let hist = Arc::new(Histogram::new("test", "test"));
        let mut handles = vec![];

        for _ in 0..10 {
            let hist_clone = Arc::clone(&hist);
            handles.push(thread::spawn(move || {
                hist_clone.observe(0.1);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(hist.count(), 10);
        assert!((hist.sum() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_custom_bucket_config() {
        let config = BucketConfig::new(vec![0.01, 0.1, 1.0]);
        let hist = Histogram::with_buckets("test", "test", config);
        
        assert_eq!(hist.boundaries().len(), 3);
        assert_eq!(hist.bucket_counts().len(), 4); // 3 boundaries + +Inf
    }

    #[test]
    fn test_histogram_large_values() {
        let hist = Histogram::new("test", "test");
        hist.observe(1000.0);
        hist.observe(10000.0);
        
        assert_eq!(hist.count(), 2);
        assert!((hist.sum() - 11000.0).abs() < EPSILON);
    }

    #[test]
    fn test_histogram_zero_value() {
        let hist = Histogram::new("test", "test");
        hist.observe(0.0);
        
        assert_eq!(hist.count(), 1);
        assert_eq!(hist.sum(), 0.0);
    }
}
