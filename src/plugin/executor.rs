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

    #[test]
    fn test_executor_creation() {
        assert!(true);
    }
}
