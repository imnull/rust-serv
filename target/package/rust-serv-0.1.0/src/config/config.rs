use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server port (default: 8080)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Root directory for serving files (default: ".")
    #[serde(default = "default_root")]
    pub root: PathBuf,

    /// Enable directory indexing (default: true)
    #[serde(default = "default_indexing")]
    pub enable_indexing: bool,

    /// Enable compression (default: true)
    #[serde(default = "default_compression")]
    pub enable_compression: bool,

    /// Log level (default: info)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable HTTPS/TLS (default: false)
    #[serde(default = "default_tls")]
    pub enable_tls: bool,

    /// TLS certificate file path
    #[serde(default)]
    pub tls_cert: Option<String>,

    /// TLS private key file path
    #[serde(default)]
    pub tls_key: Option<String>,

    /// Connection timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub connection_timeout_secs: u64,

    /// Max concurrent connections (default: 1000)
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Enable health check endpoint (default: true)
    #[serde(default = "default_health_check")]
    pub enable_health_check: bool,

    /// Enable CORS support (default: true)
    #[serde(default = "default_cors")]
    pub enable_cors: bool,

    /// CORS: Allowed origins (default: ["*"] for all origins)
    #[serde(default = "default_cors_origins")]
    pub cors_allowed_origins: Vec<String>,

    /// CORS: Allowed methods (default: ["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "PATCH"])
    #[serde(default = "default_cors_methods")]
    pub cors_allowed_methods: Vec<String>,

    /// CORS: Allowed headers (default: [])
    #[serde(default = "default_cors_headers")]
    pub cors_allowed_headers: Vec<String>,

    /// CORS: Allow credentials (default: false)
    #[serde(default = "default_cors_credentials")]
    pub cors_allow_credentials: bool,

    /// CORS: Exposed headers (default: [])
    #[serde(default = "default_cors_exposed_headers")]
    pub cors_exposed_headers: Vec<String>,

    /// CORS: Max age for preflight (default: 86400)
    #[serde(default = "default_cors_max_age")]
    pub cors_max_age: Option<u64>,

    /// Enable security features (default: true)
    #[serde(default = "default_enable_security")]
    pub enable_security: bool,

    /// Rate limiting: Max requests per window (default: 100)
    #[serde(default = "default_rate_limit_max_requests")]
    pub rate_limit_max_requests: usize,

    /// Rate limiting: Window duration in seconds (default: 60)
    #[serde(default = "default_rate_limit_window")]
    pub rate_limit_window_secs: u64,

    /// IP allowlist (empty = all allowed)
    #[serde(default = "default_ip_allowlist")]
    pub ip_allowlist: Vec<String>,

    /// IP blocklist
    #[serde(default = "default_ip_blocklist")]
    pub ip_blocklist: Vec<String>,

    /// Request size limits: Max body size in bytes (default: 10485760 = 10MB)
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    /// Request size limits: Max headers (default: 100)
    #[serde(default = "default_max_headers")]
    pub max_headers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            root: PathBuf::from("."),
            enable_indexing: true,
            enable_compression: true,
            log_level: "info".to_string(),
            enable_tls: false,
            tls_cert: None,
            tls_key: None,
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
                "HEAD".to_string(),
                "PATCH".to_string(),
            ],
            cors_allowed_headers: vec![],
            cors_allow_credentials: false,
            cors_exposed_headers: vec![],
            cors_max_age: Some(86400),
            enable_security: true,
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
            ip_allowlist: vec![],
            ip_blocklist: vec![],
            max_body_size: 10 * 1024 * 1024, // 10 MB
            max_headers: 100,
        }
    }
}

fn default_port() -> u16 {
    8080
}

fn default_root() -> PathBuf {
    PathBuf::from(".")
}

fn default_indexing() -> bool {
    true
}

fn default_compression() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_tls() -> bool {
    false
}

fn default_timeout() -> u64 {
    30
}

fn default_max_connections() -> usize {
    1000
}

fn default_health_check() -> bool {
    true
}

fn default_cors() -> bool {
    true
}

fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
        "HEAD".to_string(),
        "PATCH".to_string(),
    ]
}

fn default_cors_headers() -> Vec<String> {
    vec![]
}

fn default_cors_credentials() -> bool {
    false
}

fn default_cors_exposed_headers() -> Vec<String> {
    vec![]
}

fn default_cors_max_age() -> Option<u64> {
    Some(86400)
}

fn default_enable_security() -> bool {
    true
}

fn default_rate_limit_max_requests() -> usize {
    100
}

fn default_rate_limit_window() -> u64 {
    60
}

fn default_ip_allowlist() -> Vec<String> {
    vec![]
}

fn default_ip_blocklist() -> Vec<String> {
    vec![]
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10 MB
}

fn default_max_headers() -> usize {
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.root, PathBuf::from("."));
        assert!(config.enable_indexing);
        assert!(config.enable_compression);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_deserialize_empty_config() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_deserialize_custom_config() {
        let toml = r#"
            port = 9000
            root = "/var/www"
            enable_indexing = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.root, PathBuf::from("/var/www"));
        assert!(!config.enable_indexing);
    }

    #[test]
    fn test_tls_config() {
        let toml = r#"
            enable_tls = true
            tls_cert = "/path/to/cert.pem"
            tls_key = "/path/to/key.pem"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.enable_tls);
        assert_eq!(config.tls_cert, Some("/path/to/cert.pem".to_string()));
        assert_eq!(config.tls_key, Some("/path/to/key.pem".to_string()));
    }

    #[test]
    fn test_timeout_config() {
        let config = Config::default();
        assert_eq!(config.connection_timeout_secs, 30);
    }

    #[test]
    fn test_max_connections_config() {
        let config = Config::default();
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_health_check_config() {
        let config = Config::default();
        assert!(config.enable_health_check);
    }
}
