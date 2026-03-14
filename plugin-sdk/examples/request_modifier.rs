//! Request Modifier Plugin Example
//!
//! 请求修改插件，添加/修改/删除请求头
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.request-modifier"
//! path = "./plugins/request_modifier.wasm"
//! priority = 80
//!
//! [plugins.load.config]
//! # 添加请求头
//! [plugins.load.config.add_headers]
//! "X-Request-ID" = "${REQUEST_ID}"
//! "X-Real-IP" = "${CLIENT_IP}"
//! "X-Forwarded-For" = "${CLIENT_IP}"
//!
//! # 删除请求头
//! remove_headers = ["X-Internal-Token", "X-Debug"]
//!
//! # 修改请求头（如果存在）
//! [plugins.load.config.set_headers]
//! "User-Agent" = "rust-serv/1.0"
//!
//! # 条件规则
//! [[plugins.load.config.conditions]]
//! path_prefix = "/api/"
//! add_headers = { "X-API-Version" = "v1" }
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;
use std::collections::HashMap;

/// 请求修改插件
pub struct RequestModifierPlugin {
    metadata: PluginMetadata,
    add_headers: HashMap<String, String>,
    remove_headers: Vec<String>,
    set_headers: HashMap<String, String>,
    conditions: Vec<Condition>,
}

/// 条件规则
#[derive(Debug, Clone)]
struct Condition {
    /// 路径前缀
    pub path_prefix: String,
    /// 要添加的头部
    pub add_headers: HashMap<String, String>,
}

impl RequestModifierPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.request-modifier".to_string(),
                name: "Request Modifier Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Modify request headers".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 80,
                capabilities: vec![Capability::ModifyRequest],
                permissions: vec![],
            },
            add_headers: HashMap::new(),
            remove_headers: vec![],
            set_headers: HashMap::new(),
            conditions: vec![],
        }
    }

    /// 替换变量
    fn replace_variables(&self, template: &str, request: &PluginRequest) -> String {
        template
            .replace("${REQUEST_ID}", &request.request_id)
            .replace("${CLIENT_IP}", &request.client_ip)
            .replace("${HOST}", &request.host)
            .replace("${PATH}", &request.path)
            .replace("${METHOD}", &request.method)
    }

    /// 应用条件规则
    fn apply_conditions(&self, request: &mut PluginRequest) {
        for condition in &self.conditions {
            if request.path.starts_with(&condition.path_prefix) {
                for (name, value) in &condition.add_headers {
                    let resolved_value = self.replace_variables(value, request);
                    request.headers.insert(name.to_lowercase(), resolved_value);
                }
            }
        }
    }
}

impl Default for RequestModifierPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RequestModifierPlugin {
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

        // 加载条件规则
        if let Some(conditions) = config.get::<Vec<ConditionConfig>>("conditions") {
            for c in conditions {
                self.conditions.push(Condition {
                    path_prefix: c.path_prefix,
                    add_headers: c.add_headers.unwrap_or_default(),
                });
            }
        }

        log_info!(
            "RequestModifierPlugin loaded: {} add, {} remove, {} set, {} conditions",
            self.add_headers.len(),
            self.remove_headers.len(),
            self.set_headers.len(),
            self.conditions.len()
        );
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let mut modified = false;

        // 1. 删除请求头
        for name in &self.remove_headers {
            if request.headers.remove(&name.to_lowercase()).is_some() {
                modified = true;
            }
        }

        // 2. 设置请求头（覆盖已存在的）
        for (name, value) in &self.set_headers {
            let resolved_value = self.replace_variables(value, request);
            request.headers.insert(name.to_lowercase(), resolved_value);
            modified = true;
        }

        // 3. 添加请求头（不覆盖已存在的）
        for (name, value) in &self.add_headers {
            if !request.headers.contains_key(&name.to_lowercase()) {
                let resolved_value = self.replace_variables(value, request);
                request.headers.insert(name.to_lowercase(), resolved_value);
                modified = true;
            }
        }

        // 4. 应用条件规则
        self.apply_conditions(request);
        modified = true;

        if modified {
            Ok(PluginAction::ModifyRequest(request.clone()))
        } else {
            Ok(PluginAction::Continue)
        }
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("RequestModifierPlugin unloaded");
        Ok(())
    }
}

/// 条件配置结构
#[derive(Debug, Clone, serde::Deserialize)]
struct ConditionConfig {
    pub path_prefix: String,
    pub add_headers: Option<HashMap<String, String>>,
}

export_plugin!(RequestModifierPlugin);
