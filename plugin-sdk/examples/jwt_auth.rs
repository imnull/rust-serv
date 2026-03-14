//! JWT Auth Plugin Example
//!
//! JWT 认证插件，验证请求中的 Bearer Token
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.jwt-auth"
//! path = "./plugins/jwt_auth.wasm"
//! priority = 300
//!
//! [plugins.load.config]
//! secret = "your-jwt-secret"
//! header_name = "Authorization"
//! prefix = "Bearer "
//! excluded_paths = ["/health", "/public"]
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::{log_info, log_error};

/// JWT 认证插件
pub struct JwtAuthPlugin {
    metadata: PluginMetadata,
    secret: String,
    header_name: String,
    prefix: String,
    excluded_paths: Vec<String>,
}

impl JwtAuthPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.jwt-auth".to_string(),
                name: "JWT Auth Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "JWT authentication for requests".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 300,
                capabilities: vec![Capability::InterceptRequest],
                permissions: vec![],
            },
            secret: "default-secret".to_string(),
            header_name: "Authorization".to_string(),
            prefix: "Bearer ".to_string(),
            excluded_paths: vec!["/health".to_string(), "/ready".to_string()],
        }
    }

    /// 检查路径是否在排除列表中
    fn is_excluded(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|p| path.starts_with(p))
    }

    /// 提取 Token
    fn extract_token(&self, auth_header: &str) -> Option<String> {
        if auth_header.starts_with(&self.prefix) {
            Some(auth_header[self.prefix.len()..].to_string())
        } else {
            None
        }
    }

    /// 验证 JWT（简化版）
    fn verify_jwt(&self, token: &str) -> Result<JwtPayload, String> {
        // 解析 JWT：header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }

        // Base64 解码 payload
        let payload_json = match base64_decode(parts[1]) {
            Ok(s) => s,
            Err(_) => return Err("Invalid base64 encoding".to_string()),
        };

        // 解析 JSON
        let payload: JwtPayload = match serde_json::from_str(&payload_json) {
            Ok(p) => p,
            Err(_) => return Err("Invalid JSON payload".to_string()),
        };

        // 检查过期时间
        let now = current_timestamp();
        if payload.exp < now {
            return Err("Token expired".to_string());
        }

        // 简化：不验证签名（实际应用需要 HMAC/RS256 验证）
        // 真实环境应该使用 crypto 库验证签名
        
        Ok(payload)
    }
}

impl Default for JwtAuthPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for JwtAuthPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(secret) = config.get::<String>("secret") {
            self.secret = secret;
        }
        if let Some(header) = config.get::<String>("header_name") {
            self.header_name = header;
        }
        if let Some(prefix) = config.get::<String>("prefix") {
            self.prefix = prefix;
        }
        if let Some(paths) = config.get::<Vec<String>>("excluded_paths") {
            self.excluded_paths = paths;
        }

        log_info!("JwtAuthPlugin loaded with {} excluded paths", self.excluded_paths.len());
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        // 检查是否在排除列表
        if self.is_excluded(&request.path) {
            return Ok(PluginAction::Continue);
        }

        // 获取 Authorization 头
        let auth_header = match request.header(&self.header_name) {
            Some(h) => h,
            None => {
                return Ok(PluginAction::Intercept(
                    PluginResponse::unauthorized()
                        .with_header("WWW-Authenticate", &format!("Bearer"))
                        .with_body("Missing authorization header")
                ));
            }
        };

        // 提取 Token
        let token = match self.extract_token(auth_header) {
            Some(t) => t,
            None => {
                return Ok(PluginAction::Intercept(
                    PluginResponse::unauthorized()
                        .with_body("Invalid authorization format")
                ));
            }
        };

        // 验证 JWT
        match self.verify_jwt(&token) {
            Ok(payload) => {
                // 可以在这里将用户信息添加到请求头
                log_info!("Authenticated user: {}", payload.sub);
                Ok(PluginAction::Continue)
            }
            Err(e) => {
                log_error!("JWT verification failed: {}", e);
                Ok(PluginAction::Intercept(
                    PluginResponse::unauthorized()
                        .with_body("Invalid or expired token")
                ))
            }
        }
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("JwtAuthPlugin unloaded");
        Ok(())
    }
}

/// JWT Payload 结构
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct JwtPayload {
    /// 主题（用户ID）
    pub sub: String,
    /// 签发时间
    pub iat: u64,
    /// 过期时间
    pub exp: u64,
    /// 发行者
    pub iss: Option<String>,
    /// 受众
    pub aud: Option<String>,
    /// 自定义声明
    #[serde(flatten)]
    pub claims: std::collections::HashMap<String, serde_json::Value>,
}

/// Base64 解码（URL-safe）
fn base64_decode(input: &str) -> Result<String, ()> {
    // 替换 URL-safe 字符
    let normalized = input.replace('-', "+").replace('_', "/");
    
    // 添加填充
    let padding = (4 - normalized.len() % 4) % 4;
    let padded = format!("{}{}", normalized, "=".repeat(padding));
    
    // 解码
    match rust_serv_plugin::base64_decode(&padded) {
        Ok(s) => Ok(s),
        Err(_) => Err(()),
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    // 简化实现，实际应该调用 host 函数
    1700000000
}

export_plugin!(JwtAuthPlugin);
