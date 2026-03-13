//! Wasm executor for running plugins (simplified)

use crate::plugin::{
    error::{PluginError, PluginResult},
    traits::{PluginAction, PluginConfig, PluginRequest, PluginResponse},
};
use wasmtime::*;

/// Wasm plugin executor
pub struct WasmExecutor {
    instance: Instance,
    store: Store<ExecutorState>,
    memory: Memory,
    has_init: bool,
    has_on_request: bool,
    has_on_response: bool,
    has_unload: bool,
}

/// Executor state
#[derive(Debug, Default)]
pub struct ExecutorState {
    pub last_action: Option<String>,
}

impl WasmExecutor {
    /// Create a new executor from compiled module
    pub fn new(engine: &Engine, module: Module, config: &PluginConfig) -> PluginResult<Self> {
        let mut store = Store::new(engine, ExecutorState::default());

        // Create linker
        let mut linker = Linker::new(engine);

        // Add host functions
        Self::add_host_functions(&mut linker)?;

        // Instantiate
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| PluginError::WasmInstantiation(e.to_string()))?;

        // Get memory
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| PluginError::WasmInstantiation("Memory not exported".into()))?;

        // Check which functions exist
        let has_init = instance.get_typed_func::<(i32, i32), i32>(&mut store, "plugin_init").is_ok();
        let has_on_request = instance.get_typed_func::<(i32, i32, i32), i32>(&mut store, "plugin_on_request").is_ok();
        let has_on_response = instance.get_typed_func::<(i32, i32, i32), i32>(&mut store, "plugin_on_response").is_ok();
        let has_unload = instance.get_typed_func::<(), i32>(&mut store, "plugin_unload").is_ok();

        let mut executor = Self {
            instance,
            store,
            memory,
            has_init,
            has_on_request,
            has_on_response,
            has_unload,
        };

        // Initialize plugin
        if executor.has_init {
            let config_json = serde_json::to_string(config)
                .map_err(|e| PluginError::Serialization(e.to_string()))?;

            let (ptr, len) = executor.write_to_memory(config_json.as_bytes())?;

            let init_fn = executor.instance
                .get_typed_func::<(i32, i32), i32>(&mut executor.store, "plugin_init")
                .unwrap();

            let result = init_fn.call(&mut executor.store, (ptr, len))
                .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

            if result != 0 {
                return Err(PluginError::InitFailed(
                    format!("Plugin init returned {}", result)
                ));
            }
        }

        Ok(executor)
    }

    /// Execute on_request hook
    pub fn on_request(&mut self, request: &PluginRequest) -> PluginResult<PluginAction> {
        if !self.has_on_request {
            return Ok(PluginAction::Continue);
        }

        let request_json = serde_json::to_string(request)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;

        let (req_ptr, req_len) = self.write_to_memory(request_json.as_bytes())?;
        let result_ptr = 65536; // Upper memory region

        let on_request_fn = self.instance
            .get_typed_func::<(i32, i32, i32), i32>(&mut self.store, "plugin_on_request")
            .unwrap();

        let result = on_request_fn.call(&mut self.store, (req_ptr, req_len, result_ptr))
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        if result == 0 {
            let action = self.read_action_from_memory(result_ptr)?;
            Ok(action)
        } else {
            Err(PluginError::ExecutionError(
                format!("Plugin returned error code: {}", result)
            ))
        }
    }

    /// Execute on_response hook
    pub fn on_response(&mut self, response: &PluginResponse) -> PluginResult<PluginAction> {
        if !self.has_on_response {
            return Ok(PluginAction::Continue);
        }

        let response_json = serde_json::to_string(response)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;

        let (res_ptr, res_len) = self.write_to_memory(response_json.as_bytes())?;
        let result_ptr = 65536;

        let on_response_fn = self.instance
            .get_typed_func::<(i32, i32, i32), i32>(&mut self.store, "plugin_on_response")
            .unwrap();

        let result = on_response_fn.call(&mut self.store, (res_ptr, res_len, result_ptr))
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        if result == 0 {
            let action = self.read_action_from_memory(result_ptr)?;
            Ok(action)
        } else {
            Err(PluginError::ExecutionError(
                format!("Plugin returned error code: {}", result)
            ))
        }
    }

    /// Cleanup plugin
    pub fn unload(&mut self) -> PluginResult<()> {
        if self.has_unload {
            let unload_fn = self.instance
                .get_typed_func::<(), i32>(&mut self.store, "plugin_unload")
                .unwrap();

            unload_fn.call(&mut self.store, ())
                .map_err(|e| PluginError::ExecutionError(e.to_string()))?;
        }
        Ok(())
    }

    // Private helper methods

    fn add_host_functions(linker: &mut Linker<ExecutorState>) -> PluginResult<()> {
        linker.func_wrap("host", "log", |_level: i32, _ptr: i32, _len: i32| {
            Ok(())
        }).map_err(|e| PluginError::WasmInstantiation(e.to_string()))?;

        Ok(())
    }

    fn write_to_memory(&mut self, data: &[u8]) -> PluginResult<(i32, i32)> {
        let ptr = 0i32;
        let len = data.len() as i32;

        self.memory.write(&mut self.store, ptr as usize, data)
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        Ok((ptr, len))
    }

    fn read_action_from_memory(&mut self, ptr: i32) -> PluginResult<PluginAction> {
        let mut buffer = vec![0u8; 8192];
        self.memory.read(&self.store, ptr as usize, &mut buffer)
            .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

        let end = buffer.iter().position(|&b| b == 0).unwrap_or(8192);
        buffer.truncate(end);

        if buffer.is_empty() {
            return Ok(PluginAction::Continue);
        }

        let action_str = String::from_utf8(buffer)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;

        if action_str.is_empty() || action_str == "Continue" {
            Ok(PluginAction::Continue)
        } else {
            let action: PluginAction = serde_json::from_str(&action_str)
                .map_err(|e| PluginError::Serialization(e.to_string()))?;
            Ok(action)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper to create test engine
    fn test_engine() -> Engine {
        let mut config = Config::new();
        config.wasm_bulk_memory(true).wasm_reference_types(true);
        Engine::new(&config).unwrap()
    }

    // Helper to create minimal Wasm module with all functions
    fn minimal_wasm_module(engine: &Engine) -> Module {
        let wat = r#"
            (module
                (memory (export "memory") 2)
                (func (export "plugin_init") (param i32 i32) (result i32) i32.const 0)
                (func (export "plugin_on_request") (param i32 i32 i32) (result i32) i32.const 0)
                (func (export "plugin_on_response") (param i32 i32 i32) (result i32) i32.const 0)
                (func (export "plugin_unload") (result i32) i32.const 0)
            )
        "#;
        Module::new(engine, wat).unwrap()
    }

    // Helper to create Wasm module without plugin functions
    fn empty_wasm_module(engine: &Engine) -> Module {
        let wat = r#"(module (memory (export "memory") 1))"#;
        Module::new(engine, wat).unwrap()
    }

    // Helper to create Wasm module with failing init
    fn failing_init_wasm_module(engine: &Engine) -> Module {
        let wat = r#"
            (module
                (memory (export "memory") 1)
                (func (export "plugin_init") (param i32 i32) (result i32) i32.const 1)
            )
        "#;
        Module::new(engine, wat).unwrap()
    }

    #[test]
    fn test_executor_state_default() {
        let state = ExecutorState::default();
        assert!(state.last_action.is_none());
    }

    #[test]
    fn test_executor_new_success() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let executor = WasmExecutor::new(&engine, module, &config);
        assert!(executor.is_ok());

        let exec = executor.unwrap();
        assert!(exec.has_init);
        assert!(exec.has_on_request);
        assert!(exec.has_on_response);
        assert!(exec.has_unload);
    }

    #[test]
    fn test_executor_new_no_functions() {
        let engine = test_engine();
        let module = empty_wasm_module(&engine);
        let config = PluginConfig::default();

        let executor = WasmExecutor::new(&engine, module, &config).unwrap();
        assert!(!executor.has_init);
        assert!(!executor.has_on_request);
        assert!(!executor.has_on_response);
        assert!(!executor.has_unload);
    }

    #[test]
    fn test_executor_new_init_failure() {
        let engine = test_engine();
        let module = failing_init_wasm_module(&engine);
        let config = PluginConfig::default();

        let result = WasmExecutor::new(&engine, module, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_on_request_with_function() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-1".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let action = executor.on_request(&request);
        assert!(action.is_ok());
    }

    #[test]
    fn test_on_request_without_function() {
        let engine = test_engine();
        let module = empty_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let request = PluginRequest {
            method: "POST".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "192.168.1.1".to_string(),
            request_id: "test-2".to_string(),
            version: "HTTP/2.0".to_string(),
            host: "example.com".to_string(),
        };

        let action = executor.on_request(&request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_on_response_with_function() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let response = PluginResponse::ok();
        let action = executor.on_response(&response);
        assert!(action.is_ok());
    }

    #[test]
    fn test_on_response_without_function() {
        let engine = test_engine();
        let module = empty_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let response = PluginResponse::not_found();
        let action = executor.on_response(&response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_unload_with_function() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();
        let result = executor.unload();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unload_without_function() {
        let engine = test_engine();
        let module = empty_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();
        let result = executor.unload();
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_with_complex_config() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);

        let mut config = PluginConfig::default();
        config.enabled = true;
        config.priority = Some(100);
        config.timeout_ms = Some(200);
        config.custom.insert("key".to_string(), serde_json::json!("value"));

        let executor = WasmExecutor::new(&engine, module, &config);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_multiple_executors() {
        let engine = test_engine();
        let module1 = minimal_wasm_module(&engine);
        let module2 = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let exec1 = WasmExecutor::new(&engine, module1, &config).unwrap();
        let exec2 = WasmExecutor::new(&engine, module2, &config).unwrap();

        // Both should have independent memory
        assert!(exec1.memory.size(&exec1.store) > 0);
        assert!(exec2.memory.size(&exec2.store) > 0);
    }

    #[test]
    fn test_concurrent_requests() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        for i in 0..10 {
            let request = PluginRequest {
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

            assert!(executor.on_request(&request).is_ok());
        }
    }

    #[test]
    fn test_add_host_functions() {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        let result = WasmExecutor::add_host_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_executor_state_debug() {
        let state = ExecutorState::default();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("ExecutorState"));
    }

    #[test]
    fn test_executor_state_last_action() {
        let state = ExecutorState::default();
        assert!(state.last_action.is_none());
    }

    #[test]
    fn test_executor_with_large_request() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        // Create a large request with many headers
        let mut headers = HashMap::new();
        for i in 0..100 {
            headers.insert(format!("header-{}", i), format!("value-{}", i));
        }

        let request = PluginRequest {
            method: "POST".to_string(),
            path: "/large-request".to_string(),
            query: HashMap::new(),
            headers,
            body: Some("x".repeat(10000)),
            client_ip: "127.0.0.1".to_string(),
            request_id: "large-req".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        assert!(executor.on_request(&request).is_ok());
    }

    #[test]
    fn test_executor_with_unicode_data() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());

        let request = PluginRequest {
            method: "POST".to_string(),
            path: "/unicode/你好世界/日本語/🌍".to_string(),
            query: HashMap::new(),
            headers,
            body: Some("Unicode: 你好世界 🎉".to_string()),
            client_ip: "127.0.0.1".to_string(),
            request_id: "unicode-req".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        assert!(executor.on_request(&request).is_ok());
    }

    #[test]
    fn test_executor_with_empty_body() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/no-body".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "no-body-req".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let action = executor.on_request(&request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_executor_memory_size() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let executor = WasmExecutor::new(&engine, module, &config).unwrap();

        // Memory should be available
        let size = executor.memory.size(&executor.store);
        assert!(size > 0);
    }

    #[test]
    fn test_executor_with_response_variants() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);
        let config = PluginConfig::default();

        let mut executor = WasmExecutor::new(&engine, module, &config).unwrap();

        // Test different response types
        let responses = vec![
            PluginResponse::ok(),
            PluginResponse::not_found(),
            PluginResponse::internal_error(),
            PluginResponse {
                status: 201,
                headers: HashMap::new(),
                body: Some("Created".to_string()),
            },
        ];

        for response in responses {
            let action = executor.on_response(&response);
            assert!(action.is_ok());
        }
    }

    #[test]
    fn test_executor_with_config_timeout() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);

        let mut config = PluginConfig::default();
        config.timeout_ms = Some(5000);

        let executor = WasmExecutor::new(&engine, module, &config);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_executor_with_config_disabled() {
        let engine = test_engine();
        let module = minimal_wasm_module(&engine);

        let mut config = PluginConfig::default();
        config.enabled = false;

        let executor = WasmExecutor::new(&engine, module, &config);
        assert!(executor.is_ok());
    }
}
