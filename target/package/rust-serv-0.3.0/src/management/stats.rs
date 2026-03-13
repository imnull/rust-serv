//! Server statistics collection
//!
//! This module provides statistics collection for the management API.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Total number of requests served
    pub total_requests: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Server uptime in seconds
    pub uptime_secs: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
}

/// Statistics collector
#[derive(Debug)]
pub struct StatsCollector {
    /// Start time for uptime calculation
    start_time: Instant,
    /// Active connections counter
    active_connections: Arc<AtomicUsize>,
    /// Total requests counter
    total_requests: Arc<AtomicU64>,
    /// Cache hits counter
    cache_hits: Arc<AtomicU64>,
    /// Cache misses counter
    cache_misses: Arc<AtomicU64>,
    /// Bytes sent counter
    bytes_sent: Arc<AtomicU64>,
    /// Bytes received counter
    bytes_received: Arc<AtomicU64>,
    /// Server ready flag
    ready: Arc<AtomicBool>,
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            active_connections: Arc::new(AtomicUsize::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            ready: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Get the current statistics snapshot
    pub fn get_stats(&self) -> ServerStats {
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let total_cache_requests = cache_hits + cache_misses;

        let cache_hit_rate = if total_cache_requests > 0 {
            cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        };

        ServerStats {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            cache_hit_rate,
            uptime_secs: self.start_time.elapsed().as_secs(),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
        }
    }

    /// Increment active connections count
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections count
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment total requests count
    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Add to bytes sent counter
    pub fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Add to bytes received counter
    pub fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Check if the server is ready
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    /// Set the server ready state
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Relaxed);
    }

    /// Get a clone of the active connections counter
    pub fn active_connections_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.active_connections)
    }

    /// Get a clone of the total requests counter
    pub fn total_requests_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.total_requests)
    }

    /// Get a clone of the cache hits counter
    pub fn cache_hits_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.cache_hits)
    }

    /// Get a clone of the cache misses counter
    pub fn cache_misses_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.cache_misses)
    }

    /// Get a clone of the bytes sent counter
    pub fn bytes_sent_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.bytes_sent)
    }

    /// Get a clone of the bytes received counter
    pub fn bytes_received_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.bytes_received)
    }
}

impl Clone for StatsCollector {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            active_connections: Arc::clone(&self.active_connections),
            total_requests: Arc::clone(&self.total_requests),
            cache_hits: Arc::clone(&self.cache_hits),
            cache_misses: Arc::clone(&self.cache_misses),
            bytes_sent: Arc::clone(&self.bytes_sent),
            bytes_received: Arc::clone(&self.bytes_received),
            ready: Arc::clone(&self.ready),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_stats_collector_creation() {
        let collector = StatsCollector::new();
        let stats = collector.get_stats();

        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
    }

    #[test]
    fn test_increment_connections() {
        let collector = StatsCollector::new();
        collector.increment_connections();
        collector.increment_connections();

        let stats = collector.get_stats();
        assert_eq!(stats.active_connections, 2);
    }

    #[test]
    fn test_decrement_connections() {
        let collector = StatsCollector::new();
        collector.increment_connections();
        collector.increment_connections();
        collector.decrement_connections();

        let stats = collector.get_stats();
        assert_eq!(stats.active_connections, 1);
    }

    #[test]
    fn test_increment_requests() {
        let collector = StatsCollector::new();
        collector.increment_requests();
        collector.increment_requests();
        collector.increment_requests();

        let stats = collector.get_stats();
        assert_eq!(stats.total_requests, 3);
    }

    #[test]
    fn test_cache_hit_rate_no_requests() {
        let collector = StatsCollector::new();
        let stats = collector.get_stats();
        assert_eq!(stats.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_cache_hit_rate_all_hits() {
        let collector = StatsCollector::new();
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_hit();

        let stats = collector.get_stats();
        assert!((stats.cache_hit_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_all_misses() {
        let collector = StatsCollector::new();
        collector.record_cache_miss();
        collector.record_cache_miss();

        let stats = collector.get_stats();
        assert!((stats.cache_hit_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_mixed() {
        let collector = StatsCollector::new();
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();

        let stats = collector.get_stats();
        assert!((stats.cache_hit_rate - 0.6666666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_bytes_sent() {
        let collector = StatsCollector::new();
        collector.add_bytes_sent(100);
        collector.add_bytes_sent(200);

        let stats = collector.get_stats();
        assert_eq!(stats.bytes_sent, 300);
    }

    #[test]
    fn test_bytes_received() {
        let collector = StatsCollector::new();
        collector.add_bytes_received(50);
        collector.add_bytes_received(150);

        let stats = collector.get_stats();
        assert_eq!(stats.bytes_received, 200);
    }

    #[test]
    fn test_uptime() {
        let collector = StatsCollector::new();
        thread::sleep(Duration::from_millis(100));
        let stats = collector.get_stats();
        // Uptime should be at least 0 seconds (might be 0 due to rounding)
        assert!(stats.uptime_secs < 2);
    }

    #[test]
    fn test_ready_state() {
        let collector = StatsCollector::new();
        assert!(collector.is_ready());

        collector.set_ready(false);
        assert!(!collector.is_ready());

        collector.set_ready(true);
        assert!(collector.is_ready());
    }

    #[test]
    fn test_stats_collector_clone() {
        let collector = StatsCollector::new();
        collector.increment_requests();

        let cloned = collector.clone();
        cloned.increment_requests();

        let stats = collector.get_stats();
        assert_eq!(stats.total_requests, 2); // Both share the same counter
    }

    #[test]
    fn test_server_stats_serialization() {
        let stats = ServerStats {
            active_connections: 5,
            total_requests: 100,
            cache_hit_rate: 0.85,
            uptime_secs: 3600,
            bytes_sent: 1024000,
            bytes_received: 512000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"active_connections\":5"));
        assert!(json.contains("\"total_requests\":100"));
        assert!(json.contains("\"cache_hit_rate\":0.85"));
    }

    #[test]
    fn test_server_stats_deserialization() {
        let json = r#"{
            "active_connections": 10,
            "total_requests": 500,
            "cache_hit_rate": 0.75,
            "uptime_secs": 7200,
            "bytes_sent": 2048000,
            "bytes_received": 1024000
        }"#;

        let stats: ServerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.active_connections, 10);
        assert_eq!(stats.total_requests, 500);
        assert!((stats.cache_hit_rate - 0.75).abs() < f64::EPSILON);
        assert_eq!(stats.uptime_secs, 7200);
        assert_eq!(stats.bytes_sent, 2048000);
        assert_eq!(stats.bytes_received, 1024000);
    }

    #[test]
    fn test_server_stats_equality() {
        let stats1 = ServerStats {
            active_connections: 5,
            total_requests: 100,
            cache_hit_rate: 0.85,
            uptime_secs: 3600,
            bytes_sent: 1024000,
            bytes_received: 512000,
        };
        let stats2 = ServerStats {
            active_connections: 5,
            total_requests: 100,
            cache_hit_rate: 0.85,
            uptime_secs: 3600,
            bytes_sent: 1024000,
            bytes_received: 512000,
        };
        assert_eq!(stats1, stats2);
    }

    #[test]
    fn test_server_stats_clone() {
        let stats = ServerStats {
            active_connections: 5,
            total_requests: 100,
            cache_hit_rate: 0.85,
            uptime_secs: 3600,
            bytes_sent: 1024000,
            bytes_received: 512000,
        };
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn test_server_stats_debug() {
        let stats = ServerStats {
            active_connections: 5,
            total_requests: 100,
            cache_hit_rate: 0.85,
            uptime_secs: 3600,
            bytes_sent: 1024000,
            bytes_received: 512000,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("ServerStats"));
        assert!(debug_str.contains("active_connections"));
    }

    #[test]
    fn test_stats_collector_default() {
        let collector = StatsCollector::default();
        let stats = collector.get_stats();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_requests, 0);
    }

    #[test]
    fn test_concurrent_increment_requests() {
        let collector = Arc::new(StatsCollector::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let collector_clone = Arc::clone(&collector);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    collector_clone.increment_requests();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = collector.get_stats();
        assert_eq!(stats.total_requests, 1000);
    }

    #[test]
    fn test_concurrent_connections() {
        let collector = Arc::new(StatsCollector::new());
        let mut handles = vec![];

        for _ in 0..5 {
            let collector_clone = Arc::clone(&collector);
            handles.push(thread::spawn(move || {
                collector_clone.increment_connections();
                thread::sleep(Duration::from_millis(10));
                collector_clone.decrement_connections();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = collector.get_stats();
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_get_counter_clones() {
        let collector = StatsCollector::new();

        let active_conn = collector.active_connections_counter();
        active_conn.fetch_add(5, Ordering::Relaxed);

        let stats = collector.get_stats();
        assert_eq!(stats.active_connections, 5);
    }

    #[test]
    fn test_cache_counters_clone() {
        let collector = StatsCollector::new();

        let hits = collector.cache_hits_counter();
        let misses = collector.cache_misses_counter();

        hits.fetch_add(10, Ordering::Relaxed);
        misses.fetch_add(5, Ordering::Relaxed);

        let stats = collector.get_stats();
        assert!((stats.cache_hit_rate - 0.6666666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_bytes_counters_clone() {
        let collector = StatsCollector::new();

        let sent = collector.bytes_sent_counter();
        let received = collector.bytes_received_counter();

        sent.fetch_add(1000, Ordering::Relaxed);
        received.fetch_add(500, Ordering::Relaxed);

        let stats = collector.get_stats();
        assert_eq!(stats.bytes_sent, 1000);
        assert_eq!(stats.bytes_received, 500);
    }

    #[test]
    fn test_large_numbers() {
        let collector = StatsCollector::new();

        collector.add_bytes_sent(u64::MAX / 2);
        collector.add_bytes_sent(u64::MAX / 2 + 1);

        let stats = collector.get_stats();
        assert_eq!(stats.bytes_sent, u64::MAX);
    }

    #[test]
    fn test_cache_hit_rate_precision() {
        let collector = StatsCollector::new();

        // 7 hits, 3 misses = 70% hit rate
        for _ in 0..7 {
            collector.record_cache_hit();
        }
        for _ in 0..3 {
            collector.record_cache_miss();
        }

        let stats = collector.get_stats();
        assert!((stats.cache_hit_rate - 0.7).abs() < 0.0001);
    }
}
