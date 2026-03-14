//! Response Modifier Plugin Example
//!
//! 响应修改插件，修改响应头和响应体
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.response-modifier"
//! path = "./plugins/response_modifier.wasm"
//! priority = 110
//!
//! [plugins.load.config]
//! # 添加响应头
//! [plugins.load.config.add_headers]
//! "X-Frame-Options" = "DENY"
//! "X-Content-Type-Options" = "nosniff"
//! "X-XSS-Protection" = "1; mode=block"
//! "Referrer-Policy" = "strict-origin-when-cross-origin"
//!
//! # 删除响应头
//! remove_headers = ["X-Powered-By", "Server", "X-AspNet-Version"]
//!
//! # 基于状态码的修改
//! [[plugins.load.config.status_rules]]
//! status = 500
//! add_headers = { "X-Error-Code" = "INTERNAL_ERROR" }
//! body_template = "Error: Internal Server Error"
//!
//! # 基于 Content-Type 的修改
//! [[plugins.load.config.content_type_rules]]
#'content_type = "application/json"
//! add_headers = { "X-API-Response" = "true" }
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;
use std::collections::HashMap;

/// 响应修改插件
pub struct ResponseModifierPlugin {
    metadata: PluginMetadata,
    add_headers: HashMap<String, String>,
    remove_headers: Vec<String>,
    set_headers: HashMap<String, String>,
    status_rules: Vec<StatusRule>,
    content_type_rules: Vec<ContentTypeRule>,
}

/// 状态码规则
#[derive(Debug, Clone)]
struct StatusRule {
    pub status: u16,
    pub add_headers: HashMap<String, String>,
    pub body_template: Option<String>,
}

/// Content-Type 规则
#[derive(Debug, Clone)]
struct ContentTypeRule {
    pub content_type: String,
    pub add_headers: HashMap<String, String>,
}

impl ResponseModifierPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.response-modifier".to_string(),
                name: "Response Modifier Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Modify response headers and body".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 110,
                capabilities: vec![Capability::ModifyResponse],
                permissions: vec![],
            },
            add_headers: HashMap::new(),
            remove_headers: vec![],
            set_headers: HashMap::new(),
            status_rules: vec![],
            content_type_rules: vec![],
        }
    }

    /// 查找状态码规则
    fn find_status_rule(&self,
        status: u16,
    ) -> Option<&StatusRule> {
        self.status_rules.iter().find(|r| r.status == status)
    }

    /// 查找 Content-Type 规则
    fn find_content_type_rule(&self,
        content_type: &str,
    ) -> Option<&ContentTypeRule> {
        self.content_type_rules.iter().find(|r| {
            content_type.starts_with(&r.content_type) ||
            content_type == r.content_type
        })
    }
}

impl Default for ResponseModifierPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ResponseModifierPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(headers) = config.get::<HashMap<String, String>>("add_headers") {
            self.add_headers = headers;
        }
        if let Some(headers) = config.get::<Vec<String>>("remove_headers") {
            self.remove_headers = headers;
        }
        if let Some(headers) = config.get::<HashMap<String, String>>("set_headers") {
            self.set_headers = headers;
        }

        // 加载状态码规则
        if let Some(rules) = config.get::<Vec<StatusRuleConfig>>("status_rules") {
            for r in rules {
                self.status_rules.push(StatusRule {
                    status: r.status,
                    add_headers: r.add_headers.unwrap_or_default(),
                    body_template: r.body_template,
                });
            }
        }

        // 加载 Content-Type 规则
        if let Some(rules) = config.get::<Vec<ContentTypeRuleConfig>>("content_type_rules") {
            for r in rules {
                self.content_type_rules.push(ContentTypeRule {
                    content_type: r.content_type,
                    add_headers: r.add_headers.unwrap_or_default(),
                });
            }
        }

        log_info!(
            "ResponseModifierPlugin loaded: {} add, {} remove, {} set, {} status rules, {} content-type rules",
            self.add_headers.len(),
            self.remove_headers.len(),
            self.set_headers.len(),
            self.status_rules.len(),
            self.content_type_rules.len()
        );
        Ok(())
    }

    fn on_response(
        &mut self,
        response: &mut PluginResponse,
    ) -> Result<PluginAction, PluginError> {
        let mut modified = false;

        // 1. 删除响应头
        for name in &self.remove_headers {
            if response.headers.remove(&name.to_lowercase()).is_some() {
                modified = true;
            }
        }

        // 2. 设置响应头
        for (name, value) in &self.set_headers {
            response.headers.insert(name.to_string(), value.clone());
            modified = true;
        }

        // 3. 添加响应头（不覆盖已存在的）
        for (name, value) in &self.add_headers {
            response
                .headers
                .entry(name.to_string())
                .or_insert_with(|| value.clone());
            modified = true;
        }

        // 4. 应用状态码规则
        if let Some(rule) = self.find_status_rule(response.status) {
            for (name, value) in &rule.add_headers {
                response.headers.insert(name.to_string(), value.clone());
            }
            if let Some(body) = &rule.body_template {
                response.body = Some(body.clone());
            }
            modified = true;
        }

        // 5. 应用 Content-Type 规则
        if let Some(content_type) = response.header("content-type") {
            if let Some(rule) = self.find_content_type_rule(content_type) {
                for (name, value) in &rule.add_headers {
                    response.headers.insert(name.to_string(), value.clone());
                }
                modified = true;
            }
        }

        if modified {
            Ok(PluginAction::ModifyResponse(response.clone()))
        } else {
            Ok(PluginAction::Continue)
        }
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("ResponseModifierPlugin unloaded");
        Ok(())
    }
}

/// 状态码规则配置
#[derive(Debug, Clone, serde::Deserialize)]
struct StatusRuleConfig {
    pub status: u16,
    pub add_headers: Option<HashMap<String, String>>,
    pub body_template: Option<String>,
}

/// Content-Type 规则配置
#[derive(Debug, Clone, serde::Deserialize)]
struct ContentTypeRuleConfig {
    pub content_type: String,
    pub add_headers: Option<HashMap<String, String>>,
}

export_plugin!(ResponseModifierPlugin);
