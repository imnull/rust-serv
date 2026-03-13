/// Example: Add Header Plugin
/// 
/// A simple plugin that adds custom headers to all responses

use rust_serv_plugin::*;
use std::collections::HashMap;

#[derive(Debug)]
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
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
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
        // 从配置读取 header 名称和值
        if let Some(name) = config.get::<String>("header_name") {
            self.header_name = name;
        }
        if let Some(value) = config.get::<String>("header_value") {
            self.header_value = value;
        }
        
        // 日志输出
        println!("Plugin loaded: {} v{}", self.metadata.name, self.metadata.version);
        println!("Header: {} = {}", self.header_name, self.header_value);
        
        Ok(())
    }
    
    fn on_response(&mut self, response: &mut PluginResponse) -> Result<PluginAction, PluginError> {
        // 为响应添加自定义 Header
        response.headers.insert(
            self.header_name.clone(),
            self.header_value.clone(),
        );
        
        // 继续执行下一个插件
        Ok(PluginAction::Continue)
    }
    
    fn on_unload(&mut self) -> Result<(), PluginError> {
        println!("Plugin unloaded: {}", self.metadata.name);
        Ok(())
    }
}

// 导出插件
rust_serv_plugin::export_plugin!(AddHeaderPlugin);
