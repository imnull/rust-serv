//! Plugin manager - Simplified implementation

use crate::plugin::{
    error::{PluginError, PluginResult},
    executor::WasmExecutor,
    loader::PluginLoader,
    traits::{PluginConfig, PluginMetadata, PluginRequest, PluginResponse, PluginAction},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wasmtime::*;

/// Loaded plugin instance
pub struct LoadedPlugin {
    pub id: String,
    pub path: PathBuf,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub module: Module,
    pub stats: PluginStats,
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

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    executors: HashMap<String, WasmExecutor>,
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
            executors: HashMap::new(),
            loader: PluginLoader::new()?,
            execution_order: vec![],
            config,
        })
    }

    /// Load a plugin from file
    pub fn load(&mut self, path: &Path, config: PluginConfig) -> PluginResult<String> {
        if !self.config.enabled {
            return Err(PluginError::InvalidConfig("Plugin system disabled".into()));
        }

        if self.plugins.len() >= self.config.max_plugins {
            return Err(PluginError::Other("Maximum plugins reached".into()));
        }

        // Compile module
        let module = self.loader.compile(path)?;

        // Extract metadata
        let metadata = self.loader.extract_metadata(&module)?;
        let plugin_id = metadata.id.clone();

        // Check if already loaded
        if self.plugins.contains_key(&plugin_id) {
            return Err(PluginError::AlreadyLoaded(plugin_id));
        }

        // Create executor
        let executor = WasmExecutor::new(self.loader.engine(), module.clone(), &config)?;

        // Create loaded plugin
        let loaded = LoadedPlugin {
            id: plugin_id.clone(),
            path: path.to_path_buf(),
            metadata,
            config,
            module,
            stats: PluginStats::default(),
        };

        // Store plugin and executor
        self.plugins.insert(plugin_id.clone(), loaded);
        self.executors.insert(plugin_id.clone(), executor);

        // Update execution order
        self.update_execution_order();

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub fn unload(&mut self, plugin_id: &str) -> PluginResult<()> {
        // Unload executor
        if let Some(mut executor) = self.executors.remove(plugin_id) {
            let _ = executor.unload();
        }

        // Remove plugin
        let _plugin = self.plugins.remove(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        // Update execution order
        self.update_execution_order();

        Ok(())
    }

    /// Reload a plugin
    pub fn reload(&mut self, plugin_id: &str) -> PluginResult<()> {
        let plugin = self.plugins.get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        let path = plugin.path.clone();
        let config = plugin.config.clone();

        self.unload(plugin_id)?;
        self.load(&path, config)?;

        Ok(())
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

    /// Execute plugins on request
    pub fn on_request(&mut self, request: &mut PluginRequest) -> PluginResult<PluginAction> {
        if !self.config.enabled {
            return Ok(PluginAction::Continue);
        }

        // Execute plugins in order
        for plugin_id in self.execution_order.clone() {
            let plugin = self.plugins.get_mut(&plugin_id)
                .ok_or_else(|| PluginError::NotFound(plugin_id.clone()))?;

            if !plugin.config.enabled {
                continue;
            }

            // Get executor
            let executor = self.executors.get_mut(&plugin_id)
                .ok_or_else(|| PluginError::NotFound(plugin_id.clone()))?;

            let start = std::time::Instant::now();

            // Execute plugin
            let action = executor.on_request(request)?;

            plugin.stats.request_count += 1;
            plugin.stats.total_latency_us += start.elapsed().as_micros() as u64;

            // Handle action
            match action {
                PluginAction::Continue => continue,
                other => return Ok(other),
            }
        }

        Ok(PluginAction::Continue)
    }

    /// Execute plugins on response
    pub fn on_response(&mut self, response: &mut PluginResponse) -> PluginResult<PluginAction> {
        if !self.config.enabled {
            return Ok(PluginAction::Continue);
        }

        // Execute plugins in reverse order
        for plugin_id in self.execution_order.iter().rev().cloned() {
            let plugin = self.plugins.get_mut(&plugin_id)
                .ok_or_else(|| PluginError::NotFound(plugin_id.clone()))?;

            if !plugin.config.enabled {
                continue;
            }

            // Get executor
            let executor = self.executors.get_mut(&plugin_id)
                .ok_or_else(|| PluginError::NotFound(plugin_id.clone()))?;

            let start = std::time::Instant::now();

            // Execute plugin
            let action = executor.on_response(response)?;

            plugin.stats.response_count += 1;
            plugin.stats.total_latency_us += start.elapsed().as_micros() as u64;

            // Handle action
            match action {
                PluginAction::Continue => continue,
                other => return Ok(other),
            }
        }

        Ok(PluginAction::Continue)
    }

    /// Update plugin configuration
    pub fn update_config(&mut self, plugin_id: &str, new_config: PluginConfig) -> PluginResult<()> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        plugin.config = new_config;

        // Update execution order if priority changed
        self.update_execution_order();

        Ok(())
    }

    // Private helper methods

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

    #[test]
    fn test_manager_config() {
        let config = PluginManagerConfig {
            enabled: false,
            max_plugins: 10,
            default_timeout_ms: 50,
            auto_reload: false,
        };

        let manager = PluginManager::with_config(config);
        assert!(manager.is_ok());
        assert!(!manager.unwrap().is_enabled());
    }
}
