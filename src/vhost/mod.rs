//! Virtual Host module
//!
//! This module provides multi-site/virtual host support.

mod config;
mod host;
mod matcher;

pub use config::VHostConfig;
pub use host::VirtualHost;
pub use matcher::HostMatcher;
