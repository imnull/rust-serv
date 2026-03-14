//! Rate Limiter Plugin Example
//!
//! 基于 IP 的请求限流插件
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.rate-limiter"
//! path = "./plugins/rate_limiter.wasm"
//! priority = 200
//!
//! [plugins.load.config]
//! requests_per_minute = 100
//! burst_size = 20
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::log_warn;
use std::collections::HashMap;

/// 速率限制插件
pub struct RateLimiterPlugin {
    metadata: PluginMetadata,
    requests_per_minute: u32,
    burst_size: u32,
    // IP -> (请求计数, 时间窗口开始)
    storage: HashMap<String, (u32, u64)>,
}

impl RateLimiterPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.rate-limiter".to_string(),
                name: "Rate Limiter Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "IP-based rate limiting".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 200,
                capabilities: vec![Capability::InterceptRequest],
                permissions: vec![],
            },
            requests_per_minute: 100,
            burst_size: 20,
            storage: HashMap::new(),
        }
    }

    /// 获取当前时间戳（秒）
    fn now() -> u64 {
        // 在真实环境中，这里应该调用 host 函数获取系统时间
        // 简化处理：使用递增计数器模拟
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// 清理过期的记录
    fn cleanup(&mut self) {
        let now = Self::now();
        let window = 60; // 1 分钟窗口
        self.storage.retain(|_, (_, timestamp)| now - *timestamp < window);
    }

    /// 检查 IP 是否超过限流
    fn is_rate_limited(&mut self, ip: &str) -> bool {
        let now = Self::now();
        let window = 60; // 1 分钟窗口
        
        // 每 100 次请求清理一次过期记录
        if self.storage.len() > 100 {
            self.cleanup();
        }

        let entry = self.storage.entry(ip.to_string()).or_insert((0, now));
        
        if now - entry.1 >= window {
            // 新窗口，重置计数
            *entry = (0, now);
        }

        if entry.0 >= self.requests_per_minute + self.burst_size {
            return true;
        }

        entry.0 += 1;
        false
    }
}

impl Default for RateLimiterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RateLimiterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(rpm) = config.get::<u32>("requests_per_minute") {
            self.requests_per_minute = rpm;
        }
        if let Some(burst) = config.get::<u32>("burst_size") {
            self.burst_size = burst;
        }
        
        log_warn!("RateLimiterPlugin loaded: {} req/min, burst: {}", 
            self.requests_per_minute, self.burst_size);
        
        Ok(())
    }

    fn on_request(
        &mut self,
        request: &mut PluginRequest,
    ) -> Result<PluginAction, PluginError> {
        let ip = &request.client_ip;

        if self.is_rate_limited(ip) {
            let response = PluginResponse::too_many_requests()
                .with_header("X-RateLimit-Limit", &self.requests_per_minute.to_string())
                .with_header("X-RateLimit-Remaining", "0")
                .with_body("Rate limit exceeded. Please try again later.");

            return Ok(PluginAction::Intercept(response));
        }

        Ok(PluginAction::Continue)
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_warn!("RateLimiterPlugin unloaded");
        Ok(())
    }
}

export_plugin!(RateLimiterPlugin);
