//! ACME client
//!
//! Client for communicating with Let's Encrypt ACME servers.

use crate::config::AutoTlsConfig;

/// ACME client for certificate management
#[derive(Debug)]
pub struct AcmeClient {
    config: AutoTlsConfig,
}

impl AcmeClient {
    /// Create a new ACME client
    pub fn new(config: AutoTlsConfig) -> Self {
        Self { config }
    }

    /// Get the configured domains
    pub fn domains(&self) -> &[String] {
        &self.config.domains
    }

    /// Check if the client is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acme_client_creation() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let client = AcmeClient::new(config);
        assert!(client.is_enabled());
        assert_eq!(client.domains(), &["example.com"]);
    }
}
