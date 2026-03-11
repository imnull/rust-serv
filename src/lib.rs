//! Rust HTTP Static Server
//!
//! A high-performance, secure HTTP static file server built with Rust.

pub mod config;
pub mod error;
pub mod file_service;
pub mod handler;
pub mod memory_cache;
pub mod metrics;
pub mod middleware;
pub mod mime_types;
pub mod path_security;
pub mod server;
pub mod utils;

pub use config::Config;
pub use error::{Error, Result};
pub use memory_cache::{CacheConfig, CacheStats, CachedFile, MemoryCache};
pub use metrics::{Counter, Gauge, Histogram, MetricsCollector, PrometheusExporter};
pub use server::Server;
