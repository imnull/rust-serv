//! Plugin traits and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Capability, Permission};
use crate::plugin::PluginError;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,
    
    /// Plugin name
    pub name: String,
    
    /// Plugin version (semantic versioning)
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Author information
    pub author: String,
    
    /// Plugin homepage URL
    pub homepage: Option<String>,
    
    /// License (SPDX identifier)
    pub license: String,
    
    /// Minimum rust-serv version required
    pub min_server_version: String,
    
    /// Plugin priority (higher = earlier execution)
    pub priority: i32,
    
    /// Plugin capabilities
    pub capabilities: Vec<Capability>,
    
    /// Required permissions
    pub permissions: Vec<Permission>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether plugin is enabled
    pub enabled: bool,
    
    /// Plugin priority override
    pub priority: Option<i32>,
    
    /// Execution timeout in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Custom configuration values
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    /// Get a configuration value
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.custom.get(key).and_then(|v| {
            T::deserialize(v.clone()).ok()
        })
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: None,
            timeout_ms: Some(100),
            custom: HashMap::new(),
        }
    }
}

/// HTTP request for plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    /// HTTP method
    pub method: String,
    
    /// Request path
    pub path: String,
    
    /// Query parameters
    pub query: HashMap<String, String>,
    
    /// Request headers
    pub headers: HashMap<String, String>,
    
    /// Request body (Base64 encoded)
    pub body: Option<String>,
    
    /// Client IP address
    pub client_ip: String,
    
    /// Request ID
    pub request_id: String,
    
    /// HTTP version
    pub version: String,
    
    /// Host header
    pub host: String,
}

impl PluginRequest {
    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
    
    /// Get a query parameter
    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }
}

/// HTTP response for plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// HTTP status code
    pub status: u16,
    
    /// Response headers
    pub headers: HashMap<String, String>,
    
    /// Response body (Base64 encoded)
    pub body: Option<String>,
}

impl PluginResponse {
    /// Create new response with status
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }
    
    /// Create 200 OK response
    pub fn ok() -> Self {
        Self::new(200)
    }
    
    /// Create 404 Not Found response
    pub fn not_found() -> Self {
        Self::new(404)
    }
    
    /// Create 500 Internal Server Error response
    pub fn internal_error() -> Self {
        Self::new(500)
    }
    
    /// Add a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }
    
    /// Set body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    
    /// Set JSON body
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, PluginError> {
        let json = serde_json::to_string(data)
            .map_err(|e| PluginError::Serialization(e.to_string()))?;
        
        self.headers.insert(
            "content-type".to_string(),
            "application/json".to_string(),
        );
        self.body = Some(base64_encode(&json));
        Ok(self)
    }
}

/// Plugin action returned from plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginAction {
    /// Continue to next plugin
    Continue,
    
    /// Intercept and return response immediately
    Intercept(PluginResponse),
    
    /// Modify request and continue
    ModifyRequest(PluginRequest),
    
    /// Modify response and continue
    ModifyResponse(PluginResponse),
    
    /// Plugin encountered an error
    Error {
        message: String,
    },
}

/// Plugin trait
///
/// All plugins must implement this trait
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Called when plugin is loaded
    fn on_load(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Called when configuration changes
    fn on_config_change(&mut self, _new_config: &PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Called for each HTTP request
    fn on_request(&mut self, _request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        Ok(PluginAction::Continue)
    }
    
    /// Called for each HTTP response
    fn on_response(&mut self, _response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        Ok(PluginAction::Continue)
    }
    
    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

// Helper functions

/// Base64 encode
pub fn base64_encode(data: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}

/// Base64 decode
pub fn base64_decode(data: &str) -> Result<String, PluginError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map(|v| String::from_utf8(v).unwrap_or_default())
        .map_err(|e| PluginError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_response() {
        let res = PluginResponse::ok()
            .with_header("X-Custom", "value")
            .with_body("test");
        
        assert_eq!(res.status, 200);
        assert_eq!(res.headers.get("X-Custom"), Some(&"value".to_string()));
        assert_eq!(res.body, Some("test".to_string()));
    }
    
    #[test]
    fn test_plugin_config() {
        let mut custom = HashMap::new();
        custom.insert("key".to_string(), serde_json::json!("value"));
        
        let config = PluginConfig {
            enabled: true,
            priority: Some(100),
            timeout_ms: Some(50),
            custom,
        };
        
        assert_eq!(config.get::<String>("key"), Some("value".to_string()));
    }
    
    #[test]
    fn test_base64() {
        let original = "Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_plugin_metadata_serialization() {
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

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        
        assert!(config.enabled);
        assert_eq!(config.priority, None);
        assert_eq!(config.timeout_ms, Some(100));
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_plugin_config_get_nonexistent() {
        let config = PluginConfig::default();
        let result: Option<String> = config.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_plugin_request_header() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-1".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        assert_eq!(request.header("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.header("CONTENT-TYPE"), Some(&"application/json".to_string()));
        assert_eq!(request.header("authorization"), None);
    }

    #[test]
    fn test_plugin_request_query_param() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "2".to_string());
        query.insert("limit".to_string(), "10".to_string());
        
        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/users".to_string(),
            query,
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "test-2".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        assert_eq!(request.query_param("page"), Some(&"2".to_string()));
        assert_eq!(request.query_param("limit"), Some(&"10".to_string()));
        assert_eq!(request.query_param("sort"), None);
    }

    #[test]
    fn test_plugin_response_status_codes() {
        assert_eq!(PluginResponse::ok().status, 200);
        assert_eq!(PluginResponse::not_found().status, 404);
        assert_eq!(PluginResponse::internal_error().status, 500);
        assert_eq!(PluginResponse::new(201).status, 201);
    }

    #[test]
    fn test_plugin_response_with_methods() {
        let response = PluginResponse::ok()
            .with_header("X-Request-Id", "12345")
            .with_header("X-Rate-Limit", "100")
            .with_body("Response body");

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.body, Some("Response body".to_string()));
    }

    #[test]
    fn test_plugin_response_json() {
        #[derive(Serialize)]
        struct TestData {
            message: String,
            count: i32,
        }

        let data = TestData {
            message: "Hello".to_string(),
            count: 42,
        };

        let response = PluginResponse::ok().json(&data).unwrap();

        assert_eq!(response.headers.get("content-type"), Some(&"application/json".to_string()));
        assert!(response.body.is_some());
    }

    #[test]
    fn test_plugin_action_continue() {
        let action = PluginAction::Continue;
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: PluginAction = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized, PluginAction::Continue));
    }

    #[test]
    fn test_plugin_action_intercept() {
        let response = PluginResponse::ok();
        let action = PluginAction::Intercept(response);
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: PluginAction = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized, PluginAction::Intercept(_)));
    }

    #[test]
    fn test_plugin_action_error() {
        let action = PluginAction::Error {
            message: "Something went wrong".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: PluginAction = serde_json::from_str(&json).unwrap();
        
        if let PluginAction::Error { message } = deserialized {
            assert_eq!(message, "Something went wrong");
        } else {
            panic!("Expected Error variant");
        }
    }

    #[test]
    fn test_base64_empty() {
        let original = "";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_unicode() {
        let original = "你好世界 🌍";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_decode_invalid() {
        let invalid = "!!!not-valid-base64!!!";
        let result = base64_decode(invalid);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_request_serialization() {
        let request = PluginRequest {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            query: [("key".to_string(), "value".to_string())].into_iter().collect(),
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
    fn test_plugin_response_serialization() {
        let response = PluginResponse {
            status: 200,
            headers: [("x-custom".to_string(), "value".to_string())].into_iter().collect(),
            body: Some("response body".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PluginResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.headers.len(), deserialized.headers.len());
    }

    #[test]
    fn test_plugin_metadata_default() {
        let metadata = PluginMetadata {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            homepage: None,
            license: "MIT".to_string(),
            min_server_version: "0.1.0".to_string(),
            priority: 100,
            capabilities: vec![Capability::ModifyRequest],
            permissions: vec![Permission::ReadEnv { allowed: vec!["PATH".to_string()] }],
        };

        assert_eq!(metadata.id, "test");
        assert!(metadata.homepage.is_none());
        assert_eq!(metadata.capabilities.len(), 1);
    }
}
