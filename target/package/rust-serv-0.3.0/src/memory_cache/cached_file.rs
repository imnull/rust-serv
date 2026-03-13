//! Cached file representation

use std::time::{Duration, Instant};
use bytes::Bytes;

/// A cached file with metadata
#[derive(Clone, Debug)]
pub struct CachedFile {
    /// File content
    pub content: Bytes,
    /// MIME type
    pub mime_type: String,
    /// ETag for cache validation
    pub etag: String,
    /// Last modified timestamp (Unix timestamp)
    pub last_modified: u64,
    /// When this cache entry was created
    pub created_at: Instant,
    /// Time-to-live duration
    pub ttl: Duration,
    /// File size in bytes
    pub size: usize,
}

impl CachedFile {
    /// Create a new cached file
    pub fn new(
        content: Bytes,
        mime_type: String,
        etag: String,
        last_modified: u64,
        ttl: Duration,
    ) -> Self {
        let size = content.len();
        Self {
            content,
            mime_type,
            etag,
            last_modified,
            created_at: Instant::now(),
            ttl,
            size,
        }
    }

    /// Check if the cache entry has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Get the age of the cache entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if the ETag matches
    pub fn etag_matches(&self, etag: &str) -> bool {
        self.etag == etag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_file_creation() {
        let content = Bytes::from("test content");
        let cached = CachedFile::new(
            content.clone(),
            "text/plain".to_string(),
            "\"abc123\"".to_string(),
            1234567890,
            Duration::from_secs(300),
        );

        assert_eq!(cached.content, content);
        assert_eq!(cached.mime_type, "text/plain");
        assert_eq!(cached.etag, "\"abc123\"");
        assert_eq!(cached.last_modified, 1234567890);
        assert_eq!(cached.size, 12);
    }

    #[test]
    fn test_cached_file_not_expired() {
        let cached = CachedFile::new(
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        assert!(!cached.is_expired());
    }

    #[test]
    fn test_cached_file_expired() {
        let mut cached = CachedFile::new(
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(0),
        );
        // Force expiration by setting created_at to the past
        cached.created_at = Instant::now() - Duration::from_secs(1);

        assert!(cached.is_expired());
    }

    #[test]
    fn test_cached_file_age() {
        let cached = CachedFile::new(
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        let age = cached.age();
        assert!(age < Duration::from_millis(100));
    }

    #[test]
    fn test_etag_matches() {
        let cached = CachedFile::new(
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"abc123\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        assert!(cached.etag_matches("\"abc123\""));
        assert!(!cached.etag_matches("\"xyz789\""));
    }

    #[test]
    fn test_cached_file_size() {
        let content = Bytes::from("Hello, World!");
        let cached = CachedFile::new(
            content,
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        assert_eq!(cached.size, 13);
    }

    #[test]
    fn test_cached_file_clone() {
        let cached = CachedFile::new(
            Bytes::from("test"),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        let cloned = cached.clone();
        assert_eq!(cached.content, cloned.content);
        assert_eq!(cached.mime_type, cloned.mime_type);
        assert_eq!(cached.etag, cloned.etag);
    }

    #[test]
    fn test_empty_cached_file() {
        let cached = CachedFile::new(
            Bytes::new(),
            "text/plain".to_string(),
            "\"etag\"".to_string(),
            0,
            Duration::from_secs(300),
        );

        assert_eq!(cached.size, 0);
        assert_eq!(cached.content.len(), 0);
    }
}
