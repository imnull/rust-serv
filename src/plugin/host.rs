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
    fn test_host_metrics_histogram() {
        HostFunctions::host_metrics_histogram("test_histogram", 100.0);
    }

    #[test]
    fn test_host_metrics_histogram_zero() {
        HostFunctions::host_metrics_histogram("zero_histogram", 0.0);
    }

    #[test]
    fn test_host_metrics_histogram_negative() {
        HostFunctions::host_metrics_histogram("negative_histogram", -50.0);
    }

    #[test]
    fn test_host_set_header_unicode() {
        HostFunctions::host_set_header("X-Custom-Header", "你好世界 🌍");
    }

    #[test]
    fn test_host_set_header_long_value() {
        HostFunctions::host_set_header("X-Long-Header", &"a".repeat(1000));
    }

    #[test]
    fn test_host_log_long_message() {
        HostFunctions::host_log(1, &"a".repeat(10000));
    }

    #[test]
    fn test_host_log_special_chars() {
        HostFunctions::host_log(1, "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?");
    }

    #[test]
    fn test_host_get_config_special_chars() {
        let result = HostFunctions::host_get_config("key.with.special!chars@123");
        assert!(result.is_none());
    }

    #[test]
    fn test_host_metrics_counter_large_value() {
        HostFunctions::host_metrics_counter("large_counter", 1_000_000.0);
    }

    #[test]
    fn test_host_metrics_gauge_large_value() {
        HostFunctions::host_metrics_gauge("large_gauge", 1_000_000.0);
    }

    #[test]
    fn test_host_metrics_histogram_large_value() {
        HostFunctions::host_metrics_histogram("large_histogram", 1_000_000.0);
    }

    #[test]
    fn test_host_metrics_counter_small_value() {
        HostFunctions::host_metrics_counter("small_counter", 0.0001);
    }

    #[test]
    fn test_host_metrics_gauge_small_value() {
        HostFunctions::host_metrics_gauge("small_gauge", 0.0001);
    }

    #[test]
    fn test_host_metrics_histogram_small_value() {
        HostFunctions::host_metrics_histogram("small_histogram", 0.0001);
    }

    #[test]
    fn test_define_host_functions_multiple_calls() {
        let engine = Engine::default();
        
        // First call should succeed
        let mut linker1: Linker<()> = Linker::new(&engine);
        let result1 = define_host_functions(&mut linker1);
        assert!(result1.is_ok());
        
        // Second call on new linker should also succeed
        let mut linker2: Linker<()> = Linker::new(&engine);
        let result2 = define_host_functions(&mut linker2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_host_log_all_levels() {
        // Test all valid levels
        HostFunctions::host_log(0, "Debug");
        HostFunctions::host_log(1, "Info");
        HostFunctions::host_log(2, "Warn");
        HostFunctions::host_log(3, "Error");
        
        // Test boundary levels
        HostFunctions::host_log(-1, "Negative level");
        HostFunctions::host_log(4, "Level 4");
        HostFunctions::host_log(100, "Large level");
    }

    #[test]
    fn test_host_get_config_various_keys() {
        // Test various key formats
        assert!(HostFunctions::host_get_config("simple").is_none());
        assert!(HostFunctions::host_get_config("key.with.dots").is_none());
        assert!(HostFunctions::host_get_config("key-with-dashes").is_none());
        assert!(HostFunctions::host_get_config("key_with_underscores").is_none());
        assert!(HostFunctions::host_get_config("123numeric").is_none());
        assert!(HostFunctions::host_get_config("UPPERCASE").is_none());
        assert!(HostFunctions::host_get_config("mixedCase").is_none());
    }

    #[test]
    fn test_host_set_header_various_names() {
        HostFunctions::host_set_header("X-Custom", "value");
        HostFunctions::host_set_header("X-Custom-Header", "value");
        HostFunctions::host_set_header("X-Custom_Header", "value");
        HostFunctions::host_set_header("Custom", "value");
        HostFunctions::host_set_header("content-type", "application/json");
        HostFunctions::host_set_header("Authorization", "Bearer token");
    }

    #[test]
    fn test_host_set_header_various_values() {
        HostFunctions::host_set_header("X-Test", "simple");
        HostFunctions::host_set_header("X-Test", "with spaces");
        HostFunctions::host_set_header("X-Test", "with,commas");
        HostFunctions::host_set_header("X-Test", "with;semicolons");
        HostFunctions::host_set_header("X-Test", "with=equals");
        HostFunctions::host_set_header("X-Test", "");
    }

    #[test]
    fn test_host_metrics_various_names() {
        let names = vec![
            "simple",
            "with_dots",
            "with-dashes",
            "with/slashes",
            "with:colons",
            "request_count",
            "response_time_ms",
            "cache.hit_ratio",
        ];
        
        for name in names {
            HostFunctions::host_metrics_counter(name, 1.0);
            HostFunctions::host_metrics_gauge(name, 1.0);
            HostFunctions::host_metrics_histogram(name, 1.0);
        }
    }

    #[test]
    fn test_host_metrics_various_values() {
        // Test integer values
        HostFunctions::host_metrics_counter("test", 0.0);
        HostFunctions::host_metrics_counter("test", 1.0);
        HostFunctions::host_metrics_counter("test", 100.0);
        HostFunctions::host_metrics_counter("test", 1000000.0);
        
        // Test decimal values
        HostFunctions::host_metrics_gauge("test", 0.5);
        HostFunctions::host_metrics_gauge("test", 1.5);
        HostFunctions::host_metrics_gauge("test", 99.99);
        
        // Test negative values
        HostFunctions::host_metrics_gauge("test", -1.0);
        HostFunctions::host_metrics_gauge("test", -100.5);
        
        // Test very small values
        HostFunctions::host_metrics_histogram("test", 0.001);
        HostFunctions::host_metrics_histogram("test", 0.00001);
        
        // Test very large values
        HostFunctions::host_metrics_histogram("test", 1000000000.0);
    }

    #[test]
    fn test_host_functions_concurrent_calls() {
        use std::sync::Arc;
        use std::thread;
        
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    HostFunctions::host_log(1, &format!("Thread {} message", i));
                    HostFunctions::host_metrics_counter("thread_counter", i as f64);
                    HostFunctions::host_metrics_gauge("thread_gauge", i as f64);
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_define_host_functions_with_state() {
        let engine = Engine::default();
        
        // Test with i32 state
        let mut linker1: Linker<i32> = Linker::new(&engine);
        let result1 = define_host_functions(&mut linker1);
        assert!(result1.is_ok());
        
        // Test with String state
        let mut linker2: Linker<String> = Linker::new(&engine);
        let result2 = define_host_functions(&mut linker2);
        assert!(result2.is_ok());
        
        // Test with () state
        let mut linker3: Linker<()> = Linker::new(&engine);
        let result3 = define_host_functions(&mut linker3);
        assert!(result3.is_ok());
    }
}
