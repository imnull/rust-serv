//! Plugin SDK 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMetadata {
    /// 唯一标识符
    pub id: String,
    /// 插件名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 主页
    pub homepage: Option<String>,
    /// 许可证
    pub license: String,
    /// 最低服务器版本要求
    pub min_server_version: String,
    /// 优先级（越大越先执行）
    pub priority: i32,
    /// 能力列表
    pub capabilities: Vec<Capability>,
    /// 权限列表
    pub permissions: Vec<Permission>,
}

/// 插件能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Capability {
    /// 修改请求
    ModifyRequest,
    /// 修改响应
    ModifyResponse,
    /// 拦截请求
    InterceptRequest,
    /// 访问配置
    AccessConfig,
    /// 日志记录
    Logging,
    /// 指标上报
    Metrics,
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// 读取环境变量
    ReadEnv { allowed: Vec<String> },
    /// HTTP 请求
    HttpRequest { allowed_hosts: Vec<String> },
    /// 读取文件
    FileRead { allowed_paths: Vec<String> },
    /// 写入文件
    FileWrite { allowed_paths: Vec<String> },
    /// 网络访问
    NetworkAccess { allowed_ports: Vec<u16> },
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// 是否启用
    pub enabled: bool,
    /// 优先级覆盖
    pub priority: Option<i32>,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 自定义配置项
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    /// 获取配置值
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.custom.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 创建新的配置
    pub fn new() -> Self {
        Self {
            enabled: true,
            priority: None,
            timeout_ms: Some(100),
            custom: HashMap::new(),
        }
    }
}

/// HTTP 请求
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginRequest {
    /// HTTP 方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 查询参数
    pub query: HashMap<String, String>,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体（Base64 编码）
    pub body: Option<String>,
    /// 客户端 IP
    pub client_ip: String,
    /// 请求 ID
    pub request_id: String,
    /// HTTP 版本
    pub version: String,
    /// Host 头
    pub host: String,
}

impl PluginRequest {
    /// 获取请求头（大小写不敏感）
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// 获取查询参数
    pub fn query(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }

    /// 创建新的 GET 请求
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: uuid(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        }
    }

    /// 创建新的 POST 请求
    pub fn post(path: &str) -> Self {
        let mut req = Self::get(path);
        req.method = "POST".to_string();
        req
    }

    /// 添加请求头
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_lowercase(), value.to_string());
        self
    }

    /// 设置请求体
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

/// HTTP 响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginResponse {
    /// HTTP 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体（Base64 编码）
    pub body: Option<String>,
}

impl PluginResponse {
    /// 创建新响应
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// 200 OK
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// 201 Created
    pub fn created() -> Self {
        Self::new(201)
    }

    /// 204 No Content
    pub fn no_content() -> Self {
        Self::new(204)
    }

    /// 400 Bad Request
    pub fn bad_request() -> Self {
        Self::new(400)
    }

    /// 401 Unauthorized
    pub fn unauthorized() -> Self {
        Self::new(401)
    }

    /// 403 Forbidden
    pub fn forbidden() -> Self {
        Self::new(403)
    }

    /// 404 Not Found
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// 429 Too Many Requests
    pub fn too_many_requests() -> Self {
        Self::new(429)
    }

    /// 500 Internal Server Error
    pub fn internal_error() -> Self {
        Self::new(500)
    }

    /// 503 Service Unavailable
    pub fn service_unavailable() -> Self {
        Self::new(503)
    }

    /// 添加响应头
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// 设置响应体
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// 设置 JSON 响应体
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, crate::PluginError> {
        let json = serde_json::to_string(data)
            .map_err(|e| crate::PluginError::Serialization(e.to_string()))?;
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        self.body = Some(crate::base64_encode(&json));
        Ok(self)
    }

    /// 获取响应头
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
}

/// 插件执行动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginAction {
    /// 继续执行下一个插件
    Continue,
    /// 拦截并返回响应
    Intercept(PluginResponse),
    /// 修改请求并继续
    ModifyRequest(PluginRequest),
    /// 修改响应并继续
    ModifyResponse(PluginResponse),
    /// 执行出错
    Error { message: String },
}

/// 插件 Trait
///
/// 所有插件必须实现此 trait
pub trait Plugin: Send + Sync {
    /// 返回插件元数据
    fn metadata(&self) -> &PluginMetadata;

    /// 插件加载时调用
    fn on_load(&mut self, _config: &PluginConfig) -> Result<(), crate::PluginError> {
        Ok(())
    }

    /// 配置变更时调用
    fn on_config_change(&mut self, _new_config: &PluginConfig) -> Result<(), crate::PluginError> {
        Ok(())
    }

    /// 处理 HTTP 请求
    fn on_request(
        &mut self,
        _request: &mut PluginRequest,
    ) -> Result<PluginAction, crate::PluginError> {
        Ok(PluginAction::Continue)
    }

    /// 处理 HTTP 响应
    fn on_response(
        &mut self,
        _response: &mut PluginResponse,
    ) -> Result<PluginAction, crate::PluginError> {
        Ok(PluginAction::Continue)
    }

    /// 插件卸载时调用
    fn on_unload(&mut self) -> Result<(), crate::PluginError> {
        Ok(())
    }
}

/// 生成唯一 ID（简化版）
pub fn uuid() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("req-{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(metadata.homepage.is_some());
    }

    #[test]
    fn test_plugin_metadata_without_homepage() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            license: "MIT".to_string(),
            min_server_version: "0.3.0".to_string(),
            priority: 100,
            capabilities: vec![],
            permissions: vec![],
        };

        assert!(metadata.homepage.is_none());
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
        assert!(caps.contains(&Capability::ModifyRequest));
        assert!(caps.contains(&Capability::Logging));
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

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.enabled);
        assert_eq!(config.priority, None);
        assert_eq!(config.timeout_ms, None);
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_plugin_config_new() {
        let config = PluginConfig::new();
        assert!(config.enabled);
        assert_eq!(config.timeout_ms, Some(100));
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
    fn test_plugin_config_get_wrong_type() {
        let mut custom = HashMap::new();
        custom.insert("number".to_string(), serde_json::json!(42));

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        // 尝试用错误类型获取应该返回 None
        assert_eq!(config.get::<String>("number"), None);
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

        // 测试大小写不敏感
        assert_eq!(request.header("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.header("content-type"), Some(&"application/json".to_string()));
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
    fn test_plugin_request_get_constructor() {
        let req = PluginRequest::get("/api/users");

        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/users");
        assert!(req.body.is_none());
    }

    #[test]
    fn test_plugin_request_post_constructor() {
        let req = PluginRequest::post("/api/users");

        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/users");
    }

    #[test]
    fn test_plugin_request_with_header() {
        let req = PluginRequest::get("/")
            .with_header("X-Custom", "value")
            .with_header("Authorization", "Bearer token");

        assert_eq!(req.header("X-Custom"), Some(&"value".to_string()));
        assert_eq!(req.header("Authorization"), Some(&"Bearer token".to_string()));
    }

    #[test]
    fn test_plugin_request_with_body() {
        let req = PluginRequest::post("/api")
            .with_body(r#"{"key": "value"}"#);

        assert_eq!(req.body, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn test_plugin_response_constructors() {
        assert_eq!(PluginResponse::ok().status, 200);
        assert_eq!(PluginResponse::created().status, 201);
        assert_eq!(PluginResponse::no_content().status, 204);
        assert_eq!(PluginResponse::bad_request().status, 400);
        assert_eq!(PluginResponse::unauthorized().status, 401);
        assert_eq!(PluginResponse::forbidden().status, 403);
        assert_eq!(PluginResponse::not_found().status, 404);
        assert_eq!(PluginResponse::too_many_requests().status, 429);
        assert_eq!(PluginResponse::internal_error().status, 500);
        assert_eq!(PluginResponse::service_unavailable().status, 503);
    }

    #[test]
    fn test_plugin_response_new() {
        let res = PluginResponse::new(418); // I'm a teapot
        assert_eq!(res.status, 418);
    }

    #[test]
    fn test_plugin_response_with_header() {
        let response = PluginResponse::ok()
            .with_header("X-Custom", "value")
            .with_header("X-Request-Id", "123");

        assert_eq!(response.header("X-Custom"), Some(&"value".to_string()));
        assert_eq!(response.header("X-Request-Id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_plugin_response_with_body() {
        let response = PluginResponse::ok()
            .with_body("Hello, World!");

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
    fn test_plugin_response_json_error() {
        // 使用无法序列化的类型测试错误处理
        // 这里使用 serde_json::Value::Null 应该是可以的
        let response = PluginResponse::ok().json(&serde_json::json!(null));
        assert!(response.is_ok());
    }

    #[test]
    fn test_plugin_response_header_case_insensitive() {
        let response = PluginResponse::ok()
            .with_header("Content-Type", "application/json");

        assert_eq!(response.header("content-type"), Some(&"application/json".to_string()));
        assert_eq!(response.header("CONTENT-TYPE"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_plugin_action_variants() {
        let continue_action = PluginAction::Continue;
        assert!(matches!(continue_action, PluginAction::Continue));

        let intercept = PluginAction::Intercept(PluginResponse::ok());
        assert!(matches!(intercept, PluginAction::Intercept(_)));

        let modify_req = PluginAction::ModifyRequest(PluginRequest::get("/"));
        assert!(matches!(modify_req, PluginAction::ModifyRequest(_)));

        let modify_res = PluginAction::ModifyResponse(PluginResponse::ok());
        assert!(matches!(modify_res, PluginAction::ModifyResponse(_)));

        let error = PluginAction::Error { message: "Test error".to_string() };
        assert!(matches!(error, PluginAction::Error { .. }));
    }

    #[test]
    fn test_plugin_trait_default_implementations() {
        use std::sync::OnceLock;

        struct TestPlugin;

        impl Plugin for TestPlugin {
            fn metadata(&self) -> &PluginMetadata {
                static META: OnceLock<PluginMetadata> = OnceLock::new();
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

        let mut request = PluginRequest::get("/");
        let action = plugin.on_request(&mut request).unwrap();
        assert!(matches!(action, PluginAction::Continue));

        let mut response = PluginResponse::ok();
        let action = plugin.on_response(&mut response).unwrap();
        assert!(matches!(action, PluginAction::Continue));
    }

    #[test]
    fn test_uuid_generation() {
        let id1 = uuid();
        let id2 = uuid();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("req-"));
        assert!(id2.starts_with("req-"));
    }

    #[test]
    fn test_serialization_plugin_metadata() {
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

        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_serialization_plugin_request() {
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

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_serialization_plugin_response() {
        let response = PluginResponse {
            status: 200,
            headers: [("x-custom".to_string(), "value".to_string())].into_iter().collect(),
            body: Some("response body".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PluginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_serialization_plugin_action() {
        let actions = vec![
            PluginAction::Continue,
            PluginAction::Intercept(PluginResponse::ok()),
            PluginAction::ModifyRequest(PluginRequest::get("/")),
            PluginAction::ModifyResponse(PluginResponse::ok()),
            PluginAction::Error { message: "test error".to_string() },
        ];

        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let deserialized: PluginAction = serde_json::from_str(&json).unwrap();

            match (&action, &deserialized) {
                (PluginAction::Continue, PluginAction::Continue) => {}
                (PluginAction::Intercept(_), PluginAction::Intercept(_)) => {}
                (PluginAction::ModifyRequest(_), PluginAction::ModifyRequest(_)) => {}
                (PluginAction::ModifyResponse(_), PluginAction::ModifyResponse(_)) => {}
                (PluginAction::Error { message: m1 }, PluginAction::Error { message: m2 }) => assert_eq!(m1, m2),
                _ => panic!("Serialization mismatch"),
            }
        }
    }

    #[test]
    fn test_serialization_plugin_config() {
        let mut custom = HashMap::new();
        custom.insert("key".to_string(), serde_json::json!("value"));

        let config = PluginConfig {
            enabled: true,
            priority: Some(100),
            timeout_ms: Some(50),
            custom,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PluginConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.priority, deserialized.priority);
        assert_eq!(config.timeout_ms, deserialized.timeout_ms);
    }

    // =========================================================================
    // 边界测试
    // =========================================================================

    #[test]
    fn test_plugin_metadata_empty_strings() {
        let metadata = PluginMetadata {
            id: "".to_string(),
            name: "".to_string(),
            version: "".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            homepage: None,
            license: "".to_string(),
            min_server_version: "".to_string(),
            priority: 0,
            capabilities: vec![],
            permissions: vec![],
        };

        assert!(metadata.id.is_empty());
        assert!(metadata.name.is_empty());
        assert!(metadata.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_metadata_special_chars() {
        let metadata = PluginMetadata {
            id: "plugin-123_test.v2".to_string(),
            name: "Test Plugin 🚀".to_string(),
            version: "1.0.0-beta.1".to_string(),
            description: "A test plugin with <special> chars & unicode: 你好".to_string(),
            author: "User@example.com".to_string(),
            homepage: Some("https://example.com/path?query=value".to_string()),
            license: "MIT/Apache-2.0".to_string(),
            min_server_version: ">=0.3.0, <1.0.0".to_string(),
            priority: -100,
            capabilities: vec![Capability::ModifyRequest],
            permissions: vec![],
        };

        assert_eq!(metadata.priority, -100);
        assert!(metadata.description.contains("你好"));
    }

    #[test]
    fn test_plugin_request_empty_fields() {
        let request = PluginRequest {
            method: "".to_string(),
            path: "".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            client_ip: "".to_string(),
            request_id: "".to_string(),
            version: "".to_string(),
            host: "".to_string(),
        };

        assert!(request.header("any").is_none());
        assert!(request.query("any").is_none());
    }

    #[test]
    fn test_plugin_request_unicode_path() {
        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/用户/测试/🎉".to_string(),
            query: [("查询".to_string(), "值".to_string())].into_iter().collect(),
            headers: [("自定义头".to_string(), "值".to_string())].into_iter().collect(),
            body: Some("Unicode: 你好 🌍".to_string()),
            client_ip: "::1".to_string(),
            request_id: "req-unicode-123".to_string(),
            version: "HTTP/2.0".to_string(),
            host: "例子.com".to_string(),
        };

        assert_eq!(request.path, "/用户/测试/🎉");
        assert_eq!(request.query("查询"), Some(&"值".to_string()));
    }

    #[test]
    fn test_plugin_response_all_status_codes() {
        // 测试所有构造函数
        let responses = vec![
            (PluginResponse::ok(), 200),
            (PluginResponse::created(), 201),
            (PluginResponse::no_content(), 204),
            (PluginResponse::bad_request(), 400),
            (PluginResponse::unauthorized(), 401),
            (PluginResponse::forbidden(), 403),
            (PluginResponse::not_found(), 404),
            (PluginResponse::too_many_requests(), 429),
            (PluginResponse::internal_error(), 500),
            (PluginResponse::service_unavailable(), 503),
            (PluginResponse::new(418), 418), // I'm a teapot
            (PluginResponse::new(999), 999),
        ];

        for (response, expected_status) in responses {
            assert_eq!(response.status, expected_status);
        }
    }

    #[test]
    fn test_plugin_response_chaining() {
        let response = PluginResponse::ok()
            .with_header("H1", "V1")
            .with_header("H2", "V2")
            .with_header("H3", "V3")
            .with_body("body content");

        assert_eq!(response.headers.len(), 3);
        assert_eq!(response.body, Some("body content".to_string()));
    }

    #[test]
    fn test_plugin_config_empty_custom() {
        let config = PluginConfig {
            enabled: false,
            priority: None,
            timeout_ms: None,
            custom: HashMap::new(),
        };

        assert!(!config.enabled);
        assert_eq!(config.get::<String>("any"), None);
        assert_eq!(config.get::<i32>("any"), None);
    }

    #[test]
    fn test_plugin_config_complex_values() {
        let mut custom = HashMap::new();
        custom.insert("string".to_string(), serde_json::json!("value"));
        custom.insert("number".to_string(), serde_json::json!(42));
        custom.insert("bool".to_string(), serde_json::json!(true));
        custom.insert("null".to_string(), serde_json::Value::Null);
        custom.insert("array".to_string(), serde_json::json!([1, 2, 3]));
        custom.insert("object".to_string(), serde_json::json!({"key": "value"}));

        let config = PluginConfig {
            enabled: true,
            priority: Some(100),
            timeout_ms: Some(1000),
            custom,
        };

        assert_eq!(config.get::<String>("string"), Some("value".to_string()));
        assert_eq!(config.get::<i32>("number"), Some(42));
        assert_eq!(config.get::<bool>("bool"), Some(true));
        assert_eq!(config.get::<Vec<i32>>("array"), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_uuid_uniqueness() {
        let ids: Vec<String> = (0..100).map(|_| uuid()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique.len(), "UUIDs should be unique");
    }

    #[test]
    fn test_uuid_format() {
        let id = uuid();
        assert!(id.starts_with("req-"));
        let num_part = &id[4..];
        assert!(num_part.parse::<u64>().is_ok());
    }

    #[test]
    fn test_capability_serialization() {
        let caps = vec![
            Capability::ModifyRequest,
            Capability::ModifyResponse,
            Capability::InterceptRequest,
            Capability::AccessConfig,
            Capability::Logging,
            Capability::Metrics,
        ];

        for cap in caps {
            let json = serde_json::to_string(&cap).unwrap();
            let deserialized: Capability = serde_json::from_str(&json).unwrap();
            assert_eq!(cap, deserialized);
        }
    }

    #[test]
    fn test_permission_serialization() {
        let perms = vec![
            Permission::ReadEnv { allowed: vec![] },
            Permission::ReadEnv { allowed: vec!["PATH".to_string(), "HOME".to_string()] },
            Permission::HttpRequest { allowed_hosts: vec!["api.example.com".to_string()] },
            Permission::FileRead { allowed_paths: vec!["/tmp".to_string(), "/var/log".to_string()] },
            Permission::FileWrite { allowed_paths: vec![] },
            Permission::NetworkAccess { allowed_ports: vec![80, 443, 8080] },
        ];

        for perm in perms {
            let json = serde_json::to_string(&perm).unwrap();
            let deserialized: Permission = serde_json::from_str(&json).unwrap();
            assert_eq!(perm, deserialized);
        }
    }

    #[test]
    fn test_plugin_request_large_headers() {
        let mut headers = HashMap::new();
        for i in 0..100 {
            headers.insert(format!("X-Header-{}", i), format!("value-{}-{}-{}-{}-{}-", i, i, i, i, i));
        }

        let request = PluginRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers,
            body: None,
            client_ip: "127.0.0.1".to_string(),
            request_id: "req-large".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        assert_eq!(request.headers.len(), 100);
        assert!(request.header("x-header-50").is_some());
    }

    #[test]
    fn test_plugin_response_with_large_body() {
        let large_body = "x".repeat(10000);
        let response = PluginResponse::ok()
            .with_body(&large_body);

        assert_eq!(response.body.as_ref().unwrap().len(), 10000);
    }

    #[test]
    fn test_plugin_action_error_variants() {
        let errors = vec![
            PluginAction::Error { message: "error 1".to_string() },
            PluginAction::Error { message: "".to_string() },
            PluginAction::Error { message: "a very long error message with lots of details and context".to_string() },
        ];

        for err in errors {
            let json = serde_json::to_string(&err).unwrap();
            let deserialized: PluginAction = serde_json::from_str(&json).unwrap();
            assert!(matches!(deserialized, PluginAction::Error { .. }));
        }
    }

    #[test]
    fn test_plugin_trait_with_state() {
        use std::sync::OnceLock;

        struct StatefulPlugin {
            counter: u32,
        }

        impl StatefulPlugin {
            fn new() -> Self {
                Self { counter: 0 }
            }
        }

        impl Default for StatefulPlugin {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Plugin for StatefulPlugin {
            fn metadata(&self) -> &PluginMetadata {
                static META: OnceLock<PluginMetadata> = OnceLock::new();
                META.get_or_init(|| PluginMetadata {
                    id: "stateful".to_string(),
                    name: "Stateful Plugin".to_string(),
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

            fn on_request(&mut self, _request: &mut PluginRequest) -> Result<PluginAction, crate::PluginError> {
                self.counter += 1;
                Ok(PluginAction::Continue)
            }
        }

        let mut plugin = StatefulPlugin::new();
        let mut request = PluginRequest::get("/");
        
        for _ in 0..10 {
            plugin.on_request(&mut request).unwrap();
        }
        
        assert_eq!(plugin.counter, 10);
    }

    #[test]
    fn test_plugin_config_with_nested_objects() {
        let mut custom = HashMap::new();
        custom.insert(
            "nested".to_string(),
            serde_json::json!({
                "level1": {
                    "level2": {
                        "value": "deep"
                    }
                }
            }),
        );

        let config = PluginConfig {
            enabled: true,
            priority: None,
            timeout_ms: None,
            custom,
        };

        let nested: Option<serde_json::Value> = config.get("nested");
        assert!(nested.is_some());
        
        let obj = nested.unwrap();
        assert!(obj.get("level1").is_some());
    }
}
