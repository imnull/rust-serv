//! Virtual host configuration

use std::path::PathBuf;

/// Virtual host configuration
#[derive(Debug, Clone, PartialEq)]
pub struct VHostConfig {
    /// Host name (domain)
    pub host: String,
    /// Root directory for this host
    pub root: PathBuf,
    /// Enable directory indexing
    pub enable_indexing: bool,
    /// Enable compression
    pub enable_compression: bool,
    /// Custom index files
    pub index_files: Vec<String>,
}

impl VHostConfig {
    /// Create a new virtual host config
    pub fn new(host: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            host: host.into(),
            root: root.into(),
            enable_indexing: true,
            enable_compression: true,
            index_files: vec!["index.html".to_string(), "index.htm".to_string()],
        }
    }

    /// Set enable indexing
    pub fn with_indexing(mut self, enabled: bool) -> Self {
        self.enable_indexing = enabled;
        self
    }

    /// Set enable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Set index files
    pub fn with_index_files(mut self, files: Vec<String>) -> Self {
        self.index_files = files;
        self
    }

    /// Check if this host matches the given hostname
    pub fn matches(&self, hostname: &str) -> bool {
        self.host.eq_ignore_ascii_case(hostname)
    }
}

impl Default for VHostConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            root: PathBuf::from("."),
            enable_indexing: true,
            enable_compression: true,
            index_files: vec!["index.html".to_string(), "index.htm".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhost_config_creation() {
        let config = VHostConfig::new("example.com", "/var/www/example");
        assert_eq!(config.host, "example.com");
        assert_eq!(config.root, PathBuf::from("/var/www/example"));
    }

    #[test]
    fn test_vhost_config_with_indexing() {
        let config = VHostConfig::new("example.com", "/var/www")
            .with_indexing(false);
        assert!(!config.enable_indexing);
    }

    #[test]
    fn test_vhost_config_with_compression() {
        let config = VHostConfig::new("example.com", "/var/www")
            .with_compression(false);
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_vhost_config_with_index_files() {
        let config = VHostConfig::new("example.com", "/var/www")
            .with_index_files(vec!["default.html".to_string()]);
        assert_eq!(config.index_files, vec!["default.html"]);
    }

    #[test]
    fn test_vhost_config_matches() {
        let config = VHostConfig::new("example.com", "/var/www");
        assert!(config.matches("example.com"));
        assert!(config.matches("EXAMPLE.COM"));
        assert!(!config.matches("other.com"));
    }

    #[test]
    fn test_vhost_config_default() {
        let config = VHostConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.root, PathBuf::from("."));
    }

    #[test]
    fn test_vhost_config_clone() {
        let config = VHostConfig::new("example.com", "/var/www");
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_vhost_config_default_index_files() {
        let config = VHostConfig::default();
        assert!(config.index_files.contains(&"index.html".to_string()));
        assert!(config.index_files.contains(&"index.htm".to_string()));
    }
}
