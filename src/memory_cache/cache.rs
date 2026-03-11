//! Memory cache with LRU eviction strategy

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use bytes::Bytes;

use super::cached_file::CachedFile;
use super::stats::CacheStats;

/// Configuration for memory cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,
    /// Maximum total size in bytes
    pub max_size: usize,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_size: 100 * 1024 * 1024, // 100 MB
            default_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// LRU cache entry metadata
#[derive(Debug, Clone)]
struct LruEntry {
    /// Cached file data
    file: CachedFile,
}

/// Thread-safe memory cache with LRU eviction
pub struct MemoryCache {
    /// Cache storage
    cache: RwLock<HashMap<PathBuf, LruEntry>>,
    /// Access order for LRU (most recently used at the end)
    access_order: RwLock<Vec<PathBuf>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<CacheStats>,
}

impl MemoryCache {
    /// Create a new memory cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new memory cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            access_order: RwLock::new(Vec::new()),
            config,
            stats: Arc::new(CacheStats::new()),
        }
    }

    /// Get a cached file by path
    pub fn get(&self, path: &PathBuf) -> Option<CachedFile> {
        let cache = self.cache.read().unwrap();
        
        if let Some(entry) = cache.get(path) {
            // Check if expired
            if entry.file.is_expired() {
                drop(cache);
                self.stats.record_miss();
                self.remove(path);
                return None;
            }
            
            let file = entry.file.clone();
            drop(cache);
            
            // Update access order
            self.touch(path);
            self.stats.record_hit();
            
            return Some(file);
        }
        
        self.stats.record_miss();
        None
    }

    /// Insert a file into the cache
    pub fn insert(&self, path: PathBuf, content: Bytes, mime_type: String, etag: String, last_modified: u64) {
        self.insert_with_ttl(path, content, mime_type, etag, last_modified, self.config.default_ttl)
    }

    /// Insert a file into the cache with custom TTL
    pub fn insert_with_ttl(
        &self,
        path: PathBuf,
        content: Bytes,
        mime_type: String,
        etag: String,
        last_modified: u64,
        ttl: Duration,
    ) {
        let size = content.len();
        
        // Check if we need to evict entries
        self.evict_if_needed(size);
        
        let file = CachedFile::new(content, mime_type, etag, last_modified, ttl);
        let entry = LruEntry { file };
        
        {
            let mut cache = self.cache.write().unwrap();
            
            // If path already exists, remove old entry first
            if cache.contains_key(&path) {
                if let Some(old) = cache.get(&path) {
                    self.stats.remove_entry(old.file.size);
                }
                cache.remove(&path);
                self.remove_from_access_order(&path);
            }
            
            cache.insert(path.clone(), entry);
        }
        
        // Add to access order
        {
            let mut order = self.access_order.write().unwrap();
            order.push(path);
        }
        
        self.stats.add_entry(size);
    }

    /// Remove a file from the cache
    pub fn remove(&self, path: &PathBuf) -> bool {
        let mut cache = self.cache.write().unwrap();
        
        if let Some(entry) = cache.remove(path) {
            self.stats.remove_entry(entry.file.size);
            drop(cache);
            self.remove_from_access_order(path);
            return true;
        }
        
        false
    }

    /// Check if a path is cached
    pub fn contains(&self, path: &PathBuf) -> bool {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(path) {
            !entry.file.is_expired()
        } else {
            false
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut order = self.access_order.write().unwrap();
        
        cache.clear();
        order.clear();
        
        self.stats.reset();
    }

    /// Get cache statistics
    pub fn stats(&self) -> Arc<CacheStats> {
        Arc::clone(&self.stats)
    }

    /// Get current number of entries
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove expired entries
    pub fn remove_expired(&self) -> usize {
        let cache = self.cache.read().unwrap();
        let expired: Vec<PathBuf> = cache
            .iter()
            .filter(|(_, entry)| entry.file.is_expired())
            .map(|(path, _)| path.clone())
            .collect();
        drop(cache);
        
        let count = expired.len();
        for path in expired {
            self.remove(&path);
            self.stats.record_expired();
        }
        
        count
    }

    /// Touch a path (move to end of access order)
    fn touch(&self, path: &PathBuf) {
        self.remove_from_access_order(path);
        let mut order = self.access_order.write().unwrap();
        order.push(path.clone());
    }

    /// Remove from access order
    fn remove_from_access_order(&self, path: &PathBuf) {
        let mut order = self.access_order.write().unwrap();
        order.retain(|p| p != path);
    }

    /// Evict entries if needed
    fn evict_if_needed(&self, new_size: usize) {
        // Evict by entry count
        while self.len() >= self.config.max_entries {
            self.evict_one();
        }
        
        // Evict by size
        let current_size = self.stats.total_size() as usize;
        while current_size + new_size > self.config.max_size && self.len() > 0 {
            self.evict_one();
        }
    }

    /// Evict one entry (LRU)
    fn evict_one(&self) {
        let path = {
            let order = self.access_order.read().unwrap();
            if order.is_empty() {
                return;
            }
            order.first().unwrap().clone()
        };
        
        self.remove(&path);
        self.stats.record_eviction();
    }

    /// Get configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cache() -> MemoryCache {
        MemoryCache::with_config(CacheConfig {
            max_entries: 10,
            max_size: 1024, // 1 KB for testing
            default_ttl: Duration::from_secs(60),
        })
    }

    fn make_path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn test_cache_creation() {
        let cache = MemoryCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_with_config() {
        let config = CacheConfig {
            max_entries: 100,
            max_size: 1024 * 1024,
            default_ttl: Duration::from_secs(120),
        };
        let cache = MemoryCache::with_config(config);
        assert_eq!(cache.config().max_entries, 100);
    }

    #[test]
    fn test_insert_and_get() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        cache.insert(
            path.clone(),
            Bytes::from("Hello, World!"),
            "text/plain".to_string(),
            "\"etag1\"".to_string(),
            1234567890,
        );
        
        assert_eq!(cache.len(), 1);
        
        let cached = cache.get(&path).unwrap();
        assert_eq!(cached.content, Bytes::from("Hello, World!"));
        assert_eq!(cached.mime_type, "text/plain");
    }

    #[test]
    fn test_get_nonexistent() {
        let cache = create_test_cache();
        let path = make_path("/nonexistent.txt");
        
        let result = cache.get(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_remove() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        cache.insert(
            path.clone(),
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
        );
        
        assert_eq!(cache.len(), 1);
        
        let removed = cache.remove(&path);
        assert!(removed);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let cache = create_test_cache();
        let path = make_path("/nonexistent.txt");
        
        let removed = cache.remove(&path);
        assert!(!removed);
    }

    #[test]
    fn test_contains() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        assert!(!cache.contains(&path));
        
        cache.insert(
            path.clone(),
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
        );
        
        assert!(cache.contains(&path));
    }

    #[test]
    fn test_clear() {
        let cache = create_test_cache();
        
        for i in 0..5 {
            let path = make_path(&format!("/file{}.txt", i));
            cache.insert(
                path,
                Bytes::from("test"),
                "text/plain".to_string(),
                "\"etag\"".to_string(),
                0,
            );
        }
        
        assert_eq!(cache.len(), 5);
        
        cache.clear();
        
        assert!(cache.is_empty());
    }

    #[test]
    fn test_stats_hits_and_misses() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        // Miss
        cache.get(&path);
        assert_eq!(cache.stats().misses(), 1);
        
        // Insert
        cache.insert(
            path.clone(),
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
        );
        
        // Hit
        cache.get(&path);
        assert_eq!(cache.stats().hits(), 1);
        assert_eq!(cache.stats().misses(), 1);
    }

    #[test]
    fn test_lru_eviction_by_count() {
        let cache = create_test_cache();
        
        // Insert 10 entries (max_entries = 10)
        for i in 0..10 {
            let path = make_path(&format!("/file{}.txt", i));
            cache.insert(
                path,
                Bytes::from("x"),
                "text/plain".to_string(),
                format!("\"etag{}\"", i),
                0,
            );
        }
        
        assert_eq!(cache.len(), 10);
        
        // Insert one more - should evict the oldest
        let new_path = make_path("/new.txt");
        cache.insert(
            new_path.clone(),
            Bytes::from("y"),
            "text/plain".to_string(),
            "\"newetag\"".to_string(),
            0,
        );
        
        assert_eq!(cache.len(), 10);
        
        // First entry should be evicted
        let first_path = make_path("/file0.txt");
        assert!(!cache.contains(&first_path));
        
        // New entry should exist
        assert!(cache.contains(&new_path));
        
        // Should have recorded an eviction
        assert!(cache.stats().evictions() > 0);
    }

    #[test]
    fn test_lru_access_order() {
        let cache = create_test_cache();
        
        // Insert 3 entries
        let path1 = make_path("/file1.txt");
        let path2 = make_path("/file2.txt");
        let path3 = make_path("/file3.txt");
        
        cache.insert(path1.clone(), Bytes::from("a"), "text/plain".to_string(), "\"e1\"".to_string(), 0);
        cache.insert(path2.clone(), Bytes::from("b"), "text/plain".to_string(), "\"e2\"".to_string(), 0);
        cache.insert(path3.clone(), Bytes::from("c"), "text/plain".to_string(), "\"e3\"".to_string(), 0);
        
        // Access file1 to move it to the end
        cache.get(&path1);
        
        // Insert entries until eviction
        for i in 4..=12 {
            let path = make_path(&format!("/file{}.txt", i));
            cache.insert(path, Bytes::from("x"), "text/plain".to_string(), format!("\"e{}\"", i), 0);
        }
        
        // file2 should be evicted before file1 (file1 was accessed recently)
        assert!(cache.contains(&path1));
    }

    #[test]
    fn test_size_eviction() {
        let config = CacheConfig {
            max_entries: 100,
            max_size: 10, // Very small for testing
            default_ttl: Duration::from_secs(60),
        };
        let cache = MemoryCache::with_config(config);
        
        // Insert a 5-byte entry
        let path1 = make_path("/file1.txt");
        cache.insert(path1.clone(), Bytes::from("12345"), "text/plain".to_string(), "\"e1\"".to_string(), 0);
        
        assert_eq!(cache.len(), 1);
        
        // Insert another 5-byte entry
        let path2 = make_path("/file2.txt");
        cache.insert(path2.clone(), Bytes::from("67890"), "text/plain".to_string(), "\"e2\"".to_string(), 0);
        
        // Total size is now 10 bytes (max)
        assert_eq!(cache.len(), 2);
        
        // Insert a 6-byte entry - should evict file1
        let path3 = make_path("/file3.txt");
        cache.insert(path3.clone(), Bytes::from("123456"), "text/plain".to_string(), "\"e3\"".to_string(), 0);
        
        // file1 should be evicted due to size
        assert!(!cache.contains(&path1));
    }

    #[test]
    fn test_expired_entry() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        // Insert with very short TTL
        cache.insert_with_ttl(
            path.clone(),
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_millis(1),
        );
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));
        
        // Get should return None
        let result = cache.get(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_expired() {
        let cache = create_test_cache();
        
        // Insert entries with different TTLs
        let path1 = make_path("/long.txt");
        let path2 = make_path("/short.txt");
        
        cache.insert_with_ttl(
            path1.clone(),
            Bytes::from("test1"),
            "text/plain".to_string(),
            "\"etag1\"".to_string(),
            0,
            Duration::from_secs(60),
        );
        
        cache.insert_with_ttl(
            path2.clone(),
            Bytes::from("test2"),
            "text/plain".to_string(),
            "\"etag2\"".to_string(),
            0,
            Duration::from_millis(1),
        );
        
        // Wait for short to expire
        std::thread::sleep(Duration::from_millis(10));
        
        // Remove expired
        let removed = cache.remove_expired();
        
        assert_eq!(removed, 1);
        assert!(cache.contains(&path1));
        assert!(!cache.contains(&path2));
    }

    #[test]
    fn test_update_existing_entry() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        cache.insert(
            path.clone(),
            Bytes::from("original"),
            "text/plain".to_string(),
            "\"etag1\"".to_string(),
            0,
        );
        
        assert_eq!(cache.len(), 1);
        
        cache.insert(
            path.clone(),
            Bytes::from("updated"),
            "text/plain".to_string(),
            "\"etag2\"".to_string(),
            0,
        );
        
        assert_eq!(cache.len(), 1);
        
        let cached = cache.get(&path).unwrap();
        assert_eq!(cached.content, Bytes::from("updated"));
        assert_eq!(cached.etag, "\"etag2\"");
    }

    #[test]
    fn test_stats_evictions() {
        let cache = create_test_cache();
        
        // Fill cache
        for i in 0..10 {
            let path = make_path(&format!("/file{}.txt", i));
            cache.insert(
                path,
                Bytes::from("x"),
                "text/plain".to_string(),
                format!("\"etag{}\"", i),
                0,
            );
        }
        
        // Insert one more to trigger eviction
        let path = make_path("/overflow.txt");
        cache.insert(path, Bytes::from("y"), "text/plain".to_string(), "\"overflow\"".to_string(), 0);
        
        assert!(cache.stats().evictions() > 0);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let cache = Arc::new(create_test_cache());
        let mut handles = vec![];

        // Spawn multiple threads for concurrent writes
        for i in 0..5 {
            let cache_clone = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                let path = make_path(&format!("/file{}.txt", i));
                cache_clone.insert(
                    path,
                    Bytes::from(format!("content{}", i)),
                    "text/plain".to_string(),
                    format!("\"etag{}\"", i),
                    0,
                );
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(cache.len(), 5);
    }

    #[test]
    fn test_concurrent_reads() {
        use std::thread;

        let cache = Arc::new(create_test_cache());
        
        // Insert a file
        let path = make_path("/test.txt");
        cache.insert(
            path.clone(),
            Bytes::from("test content"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
        );

        let mut handles = vec![];

        // Spawn multiple threads for concurrent reads
        for _ in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let path_clone = path.clone();
            handles.push(thread::spawn(move || {
                cache_clone.get(&path_clone)
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        
        // All reads should succeed
        assert!(results.iter().all(|r| r.is_some()));
        assert_eq!(cache.stats().hits(), 10);
    }

    #[test]
    fn test_stats_total_size() {
        let cache = create_test_cache();
        
        cache.insert(
            make_path("/file1.txt"),
            Bytes::from("12345"), // 5 bytes
            "text/plain".to_string(),
            "\"e1\"".to_string(),
            0,
        );
        
        assert_eq!(cache.stats().total_size(), 5);
        
        cache.insert(
            make_path("/file2.txt"),
            Bytes::from("67890"), // 5 bytes
            "text/plain".to_string(),
            "\"e2\"".to_string(),
            0,
        );
        
        assert_eq!(cache.stats().total_size(), 10);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let cache = create_test_cache();
        let path = make_path("/test.txt");
        
        // 2 misses
        cache.get(&path);
        cache.get(&path);
        
        // Insert
        cache.insert(
            path.clone(),
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
        );
        
        // 3 hits
        cache.get(&path);
        cache.get(&path);
        cache.get(&path);
        
        // Hit rate should be 60% (3 hits / 5 total requests)
        let expected = 3.0 / 5.0;
        assert!((cache.stats().hit_rate() - expected).abs() < 0.001);
    }
}
