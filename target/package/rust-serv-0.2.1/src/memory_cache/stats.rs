//! Cache statistics for monitoring

use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of cache hits
    hits: AtomicU64,
    /// Number of cache misses
    misses: AtomicU64,
    /// Number of cache evictions
    evictions: AtomicU64,
    /// Number of cache entries
    entries: AtomicU64,
    /// Total size of cached data in bytes
    total_size: AtomicU64,
    /// Number of expired entries removed
    expired: AtomicU64,
}

impl CacheStats {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an expired entry removal
    pub fn record_expired(&self) {
        self.expired.fetch_add(1, Ordering::Relaxed);
    }

    /// Add to entry count
    pub fn add_entry(&self, size: usize) {
        self.entries.fetch_add(1, Ordering::Relaxed);
        self.total_size.fetch_add(size as u64, Ordering::Relaxed);
    }

    /// Remove from entry count
    pub fn remove_entry(&self, size: usize) {
        self.entries.fetch_sub(1, Ordering::Relaxed);
        self.total_size.fetch_sub(size as u64, Ordering::Relaxed);
    }

    /// Update size (delta can be positive or negative)
    pub fn update_size(&self, delta: i64) {
        if delta >= 0 {
            self.total_size.fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            self.total_size.fetch_sub((-delta) as u64, Ordering::Relaxed);
        }
    }

    /// Get cache hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get cache misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get cache evictions
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get number of entries
    pub fn entries(&self) -> u64 {
        self.entries.load(Ordering::Relaxed)
    }

    /// Get total size in bytes
    pub fn total_size(&self) -> u64 {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Get expired count
    pub fn expired(&self) -> u64 {
        self.expired.load(Ordering::Relaxed)
    }

    /// Calculate hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits();
        let misses = self.misses();
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get total requests
    pub fn total_requests(&self) -> u64 {
        self.hits() + self.misses()
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.entries.store(0, Ordering::Relaxed);
        self.total_size.store(0, Ordering::Relaxed);
        self.expired.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.evictions(), 0);
        assert_eq!(stats.entries(), 0);
        assert_eq!(stats.total_size(), 0);
    }

    #[test]
    fn test_record_hit() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hits(), 3);
    }

    #[test]
    fn test_record_miss() {
        let stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.misses(), 2);
    }

    #[test]
    fn test_record_eviction() {
        let stats = CacheStats::new();
        stats.record_eviction();
        assert_eq!(stats.evictions(), 1);
    }

    #[test]
    fn test_record_expired() {
        let stats = CacheStats::new();
        stats.record_expired();
        stats.record_expired();
        assert_eq!(stats.expired(), 2);
    }

    #[test]
    fn test_add_remove_entry() {
        let stats = CacheStats::new();
        
        stats.add_entry(100);
        assert_eq!(stats.entries(), 1);
        assert_eq!(stats.total_size(), 100);
        
        stats.add_entry(200);
        assert_eq!(stats.entries(), 2);
        assert_eq!(stats.total_size(), 300);
        
        stats.remove_entry(100);
        assert_eq!(stats.entries(), 1);
        assert_eq!(stats.total_size(), 200);
    }

    #[test]
    fn test_update_size_positive() {
        let stats = CacheStats::new();
        stats.update_size(100);
        assert_eq!(stats.total_size(), 100);
    }

    #[test]
    fn test_update_size_negative() {
        let stats = CacheStats::new();
        stats.add_entry(200);
        stats.update_size(-50);
        assert_eq!(stats.total_size(), 150);
    }

    #[test]
    fn test_hit_rate_no_requests() {
        let stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_all_hits() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_hit_rate_all_misses() {
        let stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_mixed() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        stats.record_miss();
        // 2 hits, 2 misses = 50% hit rate
        assert!((stats.hit_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_total_requests() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.total_requests(), 3);
    }

    #[test]
    fn test_reset() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.add_entry(100);
        stats.record_eviction();
        
        stats.reset();
        
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.entries(), 0);
        assert_eq!(stats.total_size(), 0);
        assert_eq!(stats.evictions(), 0);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let stats = Arc::new(CacheStats::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let stats_clone = Arc::clone(&stats);
            handles.push(thread::spawn(move || {
                stats_clone.record_hit();
                stats_clone.record_miss();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(stats.hits(), 10);
        assert_eq!(stats.misses(), 10);
    }
}
