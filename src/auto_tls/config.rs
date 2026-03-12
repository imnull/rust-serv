//! Auto TLS configuration
//!
//! Configuration for automatic TLS certificate management.

/// Re-export AutoTlsConfig from the main config module
pub use crate::config::AutoTlsConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_tls_config_creation() {
        let config = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        assert!(config.enabled);
        assert_eq!(config.domains, vec!["example.com"]);
    }
}
