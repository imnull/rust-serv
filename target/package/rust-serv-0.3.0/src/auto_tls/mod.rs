//! Auto TLS (Let's Encrypt) module
//!
//! This module provides automatic TLS certificate management using Let's Encrypt.

mod account;
mod challenge;
mod client;
mod store;

pub use account::AcmeAccount;
pub use challenge::ChallengeHandler;
pub use client::AcmeClient;
pub use store::{CertificateStore, StoredCertificate};

use serde::{Deserialize, Serialize};

/// Auto TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoTlsConfig {
    /// Enable auto TLS
    #[serde(default)]
    pub enabled: bool,

    /// Domains for certificate
    #[serde(default)]
    pub domains: Vec<String>,

    /// Email for Let's Encrypt registration
    #[serde(default)]
    pub email: String,

    /// ACME challenge type: "http-01" or "dns-01" (default: "http-01")
    #[serde(default = "default_challenge_type")]
    pub challenge_type: String,

    /// Certificate cache directory (default: "./certs")
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,

    /// Days before expiration to renew (default: 30)
    #[serde(default = "default_renew_days")]
    pub renew_before_days: u32,
}

fn default_challenge_type() -> String { "http-01".to_string() }
fn default_cache_dir() -> String { "./certs".to_string() }
fn default_renew_days() -> u32 { 30 }

impl Default for AutoTlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domains: vec![],
            email: String::new(),
            challenge_type: default_challenge_type(),
            cache_dir: default_cache_dir(),
            renew_before_days: default_renew_days(),
        }
    }
}

/// ACME directory URL for Let's Encrypt production
pub const LETSENCRYPT_PRODUCTION_URL: &str = "https://acme-v02.api.letsencrypt.org/directory";

/// ACME directory URL for Let's Encrypt staging
pub const LETSENCRYPT_STAGING_URL: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";

/// Error type for auto TLS operations
#[derive(Debug, thiserror::Error)]
pub enum AutoTlsError {
    /// ACME protocol error
    #[error("ACME error: {0}")]
    AcmeError(String),

    /// Challenge error
    #[error("Challenge error: {0}")]
    ChallengeError(String),

    /// Certificate error
    #[error("Certificate error: {0}")]
    CertificateError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Store error
    #[error("Store error: {0}")]
    StoreError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for auto TLS operations
pub type AutoTlsResult<T> = Result<T, AutoTlsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_tls_config_default() {
        let config = AutoTlsConfig::default();
        assert!(!config.enabled);
        assert!(config.domains.is_empty());
        assert_eq!(config.challenge_type, "http-01");
        assert_eq!(config.cache_dir, "./certs");
        assert_eq!(config.renew_before_days, 30);
    }

    #[test]
    fn test_auto_tls_config_custom() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "dns-01".to_string(),
            cache_dir: "/etc/certs".to_string(),
            renew_before_days: 14,
        };
        assert!(config.enabled);
        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.challenge_type, "dns-01");
        assert_eq!(config.renew_before_days, 14);
    }

    #[test]
    fn test_letsencrypt_urls() {
        assert!(LETSENCRYPT_PRODUCTION_URL.contains("letsencrypt.org"));
        assert!(LETSENCRYPT_STAGING_URL.contains("letsencrypt.org"));
        assert!(LETSENCRYPT_STAGING_URL.contains("staging"));
    }

    #[test]
    fn test_auto_tls_error_display() {
        let error = AutoTlsError::AcmeError("test".to_string());
        assert!(error.to_string().contains("ACME error"));

        let error = AutoTlsError::ChallengeError("fail".to_string());
        assert!(error.to_string().contains("Challenge error"));
    }
}
