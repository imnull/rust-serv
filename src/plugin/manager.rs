//! Plugin manager for lifecycle management

use crate::plugin::{
    error::{PluginError, PluginResult},
    loader::PluginLoader,
    traits::{PluginConfig, PluginRequest, PluginResponse, PluginAction},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wasmtime::*;

/// Loaded plugin instance
pub struct LoadedPlugin {
    pub id: String,
    pub path: PathBuf,
    pub metadata: crate::plugin::PluginMetadata,
    pub config: PluginConfig,
    pub instance: Instance,
    pub store: Store<HostState>,
    pub stats: PluginStats,
    pub functions: PluginFunctions,
}

/// Plugin statistics
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub load_time: std::time::Instant,
    pub request_count: u64,
    pub response_count: u64,
    pub error_count: u64,
    pub total_latency_us: u64,
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            load_time: std::time::Instant::now(),
            request_count: 0,
            response_count: 0,
            error_count: 0,
            total_latency_us: 0,
        }
    }
}

impl PluginStats {
    pub fn avg_latency_us(&self) -> f64 {
        let total = self.request_count + self.response_count;
        if total == 0 {
            0.0
        } else {
            self.total_latency_us as f64 / total as f64
        }
    }
}

/// Host state for Wasm execution
#[derive(Debug)]
pub struct HostState {
    pub request_data: Option<Vec<u8>>,
    pub response_data: Option<Vec<u8>>,
    pub action_data: Option<Vec<u8>>,
}

/// Wasm function references
pub struct PluginFunctions {
    pub on_init: Option<TypedFunc<(i32, i32), i32>>,
    pub on_request: Option<TypedFunc<(i32, i32, i32, i32), i32>>,
    pub on_response: Option<TypedFunc<(i32, i32, i32, i32), i32>>,
    pub on_unload: Option<TypedFunc<(), i32>>,
}

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    loader: PluginLoader,
    execution_order: Vec<String>,
    config: PluginManagerConfig,
}

/// Plugin manager configuration
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    pub enabled: bool,
    pub max_plugins: usize,
    pub default_timeout_ms: u64,
    pub auto_reload: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_plugins: 100,
            default_timeout_ms: 100,
            auto_reload: true,
        }
    }
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> PluginResult<Self> {
        Self::with_config(PluginManagerConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PluginManagerConfig) -> PluginResult<Self> {
        Ok(Self {
            plugins: HashMap::new(),
            loader: PluginLoader::new()?,
            execution_order: vec![],
            config,
        })
    }

    /// Get plugin by ID
    pub fn get(&self, plugin_id: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(plugin_id)
    }

    /// List all loaded plugins
    pub fn list(&self) -> Vec<&LoadedPlugin> {
        self.execution_order
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .collect()
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Update execution order based on priority
    fn update_execution_order(&mut self) {
        let mut plugins: Vec<_> = self.plugins
            .values()
            .map(|p| (p.config.priority.unwrap_or(p.metadata.priority), p.id.clone()))
            .collect();

        // Sort by priority (descending)
        plugins.sort_by(|a, b| b.0.cmp(&a.0));

        self.execution_order = plugins.into_iter().map(|(_, id)| id).collect();
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PluginManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_plugin_stats() {
        let mut stats = PluginStats::default();
        stats.request_count = 10;
        stats.total_latency_us = 500;

        assert_eq!(stats.avg_latency_us(), 50.0);
    }
}
