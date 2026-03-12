//! Certificate renewer
//!
//! Background task for automatic certificate renewal.

use crate::config::AutoTlsConfig;

/// Certificate renewer for automatic certificate management
#[derive(Debug)]
pub struct CertificateRenewer {
    config: AutoTlsConfig,
}

impl CertificateRenewer {
    /// Create a new certificate renewer
    pub fn new(config: AutoTlsConfig) -> Self {
        Self { config }
    }

    /// Get days before expiration to renew
    pub fn renew_before_days(&self) -> u32 {
        self.config.renew_before_days
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &str {
        &self.config.cache_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_renewer() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let renewer = CertificateRenewer::new(config);
        assert_eq!(renewer.renew_before_days(), 30);
        assert_eq!(renewer.cache_dir(), "./certs");
    }
}
