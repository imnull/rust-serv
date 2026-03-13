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

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PluginManager")
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_plugin_stats_default() {
        let stats = PluginStats::default();
        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.response_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.total_latency_us, 0);
    }

    #[test]
    fn test_plugin_stats_avg_latency() {
        let mut stats = PluginStats::default();
        stats.request_count = 10;
        stats.total_latency_us = 500;
        assert_eq!(stats.avg_latency_us(), 50.0);
    }

    #[test]
    fn test_plugin_stats_avg_latency_zero() {
        let stats = PluginStats::default();
        assert_eq!(stats.avg_latency_us(), 0.0);
    }

    #[test]
    fn test_manager_config_default() {
        let config = PluginManagerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_plugins, 100);
        assert_eq!(config.default_timeout_ms, 100);
        assert!(config.auto_reload);
    }

    #[test]
    fn test_manager_with_disabled_config() {
        let config = PluginManagerConfig {
            enabled: false,
            max_plugins: 10,
            default_timeout_ms: 50,
            auto_reload: false,
        };
        let manager = PluginManager::with_config(config).unwrap();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_manager_default() {
        let manager = PluginManager::default();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_manager_is_enabled() {
        let manager = PluginManager::new().unwrap();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_manager_count_empty() {
        let manager = PluginManager::new().unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_list_empty() {
        let manager = PluginManager::new().unwrap();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_manager_get_nonexistent() {
        let manager = PluginManager::new().unwrap();
        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_manager_load_when_disabled() {
        let config = PluginManagerConfig {
            enabled: false,
            ..Default::default()
        };
        let mut manager = PluginManager::with_config(config).unwrap();
        let temp_path = PathBuf::from("/tmp/test_plugin.wasm");
        let result = manager.load(&temp_path, PluginConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_load_exceeds_max() {
        let config = PluginManagerConfig {
            max_plugins: 0,
            ..Default::default()
        };
        let mut manager = PluginManager::with_config(config).unwrap();
        let temp_path = PathBuf::from("/tmp/test_plugin.wasm");
        let result = manager.load(&temp_path, PluginConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_unload_nonexistent() {
        let mut manager = PluginManager::new().unwrap();
        let result = manager.unload("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_reload_nonexistent() {
        let mut manager = PluginManager::new().unwrap();
        let result = manager.reload("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_update_config_nonexistent() {
        let mut manager = PluginManager::new().unwrap();
        let new_config = PluginConfig::default();
        let result = manager.update_config("nonexistent", new_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_on_request_when_disabled() {
        let config = PluginManagerConfig {
            enabled: false,
            ..Default::default()
        };
        let mut manager = PluginManager::with_config(config).unwrap();
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_on_response_when_disabled() {
        let config = PluginManagerConfig {
            enabled: false,
            ..Default::default()
        };
        let mut manager = PluginManager::with_config(config).unwrap();
        let mut response = PluginResponse::ok();
        let action = manager.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_on_request_empty() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_on_response_empty() {
        let mut manager = PluginManager::new().unwrap();
        let mut response = PluginResponse::ok();
        let action = manager.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_with_complex_request() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            query: [("param1".to_string(), "value1".to_string())].into_iter().collect(),
            headers: [("content-type".to_string(), "application/json".to_string())].into_iter().collect(),
            body: Some("test body".to_string()),
            client_ip: "192.168.1.1".to_string(),
            request_id: "req-123".to_string(),
            version: "HTTP/2.0".to_string(),
            host: "example.com".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_config_clone() {
        let config = PluginManagerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.max_plugins, cloned.max_plugins);
    }

    #[test]
    fn test_manager_config_debug() {
        let config = PluginManagerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("PluginManagerConfig"));
    }
}
