//! Configuration difference detection

use crate::config::Config;
use std::collections::HashSet;

/// Configuration change details
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDiff {
    /// Fields that changed
    pub changed_fields: Vec<String>,
    /// Whether the change requires a restart
    pub requires_restart: bool,
}

impl ConfigDiff {
    /// Create a new ConfigDiff
    pub fn new() -> Self {
        Self {
            changed_fields: Vec::new(),
            requires_restart: false,
        }
    }

    /// Compare two configurations and return the differences
    pub fn compare(old: &Config, new: &Config) -> Self {
        let mut diff = Self::new();
        
        // Check each field
        if old.port != new.port {
            diff.changed_fields.push("port".to_string());
            diff.requires_restart = true;
        }
        
        if old.root != new.root {
            diff.changed_fields.push("root".to_string());
        }
        
        if old.enable_indexing != new.enable_indexing {
            diff.changed_fields.push("enable_indexing".to_string());
        }
        
        if old.enable_compression != new.enable_compression {
            diff.changed_fields.push("enable_compression".to_string());
        }
        
        if old.log_level != new.log_level {
            diff.changed_fields.push("log_level".to_string());
        }
        
        if old.enable_tls != new.enable_tls {
            diff.changed_fields.push("enable_tls".to_string());
            diff.requires_restart = true;
        }
        
        if old.tls_cert != new.tls_cert {
            diff.changed_fields.push("tls_cert".to_string());
            diff.requires_restart = true;
        }
        
        if old.tls_key != new.tls_key {
            diff.changed_fields.push("tls_key".to_string());
            diff.requires_restart = true;
        }
        
        if old.connection_timeout_secs != new.connection_timeout_secs {
            diff.changed_fields.push("connection_timeout_secs".to_string());
        }
        
        if old.max_connections != new.max_connections {
            diff.changed_fields.push("max_connections".to_string());
        }
        
        if old.enable_health_check != new.enable_health_check {
            diff.changed_fields.push("enable_health_check".to_string());
        }
        
        if old.enable_cors != new.enable_cors {
            diff.changed_fields.push("enable_cors".to_string());
        }
        
        if !Self::vec_eq(&old.cors_allowed_origins, &new.cors_allowed_origins) {
            diff.changed_fields.push("cors_allowed_origins".to_string());
        }
        
        if !Self::vec_eq(&old.cors_allowed_methods, &new.cors_allowed_methods) {
            diff.changed_fields.push("cors_allowed_methods".to_string());
        }
        
        if !Self::vec_eq(&old.cors_allowed_headers, &new.cors_allowed_headers) {
            diff.changed_fields.push("cors_allowed_headers".to_string());
        }
        
        if old.cors_allow_credentials != new.cors_allow_credentials {
            diff.changed_fields.push("cors_allow_credentials".to_string());
        }
        
        if !Self::vec_eq(&old.cors_exposed_headers, &new.cors_exposed_headers) {
            diff.changed_fields.push("cors_exposed_headers".to_string());
        }
        
        if old.cors_max_age != new.cors_max_age {
            diff.changed_fields.push("cors_max_age".to_string());
        }
        
        if old.enable_security != new.enable_security {
            diff.changed_fields.push("enable_security".to_string());
        }
        
        if old.rate_limit_max_requests != new.rate_limit_max_requests {
            diff.changed_fields.push("rate_limit_max_requests".to_string());
        }
        
        if old.rate_limit_window_secs != new.rate_limit_window_secs {
            diff.changed_fields.push("rate_limit_window_secs".to_string());
        }
        
        if !Self::vec_eq(&old.ip_allowlist, &new.ip_allowlist) {
            diff.changed_fields.push("ip_allowlist".to_string());
        }
        
        if !Self::vec_eq(&old.ip_blocklist, &new.ip_blocklist) {
            diff.changed_fields.push("ip_blocklist".to_string());
        }
        
        if old.max_body_size != new.max_body_size {
            diff.changed_fields.push("max_body_size".to_string());
        }
        
        if old.max_headers != new.max_headers {
            diff.changed_fields.push("max_headers".to_string());
        }
        
        diff
    }
    
    /// Check if two vectors are equal (order-independent)
    fn vec_eq(a: &[String], b: &[String]) -> bool {
        let set_a: HashSet<_> = a.iter().collect();
        let set_b: HashSet<_> = b.iter().collect();
        set_a == set_b
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changed_fields.is_empty()
    }

    /// Get the number of changed fields
    pub fn change_count(&self) -> usize {
        self.changed_fields.len()
    }

    /// Check if a specific field changed
    pub fn field_changed(&self, field: &str) -> bool {
        self.changed_fields.iter().any(|f| f == field)
    }
}

impl Default for ConfigDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_diff_creation() {
        let diff = ConfigDiff::new();
        assert!(diff.changed_fields.is_empty());
        assert!(!diff.requires_restart);
    }

    #[test]
    fn test_no_changes() {
        let config1 = Config::default();
        let config2 = Config::default();
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(!diff.has_changes());
        assert_eq!(diff.change_count(), 0);
    }

    #[test]
    fn test_port_change_requires_restart() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.port = 8080;
        config2.port = 9090;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("port"));
        assert!(diff.requires_restart);
    }

    #[test]
    fn test_root_change_no_restart() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.root = PathBuf::from("/var/www1");
        config2.root = PathBuf::from("/var/www2");
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("root"));
        assert!(!diff.requires_restart);
    }

    #[test]
    fn test_tls_change_requires_restart() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_tls = false;
        config2.enable_tls = true;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_tls"));
        assert!(diff.requires_restart);
    }

    #[test]
    fn test_tls_cert_change_requires_restart() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.tls_cert = Some("/old/cert.pem".to_string());
        config2.tls_cert = Some("/new/cert.pem".to_string());
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("tls_cert"));
        assert!(diff.requires_restart);
    }

    #[test]
    fn test_compression_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_compression = true;
        config2.enable_compression = false;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_compression"));
        assert!(!diff.requires_restart);
    }

    #[test]
    fn test_log_level_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.log_level = "info".to_string();
        config2.log_level = "debug".to_string();
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("log_level"));
    }

    #[test]
    fn test_max_connections_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.max_connections = 1000;
        config2.max_connections = 2000;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("max_connections"));
    }

    #[test]
    fn test_cors_origins_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_allowed_origins = vec!["http://localhost".to_string()];
        config2.cors_allowed_origins = vec!["http://example.com".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_allowed_origins"));
    }

    #[test]
    fn test_cors_origins_order_independent() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_allowed_origins = vec!["http://a.com".to_string(), "http://b.com".to_string()];
        config2.cors_allowed_origins = vec!["http://b.com".to_string(), "http://a.com".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        // Should not detect changes when order differs
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_rate_limit_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.rate_limit_max_requests = 100;
        config2.rate_limit_max_requests = 200;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("rate_limit_max_requests"));
    }

    #[test]
    fn test_ip_list_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.ip_allowlist = vec!["127.0.0.1".to_string()];
        config2.ip_allowlist = vec!["127.0.0.1".to_string(), "192.168.1.1".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("ip_allowlist"));
    }

    #[test]
    fn test_multiple_changes() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.port = 8080;
        config1.enable_compression = true;
        config1.log_level = "info".to_string();
        
        config2.port = 9090;
        config2.enable_compression = false;
        config2.log_level = "debug".to_string();
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert_eq!(diff.change_count(), 3);
        assert!(diff.requires_restart); // port change
    }

    #[test]
    fn test_change_count() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.port = 8080;
        config2.port = 9090;
        config1.max_connections = 1000;
        config2.max_connections = 2000;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert_eq!(diff.change_count(), 2);
    }

    #[test]
    fn test_default() {
        let diff = ConfigDiff::default();
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_connection_timeout_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.connection_timeout_secs = 30;
        config2.connection_timeout_secs = 60;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("connection_timeout_secs"));
    }

    #[test]
    fn test_health_check_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_health_check = true;
        config2.enable_health_check = false;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_health_check"));
    }

    #[test]
    fn test_cors_enabled_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_cors = true;
        config2.enable_cors = false;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_cors"));
    }

    #[test]
    fn test_cors_methods_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_allowed_methods = vec!["GET".to_string()];
        config2.cors_allowed_methods = vec!["GET".to_string(), "POST".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_allowed_methods"));
    }

    #[test]
    fn test_cors_headers_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_allowed_headers = vec![];
        config2.cors_allowed_headers = vec!["Content-Type".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_allowed_headers"));
    }

    #[test]
    fn test_cors_credentials_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_allow_credentials = false;
        config2.cors_allow_credentials = true;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_allow_credentials"));
    }

    #[test]
    fn test_cors_exposed_headers_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_exposed_headers = vec![];
        config2.cors_exposed_headers = vec!["X-Custom".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_exposed_headers"));
    }

    #[test]
    fn test_cors_max_age_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.cors_max_age = Some(86400);
        config2.cors_max_age = Some(3600);
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("cors_max_age"));
    }

    #[test]
    fn test_enable_security_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_security = true;
        config2.enable_security = false;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_security"));
    }

    #[test]
    fn test_rate_limit_window_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.rate_limit_window_secs = 60;
        config2.rate_limit_window_secs = 120;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("rate_limit_window_secs"));
    }

    #[test]
    fn test_ip_blocklist_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.ip_blocklist = vec![];
        config2.ip_blocklist = vec!["192.168.1.1".to_string()];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("ip_blocklist"));
    }

    #[test]
    fn test_max_body_size_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.max_body_size = 10485760;
        config2.max_body_size = 20971520;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("max_body_size"));
    }

    #[test]
    fn test_max_headers_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.max_headers = 100;
        config2.max_headers = 200;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("max_headers"));
    }

    #[test]
    fn test_tls_key_change_requires_restart() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.tls_key = Some("/old/key.pem".to_string());
        config2.tls_key = Some("/new/key.pem".to_string());
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("tls_key"));
        assert!(diff.requires_restart);
    }

    #[test]
    fn test_indexing_change() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        config1.enable_indexing = true;
        config2.enable_indexing = false;
        
        let diff = ConfigDiff::compare(&config1, &config2);
        
        assert!(diff.has_changes());
        assert!(diff.field_changed("enable_indexing"));
    }

    #[test]
    fn test_empty_ip_list_comparison() {
        let mut config1 = Config::default();
        let mut config2 = Config::default();
        
        // Both empty - should be equal
        config1.ip_allowlist = vec![];
        config2.ip_allowlist = vec![];
        
        let diff = ConfigDiff::compare(&config1, &config2);
        assert!(!diff.field_changed("ip_allowlist"));
    }
}
