//! URL Rewrite Plugin Example
//!
//! URL 重写/重定向插件
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.url-rewrite"
//! path = "./plugins/url_rewrite.wasm"
//! priority = 100
//!
//! [plugins.load.config]
//! # 重写规则
//! [[plugins.load.config.rules]]
//! from = "/old-api/*"
//! to = "/api/v1/*"
//! type = "rewrite"
//!
//! [[plugins.load.config.rules]]
//! from = "/docs"
//! to = "/documentation"
//! type = "redirect"
//! status = 301
//!
//! [[plugins.load.config.rules]]
//! from = "/api/*"
//! to = "/internal/api/*"
//! type = "rewrite"
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;

/// URL 重写插件
pub struct UrlRewritePlugin {
    metadata: PluginMetadata,
    rules: Vec<RewriteRule>,
}

/// 重写规则
#[derive(Debug, Clone)]
struct RewriteRule {
    /// 匹配模式
    pub from: String,
    /// 目标路径
    pub to: String,
    /// 规则类型
    pub rule_type: RuleType,
    /// 重定向状态码（仅用于 redirect 类型）
    pub status: u16,
}

/// 规则类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum RuleType {
    /// 内部重写
    Rewrite,
    /// 外部重定向
    Redirect,
}

impl UrlRewritePlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.url-rewrite".to_string(),
                name: "URL Rewrite Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "URL rewriting and redirection".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 100,
                capabilities: vec![Capability::ModifyRequest, Capability::InterceptRequest],
                permissions: vec![],
            },
            rules: vec![],
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, from: &str, to: &str, rule_type: RuleType, status: u16) {
        self.rules.push(RewriteRule {
            from: from.to_string(),
            to: to.to_string(),
            rule_type,
            status,
        });
    }

    /// 匹配路径
    fn match_path(&self, pattern: &str, path: &str) -> Option<Vec<String>> {
        // 支持通配符 * 匹配
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if path.starts_with(prefix) {
                let captured = path[prefix.len()..].trim_start_matches('/').to_string();
                return Some(vec![captured]);
            }
        }
        
        // 精确匹配
        if pattern == path {
            return Some(vec![]);
        }
        
        // 前缀匹配
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            if path.starts_with(prefix) {
                let captured = path[prefix.len()..].to_string();
                return Some(vec![captured]);
            }
        }
        
        None
    }

    /// 应用重写规则
    fn apply_rewrite(&self, pattern: &str, captures: &[String]) -> String {
        let mut result = pattern.to_string();
        
        // 替换 * 通配符
        if result.contains("/*") && !captures.is_empty() {
            result = result.replace("/*", &format!("/{}", captures[0]));
        } else if result.ends_with('*') && !captures.is_empty() {
            result = format!("{}{}", &result[..result.len()-1], captures[0]);
        }
        
        result
    }

    /// 查找匹配的规则
    fn find_matching_rule(&self, path: &str) -> Option<(&RewriteRule, Vec<String>)> {
        for rule in &self.rules {
            if let Some(captures) = self.match_path(&rule.from, path) {
                return Some((rule, captures));
            }
        }
        None
    }
}

impl Default for UrlRewritePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for UrlRewritePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        // 从配置加载规则
        if let Some(rules) = config.get::<Vec<RuleConfig>>("rules") {
            for rule in rules {
                let rule_type = match rule.rule_type.as_str() {
                    "rewrite" => RuleType::Rewrite,
                    "redirect" => RuleType::Redirect,
                    _ => RuleType::Rewrite,
                };
                
                let status = rule.status.unwrap_or(if rule_type == RuleType::Redirect { 302 } else { 200 });
                
                self.add_rule(&rule.from, &rule.to, rule_type, status);
            }
        }

        // 默认规则
        if self.rules.is_empty() {
            self.add_rule("/old/*", "/new/*", RuleType::Rewrite, 200);
        }

        log_info!("UrlRewritePlugin loaded with {} rules", self.rules.len());
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        if let Some((rule, captures)) = self.find_matching_rule(&request.path) {
            let new_path = self.apply_rewrite(&rule.to, &captures);
            
            match rule.rule_type {
                RuleType::Rewrite => {
                    log_info!("Rewriting {} to {}", request.path, new_path);
                    request.path = new_path;
                    Ok(PluginAction::ModifyRequest(request.clone()))
                }
                RuleType::Redirect => {
                    log_info!("Redirecting {} to {}", request.path, new_path);
                    let location = if new_path.starts_with("http") {
                        new_path
                    } else {
                        format!("{}{}", request.host, new_path)
                    };
                    
                    let response = PluginResponse::new(rule.status)
                        .with_header("Location", &location);
                    
                    Ok(PluginAction::Intercept(response))
                }
            }
        } else {
            Ok(PluginAction::Continue)
        }
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("UrlRewritePlugin unloaded");
        Ok(())
    }
}

/// 规则配置结构（用于反序列化）
#[derive(Debug, Clone, serde::Deserialize)]
struct RuleConfig {
    pub from: String,
    pub to: String,
    #[serde(rename = "type", default = "default_rule_type")]
    pub rule_type: String,
    #[serde(default)]
    pub status: Option<u16>,
}

fn default_rule_type() -> String {
    "rewrite".to_string()
}

export_plugin!(UrlRewritePlugin);
