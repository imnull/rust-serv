//! Auto TLS (Let's Encrypt) module - Placeholder
//!
//! TODO: Complete implementation in v0.3.0

use serde::{Deserialize, Serialize};

/// Auto TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoTlsConfig {
    /// Enable auto TLS
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Domains for certificate
    #[serde(default)]
    pub domains: Vec<String>,

    /// Contact email
    #[serde(default)]
    pub email: String,

    /// Challenge type (http-01 or dns-01)
    #[serde(default = "default_challenge_type")]
    pub challenge_type: String,

    /// Certificate cache directory
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,

    /// Days before expiration to renew
    #[serde(default = "default_renew_days")]
    pub renew_before_days: u32,
}

fn default_enabled() -> bool { false }
fn default_challenge_type() -> String { "http-01".to_string() }
fn default_cache_dir() -> String { "./certs".to_string() }
fn default_renew_days() -> u32 { 30 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_tls_config_default() {
        let config = AutoTlsConfig {
            enabled: false,
            domains: vec![],
            email: "".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        assert!(!config.enabled);
        assert_eq!(config.challenge_type, "http-01");
    }
}
