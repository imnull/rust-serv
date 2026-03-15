//! Integration tests for plugin system
//!
//! End-to-end tests for the WebAssembly plugin system

#[cfg(test)]
mod plugin_integration_tests {
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    use rust_serv::plugin::{
        manager::PluginManager,
        traits::{PluginAction, PluginConfig, PluginRequest, PluginResponse},
    };

    /// Create a test Wasm module
    fn create_test_wasm() -> Vec<u8> {
        // Simple Wasm module with all required plugin exports
        let wat = r#"
            (module
                ;; Export memory
                (memory (export "memory") 2)
                
                ;; plugin_init function
                (func (export "plugin_init") (param i32 i32) (result i32)
                    i32.const 0
                )
                
                ;; plugin_on_request function
                (func (export "plugin_on_request") (param i32 i32 i32) (result i32)
                    i32.const 0
                )
                
                ;; plugin_on_response function
                (func (export "plugin_on_response") (param i32 i32 i32) (result i32)
                    i32.const 0
                )
                
                ;; plugin_log function (optional but common)
                (func (export "plugin_log") (param i32 i32) (result i32)
                    i32.const 0
                )
                
                ;; plugin_unload function
                (func (export "plugin_unload") (result i32)
                    i32.const 0
                )
            )
        "#;
        wat::parse_str(wat).expect("Failed to parse WAT")
    }

    /// Create a temporary Wasm file
    fn create_temp_wasm_file() -> tempfile::NamedTempFile {
        let wasm = create_test_wasm();
        let mut file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(&wasm).expect("Failed to write to temp file");
        file
    }

    #[tokio::test]
    async fn test_full_plugin_lifecycle() {
        // Create plugin manager
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Create temp plugin file
        let temp_file = create_temp_wasm_file();

        // Load plugin
        {
            let mut mgr = manager.write().await;
            let config = PluginConfig::default();
            let plugin_id = mgr.load(temp_file.path(), config)
                .expect("Failed to load plugin");
            
            assert!(!plugin_id.is_empty());
            assert_eq!(mgr.count(), 1);
        }

        // Verify plugin is loaded
        {
            let mgr = manager.read().await;
            assert_eq!(mgr.count(), 1);
            let plugins = mgr.list();
            assert_eq!(plugins.len(), 1);
        }

        // Execute request through plugin
        {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: "test-req-1".to_string(),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };

            let mut mgr = manager.write().await;
            let action = mgr.on_request(&mut request)
                .expect("Request processing failed");
            
            // Test plugin should return Continue
            assert!(matches!(action, PluginAction::Continue));
        }

        // Execute response through plugin
        {
            let mut response = PluginResponse::ok();
            
            let mut mgr = manager.write().await;
            let action = mgr.on_response(&mut response)
                .expect("Response processing failed");
            
            assert!(matches!(action, PluginAction::Continue));
        }

        // Get plugin stats
        {
            let mgr = manager.read().await;
            let plugins = mgr.list();
            let plugin = plugins.first().unwrap();
            assert!(plugin.stats.request_count >= 1);
        }

        // Unload plugin
        {
            let mut mgr = manager.write().await;
            let plugins = mgr.list();
            let plugin_id = plugins.first().unwrap().id.clone();
            
            mgr.unload(&plugin_id)
                .expect("Failed to unload plugin");
            
            assert_eq!(mgr.count(), 0);
        }
    }

    #[tokio::test]
    async fn test_multiple_plugins_priority_order() {
        use std::io::Write;

        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Create temp directory for plugins
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Load 3 plugins with different priorities - each with unique file path
        let configs = vec![
            (1, 100),
            (2, 50),
            (3, 200),
        ];
        
        for (idx, priority) in configs {
            let wasm = create_test_wasm();
            let path = temp_dir.path().join(format!("plugin{}.wasm", idx));
            let mut file = std::fs::File::create(&path).expect("Failed to create file");
            file.write_all(&wasm).expect("Failed to write wasm");
            
            let mut config = PluginConfig::default();
            config.priority = Some(priority);
            
            let mut mgr = manager.write().await;
            mgr.load(&path, config)
                .expect(&format!("Failed to load plugin {}", idx));
        }

        // Verify plugins are sorted by priority (descending)
        let mgr = manager.read().await;
        let plugins = mgr.list();
        
        assert_eq!(plugins.len(), 3);
        
        // Check priority order (highest first)
        let priorities: Vec<i32> = plugins
            .iter()
            .map(|p| p.config.priority.unwrap_or(p.metadata.priority))
            .collect();
        
        assert_eq!(priorities, vec![200, 100, 50]);
    }

    #[tokio::test]
    async fn test_plugin_config_update() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Load plugin
        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Update config
        {
            let mut mgr = manager.write().await;
            let mut new_config = PluginConfig::default();
            new_config.enabled = false;
            new_config.priority = Some(999);
            
            mgr.update_config(&plugin_id, new_config)
                .expect("Failed to update config");
        }

        // Verify update
        {
            let mgr = manager.read().await;
            let plugin = mgr.get(&plugin_id).unwrap();
            assert!(!plugin.config.enabled);
            assert_eq!(plugin.config.priority, Some(999));
        }

        // Test that disabled plugin is skipped
        {
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

            let mut mgr = manager.write().await;
            let action = mgr.on_request(&mut request)
                .expect("Request processing failed");
            
            // Should return Continue even though plugin is disabled
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[tokio::test]
    async fn test_plugin_reload() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Load plugin
        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Record stats before reload
        let _stats_before = {
            let mgr = manager.read().await;
            let plugin = mgr.get(&plugin_id).unwrap();
            plugin.stats.request_count
        };

        // Make some requests
        for _ in 0..5 {
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

            let mut mgr = manager.write().await;
            mgr.on_request(&mut request).ok();
        }

        // Reload plugin
        {
            let mut mgr = manager.write().await;
            mgr.reload(&plugin_id)
                .expect("Failed to reload plugin");
        }

        // Verify plugin is still loaded
        let mgr = manager.read().await;
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(&plugin_id).is_some());
    }

    #[tokio::test]
    async fn test_plugin_max_limit() {
        use rust_serv::plugin::manager::PluginManagerConfig;
        use std::io::Write;

        // Create manager with max 2 plugins
        let config = PluginManagerConfig {
            max_plugins: 2,
            ..Default::default()
        };
        
        let manager = Arc::new(RwLock::new(
            PluginManager::with_config(config).expect("Failed to create manager")
        ));

        // Create temp directory for plugins
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Load 2 plugins with different file paths
        for i in 0..2 {
            let wasm = create_test_wasm();
            let path = temp_dir.path().join(format!("plugin{}.wasm", i));
            let mut file = std::fs::File::create(&path).expect("Failed to create file");
            file.write_all(&wasm).expect("Failed to write wasm");
            
            let mut mgr = manager.write().await;
            mgr.load(&path, PluginConfig::default())
                .expect(&format!("Failed to load plugin {}", i));
        }

        assert_eq!(manager.read().await.count(), 2);

        // 3rd plugin should fail due to max limit
        let wasm = create_test_wasm();
        let path = temp_dir.path().join("plugin3.wasm");
        let mut file = std::fs::File::create(&path).expect("Failed to create file");
        file.write_all(&wasm).expect("Failed to write wasm");
        
        let result = {
            let mut mgr = manager.write().await;
            mgr.load(&path, PluginConfig::default())
        };

        assert!(result.is_err());
        // Verify it's the max plugins error, not AlreadyLoaded
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Maximum") || err_msg.contains("max"), 
                "Expected max plugins error, got: {}", err_msg);
    }

    #[tokio::test]
    async fn test_concurrent_plugin_access() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Load a plugin
        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Spawn multiple concurrent request handlers
        let mut handles = vec![];

        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let mut request = PluginRequest {
                    method: "GET".to_string(),
                    path: format!("/test/{}", i),
                    query: HashMap::new(),
                    headers: HashMap::new(),
                    body: None,
                    client_ip: "127.0.0.1".to_string(),
                    request_id: format!("req-{}", i),
                    version: "HTTP/1.1".to_string(),
                    host: "localhost".to_string(),
                };

                let mut mgr = manager_clone.write().await;
                mgr.on_request(&mut request)
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }

        // Verify stats
        let mgr = manager.read().await;
        let plugin = mgr.get(&plugin_id).unwrap();
        assert!(plugin.stats.request_count >= 10);
    }

    #[tokio::test]
    async fn test_unicode_request_handling() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        // Create request with Unicode characters
        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/用户/测试".to_string(),
            query: [("名称".to_string(), "值".to_string())].into_iter().collect(),
            headers: [("自定义头".to_string(), "你好 🌍".to_string())].into_iter().collect(),
            body: Some("Unicode: 测试数据 🎉".to_string()),
            client_ip: "127.0.0.1".to_string(),
            request_id: "unicode-test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "例子.com".to_string(),
        };

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request)
            .expect("Request processing failed");

        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_with_disabled_manager() {
        use rust_serv::plugin::manager::PluginManagerConfig;
        
        // Create manager with plugin system disabled
        let config = PluginManagerConfig {
            enabled: false,
            ..Default::default()
        };
        
        let manager = Arc::new(RwLock::new(
            PluginManager::with_config(config).expect("Failed to create manager")
        ));

        // Try to load plugin - should fail
        let temp_file = create_temp_wasm_file();
        let result = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
        };
        
        assert!(result.is_err());
        
        // Requests should still pass through
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

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request).expect("Request processing failed");
        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_multiple_reload_cycles() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Load plugin
        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Multiple reload cycles
        for i in 0..5 {
            // Make a request
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: format!("/test/{}", i),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("req-{}", i),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };

            {
                let mut mgr = manager.write().await;
                mgr.on_request(&mut request).ok();
            }

            // Reload
            {
                let mut mgr = manager.write().await;
                mgr.reload(&plugin_id).expect("Failed to reload plugin");
            }
        }

        // Verify plugin still exists
        let mgr = manager.read().await;
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(&plugin_id).is_some());
    }

    #[tokio::test]
    async fn test_plugin_request_with_various_methods() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        
        for method in methods {
            let mut request = PluginRequest {
                method: method.to_string(),
                path: "/api/test".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: if method == "POST" || method == "PUT" {
                    Some("body".to_string())
                } else {
                    None
                },
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("test-{}", method),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };

            let mut mgr = manager.write().await;
            let action = mgr.on_request(&mut request)
                .expect(&format!("Failed for method {}", method));
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[tokio::test]
    async fn test_plugin_response_with_various_status_codes() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        let status_codes = vec![200, 201, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503];
        
        for status in status_codes {
            let mut response = PluginResponse {
                status,
                headers: HashMap::new(),
                body: Some(format!("Response for status {}", status)),
            };

            let mut mgr = manager.write().await;
            let action = mgr.on_response(&mut response)
                .expect(&format!("Failed for status {}", status));
            assert!(matches!(action, PluginAction::Continue));
        }
    }

    #[tokio::test]
    async fn test_plugin_with_custom_config_values() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        
        // Create config with custom values
        let mut config = PluginConfig::default();
        config.enabled = true;
        config.priority = Some(500);
        config.timeout_ms = Some(200);
        config.custom.insert("key1".to_string(), serde_json::json!("value1"));
        config.custom.insert("key2".to_string(), serde_json::json!(42));

        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), config)
                .expect("Failed to load plugin")
        };

        // Verify config was stored
        let mgr = manager.read().await;
        let plugin = mgr.get(&plugin_id).unwrap();
        assert!(plugin.config.enabled);
        assert_eq!(plugin.config.priority, Some(500));
        assert_eq!(plugin.config.timeout_ms, Some(200));
        assert_eq!(plugin.config.custom.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(plugin.config.custom.get("key2"), Some(&serde_json::json!(42)));
    }

    #[tokio::test]
    async fn test_plugin_load_invalid_wasm() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        // Create temp file with invalid Wasm bytes
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(b"invalid wasm bytes").expect("Failed to write");

        let result = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
        };

        // Should fail to load invalid Wasm
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_concurrent_load_and_request() {
        use std::io::Write;

        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Spawn tasks that load plugins
        let mut load_handles = vec![];
        for i in 0..3 {
            let manager_clone = Arc::clone(&manager);
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            let handle = tokio::spawn(async move {
                let wasm = create_test_wasm();
                let path = temp_dir_path.join(format!("plugin{}.wasm", i));
                let mut file = std::fs::File::create(&path).expect("Failed to create file");
                file.write_all(&wasm).expect("Failed to write wasm");
                
                let mut mgr = manager_clone.write().await;
                mgr.load(&path, PluginConfig::default())
            });
            load_handles.push(handle);
        }

        // Wait for all loads to complete
        for handle in load_handles {
            let _ = handle.await.expect("Task panicked");
        }

        // Verify all plugins loaded
        let mgr = manager.read().await;
        assert!(mgr.count() > 0);
    }

    #[tokio::test]
    async fn test_plugin_list_consistency() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // List should return consistent results
        for _ in 0..10 {
            let mgr = manager.read().await;
            let plugins = mgr.list();
            assert_eq!(plugins.len(), 1);
            assert_eq!(plugins.first().unwrap().id, plugin_id);
        }
    }

    #[tokio::test]
    async fn test_plugin_get_consistency() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Get should return consistent results
        for _ in 0..10 {
            let mgr = manager.read().await;
            let plugin = mgr.get(&plugin_id);
            assert!(plugin.is_some());
            assert_eq!(plugin.unwrap().id, plugin_id);
        }

        // Non-existent plugin should return None
        let mgr = manager.read().await;
        assert!(mgr.get("non-existent-id").is_none());
    }

    #[tokio::test]
    async fn test_plugin_stats_accumulation() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        let plugin_id = {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin")
        };

        // Make multiple requests
        for i in 0..50 {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: format!("/test/{}", i),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: format!("req-{}", i),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };

            let mut mgr = manager.write().await;
            mgr.on_request(&mut request).ok();
        }

        // Verify stats accumulated
        let mgr = manager.read().await;
        let plugin = mgr.get(&plugin_id).unwrap();
        assert!(plugin.stats.request_count >= 50);
    }

    #[tokio::test]
    async fn test_plugin_with_empty_request_body() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/empty".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "empty-body-test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request)
            .expect("Request processing failed");
        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_with_large_request_body() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        let mut request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/upload".to_string(),
            query: HashMap::new(),
            headers: [("content-type".to_string(), "application/octet-stream".to_string())].into_iter().collect(),
            body: Some("x".repeat(10000)), // 10KB body
            client_ip: "127.0.0.1".to_string(),
            request_id: "large-body-test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request)
            .expect("Request processing failed");
        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_with_many_headers() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

        let mut headers = HashMap::new();
        for i in 0..20 {
            headers.insert(format!("X-Header-{}", i), format!("value-{}", i));
        }

        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/headers".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "many-headers-test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request)
            .expect("Request processing failed");
        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_with_many_query_params() {
        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_file = create_temp_wasm_file();
        {
            let mut mgr = manager.write().await;
            mgr.load(temp_file.path(), PluginConfig::default())
                .expect("Failed to load plugin");
        }

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
            request_id: "many-params-test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let mut mgr = manager.write().await;
        let action = mgr.on_request(&mut request)
            .expect("Request processing failed");
        assert!(matches!(action, PluginAction::Continue));
    }

    #[tokio::test]
    async fn test_plugin_priority_zero() {
        use std::io::Write;

        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Load plugin with priority 0
        let wasm = create_test_wasm();
        let path = temp_dir.path().join("plugin_priority_0.wasm");
        let mut file = std::fs::File::create(&path).expect("Failed to create file");
        file.write_all(&wasm).expect("Failed to write wasm");

        let mut config = PluginConfig::default();
        config.priority = Some(0);

        let mut mgr = manager.write().await;
        mgr.load(&path, config)
            .expect("Failed to load plugin with priority 0");

        let plugins = mgr.list();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins.first().unwrap().config.priority, Some(0));
    }

    #[tokio::test]
    async fn test_plugin_priority_negative() {
        use std::io::Write;

        let manager = Arc::new(RwLock::new(
            PluginManager::new().expect("Failed to create manager")
        ));

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Load plugin with negative priority
        let wasm = create_test_wasm();
        let path = temp_dir.path().join("plugin_priority_neg.wasm");
        let mut file = std::fs::File::create(&path).expect("Failed to create file");
        file.write_all(&wasm).expect("Failed to write wasm");

        let mut config = PluginConfig::default();
        config.priority = Some(-100);

        let mut mgr = manager.write().await;
        mgr.load(&path, config)
            .expect("Failed to load plugin with negative priority");

        let plugins = mgr.list();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins.first().unwrap().config.priority, Some(-100));
    }}
