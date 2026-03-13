/// Plugin SDK - 插件开发接口定义
/// 
/// 本模块定义了插件开发者需要实现的核心 trait 和数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

// ============================================================================
// 插件元数据
// ============================================================================

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub license: String,
    pub min_server_version: String,
    pub priority: i32,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
}

// ============================================================================
// 插件配置
// ============================================================================

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: Option<i32>,
    pub timeout_ms: Option<u64>,
    pub custom: HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.custom.get(key).and_then(|v| {
            serde_json::from_value(v.clone()).ok()
        })
    }
}

// ============================================================================
// 插件数据结构
// ============================================================================

/// HTTP 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,  // Base64 encoded
    pub client_ip: String,
    pub request_id: String,
    pub version: String,
    pub host: String,
}

impl PluginRequest {
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
    
    pub fn query(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }
}

/// HTTP 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,  // Base64 encoded
}

impl PluginResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }
    
    pub fn ok() -> Self {
        Self::new(200)
    }
    
    pub fn not_found() -> Self {
        Self::new(404)
    }
    
    pub fn internal_error() -> Self {
        Self::new(500)
    }
    
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.insert(name, value);
        self
    }
    
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
    
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, PluginError> {
        let json = serde_json::to_string(data)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        self.body = Some(base64_encode(&json));
        Ok(self)
    }
}

// ============================================================================
// 插件动作
// ============================================================================

/// 插件执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginAction {
    /// 继续执行
    Continue,
    
    /// 拦截并返回响应
    Intercept(PluginResponse),
    
    /// 修改请求
    ModifyRequest(PluginRequest),
    
    /// 修改响应
    ModifyResponse(PluginResponse),
    
    /// 错误
    Error { message: String },
}

// ============================================================================
// 插件错误
// ============================================================================

/// 插件错误
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("{0}")]
    Other(String),
}

// ============================================================================
// 插件 Trait
// ============================================================================

/// 插件主 trait
/// 
/// 所有插件必须实现此 trait
pub trait Plugin: Send + Sync {
    /// 返回插件元数据
    fn metadata(&self) -> &PluginMetadata;
    
    /// 插件加载时调用
    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        let _ = config;
        Ok(())
    }
    
    /// 配置变更时调用
    fn on_config_change(&mut self, _new_config: &PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// 处理 HTTP 请求
    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let _ = request;
        Ok(PluginAction::Continue)
    }
    
    /// 处理 HTTP 响应
    fn on_response(&mut self, response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        let _ = response;
        Ok(PluginAction::Continue)
    }
    
    /// 插件卸载时调用
    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// Base64 编码
pub fn base64_encode(data: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}

/// Base64 解码
pub fn base64_decode(data: &str) -> Result<String, PluginError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map(|v| String::from_utf8(v).unwrap_or_default())
        .map_err(|e| PluginError::Serialization(e.to_string()))
}

// ============================================================================
// 插件导出宏
// ============================================================================

/// 导出插件
/// 
/// 在插件 crate 的 lib.rs 中使用：
/// 
/// ```ignore
/// rust_serv_plugin::export_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        /// 插件实例（全局）
        static mut PLUGIN: Option<$plugin_type> = None;
        
        /// 初始化插件
        #[no_mangle]
        pub extern "C" fn plugin_init(
            config_ptr: i32,
            config_len: i32,
        ) -> i32 {
            unsafe {
                if PLUGIN.is_some() {
                    return 1000; // Already initialized
                }
                
                let config = parse_config(config_ptr, config_len);
                let mut plugin = <$plugin_type>::default();
                
                match plugin.on_load(&config) {
                    Ok(()) => {
                        PLUGIN = Some(plugin);
                        0
                    }
                    Err(e) => {
                        eprintln!("Plugin init failed: {}", e);
                        1001
                    }
                }
            }
        }
        
        /// 处理请求
        #[no_mangle]
        pub extern "C" fn plugin_on_request(
            req_ptr: i32,
            req_len: i32,
            action_ptr: i32,
            action_len_ptr: i32,
        ) -> i32 {
            unsafe {
                if let Some(ref mut plugin) = PLUGIN {
                    let mut req = parse_request(req_ptr, req_len);
                    match plugin.on_request(&mut req) {
                        Ok(action) => {
                            write_action(action, action_ptr, action_len_ptr);
                            0
                        }
                        Err(e) => {
                            write_action(
                                PluginAction::Error { message: e.to_string() },
                                action_ptr,
                                action_len_ptr,
                            );
                            2000
                        }
                    }
                } else {
                    1002 // Not initialized
                }
            }
        }
        
        /// 处理响应
        #[no_mangle]
        pub extern "C" fn plugin_on_response(
            res_ptr: i32,
            res_len: i32,
            action_ptr: i32,
            action_len_ptr: i32,
        ) -> i32 {
            unsafe {
                if let Some(ref mut plugin) = PLUGIN {
                    let mut res = parse_response(res_ptr, res_len);
                    match plugin.on_response(&mut res) {
                        Ok(action) => {
                            write_action(action, action_ptr, action_len_ptr);
                            0
                        }
                        Err(e) => {
                            write_action(
                                PluginAction::Error { message: e.to_string() },
                                action_ptr,
                                action_len_ptr,
                            );
                            2000
                        }
                    }
                } else {
                    1002 // Not initialized
                }
            }
        }
        
        /// 卸载插件
        #[no_mangle]
        pub extern "C" fn plugin_unload() -> i32 {
            unsafe {
                if let Some(mut plugin) = PLUGIN.take() {
                    match plugin.on_unload() {
                        Ok(()) => 0,
                        Err(_) => 1003,
                    }
                } else {
                    0
                }
            }
        }
    };
}

// ============================================================================
// 示例插件
// ============================================================================

/// 示例：简单的 Header 插件
/// 
/// 为每个响应添加自定义 Header
pub struct AddHeaderPlugin {
    metadata: PluginMetadata,
    header_name: String,
    header_value: String,
}

impl AddHeaderPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.add-header".to_string(),
                name: "Add Header Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Adds custom headers to responses".to_string(),
                author: "Your Name".to_string(),
                homepage: None,
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 50,
                capabilities: vec!["ModifyResponse".to_string()],
                permissions: vec![],
            },
            header_name: "X-Powered-By".to_string(),
            header_value: "rust-serv".to_string(),
        }
    }
}

impl Default for AddHeaderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AddHeaderPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(name) = config.get::<String>("header_name") {
            self.header_name = name;
        }
        if let Some(value) = config.get::<String>("header_value") {
            self.header_value = value;
        }
        Ok(())
    }
    
    fn on_response(&mut self, response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        response.headers.insert(
            self.header_name.clone(),
            self.header_value.clone(),
        );
        Ok(PluginAction::Continue)
    }
}

/// 示例：Rate Limiter 插件
/// 
/// 基于 IP 的请求限流
pub struct RateLimiterPlugin {
    metadata: PluginMetadata,
    rpm: u32,  // requests per minute
    storage: HashMap<String, u32>,
}

impl RateLimiterPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.rate-limiter".to_string(),
                name: "Rate Limiter".to_string(),
                version: "1.0.0".to_string(),
                description: "IP-based rate limiting".to_string(),
                author: "Your Name".to_string(),
                homepage: None,
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 200,
                capabilities: vec!["InterceptRequest".to_string()],
                permissions: vec![],
            },
            rpm: 100,
            storage: HashMap::new(),
        }
    }
}

impl Default for RateLimiterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RateLimiterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(rpm) = config.get::<u32>("requests_per_minute") {
            self.rpm = rpm;
        }
        Ok(())
    }
    
    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let ip = &request.client_ip;
        let count = self.storage.entry(ip.clone()).or_insert(0);
        
        if *count >= self.rpm {
            return Ok(PluginAction::Intercept(
                PluginResponse::new(429)
                    .with_header("X-RateLimit-Limit".to_string(), self.rpm.to_string())
                    .with_body("Rate limit exceeded".to_string())
            ));
        }
        
        *count += 1;
        Ok(PluginAction::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_header_plugin() {
        let mut plugin = AddHeaderPlugin::new();
        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom: HashMap::new(),
        };
        
        plugin.on_load(&config).unwrap();
        
        let mut response = PluginResponse::ok();
        let action = plugin.on_response(&mut response).unwrap();
        
        assert!(matches!(action, PluginAction::Continue));
        assert_eq!(response.headers.get("X-Powered-By"), Some(&"rust-serv".to_string()));
    }
    
    #[test]
    fn test_rate_limiter_plugin() {
        let mut plugin = RateLimiterPlugin::new();
        let mut config = HashMap::new();
        config.insert("requests_per_minute".to_string(), serde_json::json!(2));
        
        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom: config,
        };
        
        plugin.on_load(&config).unwrap();
        
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
        
        // First request should pass
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
        
        // Second request should pass
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
        
        // Third request should be rate limited
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Intercept(_)));
    }
}
