//! CORS Plugin Example
//!
//! 跨域资源共享支持
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.cors"
//! path = "./plugins/cors.wasm"
//! priority = 150
//!
//! [plugins.load.config]
//! allowed_origins = ["https://example.com", "https://app.example.com"]
//! allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
//! allowed_headers = ["Content-Type", "Authorization", "X-Request-ID"]
//! max_age = 86400
//! allow_credentials = true
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;

/// CORS 插件
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
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
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

    /// 构建 CORS 响应头
    fn build_cors_headers(&self,
        origin: &str,
    ) -> Vec<(String, String)> {
        let mut headers = vec![
            ("Access-Control-Allow-Origin".to_string(), origin.to_string()),
            (
                "Access-Control-Allow-Methods".to_string(),
                self.allowed_methods.join(", "),
            ),
            (
                "Access-Control-Allow-Headers".to_string(),
                self.allowed_headers.join(", "),
            ),
            ("Access-Control-Max-Age".to_string(), self.max_age.to_string()),
        ];

        if self.allow_credentials {
            headers.push((
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            ));
        }

        headers
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

        log_info!(
            "CorsPlugin loaded: origins={:?}, methods={:?}",
            self.allowed_origins, self.allowed_methods
        );

        Ok(())
    }

    fn on_request(
        &mut self,
        request: &mut PluginRequest,
    ) -> Result<PluginAction, PluginError> {
        // 获取 Origin
        let origin = request
            .header("origin")
            .cloned()
            .unwrap_or_else(|| "*".to_string());

        // 处理预检请求 (OPTIONS)
        if request.method == "OPTIONS" {
            if !self.is_origin_allowed(&origin) {
                return Ok(PluginAction::Intercept(
                    PluginResponse::forbidden().with_body("CORS origin not allowed"),
                ));
            }

            let mut response = PluginResponse::no_content();
            for (name, value) in self.build_cors_headers(&origin) {
                response = response.with_header(&name, &value);
            }

            return Ok(PluginAction::Intercept(response));
        }

        Ok(PluginAction::Continue)
    }

    fn on_response(
        &mut self,
        response: &mut PluginResponse,
    ) -> Result<PluginAction, PluginError> {
        // 为所有响应添加 CORS 头
        let origin = if self.allowed_origins.len() == 1 {
            self.allowed_origins[0].clone()
        } else {
            "*".to_string()
        };

        response
            .headers
            .insert("Access-Control-Allow-Origin".to_string(), origin);

        if self.allow_credentials {
            response
                .headers
                .insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        }

        Ok(PluginAction::ModifyResponse(response.clone()))
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("CorsPlugin unloaded");
        Ok(())
    }
}

export_plugin!(CorsPlugin);
