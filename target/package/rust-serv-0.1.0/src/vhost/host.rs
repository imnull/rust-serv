//! Virtual host representation

use std::path::PathBuf;

/// Virtual host
#[derive(Debug, Clone)]
pub struct VirtualHost {
    /// Host configuration
    pub config: super::config::VHostConfig,
    /// Whether this is the default host
    pub is_default: bool,
}

impl VirtualHost {
    /// Create a new virtual host
    pub fn new(config: super::config::VHostConfig) -> Self {
        Self {
            config,
            is_default: false,
        }
    }

    /// Create a default virtual host
    pub fn default_host(root: impl Into<PathBuf>) -> Self {
        let config = super::config::VHostConfig::new("_default_", root);
        Self {
            config,
            is_default: true,
        }
    }

    /// Set as default host
    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
    }

    /// Check if this host matches the given hostname
    pub fn matches(&self, hostname: &str) -> bool {
        if self.is_default {
            return true;
        }
        self.config.matches(hostname)
    }

    /// Get root directory
    pub fn root(&self) -> &PathBuf {
        &self.config.root
    }

    /// Get host name
    pub fn host(&self) -> &str {
        &self.config.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::VHostConfig;

    #[test]
    fn test_virtual_host_creation() {
        let config = VHostConfig::new("example.com", "/var/www");
        let vhost = VirtualHost::new(config);
        
        assert_eq!(vhost.host(), "example.com");
        assert_eq!(vhost.root(), &PathBuf::from("/var/www"));
        assert!(!vhost.is_default);
    }

    #[test]
    fn test_virtual_host_default() {
        let vhost = VirtualHost::default_host("/var/default");
        
        assert!(vhost.is_default);
        assert_eq!(vhost.host(), "_default_");
    }

    #[test]
    fn test_virtual_host_set_default() {
        let config = VHostConfig::new("example.com", "/var/www");
        let mut vhost = VirtualHost::new(config);
        
        vhost.set_default(true);
        assert!(vhost.is_default);
        
        vhost.set_default(false);
        assert!(!vhost.is_default);
    }

    #[test]
    fn test_virtual_host_matches() {
        let config = VHostConfig::new("example.com", "/var/www");
        let vhost = VirtualHost::new(config);
        
        assert!(vhost.matches("example.com"));
        assert!(vhost.matches("EXAMPLE.COM"));
        assert!(!vhost.matches("other.com"));
    }

    #[test]
    fn test_virtual_host_default_matches_all() {
        let vhost = VirtualHost::default_host("/var/default");
        
        // Default host should match everything
        assert!(vhost.matches("example.com"));
        assert!(vhost.matches("anything.org"));
        assert!(vhost.matches("localhost"));
    }

    #[test]
    fn test_virtual_host_root() {
        let config = VHostConfig::new("example.com", "/var/www/example");
        let vhost = VirtualHost::new(config);
        
        assert_eq!(vhost.root(), &PathBuf::from("/var/www/example"));
    }

    #[test]
    fn test_virtual_host_clone() {
        let config = VHostConfig::new("example.com", "/var/www");
        let vhost = VirtualHost::new(config);
        let cloned = vhost.clone();
        
        assert_eq!(vhost.host(), cloned.host());
        assert_eq!(vhost.root(), cloned.root());
    }
}
