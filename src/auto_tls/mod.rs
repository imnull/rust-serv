//! Auto TLS (Let's Encrypt) module
//!
//! This module provides automatic TLS certificate management using Let's Encrypt.

mod account;
mod challenge;
mod client;
mod config;
mod renewer;
mod store;

pub use account::AcmeAccount;
pub use challenge::{ChallengeHandler, ChallengeType, Http01Challenge};
pub use client::AcmeClient;
pub use config::AutoTlsConfig;
pub use renewer::CertificateRenewer;
pub use store::{CertificateStore, StoredCertificate};

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
    SerializationError(String),
}

/// Result type for auto TLS operations
pub type AutoTlsResult<T> = Result<T, AutoTlsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letsencrypt_urls() {
        assert!(LETSENCRYPT_PRODUCTION_URL.contains("letsencrypt.org"));
        assert!(LETSENCRYPT_STAGING_URL.contains("letsencrypt.org"));
        assert!(LETSENCRYPT_PRODUCTION_URL.contains("acme-v02"));
        assert!(LETSENCRYPT_STAGING_URL.contains("staging"));
    }

    #[test]
    fn test_auto_tls_error_display() {
        let error = AutoTlsError::AcmeError("test error".to_string());
        assert!(error.to_string().contains("ACME error"));
        assert!(error.to_string().contains("test error"));

        let error = AutoTlsError::ChallengeError("challenge failed".to_string());
        assert!(error.to_string().contains("Challenge error"));

        let error = AutoTlsError::CertificateError("invalid cert".to_string());
        assert!(error.to_string().contains("Certificate error"));

        let error = AutoTlsError::ConfigError("bad config".to_string());
        assert!(error.to_string().contains("Configuration error"));

        let error = AutoTlsError::StoreError("store failed".to_string());
        assert!(error.to_string().contains("Store error"));

        let error = AutoTlsError::SerializationError("serde failed".to_string());
        assert!(error.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_auto_tls_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: AutoTlsError = io_error.into();
        assert!(matches!(error, AutoTlsError::IoError(_)));
    }

    #[test]
    fn test_auto_tls_result_ok() {
        let result: AutoTlsResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_auto_tls_result_err() {
        let result: AutoTlsResult<i32> = Err(AutoTlsError::ConfigError("test".to_string()));
        assert!(result.is_err());
    }
}
