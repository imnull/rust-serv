//! Cache Control Plugin Example
//!
//! 缓存控制插件，动态设置 Cache-Control 头
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.cache-control"
//! path = "./plugins/cache_control.wasm"
//! priority = 90
//!
//! [plugins.load.config]
//! # 默认缓存策略
//! default_max_age = 3600
//! default_public = true
//!
//! # 路径特定规则
//! [[plugins.load.config.rules]]
//! path_pattern = "/api/*"
//! max_age = 0
//! no_cache = true
//!
//! [[plugins.load.config.rules]]
//! path_pattern = "/static/*"
//! max_age = 86400
//! immutable = true
//!
//! [[plugins.load.config.rules]]
//! path_pattern = "/images/*"
//! max_age = 604800
//! public = true
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_info;

/// 缓存控制插件
pub struct CacheControlPlugin {
    metadata: PluginMetadata,
    default_max_age: u32,
    default_public: bool,
    rules: Vec<CacheRule>,
}

/// 缓存规则
#[derive(Debug, Clone)]
struct CacheRule {
    /// 路径模式
    pub path_pattern: String,
    /// 最大年龄（秒）
    pub max_age: Option<u32>,
    /// 是否公开
    pub public: Option<bool>,
    /// 是否禁用缓存
    pub no_cache: bool,
    /// 是否禁用存储
    pub no_store: bool,
    /// 是否必须重新验证
    pub must_revalidate: bool,
    /// 是否私有
    pub private: bool,
    /// 是否不可变
    pub immutable: bool,
}

impl CacheControlPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.cache-control".to_string(),
                name: "Cache Control Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Dynamic cache control headers".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 90,
                capabilities: vec![Capability::ModifyResponse],
                permissions: vec![],
            },
            default_max_age: 3600,
            default_public: true,
            rules: vec![],
        }
    }

    /// 匹配路径
    fn matches_pattern(&self, pattern: &str, path: &str) -> bool {
        // 通配符匹配
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            path.starts_with(prefix)
        } else if pattern == "*" {
            true
        } else {
            path == pattern
        }
    }

    /// 查找匹配的规则
    fn find_matching_rule(&self, path: &str) -> Option<&CacheRule> {
        // 返回第一个匹配的规则
        self.rules.iter().find(|r| self.matches_pattern(&r.path_pattern, path))
    }

    /// 构建 Cache-Control 头值
    fn build_cache_control(&self,
        rule: Option<&CacheRule>,
        response: &PluginResponse,
    ) -> String {
        // 如果响应已经有 Cache-Control，则不覆盖（除非规则明确指定）
        if response.header("cache-control").is_some() && rule.is_none() {
            return response.header("cache-control").unwrap().clone();
        }

        let mut parts: Vec<String> = vec![];

        if let Some(r) = rule {
            if r.no_store {
                return "no-store".to_string();
            }

            if r.no_cache {
                parts.push("no-cache".to_string());
            }

            if r.private {
                parts.push("private".to_string());
            } else if r.public == Some(true) || (r.public.is_none() && self.default_public) {
                parts.push("public".to_string());
            }

            if r.must_revalidate {
                parts.push("must-revalidate".to_string());
            }

            if r.immutable {
                parts.push("immutable".to_string());
            }

            let max_age = r.max_age.unwrap_or(self.default_max_age);
            if max_age > 0 && !r.no_cache {
                parts.push(format!("max-age={}", max_age));
            }
        } else {
            // 使用默认值
            if self.default_public {
                parts.push("public".to_string());
            }
            parts.push(format!("max-age={}", self.default_max_age));
        }

        if parts.is_empty() {
            "no-cache".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// 计算 Expires 头
    fn calculate_expires(&self,
        rule: Option<&CacheRule>,
    ) -> Option<String> {
        // 简化处理：只返回 max-age 对应的相对时间
        // 实际应该返回 HTTP 日期格式
        let max_age = rule
            .and_then(|r| r.max_age)
            .unwrap_or(self.default_max_age);

        if max_age == 0 {
            Some("0".to_string())
        } else {
            None
        }
    }
}

impl Default for CacheControlPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CacheControlPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(max_age) = config.get::<u32>("default_max_age") {
            self.default_max_age = max_age;
        }
        if let Some(public) = config.get::<bool>("default_public") {
            self.default_public = public;
        }

        // 加载规则
        if let Some(rules) = config.get::<Vec<CacheRuleConfig>>("rules") {
            for r in rules {
                self.rules.push(CacheRule {
                    path_pattern: r.path_pattern,
                    max_age: r.max_age,
                    public: r.public,
                    no_cache: r.no_cache.unwrap_or(false),
                    no_store: r.no_store.unwrap_or(false),
                    must_revalidate: r.must_revalidate.unwrap_or(false),
                    private: r.private.unwrap_or(false),
                    immutable: r.immutable.unwrap_or(false),
                });
            }
        }

        log_info!(
            "CacheControlPlugin loaded: default_max_age={}, {} rules",
            self.default_max_age,
            self.rules.len()
        );
        Ok(())
    }

    fn on_response(&mut self,
        response: &mut PluginResponse,
    ) -> Result<PluginAction, PluginError> {
        // 获取请求路径（简化处理，实际应该从请求中获取）
        // 这里我们假设可以从响应的某个头或上下文中获取路径
        // 为简化，使用通配符匹配所有路径
        let path = "/"; // 默认值

        let rule = self.find_matching_rule(path);
        let cache_control = self.build_cache_control(rule, response);

        response.headers.insert(
            "Cache-Control".to_string(),
            cache_control,
        );

        // 可选：添加 Expires 头
        if let Some(expires) = self.calculate_expires(rule) {
            response.headers.insert("Expires".to_string(), expires);
        }

        Ok(PluginAction::ModifyResponse(response.clone()))
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("CacheControlPlugin unloaded");
        Ok(())
    }
}

/// 缓存规则配置
#[derive(Debug, Clone, serde::Deserialize)]
struct CacheRuleConfig {
    pub path_pattern: String,
    pub max_age: Option<u32>,
    pub public: Option<bool>,
    pub no_cache: Option<bool>,
    pub no_store: Option<bool>,
    pub must_revalidate: Option<bool>,
    pub private: Option<bool>,
    pub immutable: Option<bool>,
}

export_plugin!(CacheControlPlugin);
