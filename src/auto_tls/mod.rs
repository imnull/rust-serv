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

    #[test]
    fn test_auto_tls_error_certificate() {
        let error = AutoTlsError::CertificateError("invalid cert".to_string());
        assert!(error.to_string().contains("Certificate error"));
    }

    #[test]
    fn test_auto_tls_error_config() {
        let error = AutoTlsError::ConfigError("missing domain".to_string());
        assert!(error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_auto_tls_error_store() {
        let error = AutoTlsError::StoreError("disk full".to_string());
        assert!(error.to_string().contains("Store error"));
    }

    #[test]
    fn test_auto_tls_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: AutoTlsError = io_error.into();
        assert!(error.to_string().contains("IO error"));
    }

    #[test]
    fn test_auto_tls_config_with_multiple_domains() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec![
                "example.com".to_string(),
                "www.example.com".to_string(),
                "api.example.com".to_string(),
            ],
            email: "admin@example.com".to_string(),
            challenge_type: default_challenge_type(),
            cache_dir: default_cache_dir(),
            renew_before_days: default_renew_days(),
        };
        assert!(config.enabled);
        assert_eq!(config.domains.len(), 3);
        assert_eq!(config.domains[0], "example.com");
        assert_eq!(config.domains[1], "www.example.com");
        assert_eq!(config.domains[2], "api.example.com");
    }

    #[test]
    fn test_auto_tls_config_clone() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.domains, cloned.domains);
        assert_eq!(config.email, cloned.email);
        assert_eq!(config.challenge_type, cloned.challenge_type);
        assert_eq!(config.cache_dir, cloned.cache_dir);
        assert_eq!(config.renew_before_days, cloned.renew_before_days);
    }

    #[test]
    fn test_auto_tls_config_debug() {
        let config = AutoTlsConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("AutoTlsConfig"));
    }

    #[test]
    fn test_auto_tls_config_equality() {
        let config1 = AutoTlsConfig::default();
        let config2 = AutoTlsConfig::default();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.domains, config2.domains);
        assert_eq!(config1.email, config2.email);
    }

    #[test]
    fn test_letsencrypt_production_url_format() {
        assert!(LETSENCRYPT_PRODUCTION_URL.starts_with("https://"));
        assert!(LETSENCRYPT_PRODUCTION_URL.contains("acme-v02"));
        assert!(LETSENCRYPT_PRODUCTION_URL.ends_with("/directory"));
    }

    #[test]
    fn test_letsencrypt_staging_url_format() {
        assert!(LETSENCRYPT_STAGING_URL.starts_with("https://"));
        assert!(LETSENCRYPT_STAGING_URL.contains("acme-staging-v02"));
        assert!(LETSENCRYPT_STAGING_URL.ends_with("/directory"));
    }

    #[test]
    fn test_default_challenge_type() {
        assert_eq!(default_challenge_type(), "http-01");
    }

    #[test]
    fn test_default_cache_dir() {
        assert_eq!(default_cache_dir(), "./certs");
    }

    #[test]
    fn test_default_renew_days() {
        assert_eq!(default_renew_days(), 30);
    }

    #[test]
    fn test_auto_tls_config_with_empty_email() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: String::new(),
            challenge_type: default_challenge_type(),
            cache_dir: default_cache_dir(),
            renew_before_days: default_renew_days(),
        };
        assert!(config.enabled);
        assert!(config.email.is_empty());
    }

    #[test]
    fn test_auto_tls_config_renew_days_variations() {
        for days in [1, 7, 14, 30, 60, 90] {
            let config = AutoTlsConfig {
                enabled: true,
                domains: vec!["example.com".to_string()],
                email: "admin@example.com".to_string(),
                challenge_type: default_challenge_type(),
                cache_dir: default_cache_dir(),
                renew_before_days: days,
            };
            assert_eq!(config.renew_before_days, days);
        }
    }
}
