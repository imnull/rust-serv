//! Plugin SDK - 插件开发工具包
//!
//! 提供开发 rust-serv WebAssembly 插件所需的所有工具和接口。
//!
//! # 快速开始
//!
//! ```ignore
//! use rust_serv_plugin::{Plugin, PluginMetadata, export_plugin};
//!
//! pub struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> &PluginMetadata {
//!         &METADATA
//!     }
//!
//!     fn on_request(&mut self, req: &mut PluginRequest) -> Result<PluginAction, PluginError> {
//!         // 你的逻辑
//!         Ok(PluginAction::Continue)
//!     }
//! }
//!
//! export_plugin!(MyPlugin);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 重新导出核心类型
pub use error::{PluginError, PluginResult};
pub use types::*;

pub mod error;
pub mod host;
pub mod types;

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
/// use rust_serv_plugin::{Plugin, PluginMetadata, PluginConfig, PluginError, export_plugin};
///
/// pub struct MyPlugin {
///     metadata: PluginMetadata,
/// }
///
/// impl Default for MyPlugin {
///     fn default() -> Self {
///         Self {
///             metadata: PluginMetadata {
///                 id: "com.example.my-plugin".to_string(),
///                 name: "My Plugin".to_string(),
///                 version: "1.0.0".to_string(),
///                 description: "A test plugin".to_string(),
///                 author: "Your Name".to_string(),
///                 homepage: None,
///                 license: "MIT".to_string(),
///                 min_server_version: "0.3.0".to_string(),
///                 priority: 100,
///                 capabilities: vec![],
///                 permissions: vec![],
///             },
///         }
///     }
/// }
///
/// impl Plugin for MyPlugin {
///     fn metadata(&self) -> &PluginMetadata {
///         &self.metadata
///     }
/// }
///
/// export_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        use rust_serv_plugin::types::*;
        use std::collections::HashMap;

        /// 插件实例（全局）
        static mut PLUGIN: Option<$plugin_type> = None;

        /// 内存缓冲区（用于序列化/反序列化）
        static mut BUFFER: Vec<u8> = Vec::new();

        /// 解析配置
        unsafe fn parse_config(ptr: i32, len: i32) -> PluginConfig {
            let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            serde_json::from_slice(slice).unwrap_or_else(|_| PluginConfig {
                enabled: true,
                priority: None,
                timeout_ms: Some(100),
                custom: HashMap::new(),
            })
        }

        /// 解析请求
        unsafe fn parse_request(ptr: i32, len: i32) -> PluginRequest {
            let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            serde_json::from_slice(slice).unwrap_or_else(|_| PluginRequest {
                method: "GET".to_string(),
                path: "/".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: "127.0.0.1".to_string(),
                request_id: "unknown".to_string(),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            })
        }

        /// 解析响应
        unsafe fn parse_response(ptr: i32, len: i32) -> PluginResponse {
            let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            serde_json::from_slice(slice).unwrap_or_else(|_| PluginResponse::ok())
        }

        /// 写入动作到内存
        unsafe fn write_action(action: PluginAction, ptr: i32, len_ptr: i32) {
            let json = serde_json::to_vec(&action).unwrap_or_default();
            let len = json.len() as i32;

            // 写入长度到 len_ptr
            let len_slice = std::slice::from_raw_parts_mut(len_ptr as *mut u8, 4);
            len_slice.copy_from_slice(&len.to_le_bytes());

            // 写入数据到 ptr
            let data_slice = std::slice::from_raw_parts_mut(ptr as *mut u8, json.len());
            data_slice.copy_from_slice(&json);
        }

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
                        host_log(3, &format!("Plugin init failed: {}", e));
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

        /// Host 函数：日志
        #[link(wasm_import_module = "host")]
        extern "C" {
            fn host_log(level: i32, msg_ptr: i32, msg_len: i32);
        }

        /// 安全地调用 host_log
        unsafe fn host_log(level: i32, message: &str) {
            host_log(level, message.as_ptr() as i32, message.len() as i32);
        }
    };
}

// ============================================================================
// 内置示例插件
// ============================================================================

/// 示例：添加 Header 插件
///
/// 为每个响应添加自定义 Header
///
/// # 配置示例
/// ```toml
/// [plugins.load.config]
/// header_name = "X-Powered-By"
/// header_value = "rust-serv"
/// ```
#[derive(Debug)]
pub struct AddHeaderPlugin {
    metadata: PluginMetadata,
    header_name: String,
    header_value: String,
}

impl AddHeaderPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.add-header".to_string(),
                name: "Add Header Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Adds custom headers to responses".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 50,
                capabilities: vec![Capability::ModifyResponse],
                permissions: vec![],
            },
            header_name: "X-Powered-By".to_string(),
            header_value: "rust-serv".to_string(),
        }
    }

    /// 从配置创建
    pub fn with_config(header_name: &str, header_value: &str) -> Self {
        let mut plugin = Self::new();
        plugin.header_name = header_name.to_string();
        plugin.header_value = header_value.to_string();
        plugin
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
        Ok(PluginAction::ModifyResponse(response.clone()))
    }
}

/// 示例：速率限制插件
///
/// 基于 IP 的简单请求限流
///
/// # 配置示例
/// ```toml
/// [plugins.load.config]
/// requests_per_minute = 100
/// burst_size = 20
/// ```
#[derive(Debug)]
pub struct RateLimiterPlugin {
    metadata: PluginMetadata,
    requests_per_minute: u32,
    burst_size: u32,
    storage: HashMap<String, (u32, u64)>, // IP -> (count, timestamp)
}

impl RateLimiterPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.rate-limiter".to_string(),
                name: "Rate Limiter".to_string(),
                version: "1.0.0".to_string(),
                description: "IP-based rate limiting".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 200,
                capabilities: vec![Capability::InterceptRequest],
                permissions: vec![],
            },
            requests_per_minute: 100,
            burst_size: 20,
            storage: HashMap::new(),
        }
    }

    /// 获取当前时间戳（秒）
    fn now() -> u64 {
        // 在 Wasm 中使用 host 函数获取时间，这里简化处理
        0
    }

    /// 检查是否限流
    fn is_rate_limited(&mut self, ip: &str) -> bool {
        let now = Self::now();
        let window = 60; // 1 minute window

        if let Some((count, timestamp)) = self.storage.get(ip) {
            if now - timestamp < window {
                // 同一窗口内
                if *count >= self.requests_per_minute + self.burst_size {
                    return true;
                }
            } else {
                // 新窗口，重置计数
                self.storage.insert(ip.to_string(), (0, now));
            }
        } else {
            self.storage.insert(ip.to_string(), (0, now));
        }

        // 增加计数
        if let Some((count, _)) = self.storage.get_mut(ip) {
            *count += 1;
        }

        false
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
            self.requests_per_minute = rpm;
        }
        if let Some(burst) = config.get::<u32>("burst_size") {
            self.burst_size = burst;
        }
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let ip = &request.client_ip;

        if self.is_rate_limited(ip) {
            let response = PluginResponse::new(429)
                .with_header(
                    "X-RateLimit-Limit".to_string(),
                    self.requests_per_minute.to_string(),
                )
                .with_body("Rate limit exceeded. Please try again later.".to_string());

            Ok(PluginAction::Intercept(response))
        } else {
            Ok(PluginAction::Continue)
        }
    }
}

/// 示例：CORS 插件
///
/// 跨域资源共享支持
///
/// # 配置示例
/// ```toml
/// [plugins.load.config]
/// allowed_origins = ["https://example.com", "https://app.example.com"]
/// allowed_methods = ["GET", "POST", "PUT", "DELETE"]
/// allowed_headers = ["Content-Type", "Authorization"]
/// max_age = 86400
/// ```
#[derive(Debug)]
pub struct CorsPlugin {
    metadata: PluginMetadata,
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    max_age: u32,
    allow_credentials: bool,
}

impl CorsPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.cors".to_string(),
                name: "CORS Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Cross-Origin Resource Sharing support".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 150,
                capabilities: vec![Capability::ModifyRequest, Capability::ModifyResponse],
                permissions: vec![],
            },
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["*".to_string()],
            max_age: 86400,
            allow_credentials: false,
        }
    }

    /// 检查 Origin 是否允许
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins.contains(&"*".to_string()) {
            return true;
        }
        self.allowed_origins.contains(&origin.to_string())
    }
}

impl Default for CorsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CorsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(origins) = config.get::<Vec<String>>("allowed_origins") {
            self.allowed_origins = origins;
        }
        if let Some(methods) = config.get::<Vec<String>>("allowed_methods") {
            self.allowed_methods = methods;
        }
        if let Some(headers) = config.get::<Vec<String>>("allowed_headers") {
            self.allowed_headers = headers;
        }
        if let Some(max_age) = config.get::<u32>("max_age") {
            self.max_age = max_age;
        }
        if let Some(credentials) = config.get::<bool>("allow_credentials") {
            self.allow_credentials = credentials;
        }
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        // 处理预检请求 (OPTIONS)
        if request.method == "OPTIONS" {
            let origin = request.header("origin").unwrap_or(&"*".to_string()).clone();

            if !self.is_origin_allowed(&origin) {
                return Ok(PluginAction::Intercept(
                    PluginResponse::new(403).with_body("CORS origin not allowed".to_string())
                ));
            }

            let mut response = PluginResponse::new(204)
                .with_header("Access-Control-Allow-Origin".to_string(), origin)
                .with_header(
                    "Access-Control-Allow-Methods".to_string(),
                    self.allowed_methods.join(", "),
                )
                .with_header(
                    "Access-Control-Allow-Headers".to_string(),
                    self.allowed_headers.join(", "),
                )
                .with_header("Access-Control-Max-Age".to_string(), self.max_age.to_string());

            if self.allow_credentials {
                response = response.with_header(
                    "Access-Control-Allow-Credentials".to_string(),
                    "true".to_string(),
                );
            }

            return Ok(PluginAction::Intercept(response));
        }

        Ok(PluginAction::Continue)
    }

    fn on_response(&mut self, response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        // 添加 CORS 头到响应
        response.headers.insert(
            "Access-Control-Allow-Origin".to_string(),
            self.allowed_origins.join(", "),
        );

        if self.allow_credentials {
            response.headers.insert(
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            );
        }

        Ok(PluginAction::ModifyResponse(response.clone()))
    }
}

/// 示例：请求日志插件
///
/// 记录所有请求信息
///
/// # 配置示例
/// ```toml
/// [plugins.load.config]
/// log_level = "info"
/// include_headers = ["User-Agent", "Referer"]
/// ```
#[derive(Debug)]
pub struct RequestLogPlugin {
    metadata: PluginMetadata,
    log_level: LogLevel,
    include_headers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl RequestLogPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.request-log".to_string(),
                name: "Request Log Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Logs all incoming requests".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 10,
                capabilities: vec![Capability::Logging],
                permissions: vec![],
            },
            log_level: LogLevel::Info,
            include_headers: vec!["User-Agent".to_string()],
        }
    }

    /// 将日志级别转换为 i32
    fn level_to_i32(&self) -> i32 {
        match self.log_level {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

impl Default for RequestLogPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RequestLogPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(level) = config.get::<String>("log_level") {
            self.log_level = match level.as_str() {
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => LogLevel::Info,
            };
        }
        if let Some(headers) = config.get::<Vec<String>>("include_headers") {
            self.include_headers = headers;
        }
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let mut log_msg = format!(
            "[{}] {} {} - IP: {}",
            request.request_id, request.method, request.path, request.client_ip
        );

        // 添加 headers
        for header in &self.include_headers {
            if let Some(value) = request.header(header) {
                log_msg.push_str(&format!(" | {}: {}", header, value));
            }
        }

        // 在真实环境中，这里会调用 host_log
        // unsafe { host_log(self.level_to_i32(), &log_msg); }

        Ok(PluginAction::Continue)
    }
}

/// 示例：IP 白名单插件
///
/// 只允许特定 IP 访问
///
/// # 配置示例
/// ```toml
/// [plugins.load.config]
/// whitelist = ["192.168.1.0/24", "10.0.0.0/8", "127.0.0.1"]
/// deny_message = "Access denied from your IP"
/// ```
#[derive(Debug)]
pub struct IpWhitelistPlugin {
    metadata: PluginMetadata,
    whitelist: Vec<String>,
    deny_message: String,
}

impl IpWhitelistPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.ip-whitelist".to_string(),
                name: "IP Whitelist Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "IP-based access control".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 300,
                capabilities: vec![Capability::InterceptRequest],
                permissions: vec![],
            },
            whitelist: vec!["127.0.0.1".to_string()],
            deny_message: "Access denied from your IP address".to_string(),
        }
    }

    /// 检查 IP 是否在白名单中
    fn is_allowed(&self, ip: &str) -> bool {
        // 简化实现：支持精确匹配和 CIDR 前缀匹配
        for allowed in &self.whitelist {
            if allowed == "*" || allowed == ip {
                return true;
            }
            // CIDR 匹配简化版（实际实现会更复杂）
            if allowed.contains('/') {
                if ip.starts_with(&allowed[..allowed.rfind('/').unwrap()]) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for IpWhitelistPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for IpWhitelistPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(whitelist) = config.get::<Vec<String>>("whitelist") {
            self.whitelist = whitelist;
        }
        if let Some(message) = config.get::<String>("deny_message") {
            self.deny_message = message;
        }
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        if !self.is_allowed(&request.client_ip) {
            return Ok(PluginAction::Intercept(
                PluginResponse::new(403).with_body(self.deny_message.clone())
            ));
        }
        Ok(PluginAction::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::*;

    // =========================================================================
    // 基础类型测试
    // =========================================================================

    #[test]
    fn test_plugin_metadata_creation() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: Some("https://example.com".to_string()),
            license: "MIT".to_string(),
            min_server_version: "0.3.0".to_string(),
            priority: 100,
            capabilities: vec![Capability::ModifyRequest],
            permissions: vec![Permission::ReadEnv { allowed: vec!["PATH".to_string()] }],
        };

        assert_eq!(metadata.id, "test.plugin");
        assert_eq!(metadata.priority, 100);
    }

    #[test]
    fn test_plugin_config_get() {
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
    fn test_plugin_request_header() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("authorization".to_string(), "Bearer token123".to_string());

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "192.168.1.100".to_string(),
            request_id: "req-123".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        assert_eq!(request.header("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.header("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(request.header("X-Custom"), None);
    }

    #[test]
    fn test_plugin_request_query() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        query.insert("limit".to_string(), "20".to_string());

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            query,
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "req-456".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        assert_eq!(request.query("page"), Some(&"1".to_string()));
        assert_eq!(request.query("limit"), Some(&"20".to_string()));
        assert_eq!(request.query("sort"), None);
    }

    #[test]
    fn test_plugin_response_constructors() {
        let ok_res = PluginResponse::ok();
        assert_eq!(ok_res.status, 200);
        assert!(ok_res.headers.is_empty());

        let not_found = PluginResponse::not_found();
        assert_eq!(not_found.status, 404);

        let error = PluginResponse::internal_error();
        assert_eq!(error.status, 500);

        let custom = PluginResponse::new(201);
        assert_eq!(custom.status, 201);
    }

    #[test]
    fn test_plugin_response_with_header() {
        let response = PluginResponse::ok()
            .with_header("X-Custom".to_string(), "value".to_string())
            .with_header("X-Request-Id".to_string(), "123".to_string());

        assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(response.headers.get("X-Request-Id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_plugin_response_with_body() {
        let response = PluginResponse::ok()
            .with_body("Hello, World!".to_string());

        assert_eq!(response.body, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_plugin_response_json() {
        #[derive(Serialize)]
        struct TestData {
            message: String,
            count: i32,
        }

        let data = TestData {
            message: "Success".to_string(),
            count: 42,
        };

        let response = PluginResponse::ok().json(&data).unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
        assert!(response.body.is_some());
    }

    #[test]
    fn test_plugin_action_variants() {
        let continue_action = PluginAction::Continue;
        assert!(matches!(continue_action, PluginAction::Continue));

        let intercept = PluginAction::Intercept(PluginResponse::ok());
        assert!(matches!(intercept, PluginAction::Intercept(_)));

        let modify_req = PluginAction::ModifyRequest(PluginRequest {
            method: "POST".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        });
        assert!(matches!(modify_req, PluginAction::ModifyRequest(_)));

        let modify_res = PluginAction::ModifyResponse(PluginResponse::ok());
        assert!(matches!(modify_res, PluginAction::ModifyResponse(_)));

        let error = PluginAction::Error { message: "Test error".to_string() };
        assert!(matches!(error, PluginAction::Error { .. }));
    }

    #[test]
    fn test_capability_variants() {
        let caps = vec![
            Capability::ModifyRequest,
            Capability::ModifyResponse,
            Capability::InterceptRequest,
            Capability::AccessConfig,
            Capability::Logging,
            Capability::Metrics,
        ];

        assert_eq!(caps.len(), 6);
    }

    #[test]
    fn test_permission_variants() {
        let perms = vec![
            Permission::ReadEnv { allowed: vec!["PATH".to_string()] },
            Permission::HttpRequest { allowed_hosts: vec!["api.example.com".to_string()] },
            Permission::FileRead { allowed_paths: vec!["/tmp".to_string()] },
            Permission::FileWrite { allowed_paths: vec!["/tmp".to_string()] },
            Permission::NetworkAccess { allowed_ports: vec![80, 443] },
        ];

        assert_eq!(perms.len(), 5);
    }

    // =========================================================================
    // 辅助函数测试
    // =========================================================================

    #[test]
    fn test_base64_encode_decode() {
        let original = "Hello, World! 你好世界";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_decode_invalid() {
        let result = base64_decode("!!!invalid!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_empty() {
        let encoded = base64_encode("");
        assert_eq!(encoded, "");

        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, "");
    }

    // =========================================================================
    // AddHeaderPlugin 测试
    // =========================================================================

    #[test]
    fn test_add_header_plugin_default() {
        let plugin = AddHeaderPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "com.example.add-header");
        assert_eq!(metadata.priority, 50);
    }

    #[test]
    fn test_add_header_plugin_with_config() {
        let plugin = AddHeaderPlugin::with_config("X-Custom", "value");

        // 测试元数据
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "Add Header Plugin");
    }

    #[test]
    fn test_add_header_plugin_on_load() {
        let mut plugin = AddHeaderPlugin::new();

        let mut custom = HashMap::new();
        custom.insert("header_name".to_string(), serde_json::json!("X-Test"));
        custom.insert("header_value".to_string(), serde_json::json!("test-value"));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        plugin.on_load(&config).unwrap();

        // 测试 on_response
        let mut response = PluginResponse::ok();
        let action = plugin.on_response(&mut response).unwrap();

        assert!(matches!(action, PluginAction::ModifyResponse(_)));
        assert_eq!(response.headers.get("X-Test"), Some(&"test-value".to_string()));
    }

    #[test]
    fn test_add_header_plugin_response() {
        let mut plugin = AddHeaderPlugin::new();
        let mut response = PluginResponse::ok();

        let action = plugin.on_response(&mut response).unwrap();

        assert!(matches!(action, PluginAction::ModifyResponse(_)));
        assert_eq!(response.headers.get("X-Powered-By"), Some(&"rust-serv".to_string()));
    }

    // =========================================================================
    // RateLimiterPlugin 测试
    // =========================================================================

    #[test]
    fn test_rate_limiter_plugin_default() {
        let plugin = RateLimiterPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "com.example.rate-limiter");
        assert_eq!(metadata.priority, 200);
    }

    #[test]
    fn test_rate_limiter_plugin_on_load() {
        let mut plugin = RateLimiterPlugin::new();

        let mut custom = HashMap::new();
        custom.insert("requests_per_minute".to_string(), serde_json::json!(50));
        custom.insert("burst_size".to_string(), serde_json::json!(10));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        plugin.on_load(&config).unwrap();
        assert_eq!(plugin.requests_per_minute, 50);
        assert_eq!(plugin.burst_size, 10);
    }

    #[test]
    fn test_rate_limiter_plugin_request_allowed() {
        let mut plugin = RateLimiterPlugin::new();

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

        // 第一次请求应该通过
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    // =========================================================================
    // CorsPlugin 测试
    // =========================================================================

    #[test]
    fn test_cors_plugin_default() {
        let plugin = CorsPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "com.example.cors");
        assert_eq!(metadata.priority, 150);
    }

    #[test]
    fn test_cors_plugin_is_origin_allowed() {
        let plugin = CorsPlugin::new();

        assert!(plugin.is_origin_allowed("https://example.com"));
        assert!(plugin.is_origin_allowed("https://any.com")); // 默认允许所有
    }

    #[test]
    fn test_cors_plugin_on_load() {
        let mut plugin = CorsPlugin::new();

        let mut custom = HashMap::new();
        custom.insert(
            "allowed_origins".to_string(),
            serde_json::json!(["https://example.com"]),
        );
        custom.insert("max_age".to_string(), serde_json::json!(3600));
        custom.insert("allow_credentials".to_string(), serde_json::json!(true));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        plugin.on_load(&config).unwrap();
        assert_eq!(plugin.allowed_origins, vec!["https://example.com"]);
        assert_eq!(plugin.max_age, 3600);
        assert!(plugin.allow_credentials);
    }

    #[test]
    fn test_cors_plugin_preflight_request() {
        let mut plugin = CorsPlugin::new();

        let mut request = PluginRequest {
            method: "OPTIONS".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Intercept(_)));
    }

    #[test]
    fn test_cors_plugin_response() {
        let mut plugin = CorsPlugin::new();
        let mut response = PluginResponse::ok();

        let action = plugin.on_response(&mut response).unwrap();

        assert!(matches!(action, PluginAction::ModifyResponse(_)));
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    }

    // =========================================================================
    // RequestLogPlugin 测试
    // =========================================================================

    #[test]
    fn test_request_log_plugin_default() {
        let plugin = RequestLogPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "com.example.request-log");
        assert_eq!(metadata.priority, 10);
    }

    #[test]
    fn test_request_log_plugin_on_load() {
        let mut plugin = RequestLogPlugin::new();

        let mut custom = HashMap::new();
        custom.insert("log_level".to_string(), serde_json::json!("debug"));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        plugin.on_load(&config).unwrap();
        assert_eq!(plugin.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_request_log_plugin_request() {
        let mut plugin = RequestLogPlugin::new();

        let mut headers = HashMap::new();
        headers.insert("user-agent".to_string(), "Mozilla/5.0".to_string());

        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "req-123".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_log_level_conversion() {
        let plugin = RequestLogPlugin::new();

        assert_eq!(plugin.level_to_i32(), 1); // Info

        let mut debug_plugin = RequestLogPlugin::new();
        let mut custom = HashMap::new();
        custom.insert("log_level".to_string(), serde_json::json!("debug"));
        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };
        debug_plugin.on_load(&config).unwrap();
        assert_eq!(debug_plugin.level_to_i32(), 0); // Debug
    }

    // =========================================================================
    // IpWhitelistPlugin 测试
    // =========================================================================

    #[test]
    fn test_ip_whitelist_plugin_default() {
        let plugin = IpWhitelistPlugin::new();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "com.example.ip-whitelist");
        assert_eq!(metadata.priority, 300);
    }

    #[test]
    fn test_ip_whitelist_plugin_is_allowed() {
        let plugin = IpWhitelistPlugin::new();

        assert!(plugin.is_allowed("127.0.0.1"));
        assert!(!plugin.is_allowed("192.168.1.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_with_wildcard() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["*".to_string()];

        assert!(plugin.is_allowed("192.168.1.1"));
        assert!(plugin.is_allowed("10.0.0.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_on_load() {
        let mut plugin = IpWhitelistPlugin::new();

        let mut custom = HashMap::new();
        custom.insert(
            "whitelist".to_string(),
            serde_json::json!(["10.0.0.0/8", "192.168.1.0/24"]),
        );
        custom.insert("deny_message".to_string(), serde_json::json!("Access denied!"));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        plugin.on_load(&config).unwrap();
        assert!(plugin.whitelist.contains(&"10.0.0.0/8".to_string()));
        assert_eq!(plugin.deny_message, "Access denied!");
    }

    #[test]
    fn test_ip_whitelist_plugin_allowed_request() {
        let mut plugin = IpWhitelistPlugin::new();

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

        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_ip_whitelist_plugin_denied_request() {
        let mut plugin = IpWhitelistPlugin::new();

        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/admin".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "192.168.1.100".to_string(), // 不在白名单中
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Intercept(_)));
    }

    // =========================================================================
    // Trait 默认实现测试
    // =========================================================================

    #[test]
    fn test_plugin_default_implementations() {
        struct TestPlugin;

        impl Plugin for TestPlugin {
            fn metadata(&self) -> &PluginMetadata {
                static META: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
                META.get_or_init(|| PluginMetadata {
                    id: "test".to_string(),
                    name: "Test".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test".to_string(),
                    author: "Test".to_string(),
                    homepage: None,
                    license: "MIT".to_string(),
                    min_server_version: "0.1.0".to_string(),
                    priority: 100,
                    capabilities: vec![],
                    permissions: vec![],
                })
            }
        }

        let mut plugin = TestPlugin;
        let config = PluginConfig::default();

        // 测试默认实现
        assert!(plugin.on_load(&config).is_ok());
        assert!(plugin.on_config_change(&config).is_ok());
        assert!(plugin.on_unload().is_ok());

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
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));

        let mut response = PluginResponse::ok();
        let action = plugin.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    // =========================================================================
    // 序列化测试
    // =========================================================================

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: Some("https://example.com".to_string()),
            license: "MIT".to_string(),
            min_server_version: "0.3.0".to_string(),
            priority: 100,
            capabilities: vec![Capability::ModifyRequest, Capability::ModifyResponse],
            permissions: vec![Permission::ReadEnv { allowed: vec!["PATH".to_string()] }],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.capabilities.len(), deserialized.capabilities.len());
    }

    #[test]
    fn test_plugin_request_serialization() {
        let request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            query: [("page".to_string(), "1".to_string())].into_iter().collect(),
            headers: [("content-type".to_string(), "application/json".to_string())].into_iter().collect(),
            body: Some("test body".to_string()),
            client_ip: "192.168.1.1".to_string(),
            request_id: "req-123".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "example.com".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: PluginRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.path, deserialized.path);
    }

    #[test]
    fn test_plugin_action_serialization() {
        let actions = vec![
            PluginAction::Continue,
            PluginAction::Intercept(PluginResponse::ok()),
            PluginAction::Error { message: "test".to_string() },
        ];

        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let deserialized: PluginAction = serde_json::from_str(&json).unwrap();

            match (&action, &deserialized) {
                (PluginAction::Continue, PluginAction::Continue) => {}
                (PluginAction::Intercept(_), PluginAction::Intercept(_)) => {}
                (PluginAction::Error { message: m1 }, PluginAction::Error { message: m2 }) => {
                    assert_eq!(m1, m2);
                }
                _ => panic!("Serialization mismatch"),
            }
        }
    }

    // =========================================================================
    // 扩展测试 - AddHeaderPlugin
    // =========================================================================

    #[test]
    fn test_add_header_plugin_with_config_constructor() {
        let plugin = AddHeaderPlugin::with_config("X-Custom", "custom-value");
        let metadata = plugin.metadata();
        
        assert_eq!(metadata.id, "com.example.add-header");
        assert_eq!(metadata.priority, 50);
    }

    #[test]
    fn test_add_header_plugin_on_load_no_config() {
        let mut plugin = AddHeaderPlugin::new();
        let config = PluginConfig::default();
        
        plugin.on_load(&config).unwrap();
        
        // 应该使用默认值
        let mut response = PluginResponse::ok();
        plugin.on_response(&mut response).unwrap();
        
        assert_eq!(response.headers.get("X-Powered-By"), Some(&"rust-serv".to_string()));
    }

    #[test]
    fn test_add_header_plugin_on_unload() {
        let mut plugin = AddHeaderPlugin::new();
        assert!(plugin.on_unload().is_ok());
    }

    #[test]
    fn test_add_header_plugin_multiple_headers() {
        let mut plugin1 = AddHeaderPlugin::with_config("X-Header-1", "value1");
        let mut plugin2 = AddHeaderPlugin::with_config("X-Header-2", "value2");
        
        let mut response = PluginResponse::ok();
        plugin1.on_response(&mut response).unwrap();
        plugin2.on_response(&mut response).unwrap();
        
        assert_eq!(response.headers.get("X-Header-1"), Some(&"value1".to_string()));
        assert_eq!(response.headers.get("X-Header-2"), Some(&"value2".to_string()));
    }

    // =========================================================================
    // 扩展测试 - RateLimiterPlugin
    // =========================================================================

    #[test]
    fn test_rate_limiter_plugin_burst_limit() {
        let mut plugin = RateLimiterPlugin::new();
        plugin.requests_per_minute = 2;
        plugin.burst_size = 1;
        
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "192.168.1.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        // 前 3 个请求应该通过 (rpm=2 + burst=1)
        for _ in 0..3 {
            let action = plugin.on_request(&mut request).unwrap();
            assert!(matches!(action, PluginAction::Continue));
        }
        
        // 第 4 个请求应该被限流
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Intercept(_)));
    }

    #[test]
    fn test_rate_limiter_plugin_different_ips() {
        let mut plugin = RateLimiterPlugin::new();
        plugin.requests_per_minute = 1;
        plugin.burst_size = 0;
        
        let mut request1 = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "192.168.1.1".to_string(),
            request_id: "test1".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        let mut request2 = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "192.168.1.2".to_string(),
            request_id: "test2".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        // 每个 IP 的第一个请求都应该通过
        let action1 = plugin.on_request(&mut request1).unwrap();
        let action2 = plugin.on_request(&mut request2).unwrap();
        
        assert!(matches!(action1, PluginAction::Continue));
        assert!(matches!(action2, PluginAction::Continue));
    }

    #[test]
    fn test_rate_limiter_plugin_cleanup() {
        let mut plugin = RateLimiterPlugin::new();
        
        // 添加大量记录触发清理
        for i in 0..150 {
            let mut request = PluginRequest {
                method: "GET".to_string(),
                path: "/".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                client_ip: format!("192.168.1.{}", i % 256),
                request_id: format!("test{}", i),
                version: "HTTP/1.1".to_string(),
                host: "localhost".to_string(),
            };
            plugin.on_request(&mut request).unwrap();
        }
        
        // 应该触发清理
        assert!(plugin.storage.len() <= 100);
    }

    #[test]
    fn test_rate_limiter_plugin_on_unload() {
        let mut plugin = RateLimiterPlugin::new();
        assert!(plugin.on_unload().is_ok());
    }

    // =========================================================================
    // 扩展测试 - CorsPlugin
    // =========================================================================

    #[test]
    fn test_cors_plugin_is_origin_allowed_specific() {
        let mut plugin = CorsPlugin::new();
        plugin.allowed_origins = vec!["https://example.com".to_string(), "https://app.example.com".to_string()];
        
        assert!(plugin.is_origin_allowed("https://example.com"));
        assert!(plugin.is_origin_allowed("https://app.example.com"));
        assert!(!plugin.is_origin_allowed("https://evil.com"));
        assert!(!plugin.is_origin_allowed("https://example.com.evil.com"));
    }

    #[test]
    fn test_cors_plugin_preflight_blocked_origin() {
        let mut plugin = CorsPlugin::new();
        plugin.allowed_origins = vec!["https://allowed.com".to_string()];
        
        let mut request = PluginRequest {
            method: "OPTIONS".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers: [("origin".to_string(), "https://blocked.com".to_string())].into_iter().collect(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Intercept(resp) if resp.status == 403));
    }

    #[test]
    fn test_cors_plugin_build_cors_headers() {
        let plugin = CorsPlugin::new();
        let headers = plugin.build_cors_headers("https://example.com");
        
        let header_map: HashMap<_, _> = headers.into_iter().collect();
        assert!(header_map.contains_key("Access-Control-Allow-Origin"));
        assert!(header_map.contains_key("Access-Control-Allow-Methods"));
        assert!(header_map.contains_key("Access-Control-Allow-Headers"));
        assert!(header_map.contains_key("Access-Control-Max-Age"));
    }

    #[test]
    fn test_cors_plugin_response_with_credentials() {
        let mut plugin = CorsPlugin::new();
        plugin.allow_credentials = true;
        
        let mut response = PluginResponse::ok();
        let action = plugin.on_response(&mut response).unwrap();
        
        assert!(matches!(action, PluginAction::ModifyResponse(_)));
        assert_eq!(response.headers.get("Access-Control-Allow-Credentials"), Some(&"true".to_string()));
    }

    #[test]
    fn test_cors_plugin_regular_request_not_options() {
        let mut plugin = CorsPlugin::new();
        
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        // GET 请求应该继续处理
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_cors_plugin_on_unload() {
        let mut plugin = CorsPlugin::new();
        assert!(plugin.on_unload().is_ok());
    }

    // =========================================================================
    // 扩展测试 - RequestLogPlugin
    // =========================================================================

    #[test]
    fn test_request_log_plugin_all_levels() {
        let levels = vec![
            ("debug", LogLevel::Debug, 0),
            ("info", LogLevel::Info, 1),
            ("warn", LogLevel::Warn, 2),
            ("error", LogLevel::Error, 3),
            ("unknown", LogLevel::Info, 1), // 默认 info
        ];
        
        for (level_str, expected_level, expected_i32) in levels {
            let mut plugin = RequestLogPlugin::new();
            let mut custom = HashMap::new();
            custom.insert("log_level".to_string(), serde_json::json!(level_str));
            let config = PluginConfig {
                enabled: true,
                priority: None,
                timeout_ms: None,
                custom,
            };
            plugin.on_load(&config).unwrap();
            
            assert_eq!(plugin.log_level, expected_level, "Failed for level: {}", level_str);
            assert_eq!(plugin.level_to_i32(), expected_i32);
        }
    }

    #[test]
    fn test_request_log_plugin_on_load_with_headers() {
        let mut plugin = RequestLogPlugin::new();
        let mut custom = HashMap::new();
        custom.insert(
            "include_headers".to_string(),
            serde_json::json!(["User-Agent", "Referer", "Authorization"]),
        );
        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };
        
        plugin.on_load(&config).unwrap();
        assert_eq!(plugin.include_headers.len(), 3);
        assert!(plugin.include_headers.contains(&"User-Agent".to_string()));
    }

    #[test]
    fn test_request_log_plugin_request_without_headers() {
        let mut plugin = RequestLogPlugin::new();
        
        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/api".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "req-no-headers".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        let mut request_clone = request.clone();
        let action = plugin.on_request(&mut request_clone).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_request_log_plugin_on_unload() {
        let mut plugin = RequestLogPlugin::new();
        assert!(plugin.on_unload().is_ok());
    }

    // =========================================================================
    // 扩展测试 - IpWhitelistPlugin
    // =========================================================================

    #[test]
    fn test_ip_whitelist_plugin_cidr_24() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["192.168.1.0/24".to_string()];
        
        assert!(plugin.is_allowed("192.168.1.1"));
        assert!(plugin.is_allowed("192.168.1.100"));
        assert!(plugin.is_allowed("192.168.1.254"));
        assert!(!plugin.is_allowed("192.168.2.1"));
        assert!(!plugin.is_allowed("10.0.0.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_cidr_16() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["10.0.0.0/16".to_string()];
        
        assert!(plugin.is_allowed("10.0.1.1"));
        assert!(plugin.is_allowed("10.0.100.50"));
        assert!(!plugin.is_allowed("10.1.0.1"));
        assert!(!plugin.is_allowed("192.168.1.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_cidr_8() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["10.0.0.0/8".to_string()];
        
        assert!(plugin.is_allowed("10.1.2.3"));
        assert!(plugin.is_allowed("10.100.200.50"));
        assert!(!plugin.is_allowed("11.0.0.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_invalid_cidr() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["invalid-cidr".to_string(), "192.168.1.0/99".to_string()];
        
        // 无效 CIDR 应该被忽略，返回 false
        assert!(!plugin.is_allowed("192.168.1.1"));
        assert!(!plugin.is_allowed("anything"));
    }

    #[test]
    fn test_ip_whitelist_plugin_prefix_match() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec!["192.168.1.".to_string()];
        
        assert!(plugin.is_allowed("192.168.1.100"));
        assert!(plugin.is_allowed("192.168.1.1"));
        assert!(!plugin.is_allowed("192.168.2.1"));
        assert!(!plugin.is_allowed("192.168.10.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_on_load_default() {
        let mut plugin = IpWhitelistPlugin::new();
        let config = PluginConfig::default();
        
        plugin.on_load(&config).unwrap();
        // 应该保持默认白名单
        assert!(plugin.is_allowed("127.0.0.1"));
    }

    #[test]
    fn test_ip_whitelist_plugin_intercept_response() {
        let mut plugin = IpWhitelistPlugin::new();
        
        let mut request = PluginRequest {
            method: "GET".to_string(),
            path: "/admin".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "10.0.0.1".to_string(),
            request_id: "test".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };
        
        let action = plugin.on_request(&mut request).unwrap();
        
        if let PluginAction::Intercept(response) = action {
            assert_eq!(response.status, 403);
            assert_eq!(response.body, Some("Access denied from your IP address".to_string()));
        } else {
            panic!("Expected Intercept action");
        }
    }

    #[test]
    fn test_ip_whitelist_plugin_empty_whitelist() {
        let mut plugin = IpWhitelistPlugin::new();
        plugin.whitelist = vec![];
        
        // 空白名单应该拒绝所有
        assert!(!plugin.is_allowed("127.0.0.1"));
        assert!(!plugin.is_allowed("192.168.1.1"));
    }

    // =========================================================================
    // 边界测试
    // =========================================================================

    #[test]
    fn test_base64_roundtrip_various_strings() {
        let test_cases = vec![
            "",
            "Hello",
            "Hello, World!",
            "Special chars: !@#$%^&*()",
            "Unicode: 你好世界 🌍🎉",
            &"Very long string: ".repeat(1000),
            "With\nnewlines\nand\ttabs",
        ];
        
        for original in test_cases {
            let encoded = base64_encode(original);
            let decoded = base64_decode(&encoded).unwrap();
            assert_eq!(original, decoded, "Failed for: {}", &original[..original.len().min(50)]);
        }
    }

    #[test]
    fn test_plugin_config_invalid_type_conversion() {
        let mut custom = HashMap::new();
        custom.insert("string_value".to_string(), serde_json::json!("not a number"));
        custom.insert("null_value".to_string(), serde_json::Value::Null);
        custom.insert("array_value".to_string(), serde_json::json!([1, 2, 3]));
        
        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };
        
        // 尝试将字符串转换为数字应该失败
        assert_eq!(config.get::<i32>("string_value"), None);
        assert_eq!(config.get::<i32>("null_value"), None);
        assert_eq!(config.get::<i32>("array_value"), None);
    }
}
