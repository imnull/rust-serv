//! Plugin Management API
//!
//! 提供插件管理相关的 REST API 端点

use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::plugin::{
    manager::PluginManager,
    traits::{PluginConfig, PluginMetadata},
};

use super::json_response;

/// 插件列表响应
#[derive(Debug, Serialize)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginInfo>,
    pub total: usize,
    pub enabled: usize,
}

/// 插件信息
#[derive(Debug, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub priority: i32,
    pub stats: PluginStatsInfo,
}

/// 插件统计信息
#[derive(Debug, Serialize)]
pub struct PluginStatsInfo {
    pub request_count: u64,
    pub response_count: u64,
    pub error_count: u64,
    pub avg_latency_us: f64,
}

/// 加载插件请求
#[derive(Debug, Deserialize)]
pub struct LoadPluginRequest {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

/// 更新插件配置请求
#[derive(Debug, Deserialize)]
pub struct UpdatePluginRequest {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub config: Option<std::collections::HashMap<String, serde_json::Value>>,
}

fn default_true() -> bool {
    true
}

/// 插件管理 API 处理器
pub struct PluginManagementHandler {
    manager: Arc<RwLock<PluginManager>>,
}

impl PluginManagementHandler {
    /// 创建新的插件管理处理器
    pub fn new(manager: Arc<RwLock<PluginManager>>) -> Self {
        Self { manager }
    }

    /// 处理插件列表请求
    pub async fn list_plugins(&self,
    ) -> Response<Full<Bytes>> {
        let manager = self.manager.read().await;
        let plugins = manager.list();

        let plugin_infos: Vec<PluginInfo> = plugins
            .iter()
            .map(|p| PluginInfo {
                id: p.id.clone(),
                name: p.metadata.name.clone(),
                version: p.metadata.version.clone(),
                description: p.metadata.description.clone(),
                author: p.metadata.author.clone(),
                enabled: p.config.enabled,
                priority: p.config.priority.unwrap_or(p.metadata.priority),
                stats: PluginStatsInfo {
                    request_count: p.stats.request_count,
                    response_count: p.stats.response_count,
                    error_count: p.stats.error_count,
                    avg_latency_us: p.stats.avg_latency_us(),
                },
            })
            .collect();

        let enabled_count = plugin_infos.iter().filter(|p| p.enabled).count();

        let response = PluginListResponse {
            total: plugin_infos.len(),
            enabled: enabled_count,
            plugins: plugin_infos,
        };

        match serde_json::to_string(&response) {
            Ok(body) => json_response(200, &body),
            Err(e) => {
                error!("Failed to serialize plugin list: {}", e);
                json_response(500, r#"{"error":"Internal server error"}"#)
            }
        }
    }

    /// 获取单个插件详情
    pub async fn get_plugin(
        &self,
        plugin_id: &str,
    ) -> Response<Full<Bytes>> {
        let manager = self.manager.read().await;

        match manager.get(plugin_id) {
            Some(p) => {
                let info = PluginInfo {
                    id: p.id.clone(),
                    name: p.metadata.name.clone(),
                    version: p.metadata.version.clone(),
                    description: p.metadata.description.clone(),
                    author: p.metadata.author.clone(),
                    enabled: p.config.enabled,
                    priority: p.config.priority.unwrap_or(p.metadata.priority),
                    stats: PluginStatsInfo {
                        request_count: p.stats.request_count,
                        response_count: p.stats.response_count,
                        error_count: p.stats.error_count,
                        avg_latency_us: p.stats.avg_latency_us(),
                    },
                };

                match serde_json::to_string(&info) {
                    Ok(body) => json_response(200, &body),
                    Err(e) => {
                        error!("Failed to serialize plugin info: {}", e);
                        json_response(500, r#"{"error":"Internal server error"}"#)
                    }
                }
            }
            None => json_response(
                404,
                &format!(r#"{{"error":"Plugin '{}' not found"}}"#, plugin_id),
            ),
        }
    }

    /// 加载插件
    pub async fn load_plugin(
        &self,
        body: &str,
    ) -> Response<Full<Bytes>> {
        let request: LoadPluginRequest = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => {
                return json_response(
                    400,
                    &format!(r#"{{"error":"Invalid request: {}"}}"#, e),
                );
            }
        };

        let path = std::path::Path::new(&request.path);

        let plugin_config = PluginConfig {
            enabled: request.enabled,
            priority: request.priority,
            timeout_ms: None,
            custom: request.config,
        };

        let mut manager = self.manager.write().await;

        match manager.load(path, plugin_config) {
            Ok(id) => {
                info!("Plugin loaded: {}", id);
                json_response(
                    201,
                    &format!(r#"{{"id":"{}","status":"loaded"}}"#, id),
                )
            }
            Err(e) => {
                error!("Failed to load plugin: {}", e);
                json_response(
                    500,
                    &format!(r#"{{"error":"Failed to load plugin: {}"}}"#, e),
                )
            }
        }
    }

    /// 卸载插件
    pub async fn unload_plugin(
        &self,
        plugin_id: &str,
    ) -> Response<Full<Bytes>> {
        let mut manager = self.manager.write().await;

        match manager.unload(plugin_id) {
            Ok(_) => {
                info!("Plugin unloaded: {}", plugin_id);
                json_response(200, r#"{"status":"unloaded"}"#)
            }
            Err(e) => {
                error!("Failed to unload plugin {}: {}", plugin_id, e);
                json_response(
                    500,
                    &format!(r#"{{"error":"Failed to unload plugin: {}"}}"#, e),
                )
            }
        }
    }

    /// 重载插件
    pub async fn reload_plugin(
        &self,
        plugin_id: &str,
    ) -> Response<Full<Bytes>> {
        let mut manager = self.manager.write().await;

        match manager.reload(plugin_id) {
            Ok(_) => {
                info!("Plugin reloaded: {}", plugin_id);
                json_response(
                    200,
                    &format!(r#"{{"id":"{}","status":"reloaded"}}"#, plugin_id),
                )
            }
            Err(e) => {
                error!("Failed to reload plugin {}: {}", plugin_id, e);
                json_response(
                    500,
                    &format!(r#"{{"error":"Failed to reload plugin: {}"}}"#, e),
                )
            }
        }
    }

    /// 更新插件配置
    pub async fn update_plugin(
        &self,
        plugin_id: &str,
        body: &str,
    ) -> Response<Full<Bytes>> {
        let request: UpdatePluginRequest = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => {
                return json_response(
                    400,
                    &format!(r#"{{"error":"Invalid request: {}"}}"#, e),
                );
            }
        };

        let manager = self.manager.read().await;

        // 获取当前配置
        let current_config = match manager.get(plugin_id) {
            Some(p) => p.config.clone(),
            None => {
                return json_response(
                    404,
                    &format!(r#"{{"error":"Plugin '{}' not found"}}"#, plugin_id),
                );
            }
        };

        drop(manager);

        // 构建新配置
        let new_config = PluginConfig {
            enabled: request.enabled.unwrap_or(current_config.enabled),
            priority: request.priority.or(current_config.priority),
            timeout_ms: current_config.timeout_ms,
            custom: request.config.unwrap_or(current_config.custom),
        };

        let mut manager = self.manager.write().await;

        match manager.update_config(plugin_id, new_config) {
            Ok(_) => {
                info!("Plugin config updated: {}", plugin_id);
                json_response(200, r#"{"status":"updated"}"#)
            }
            Err(e) => {
                error!("Failed to update plugin {}: {}", plugin_id, e);
                json_response(
                    500,
                    &format!(r#"{{"error":"Failed to update plugin: {}"}}"#, e),
                )
            }
        }
    }

    /// 启用插件
    pub async fn enable_plugin(
        &self,
        plugin_id: &str,
    ) -> Response<Full<Bytes>> {
        self.update_plugin_status(plugin_id, true).await
    }

    /// 禁用插件
    pub async fn disable_plugin(
        &self,
        plugin_id: &str,
    ) -> Response<Full<Bytes>> {
        self.update_plugin_status(plugin_id, false).await
    }

    /// 更新插件启用状态
    async fn update_plugin_status(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Response<Full<Bytes>> {
        let manager = self.manager.read().await;

        let current_config = match manager.get(plugin_id) {
            Some(p) => p.config.clone(),
            None => {
                return json_response(
                    404,
                    &format!(r#"{{"error":"Plugin '{}' not found"}}"#, plugin_id),
                );
            }
        };

        drop(manager);

        let new_config = PluginConfig {
            enabled,
            priority: current_config.priority,
            timeout_ms: current_config.timeout_ms,
            custom: current_config.custom,
        };

        let mut manager = self.manager.write().await;

        match manager.update_config(plugin_id, new_config) {
            Ok(_) => {
                let status = if enabled { "enabled" } else { "disabled" };
                info!("Plugin {}: {}", status, plugin_id);
                json_response(
                    200,
                    &format!(r#"{{"id":"{}","status":"{}"}}"#, plugin_id, status),
                )
            }
            Err(e) => {
                error!("Failed to update plugin {}: {}", plugin_id, e);
                json_response(
                    500,
                    &format!(r#"{{"error":"Failed to update plugin: {}"}}"#, e),
                )
            }
        }
    }

    /// 获取插件系统状态
    pub async fn get_system_status(
        &self,
    ) -> Response<Full<Bytes>> {
        let manager = self.manager.read().await;

        #[derive(Serialize)]
        struct SystemStatus {
            enabled: bool,
            plugin_count: usize,
            loaded_plugins: Vec<String>,
        }

        let status = SystemStatus {
            enabled: manager.is_enabled(),
            plugin_count: manager.count(),
            loaded_plugins: manager.list().iter().map(|p| p.id.clone()).collect(),
        };

        match serde_json::to_string(&status) {
            Ok(body) => json_response(200, &body),
            Err(e) => {
                error!("Failed to serialize system status: {}", e);
                json_response(500, r#"{"error":"Internal server error"}"#)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_list_response_serialization() {
        let response = PluginListResponse {
            plugins: vec![],
            total: 0,
            enabled: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("plugins"));
        assert!(json.contains("total"));
    }

    #[test]
    fn test_load_plugin_request_deserialization() {
        let json = r#"{
            "id": "test-plugin",
            "path": "/plugins/test.wasm",
            "enabled": true
        }"#;

        let request: LoadPluginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.id, "test-plugin");
        assert_eq!(request.path, "/plugins/test.wasm");
        assert!(request.enabled);
    }

    #[test]
    fn test_update_plugin_request_deserialization() {
        let json = r#"{
            "enabled": false,
            "priority": 100
        }"#;

        let request: UpdatePluginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.enabled, Some(false));
        assert_eq!(request.priority, Some(100));
    }
}