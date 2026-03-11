//! Reverse proxy module
//!
//! This module provides reverse proxy capabilities for forwarding requests to backend services.

mod config;
mod handler;

pub use config::ProxyConfig;
pub use handler::ProxyHandler;
