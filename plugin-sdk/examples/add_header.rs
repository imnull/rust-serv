//! Add Header Plugin Example
//!
//! 为所有响应添加自定义 Header
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.add-header"
//! path = "./plugins/add_header.wasm"
//! priority = 50
//!
//! [plugins.load.config]
//! header_name = "X-Powered-By"
//! header_value = "rust-serv"
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;

/// 添加 Header 插件
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
                description: "Adds custom headers to all responses".to_string(),
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
        
        log_info!("AddHeaderPlugin loaded with {}: {}", 
            self.header_name, self.header_value);
        
        Ok(())
    }

    fn on_response(&mut self, response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        response.headers.insert(
            self.header_name.clone(),
            self.header_value.clone(),
        );
        
        Ok(PluginAction::ModifyResponse(response.clone()))
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("AddHeaderPlugin unloaded");
        Ok(())
    }
}

export_plugin!(AddHeaderPlugin);
