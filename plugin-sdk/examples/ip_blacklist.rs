//! IP Blacklist Plugin Example
//!
//! IP 黑名单插件，阻止特定 IP 访问
//!
//! ## 配置
//! ```toml
//! [[plugins.load]]
//! id = "com.example.ip-blacklist"
//! path = "./plugins/ip_blacklist.wasm"
//! priority = 310
//!
//! [plugins.load.config]
//! blacklist = ["192.168.1.100", "10.0.0.50", "172.16.0.0/24"]
#! deny_message = "Your IP has been blocked"
//! return_444 = false  # 如果为 true，直接断开连接而不返回响应
//! ```

use rust_serv_plugin::{export_plugin, Plugin, PluginConfig, PluginError};
use rust_serv_plugin::types::{PluginMetadata, PluginRequest, PluginResponse, PluginAction, Capability};
use rust_serv_plugin::{log_warn, log_info};

/// IP 黑名单插件
pub struct IpBlacklistPlugin {
    metadata: PluginMetadata,
    blacklist: Vec<String>,
    deny_message: String,
    return_444: bool,  // 直接断开连接
}

impl IpBlacklistPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "com.example.ip-blacklist".to_string(),
                name: "IP Blacklist Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Block requests from blacklisted IPs".to_string(),
                author: "rust-serv".to_string(),
                homepage: Some("https://github.com/imnull/rust-serv".to_string()),
                license: "MIT".to_string(),
                min_server_version: "0.3.0".to_string(),
                priority: 310,
                capabilities: vec![Capability::InterceptRequest],
                permissions: vec![],
            },
            blacklist: vec![],
            deny_message: "Access denied".to_string(),
            return_444: false,
        }
    }

    /// 检查 IP 是否在黑名单中
    fn is_blacklisted(&self, ip: &str) -> bool {
        for blocked in &self.blacklist {
            // 精确匹配
            if blocked == ip {
                return true;
            }
            
            // CIDR 匹配（简化版）
            if blocked.contains('/') {
                if self.matches_cidr(ip, blocked) {
                    return true;
                }
            }
            
            // 前缀匹配（如 192.168.1. 匹配 192.168.1.xxx）
            if blocked.ends_with('.') && ip.starts_with(blocked) {
                return true;
            }
        }
        false
    }

    /// CIDR 匹配（简化版，仅支持 /8, /16, /24）
    fn matches_cidr(&self, ip: &str, cidr: &str) -> bool {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let prefix = parts[0];
        let mask: u8 = match parts[1].parse() {
            Ok(m) => m,
            Err(_) => return false,
        };
        
        // 简化的 CIDR 匹配，仅处理常见的 /24, /16, /8
        match mask {
            32 => ip == prefix,
            24 => {
                let ip_prefix: Vec<&str> = ip.split('.').collect();
                let cidr_prefix: Vec<&str> = prefix.split('.').collect();
                ip_prefix.len() >= 3 && cidr_prefix.len() >= 3
                    && ip_prefix[0] == cidr_prefix[0]
                    && ip_prefix[1] == cidr_prefix[1]
                    && ip_prefix[2] == cidr_prefix[2]
            }
            16 => {
                let ip_prefix: Vec<&str> = ip.split('.').collect();
                let cidr_prefix: Vec<&str> = prefix.split('.').collect();
                ip_prefix.len() >= 2 && cidr_prefix.len() >= 2
                    && ip_prefix[0] == cidr_prefix[0]
                    && ip_prefix[1] == cidr_prefix[1]
            }
            8 => {
                let ip_prefix: Vec<&str> = ip.split('.').collect();
                let cidr_prefix: Vec<&str> = prefix.split('.').collect();
                ip_prefix.len() >= 1 && cidr_prefix.len() >= 1
                    && ip_prefix[0] == cidr_prefix[0]
            }
            _ => {
                // 对于其他掩码，使用前缀匹配作为近似
                let octets_to_match = (mask as usize + 7) / 8;
                let ip_octets: Vec<&str> = ip.split('.').collect();
                let prefix_octets: Vec<&str> = prefix.split('.').collect();
                
                if ip_octets.len() < octets_to_match || prefix_octets.len() < octets_to_match {
                    return false;
                }
                
                for i in 0..octets_to_match {
                    if ip_octets[i] != prefix_octets[i] {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl Default for IpBlacklistPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for IpBlacklistPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(blacklist) = config.get::<Vec<String>>("blacklist") {
            self.blacklist = blacklist;
        }
        if let Some(message) = config.get::<String>("deny_message") {
            self.deny_message = message;
        }
        if let Some(return_444) = config.get::<bool>("return_444") {
            self.return_444 = return_444;
        }

        log_warn!(
            "IpBlacklistPlugin loaded with {} blacklisted IPs",
            self.blacklist.len()
        );
        Ok(())
    }

    fn on_request(&mut self, request: &mut PluginRequest) -> Result<PluginAction, PluginError> {
        let client_ip = &request.client_ip;

        if self.is_blacklisted(client_ip) {
            log_warn!("Blocked request from blacklisted IP: {}", client_ip);

            if self.return_444 {
                // 444 是 Nginx 特殊状态码，表示直接断开连接
                // 这里我们用 444 或 400 表示
                return Ok(PluginAction::Intercept(
                    PluginResponse::new(444)
                ));
            }

            return Ok(PluginAction::Intercept(
                PluginResponse::forbidden()
                    .with_header("X-Blocked-By", "ip-blacklist")
                    .with_body(&self.deny_message)
            ));
        }

        Ok(PluginAction::Continue)
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info!("IpBlacklistPlugin unloaded");
        Ok(())
    }
}

export_plugin!(IpBlacklistPlugin);
