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
        let mut metadata = self.loader.extract_metadata(&module)?;
        
        // Generate unique plugin ID based on file path if metadata ID is "unknown"
        let plugin_id = if metadata.id == "unknown" {
            // Use file stem + hash of full path to create unique ID
            let file_stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("plugin");
            let path_hash = format!("{:x}", crc32fast::hash(path.to_string_lossy().as_bytes()));
            // Ensure we don't panic if hash is short
            let hash_prefix = if path_hash.len() >= 8 {
                &path_hash[..8]
            } else {
                &path_hash
            };
            format!("{}-{}", file_stem, hash_prefix)
        } else {
            metadata.id.clone()
        };
        metadata.id = plugin_id.clone();

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
    fn test_manager_plugin_stats_increment() {
        let mut stats = PluginStats::default();
        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.error_count, 0);
        
        // Simulate some requests
        stats.request_count += 1;
        stats.total_latency_us = 100;
        assert_eq!(stats.request_count, 1);
        assert_eq!(stats.avg_latency_us(), 100.0);
        
        stats.request_count += 1;
        stats.total_latency_us += 200;
        assert_eq!(stats.request_count, 2);
        assert_eq!(stats.avg_latency_us(), 150.0);
    }

    #[test]
    fn test_manager_plugin_stats_error_count() {
        let mut stats = PluginStats::default();
        assert_eq!(stats.error_count, 0);
        
        stats.error_count += 1;
        assert_eq!(stats.error_count, 1);
        
        stats.error_count += 5;
        assert_eq!(stats.error_count, 6);
    }

    #[test]
    fn test_manager_config_with_max_plugins_boundary() {
        // Test with 1 max plugin
        let config = PluginManagerConfig {
            max_plugins: 1,
            ..Default::default()
        };
        let manager = PluginManager::with_config(config).unwrap();
        assert!(manager.is_enabled());
        
        // Test with large max plugins
        let config = PluginManagerConfig {
            max_plugins: 10000,
            ..Default::default()
        };
        let manager = PluginManager::with_config(config).unwrap();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_manager_config_with_timeout_boundary() {
        // Test with 0 timeout
        let config = PluginManagerConfig {
            default_timeout_ms: 0,
            ..Default::default()
        };
        let manager = PluginManager::with_config(config).unwrap();
        assert!(manager.is_enabled());
        
        // Test with large timeout
        let config = PluginManagerConfig {
            default_timeout_ms: 60000, // 1 minute
            ..Default::default()
        };
        let manager = PluginManager::with_config(config).unwrap();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_manager_request_with_empty_body() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None, // Empty body
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-empty-body".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_request_with_large_body() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/upload".to_string(),
            query: HashMap::new(),
            headers: [("content-type".to_string(), "application/octet-stream".to_string())].into_iter().collect(),
            body: Some("x".repeat(100000)), // 100KB body
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-large-body".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_request_with_many_headers() {
        let mut manager = PluginManager::new().unwrap();
        let mut headers = HashMap::new();
        for i in 0..20 {
            headers.insert(format!("X-Header-{}", i), format!("value-{}", i));
        }
        
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-many-headers".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_request_with_many_query_params() {
        let mut manager = PluginManager::new().unwrap();
        let mut query = HashMap::new();
        for i in 0..10 {
            query.insert(format!("param{}", i), format!("value{}", i));
        }
        
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/search".to_string(),
            query,
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-many-params".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_response_with_various_status_codes() {
        let mut manager = PluginManager::new().unwrap();
        
        // Test various status codes
        let status_codes = vec![200, 201, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503];
        
        for status in status_codes {
            let mut response = PluginResponse {
                status,
                headers: HashMap::new(),
                body: None,
            };
            let action = manager.on_response(&mut response).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_response_with_headers() {
        let mut manager = PluginManager::new().unwrap();
        let mut response = PluginResponse {
            status: 200,
            headers: [
                ("content-type".to_string(), "application/json".to_string()),
                ("cache-control".to_string(), "no-cache".to_string()),
                ("x-custom".to_string(), "custom-value".to_string()),
            ].into_iter().collect(),
            body: Some("{\"status\": \"ok\"}".to_string()),
        };
        let action = manager.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_multiple_requests_same_manager() {
        let mut manager = PluginManager::new().unwrap();
        
        // Process multiple requests
        for i in 0..100 {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: format!("/api/item/{}", i),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("req-{}", i),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_request_with_special_characters_in_path() {
        let mut manager = PluginManager::new().unwrap();
        
        let paths = vec![
            "/api/test-123",
            "/api/test_123",
            "/api/test.123",
            "/api/test~123",
            "/api/test%20space",
            "/api/test+plus",
            "/api/test&ampersand",
            "/api/test=equals",
        ];
        
        for path in paths {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: path.to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: "test-special-chars".to_string(),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_is_empty() {
        let manager = PluginManager::new().unwrap();
        // When no plugins loaded, should be effectively empty
        assert_eq!(manager.count(), 0);
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_manager_execution_order_empty() {
        let manager = PluginManager::new().unwrap();
        let list = manager.list();
        // Empty manager should return empty list
        assert!(list.is_empty());
    }

    #[test]
    fn test_manager_stats_default() {
        let stats = PluginStats::default();
        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.total_latency_us, 0);
        assert_eq!(stats.avg_latency_us(), 0.0);
    }

    #[test]
    fn test_manager_stats_with_data() {
        let stats = PluginStats {
            load_time: std::time::Instant::now(),
            request_count: 100,
            response_count: 0,
            error_count: 5,
            total_latency_us: 50000,
        };
        assert_eq!(stats.request_count, 100);
        assert_eq!(stats.error_count, 5);
        assert_eq!(stats.total_latency_us, 50000);
        // avg = total_latency_us / (request_count + response_count) = 50000 / 100 = 500
        assert_eq!(stats.avg_latency_us(), 500.0);
    }

    #[test]
    fn test_manager_stats_high_latency() {
        let stats = PluginStats {
            load_time: std::time::Instant::now(),
            request_count: 1,
            response_count: 0,
            error_count: 0,
            total_latency_us: 10_000_000, // 10 seconds
        };
        // avg = 10000000 / 1 = 10000000
        assert_eq!(stats.avg_latency_us(), 10_000_000.0);
    }

    #[test]
    fn test_manager_request_with_all_http_methods() {
        let mut manager = PluginManager::new().unwrap();
        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "TRACE", "CONNECT"];
        
        for method in methods {
            let mut request = PluginRequest {
                method: method.to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: if method == "POST" || method == "PUT" || method == "PATCH" {
                    Some("body content".to_string())
                } else {
                    None
                },
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("test-{}", method),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue), "Failed for method: {}", method);
        }
    }

    #[test]
    fn test_manager_request_with_http_versions() {
        let mut manager = PluginManager::new().unwrap();
        let versions = vec!["HTTP/0.9", "HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"];
        
        for version in versions {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("test-{}", version.replace("/", "-")),
                version: version.to_string(),
                host: "localhost".to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue), "Failed for version: {}", version);
        }
    }

    #[test]
    fn test_manager_request_with_various_hosts() {
        let mut manager = PluginManager::new().unwrap();
        let hosts = vec![
            "localhost",
            "example.com",
            "api.example.com",
            "192.168.1.1",
            "[::1]",
            "example.com:8080",
            "127.0.0.1:3000",
        ];
        
        for host in hosts {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("test-{}", host.replace(|c: char| !c.is_alphanumeric(), "-")),
                version: "HTTP/1.1".to_string(),
                host: host.to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue), "Failed for host: {}", host);
        }
    }

    #[test]
    fn test_manager_request_with_various_ips() {
        let mut manager = PluginManager::new().unwrap();
        let ips = vec![
            "127.0.0.1",
            "192.168.1.1",
            "10.0.0.1",
            "172.16.0.1",
            "::1",
            "::ffff:192.168.1.1",
        ];
        
        for ip in ips {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: ip.to_string(),
                request_id: format!("test-{}", ip.replace(|c: char| !c.is_alphanumeric(), "-")),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };
            let action = manager.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue), "Failed for IP: {}", ip);
        }
    }

    #[test]
    fn test_manager_config_equality_detailed() {
        let config1 = PluginManagerConfig {
            enabled: true,
            max_plugins: 100,
            default_timeout_ms: 100,
            auto_reload: true,
        };
        let config2 = PluginManagerConfig {
            enabled: true,
            max_plugins: 100,
            default_timeout_ms: 100,
            auto_reload: true,
        };
        
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.max_plugins, config2.max_plugins);
        assert_eq!(config1.default_timeout_ms, config2.default_timeout_ms);
        assert_eq!(config1.auto_reload, config2.auto_reload);
    }

    #[test]
    fn test_manager_config_inequality() {
        let config1 = PluginManagerConfig::default();
        let config2 = PluginManagerConfig {
            enabled: false,
            ..Default::default()
        };
        
        assert_ne!(config1.enabled, config2.enabled);
    }

    #[test]
    fn test_manager_config_debug_output() {
        let config = PluginManagerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("PluginManagerConfig"));
        assert!(debug_str.contains("enabled") || debug_str.contains("true"));
    }

    #[test]
    fn test_manager_with_long_request_id() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "a".repeat(1000),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_response_with_empty_headers() {
        let mut manager = PluginManager::new().unwrap();
        let mut response = PluginResponse {
            status: 200,
            headers: HashMap::new(),
            body: None,
        };
        let action = manager.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_manager_response_with_redirect_status() {
        let mut manager = PluginManager::new().unwrap();
        let redirect_codes = vec![301, 302, 303, 307, 308];
        
        for code in redirect_codes {
            let mut response = PluginResponse {
                status: code,
                headers: [("Location".to_string(), "/new-path".to_string())].into_iter().collect(),
                body: None,
            };
            let action = manager.on_response(&mut response).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_response_with_error_status() {
        let mut manager = PluginManager::new().unwrap();
        let error_codes = vec![400, 401, 403, 404, 405, 422, 429, 500, 502, 503, 504];
        
        for code in error_codes {
            let mut response = PluginResponse {
                status: code,
                headers: HashMap::new(),
                body: Some(format!("Error {}", code)),
            };
            let action = manager.on_response(&mut response).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_response_with_content_type_variations() {
        let mut manager = PluginManager::new().unwrap();
        let content_types = vec![
            "application/json",
            "text/html",
            "text/plain",
            "application/xml",
            "application/octet-stream",
            "image/png",
            "image/jpeg",
        ];
        
        for ct in content_types {
            let mut response = PluginResponse {
                status: 200,
                headers: [("Content-Type".to_string(), ct.to_string())].into_iter().collect(),
                body: None,
            };
            let action = manager.on_response(&mut response).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[test]
    fn test_manager_on_request_preserves_request_data() {
        let mut manager = PluginManager::new().unwrap();
        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            query: [("page".to_string(), "1".to_string())].into_iter().collect(),
            headers: [("Authorization".to_string(), "Bearer token123".to_string())].into_iter().collect(),
            body: Some("{\"name\":\"test\"}".to_string()),
            client_ip: "192.168.1.100".to_string(),
            request_id: "req-unique-123".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "api.example.com".to_string(),
        };
        
        let action = manager.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
        
        // Verify request data is preserved
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.query.get("page"), Some(&"1".to_string()));
        assert_eq!(request.headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(request.body, Some("{\"name\":\"test\"}".to_string()));
        assert_eq!(request.client_ip, "192.168.1.100");
        assert_eq!(request.request_id, "req-unique-123");
    }

    #[test]
    fn test_manager_on_response_preserves_response_data() {
        let mut manager = PluginManager::new().unwrap();
        let mut response = PluginResponse {
            status: 201,
            headers: [
                ("Content-Type".to_string(), "application/json".to_string()),
                ("X-Request-ID".to_string(), "123".to_string()),
            ].into_iter().collect(),
            body: Some("{\"id\": 1}".to_string()),
        };
        
        let action = manager.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
        
        // Verify response data is preserved
        assert_eq!(response.status, 201);
        assert_eq!(response.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(response.headers.get("X-Request-ID"), Some(&"123".to_string()));
        assert_eq!(response.body, Some("{\"id\": 1}".to_string()));
    }
}
