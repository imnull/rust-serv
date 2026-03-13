//! Host functions for Wasm plugins

use crate::plugin::error::PluginResult;
use wasmtime::*;

/// Host functions provided to Wasm plugins
pub struct HostFunctions;

impl HostFunctions {
    /// Log a message from plugin
    pub fn host_log(
        level: i32,
        message: &str,
    ) {
        let level_str = match level {
            0 => "DEBUG",
            1 => "INFO",
            2 => "WARN",
            3 => "ERROR",
            _ => "UNKNOWN",
        };
        
        eprintln!("[Plugin {}] {}", level_str, message);
    }
    
    /// Get configuration value
    pub fn host_get_config(
        key: &str,
    ) -> Option<String> {
        // TODO: Implement config retrieval
        let _ = key;
        None
    }
    
    /// Set response header
    pub fn host_set_header(
        name: &str,
        value: &str,
    ) {
        // TODO: Implement header setting
        let _ = (name, value);
    }
    
    /// Report counter metric
    pub fn host_metrics_counter(
        name: &str,
        value: f64,
    ) {
        // TODO: Integrate with Prometheus metrics
        let _ = (name, value);
    }
    
    /// Report gauge metric
    pub fn host_metrics_gauge(
        name: &str,
        value: f64,
    ) {
        // TODO: Integrate with Prometheus metrics
        let _ = (name, value);
    }
    
    /// Report histogram metric
    pub fn host_metrics_histogram(
        name: &str,
        value: f64,
    ) {
        // TODO: Integrate with Prometheus metrics
        let _ = (name, value);
    }
}

/// Define host functions in linker
pub fn define_host_functions<T>(
    linker: &mut Linker<T>,
) -> PluginResult<()>
where
    T: Send + Sync + 'static,
{
    // host_log(level: i32, msg_ptr: i32, msg_len: i32)
    linker.func_wrap("host", "host_log", |mut caller: Caller<'_, T>, level: i32, msg_ptr: i32, msg_len: i32| {
        // Get memory
        let memory = caller.get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        // Read message from memory
        let mut buffer = vec![0u8; msg_len as usize];
        memory.read(&caller, msg_ptr as usize, &mut buffer)?;
        
        let message = String::from_utf8(buffer)
            .unwrap_or_else(|_| "Invalid UTF-8".to_string());
        
        // Log
        HostFunctions::host_log(level, &message);
        
        Ok(())
    }).map_err(|e| crate::plugin::PluginError::HostFunction(e.to_string()))?;
    
    // host_get_config(key_ptr: i32, key_len: i32, val_ptr: i32, val_len_ptr: i32) -> i32
    linker.func_wrap("host", "host_get_config", |mut caller: Caller<'_, T>, key_ptr: i32, key_len: i32, val_ptr: i32, val_len_ptr: i32| {
        // Get memory
        let memory = caller.get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        // Read key
        let mut key_buffer = vec![0u8; key_len as usize];
        memory.read(&caller, key_ptr as usize, &mut key_buffer)?;
        
        let key = String::from_utf8(key_buffer)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        
        // Get config value
        if let Some(value) = HostFunctions::host_get_config(&key) {
            // Write value to memory
            let value_bytes = value.as_bytes();
            memory.write(&mut caller, val_ptr as usize, value_bytes)?;
            
            // Write length
            let len_bytes = (value_bytes.len() as i32).to_le_bytes();
            memory.write(&mut caller, val_len_ptr as usize, &len_bytes)?;
            
            Ok(0)  // Success
        } else {
            Ok(1)  // Not found
        }
    }).map_err(|e| crate::plugin::PluginError::HostFunction(e.to_string()))?;
    
    // host_set_header(name_ptr: i32, name_len: i32, val_ptr: i32, val_len: i32)
    linker.func_wrap("host", "host_set_header", |mut caller: Caller<'_, T>, name_ptr: i32, name_len: i32, val_ptr: i32, val_len: i32| {
        // Get memory
        let memory = caller.get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        // Read name
        let mut name_buffer = vec![0u8; name_len as usize];
        memory.read(&caller, name_ptr as usize, &mut name_buffer)?;
        
        // Read value
        let mut val_buffer = vec![0u8; val_len as usize];
        memory.read(&caller, val_ptr as usize, &mut val_buffer)?;
        
        let name = String::from_utf8(name_buffer)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        let value = String::from_utf8(val_buffer)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        
        // Set header
        HostFunctions::host_set_header(&name, &value);
        
        Ok(())
    }).map_err(|e| crate::plugin::PluginError::HostFunction(e.to_string()))?;
    
    // host_metrics_counter(name_ptr: i32, name_len: i32, value: f64)
    linker.func_wrap("host", "host_metrics_counter", |mut caller: Caller<'_, T>, name_ptr: i32, name_len: i32, value: f64| {
        // Get memory
        let memory = caller.get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        // Read name
        let mut name_buffer = vec![0u8; name_len as usize];
        memory.read(&caller, name_ptr as usize, &mut name_buffer)?;
        
        let name = String::from_utf8(name_buffer)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        
        // Report metric
        HostFunctions::host_metrics_counter(&name, value);
        
        Ok(())
    }).map_err(|e| crate::plugin::PluginError::HostFunction(e.to_string()))?;
    
    // host_metrics_gauge(name_ptr: i32, name_len: i32, value: f64)
    linker.func_wrap("host", "host_metrics_gauge", |mut caller: Caller<'_, T>, name_ptr: i32, name_len: i32, value: f64| {
        // Get memory
        let memory = caller.get_export("memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        // Read name
        let mut name_buffer = vec![0u8; name_len as usize];
        memory.read(&caller, name_ptr as usize, &mut name_buffer)?;
        
        let name = String::from_utf8(name_buffer)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8"))?;
        
        // Report metric
        HostFunctions::host_metrics_gauge(&name, value);
        
        Ok(())
    }).map_err(|e| crate::plugin::PluginError::HostFunction(e.to_string()))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_log_debug() {
        HostFunctions::host_log(0, "Debug message");
    }

    #[test]
    fn test_host_log_info() {
        HostFunctions::host_log(1, "Info message");
    }

    #[test]
    fn test_host_log_warn() {
        HostFunctions::host_log(2, "Warning message");
    }

    #[test]
    fn test_host_log_error() {
        HostFunctions::host_log(3, "Error message");
    }

    #[test]
    fn test_host_log_unknown_level() {
        HostFunctions::host_log(99, "Unknown level");
    }

    #[test]
    fn test_host_log_empty_message() {
        HostFunctions::host_log(1, "");
    }

    #[test]
    fn test_host_log_unicode() {
        HostFunctions::host_log(1, "你好世界 🌍");
    }

    #[test]
    fn test_host_get_config_nonexistent() {
        let result = HostFunctions::host_get_config("nonexistent_key");
        assert!(result.is_none());
    }

    #[test]
    fn test_host_get_config_empty() {
        let result = HostFunctions::host_get_config("");
        assert!(result.is_none());
    }

    #[test]
    fn test_host_set_header() {
        HostFunctions::host_set_header("X-Custom", "value");
    }

    #[test]
    fn test_host_set_header_empty() {
        HostFunctions::host_set_header("", "");
    }

    #[test]
    fn test_host_metrics_counter() {
        HostFunctions::host_metrics_counter("test_counter", 1.0);
    }

    #[test]
    fn test_host_metrics_counter_zero() {
        HostFunctions::host_metrics_counter("zero_counter", 0.0);
    }

    #[test]
    fn test_host_metrics_counter_negative() {
        HostFunctions::host_metrics_counter("negative_counter", -1.0);
    }

    #[test]
    fn test_host_metrics_gauge() {
        HostFunctions::host_metrics_gauge("test_gauge", 42.5);
    }

    #[test]
    fn test_host_metrics_gauge_zero() {
        HostFunctions::host_metrics_gauge("zero_gauge", 0.0);
    }

    #[test]
    fn test_define_host_functions() {
        let engine = Engine::default();
        let mut linker: Linker<()> = Linker::new(&engine);

        let result = define_host_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_log_calls() {
        for level in 0..4 {
            HostFunctions::host_log(level, &format!("Level {} test", level));
        }
    }

    #[test]
    fn test_multiple_metric_calls() {
        for i in 0..10 {
            HostFunctions::host_metrics_counter(&format!("counter_{}", i), i as f64);
            HostFunctions::host_metrics_gauge(&format!("gauge_{}", i), i as f64 * 0.5);
        }
    }
}
