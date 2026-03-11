//! Memory cache module for high-performance file caching
//! 
//! This module provides an in-memory LRU cache for frequently accessed files,
//! reducing disk I/O and improving response times.

mod cache;
mod cached_file;
mod stats;

pub use cache::{CacheConfig, MemoryCache};
pub use cached_file::CachedFile;
pub use stats::CacheStats;
