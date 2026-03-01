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
