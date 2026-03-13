//! Plugin system tests

#[cfg(test)]
mod tests {
    use crate::plugin::{
        error::PluginError,
        loader::PluginLoader,
        manager::{PluginManager, PluginManagerConfig},
        traits::{PluginAction, PluginConfig, PluginMetadata, PluginRequest, PluginResponse},
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert!(manager.is_enabled());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_plugin_manager_config() {
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
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: Some("https://example.com".to_string()),
            license: "MIT".to_string(),
            min_server_version: "0.1.0".to_string(),
            priority: 100,
            capabilities: vec![],
            permissions: vec![],
        };

        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.priority, 100);
    }

    #[test]
    fn test_plugin_config() {
        let mut custom = HashMap::new();
        custom.insert("key".to_string(), serde_json::json!("value"));
        custom.insert("number".to_string(), serde_json::json!(42));

        let config = PluginConfig {
            enabled: true,
            priority: Some(100),
            timeout_ms: Some(50),
            custom,
        };

        assert_eq!(config.get::<String>("key"), Some("value".to_string()));
        assert_eq!(config.get::<i32>("number"), Some(42));
        assert_eq!(config.get::<String>("nonexistent"), None);
    }

    #[test]
    fn test_plugin_request() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            query,
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "req-123".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.header("content-type"), Some(&"application/json".to_string()));
        assert_eq!(request.query_param("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_plugin_response() {
        let response = PluginResponse::ok()
            .with_header("X-Custom", "value")
            .with_body("test body");

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(response.body, Some("test body".to_string()));
    }

    #[test]
    fn test_plugin_response_json() {
        #[derive(serde::Serialize)]
        struct TestData {
            message: String,
        }

        let data = TestData {
            message: "Hello".to_string(),
        };

        let response = PluginResponse::ok().json(&data);
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
        assert!(response.body.is_some());
    }

    #[test]
    fn test_plugin_action() {
        // Continue action
        let continue_action = PluginAction::Continue;
        let json = serde_json::to_string(&continue_action).unwrap();
        // Serialized format may vary
        assert!(json.contains("Continue"));

        // Intercept action
        let response = PluginResponse::not_found();
        let intercept = PluginAction::Intercept(response);
        let json = serde_json::to_string(&intercept).unwrap();
        assert!(json.contains("Intercept"));
    }

    #[test]
    fn test_plugin_error() {
        let err = PluginError::NotFound("test".to_string());
        assert_eq!(err.error_code(), 1002);
        assert!(!err.is_recoverable());
        assert!(!err.is_timeout());

        let timeout_err = PluginError::Timeout(100);
        assert!(timeout_err.is_timeout());
        assert!(timeout_err.is_recoverable());
        assert_eq!(timeout_err.error_code(), 2001);
    }

    #[test]
    fn test_plugin_stats() {
        let mut stats = crate::plugin::manager::PluginStats::default();
        assert_eq!(stats.request_count, 0);
        assert_eq!(stats.avg_latency_us(), 0.0);

        stats.request_count = 10;
        stats.response_count = 5;
        stats.total_latency_us = 1500;

        assert_eq!(stats.avg_latency_us(), 100.0);
    }

    #[test]
    fn test_module_cache() {
        let mut cache = crate::plugin::loader::ModuleCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let loader = PluginLoader::new().unwrap();
        let module = wasmtime::Module::new(loader.engine(), "(module)").unwrap();

        cache.insert(PathBuf::from("/test.wasm"), module);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&PathBuf::from("/test.wasm")).is_some());

        cache.remove(&PathBuf::from("/test.wasm"));
        assert_eq!(cache.len(), 0);
    }
}
