//! ACME challenge handler
//!
//! Handles HTTP-01 and DNS-01 challenges for ACME certificate validation.

use crate::config::AutoTlsConfig;

/// Challenge handler for ACME validation
#[derive(Debug)]
pub struct ChallengeHandler {
    config: AutoTlsConfig,
}

impl ChallengeHandler {
    /// Create a new challenge handler
    pub fn new(config: AutoTlsConfig) -> Self {
        Self { config }
    }

    /// Get the challenge type
    pub fn challenge_type(&self) -> &str {
        &self.config.challenge_type
    }

    /// Check if using HTTP-01 challenge
    pub fn is_http01(&self) -> bool {
        self.config.challenge_type == "http-01"
    }

    /// Check if using DNS-01 challenge
    pub fn is_dns01(&self) -> bool {
        self.config.challenge_type == "dns-01"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_handler_http01() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let handler = ChallengeHandler::new(config);
        assert!(handler.is_http01());
        assert!(!handler.is_dns01());
    }

    #[test]
    fn test_challenge_handler_dns01() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "dns-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let handler = ChallengeHandler::new(config);
        assert!(!handler.is_http01());
        assert!(handler.is_dns01());
    }
}
