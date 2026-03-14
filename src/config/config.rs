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

    /// Management API configuration
    #[serde(default)]
    pub management: Option<ManagementConfig>,

    /// Plugin system configuration
    #[serde(default)]
    pub plugins: Option<PluginSystemConfig>,

    /// Auto TLS (Let's Encrypt) configuration
    #[serde(default)]
    pub auto_tls: Option<AutoTlsConfig>,
}

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginSystemConfig {
    /// Enable plugin system (default: false)
    #[serde(default = "default_plugins_enabled")]
    pub enabled: bool,

    /// Plugin directory path (default: "./plugins")
    #[serde(default = "default_plugins_dir")]
    pub directory: PathBuf,

    /// Enable hot reload (default: true)
    #[serde(default = "default_plugins_hot_reload")]
    pub hot_reload: bool,

    /// Maximum number of plugins (default: 100)
    #[serde(default = "default_plugins_max")]
    pub max_plugins: usize,

    /// Default plugin timeout in milliseconds (default: 100)
    #[serde(default = "default_plugins_timeout")]
    pub timeout_ms: u64,

    /// Management API prefix for plugins (default: "_plugins")
    #[serde(default = "default_plugins_api_prefix")]
    pub api_prefix: String,

    /// Preload plugins on startup (default: [])
    #[serde(default)]
    pub preload: Vec<PluginLoadConfig>,
}

/// Plugin load configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginLoadConfig {
    /// Plugin ID
    pub id: String,

    /// Plugin file path (relative to plugins directory)
    pub path: String,

    /// Plugin priority (higher = earlier execution)
    #[serde(default)]
    pub priority: Option<i32>,

    /// Whether plugin is enabled
    #[serde(default = "default_plugin_enabled")]
    pub enabled: bool,

    /// Plugin-specific configuration
    #[serde(default)]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

/// Management API configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagementConfig {
    /// Enable management API endpoints (default: false)
    #[serde(default = "default_management_enabled")]
    pub enabled: bool,

    /// Health check endpoint path (default: "/health")
    #[serde(default = "default_health_path")]
    pub health_path: String,

    /// Readiness check endpoint path (default: "/ready")
    #[serde(default = "default_ready_path")]
    pub ready_path: String,

    /// Stats endpoint path (default: "/stats")
    #[serde(default = "default_stats_path")]
    pub stats_path: String,
}

/// Auto TLS (Let's Encrypt) configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoTlsConfig {
    /// Enable auto TLS certificate management (default: false)
    #[serde(default = "default_auto_tls_enabled")]
    pub enabled: bool,

    /// Domains for certificate
    #[serde(default)]
    pub domains: Vec<String>,

    /// Email for Let's Encrypt registration
    #[serde(default)]
    pub email: String,

    /// ACME challenge type: "http-01" or "dns-01" (default: "http-01")
    #[serde(default = "default_challenge_type")]
    pub challenge_type: String,

    /// Certificate cache directory (default: "./certs")
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,

    /// Days before expiration to renew (default: 30)
    #[serde(default = "default_renew_before_days")]
    pub renew_before_days: u32,
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
            management: None,
            plugins: None,
            auto_tls: None,
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

// Plugin config defaults
fn default_plugins_enabled() -> bool {
    false
}

fn default_plugins_dir() -> PathBuf {
    PathBuf::from("./plugins")
}

fn default_plugins_hot_reload() -> bool {
    true
}

fn default_plugins_max() -> usize {
    100
}

fn default_plugins_timeout() -> u64 {
    100
}

fn default_plugins_api_prefix() -> String {
    "_plugins".to_string()
}

fn default_plugin_enabled() -> bool {
    true
}

// Management config defaults
fn default_management_enabled() -> bool {
    false
}

fn default_health_path() -> String {
    "/health".to_string()
}

fn default_ready_path() -> String {
    "/ready".to_string()
}

fn default_stats_path() -> String {
    "/stats".to_string()
}

// Auto TLS config defaults
fn default_auto_tls_enabled() -> bool {
    false
}

fn default_challenge_type() -> String {
    "http-01".to_string()
}

fn default_cache_dir() -> String {
    "./certs".to_string()
}

fn default_renew_before_days() -> u32 {
    30
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

    #[test]
    fn test_management_config_default() {
        let config = Config::default();
        assert!(config.management.is_none());
    }

    #[test]
    fn test_auto_tls_config_default() {
        let config = Config::default();
        assert!(config.auto_tls.is_none());
    }

    #[test]
    fn test_management_config_deserialize() {
        let toml = r#"
            [management]
            enabled = true
            health_path = "/health"
            ready_path = "/ready"
            stats_path = "/stats"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let mgmt = config.management.unwrap();
        assert!(mgmt.enabled);
        assert_eq!(mgmt.health_path, "/health");
        assert_eq!(mgmt.ready_path, "/ready");
        assert_eq!(mgmt.stats_path, "/stats");
    }

    #[test]
    fn test_management_config_partial() {
        let toml = r#"
            [management]
            enabled = true
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let mgmt = config.management.unwrap();
        assert!(mgmt.enabled);
        assert_eq!(mgmt.health_path, "/health"); // default
        assert_eq!(mgmt.ready_path, "/ready"); // default
        assert_eq!(mgmt.stats_path, "/stats"); // default
    }

    #[test]
    fn test_auto_tls_config_deserialize() {
        let toml = r#"
            [auto_tls]
            enabled = true
            domains = ["example.com", "www.example.com"]
            email = "admin@example.com"
            challenge_type = "http-01"
            cache_dir = "./certs"
            renew_before_days = 30
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let auto_tls = config.auto_tls.unwrap();
        assert!(auto_tls.enabled);
        assert_eq!(auto_tls.domains, vec!["example.com", "www.example.com"]);
        assert_eq!(auto_tls.email, "admin@example.com");
        assert_eq!(auto_tls.challenge_type, "http-01");
        assert_eq!(auto_tls.cache_dir, "./certs");
        assert_eq!(auto_tls.renew_before_days, 30);
    }

    #[test]
    fn test_auto_tls_config_partial() {
        let toml = r#"
            [auto_tls]
            enabled = true
            domains = ["example.com"]
            email = "admin@example.com"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let auto_tls = config.auto_tls.unwrap();
        assert!(auto_tls.enabled);
        assert_eq!(auto_tls.domains, vec!["example.com"]);
        assert_eq!(auto_tls.email, "admin@example.com");
        assert_eq!(auto_tls.challenge_type, "http-01"); // default
        assert_eq!(auto_tls.cache_dir, "./certs"); // default
        assert_eq!(auto_tls.renew_before_days, 30); // default
    }

    #[test]
    fn test_management_config_custom_paths() {
        let toml = r#"
            [management]
            enabled = true
            health_path = "/api/health"
            ready_path = "/api/ready"
            stats_path = "/api/stats"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let mgmt = config.management.unwrap();
        assert_eq!(mgmt.health_path, "/api/health");
        assert_eq!(mgmt.ready_path, "/api/ready");
        assert_eq!(mgmt.stats_path, "/api/stats");
    }

    #[test]
    fn test_auto_tls_dns_challenge() {
        let toml = r#"
            [auto_tls]
            enabled = true
            domains = ["example.com"]
            email = "admin@example.com"
            challenge_type = "dns-01"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let auto_tls = config.auto_tls.unwrap();
        assert_eq!(auto_tls.challenge_type, "dns-01");
    }

    #[test]
    fn test_management_config_equality() {
        let config1 = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let config2 = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_auto_tls_config_equality() {
        let config1 = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        let config2 = AutoTlsConfig {
            enabled: true,
            domains: vec!["example.com".to_string()],
            email: "admin@example.com".to_string(),
            challenge_type: "http-01".to_string(),
            cache_dir: "./certs".to_string(),
            renew_before_days: 30,
        };
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_config_with_both_management_and_auto_tls() {
        let toml = r#"
            [management]
            enabled = true
            [auto_tls]
            enabled = true
            domains = ["example.com"]
            email = "admin@example.com"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.management.as_ref().unwrap().enabled);
        assert!(config.auto_tls.as_ref().unwrap().enabled);
    }

    #[test]
    fn test_management_config_disabled() {
        let toml = r#"
            [management]
            enabled = false
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.management.unwrap().enabled);
    }

    #[test]
    fn test_auto_tls_multiple_domains() {
        let toml = r#"
            [auto_tls]
            enabled = true
            domains = ["example.com", "www.example.com", "api.example.com"]
            email = "admin@example.com"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let auto_tls = config.auto_tls.unwrap();
        assert_eq!(auto_tls.domains.len(), 3);
    }

    #[test]
    fn test_auto_tls_custom_renew_days() {
        let toml = r#"
            [auto_tls]
            enabled = true
            domains = ["example.com"]
            email = "admin@example.com"
            renew_before_days = 14
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let auto_tls = config.auto_tls.unwrap();
        assert_eq!(auto_tls.renew_before_days, 14);
    }
}
