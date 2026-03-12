//! ACME client
//!
//! Client for communicating with Let's Encrypt ACME servers.

use super::account::AcmeAccount;
use super::challenge::ChallengeHandler;
use super::config::AutoTlsConfig;
use super::store::CertificateStore;
use super::{AutoTlsError, AutoTlsResult, LETSENCRYPT_PRODUCTION_URL, LETSENCRYPT_STAGING_URL};
use std::sync::Arc;

/// ACME client for certificate management
#[derive(Debug)]
pub struct AcmeClient {
    config: AutoTlsConfig,
    store: CertificateStore,
}

impl AcmeClient {
    /// Create a new ACME client
    pub fn new(config: AutoTlsConfig) -> Self {
        let store = CertificateStore::new(config.cache_dir.clone());
        Self { config, store }
    }

    /// Create a new ACME client with a custom store
    pub fn with_store(config: AutoTlsConfig, store: CertificateStore) -> Self {
        Self { config, store }
    }

    /// Get the configured domains
    pub fn domains(&self) -> &[String] {
        &self.config.domains
    }

    /// Check if the client is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the email address
    pub fn email(&self) -> &str {
        &self.config.email
    }

    /// Get the challenge type
    pub fn challenge_type(&self) -> &str {
        &self.config.challenge_type
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &str {
        &self.config.cache_dir
    }

    /// Get the renew before days threshold
    pub fn renew_before_days(&self) -> u32 {
        self.config.renew_before_days
    }

    /// Get the certificate store
    pub fn store(&self) -> &CertificateStore {
        &self.store
    }

    /// Get the ACME directory URL based on configuration
    pub fn directory_url(&self, staging: bool) -> &'static str {
        if staging {
            LETSENCRYPT_STAGING_URL
        } else {
            LETSENCRYPT_PRODUCTION_URL
        }
    }

    /// Check if using HTTP-01 challenge
    pub fn is_http01(&self) -> bool {
        self.config.challenge_type == "http-01"
    }

    /// Check if using DNS-01 challenge
    pub fn is_dns01(&self) -> bool {
        self.config.challenge_type == "dns-01"
    }

    /// Create a challenge handler for this client
    pub fn challenge_handler(&self) -> ChallengeHandler {
        if self.is_http01() {
            ChallengeHandler::Http01(super::challenge::Http01Challenge::new())
        } else {
            ChallengeHandler::Dns01
        }
    }

    /// Request a certificate for the configured domains
    ///
    /// This is a placeholder that simulates the ACME flow.
    /// In a real implementation, this would use instant-acme crate.
    pub async fn request_certificate(&self) -> AutoTlsResult<super::store::StoredCertificate> {
        if !self.config.enabled {
            return Err(AutoTlsError::ConfigError(
                "Auto TLS is not enabled".to_string(),
            ));
        }

        if self.config.domains.is_empty() {
            return Err(AutoTlsError::ConfigError(
                "No domains configured".to_string(),
            ));
        }

        // In a real implementation, this would:
        // 1. Load or create an ACME account
        // 2. Create a new order with the ACME server
        // 3. Complete the challenges
        // 4. Finalize the order and download the certificate

        // For now, return a mock certificate
        let cert = super::store::StoredCertificate {
            domains: self.config.domains.clone(),
            certificate: "MOCK_CERTIFICATE".to_string(),
            private_key: "MOCK_PRIVATE_KEY".to_string(),
            not_before: chrono::Utc::now(),
            not_after: chrono::Utc::now() + chrono::Duration::days(90),
        };

        // Save to store
        self.store.save_certificate(&cert)?;

        Ok(cert)
    }

    /// Load an existing certificate from the store
    pub fn load_certificate(&self) -> AutoTlsResult<Option<super::store::StoredCertificate>> {
        self.store.load_certificate()
    }

    /// Check if the certificate needs renewal
    pub fn needs_renewal(&self) -> AutoTlsResult<bool> {
        match self.store.load_certificate()? {
            Some(cert) => {
                let renewal_threshold =
                    chrono::Duration::days(self.config.renew_before_days as i64);
                let expires_in = cert.not_after - chrono::Utc::now();
                Ok(expires_in < renewal_threshold)
            }
            None => Ok(true),
        }
    }

    /// Create or load an ACME account
    pub fn create_or_load_account(&self) -> AutoTlsResult<AcmeAccount> {
        let account_path = std::path::PathBuf::from(&self.config.cache_dir).join("account.json");

        if account_path.exists() {
            AcmeAccount::load_from_file(&account_path)
        } else {
            // Create a new account with a generated key
            // In a real implementation, this would generate a proper ECDSA key
            let account = AcmeAccount::new(
                self.config.email.clone(),
                "GENERATED_PRIVATE_KEY".to_string(),
            );

            // Ensure the cache directory exists
            std::fs::create_dir_all(&self.config.cache_dir)?;

            // Save the account
            account.save_to_file(&account_path)?;

            Ok(account)
        }
    }
}

impl Clone for AcmeClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            store: self.store.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (AutoTlsConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string(), "www.example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            renew_before_days: 30,
        };
        (config, temp_dir)
    }

    #[test]
    fn test_acme_client_creation() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config.clone());
        assert!(client.is_enabled());
        assert_eq!(client.domains(), &["example.com", "www.example.com"]);
        assert_eq!(client.email(), "admin@example.com");
        assert_eq!(client.challenge_type(), "http-01");
        assert_eq!(client.renew_before_days(), 30);
    }

    #[test]
    fn test_acme_client_domains() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        assert_eq!(client.domains().len(), 2);
        assert!(client.domains().contains(&"example.com".to_string()));
    }

    #[test]
    fn test_acme_client_challenge_type() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        assert!(client.is_http01());
        assert!(!client.is_dns01());
    }

    #[test]
    fn test_acme_client_dns01_challenge() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "dns-01".to_string(),
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            renew_before_days: 30,
        };
        let client = AcmeClient::new(config);
        assert!(!client.is_http01());
        assert!(client.is_dns01());
    }

    #[test]
    fn test_acme_client_directory_url() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);

        assert!(client.directory_url(false).contains("acme-v02"));
        assert!(client.directory_url(true).contains("staging"));
    }

    #[test]
    fn test_acme_client_challenge_handler() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        let handler = client.challenge_handler();
        assert!(matches!(handler, ChallengeHandler::Http01(_)));
    }

    #[test]
    fn test_acme_client_clone() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        let cloned = client.clone();
        assert_eq!(client.domains(), cloned.domains());
    }

    #[test]
    fn test_acme_client_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutoTlsConfig {
            enabled: false,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            renew_before_days: 30,
        };
        let client = AcmeClient::new(config);
        assert!(!client.is_enabled());
    }

    #[test]
    fn test_acme_client_cache_dir() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config.clone());
        assert_eq!(client.cache_dir(), config.cache_dir);
    }

    #[test]
    fn test_acme_client_create_account() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        let account = client.create_or_load_account().unwrap();
        assert_eq!(account.email, "admin@example.com");
    }

    #[test]
    fn test_acme_client_load_existing_account() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);

        // Create account first time
        let account1 = client.create_or_load_account().unwrap();

        // Load existing account
        let account2 = client.create_or_load_account().unwrap();

        assert_eq!(account1.email, account2.email);
    }

    #[tokio::test]
    async fn test_acme_client_request_certificate_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutoTlsConfig {
            enabled: false,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            renew_before_days: 30,
        };
        let client = AcmeClient::new(config);
        let result = client.request_certificate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_acme_client_request_certificate_no_domains() {
        let temp_dir = TempDir::new().unwrap();
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec![],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            renew_before_days: 30,
        };
        let client = AcmeClient::new(config);
        let result = client.request_certificate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_acme_client_needs_renewal_no_cert() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        let needs = client.needs_renewal().unwrap();
        assert!(needs);
    }

    #[test]
    fn test_acme_client_debug() {
        let (config, _temp_dir) = create_test_config();
        let client = AcmeClient::new(config);
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("AcmeClient"));
    }
}
