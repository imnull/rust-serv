//! ACME Client for certificate management
//!
//! Note: Full ACME implementation requires careful handling of various edge cases.
//! This module provides the core structure with placeholder implementations.

use std::path::Path;
use std::sync::Arc;

use super::challenge::ChallengeHandler;
use super::store::CertificateStore;
use super::{AutoTlsConfig, AutoTlsError, AutoTlsResult};

/// ACME Client for certificate operations
pub struct AcmeClient {
    config: AutoTlsConfig,
    challenge_handler: Arc<ChallengeHandler>,
    store: CertificateStore,
}

impl AcmeClient {
    /// Create a new ACME client
    pub fn new(config: AutoTlsConfig) -> Self {
        let store = CertificateStore::new(Path::new(&config.cache_dir));
        Self {
            config,
            challenge_handler: Arc::new(ChallengeHandler::new()),
            store,
        }
    }

    /// Create client with existing challenge handler
    pub fn with_challenge_handler(
        config: AutoTlsConfig,
        challenge_handler: Arc<ChallengeHandler>,
    ) -> Self {
        let store = CertificateStore::new(Path::new(&config.cache_dir));
        Self {
            config,
            challenge_handler,
            store,
        }
    }

    /// Get the challenge handler
    pub fn challenge_handler(&self) -> Arc<ChallengeHandler> {
        self.challenge_handler.clone()
    }

    /// Initialize ACME account
    pub async fn initialize(&mut self) -> AutoTlsResult<()> {
        // Ensure cache directory exists
        self.store.init().await?;
        tracing::info!("ACME client initialized");
        Ok(())
    }

    /// Request a certificate for configured domains
    ///
    /// TODO: Full implementation using instant-acme
    /// For now, this returns an error indicating manual certificate setup is needed
    pub async fn request_certificate(&mut self) -> AutoTlsResult<Vec<String>> {
        if self.config.domains.is_empty() {
            return Err(AutoTlsError::ConfigError("No domains configured".to_string()));
        }

        // Placeholder: In production, this would:
        // 1. Create ACME account
        // 2. Create order with Let's Encrypt
        // 3. Complete HTTP-01 challenges
        // 4. Generate CSR
        // 5. Finalize order
        // 6. Download certificate

        Err(AutoTlsError::AcmeError(
            "Auto TLS not yet fully implemented. Please use manual certificate setup or certbot.".to_string()
        ))
    }

    /// Check if certificate needs renewal
    pub async fn needs_renewal(&self) -> AutoTlsResult<bool> {
        let domain = match self.config.domains.first() {
            Some(d) => d,
            None => return Ok(false),
        };

        match self.store.load_certificate(domain).await? {
            Some(cert) => Ok(cert.is_expired() || cert.days_until_expiry() <= self.config.renew_before_days as i64),
            None => Ok(true),
        }
    }

    /// Get current certificate if exists
    pub async fn get_certificate(&self) -> AutoTlsResult<Option<super::store::StoredCertificate>> {
        let domain = match self.config.domains.first() {
            Some(d) => d,
            None => return Ok(None),
        };
        self.store.load_certificate(domain).await
    }

    /// Import existing certificate (from certbot or other source)
    pub async fn import_certificate(
        &self,
        certificate: String,
        private_key: String,
        expires_at: Option<u64>,
    ) -> AutoTlsResult<()> {
        let domain = match self.config.domains.first() {
            Some(d) => d,
            None => return Err(AutoTlsError::ConfigError("No domains configured".to_string())),
        };

        let expires = expires_at.unwrap_or_else(|| {
            // Default to 90 days from now
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + (90 * 24 * 60 * 60)
        });

        self.store
            .save_certificate_with_expiry(domain, &certificate, &private_key, expires)
            .await?;

        tracing::info!("Certificate imported for {}", domain);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acme_client_creation() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            cache_dir: std::env::temp_dir().to_string_lossy().to_string(),
            ..Default::default()
        };

        let client = AcmeClient::new(config);
        assert_eq!(client.challenge_handler().challenge_count().await, 0);
    }

    #[test]
    fn test_acme_client_with_challenge_handler() {
        let config = AutoTlsConfig::default();
        let handler = Arc::new(ChallengeHandler::new());
        let client = AcmeClient::with_challenge_handler(config, handler.clone());
        
        // Both should share the same handler
        assert!(Arc::ptr_eq(&client.challenge_handler(), &handler));
    }

    #[tokio::test]
    async fn test_needs_renewal_no_certificate() {
        let config = AutoTlsConfig {
            domains: vec!["example.com".to_string()],
            cache_dir: std::env::temp_dir().to_string_lossy().to_string(),
            ..Default::default()
        };
        let client = AcmeClient::new(config);
        
        // Without certificate, should return true
        let result = client.needs_renewal().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_initialize() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = AutoTlsConfig {
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        let mut client = AcmeClient::new(config);
        
        client.initialize().await.unwrap();
    }
}
