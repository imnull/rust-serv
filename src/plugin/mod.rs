//! Plugin system for rust-serv
//!
//! This module provides WebAssembly-based plugin support with hot-reload capability.

pub mod error;
pub mod traits;
pub mod loader;
pub mod manager;
pub mod host;

pub use error::{PluginError, PluginResult};
pub use traits::{
    Plugin,
    PluginMetadata,
    PluginConfig,
    PluginRequest,
    PluginResponse,
    PluginAction,
};
pub use loader::PluginLoader;
pub use manager::PluginManager;

use serde::{Deserialize, Serialize};

/// Plugin capability flags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    /// Can modify requests
    ModifyRequest,
    
    /// Can modify responses
    ModifyResponse,
    
    /// Can intercept requests
    InterceptRequest,
    
    /// Can access configuration
    AccessConfig,
    
    /// Can log messages
    Logging,
    
    /// Can report metrics
    Metrics,
}

/// Plugin permission for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    /// Read environment variables
    ReadEnv { allowed: Vec<String> },
    
    /// Make HTTP requests
    HttpRequest { allowed_hosts: Vec<String> },
    
    /// Read files
    FileRead { allowed_paths: Vec<String> },
    
    /// Write files
    FileWrite { allowed_paths: Vec<String> },
    
    /// Access network
    NetworkAccess { allowed_ports: Vec<u16> },
}
