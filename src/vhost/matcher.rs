//! Host matcher for virtual host routing

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::config::VHostConfig;
use super::host::VirtualHost;

/// Host matcher for routing to virtual hosts
pub struct HostMatcher {
    /// Virtual hosts by hostname
    hosts: RwLock<HashMap<String, Arc<VirtualHost>>>,
    /// Default virtual host
    default_host: RwLock<Option<Arc<VirtualHost>>>,
}

impl HostMatcher {
    /// Create a new host matcher
    pub fn new() -> Self {
        Self {
            hosts: RwLock::new(HashMap::new()),
            default_host: RwLock::new(None),
        }
    }

    /// Add a virtual host
    pub fn add_host(&self, config: VHostConfig) {
        let vhost = Arc::new(VirtualHost::new(config.clone()));
        let mut hosts = self.hosts.write().unwrap();
        hosts.insert(config.host.to_lowercase(), vhost);
    }

    /// Add a default virtual host
    pub fn set_default(&self, root: impl Into<std::path::PathBuf>) {
        let vhost = Arc::new(VirtualHost::default_host(root));
        let mut default = self.default_host.write().unwrap();
        *default = Some(vhost);
    }

    /// Match a hostname to a virtual host
    pub fn match_host(&self, hostname: &str) -> Option<Arc<VirtualHost>> {
        // First try exact match
        let hosts = self.hosts.read().unwrap();
        let hostname_lower = hostname.to_lowercase();
        
        if let Some(vhost) = hosts.get(&hostname_lower) {
            return Some(Arc::clone(vhost));
        }
        
        // Try default host
        let default = self.default_host.read().unwrap();
        if let Some(vhost) = default.as_ref() {
            return Some(Arc::clone(vhost));
        }
        
        None
    }

    /// Remove a virtual host
    pub fn remove_host(&self, hostname: &str) -> bool {
        let mut hosts = self.hosts.write().unwrap();
        hosts.remove(&hostname.to_lowercase()).is_some()
    }

    /// Get host count
    pub fn host_count(&self) -> usize {
        self.hosts.read().unwrap().len()
    }

    /// Check if a host exists
    pub fn has_host(&self, hostname: &str) -> bool {
        self.hosts.read().unwrap().contains_key(&hostname.to_lowercase())
    }

    /// Clear all hosts
    pub fn clear(&self) {
        let mut hosts = self.hosts.write().unwrap();
        hosts.clear();
        
        let mut default = self.default_host.write().unwrap();
        *default = None;
    }

    /// Check if default host exists
    pub fn has_default(&self) -> bool {
        self.default_host.read().unwrap().is_some()
    }

    /// Get all hostnames
    pub fn hostnames(&self) -> Vec<String> {
        self.hosts.read().unwrap().keys().cloned().collect()
    }
}

impl Default for HostMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_matcher_creation() {
        let matcher = HostMatcher::new();
        assert_eq!(matcher.host_count(), 0);
        assert!(!matcher.has_default());
    }

    #[test]
    fn test_add_host() {
        let matcher = HostMatcher::new();
        let config = VHostConfig::new("example.com", "/var/www");
        
        matcher.add_host(config);
        
        assert_eq!(matcher.host_count(), 1);
        assert!(matcher.has_host("example.com"));
        assert!(matcher.has_host("EXAMPLE.COM"));
    }

    #[test]
    fn test_match_host_exact() {
        let matcher = HostMatcher::new();
        let config = VHostConfig::new("example.com", "/var/www/example");
        matcher.add_host(config);
        
        let vhost = matcher.match_host("example.com").unwrap();
        assert_eq!(vhost.host(), "example.com");
        assert_eq!(vhost.root(), &PathBuf::from("/var/www/example"));
    }

    #[test]
    fn test_match_host_case_insensitive() {
        let matcher = HostMatcher::new();
        let config = VHostConfig::new("Example.Com", "/var/www");
        matcher.add_host(config);
        
        let vhost = matcher.match_host("EXAMPLE.COM").unwrap();
        assert_eq!(vhost.host(), "Example.Com");
    }

    #[test]
    fn test_match_host_default() {
        let matcher = HostMatcher::new();
        matcher.set_default("/var/default");
        
        // Should match any hostname
        let vhost = matcher.match_host("anything.com").unwrap();
        assert!(vhost.is_default);
    }

    #[test]
    fn test_match_host_no_match() {
        let matcher = HostMatcher::new();
        let config = VHostConfig::new("example.com", "/var/www");
        matcher.add_host(config);
        
        // No default, no match
        let result = matcher.match_host("other.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_host_prefer_exact_over_default() {
        let matcher = HostMatcher::new();
        
        // Add default first
        matcher.set_default("/var/default");
        
        // Add specific host
        let config = VHostConfig::new("example.com", "/var/www/example");
        matcher.add_host(config);
        
        // Should return specific host, not default
        let vhost = matcher.match_host("example.com").unwrap();
        assert_eq!(vhost.host(), "example.com");
        assert!(!vhost.is_default);
        
        // Should return default for other hosts
        let vhost = matcher.match_host("other.com").unwrap();
        assert!(vhost.is_default);
    }

    #[test]
    fn test_remove_host() {
        let matcher = HostMatcher::new();
        let config = VHostConfig::new("example.com", "/var/www");
        matcher.add_host(config);
        
        assert!(matcher.remove_host("example.com"));
        assert_eq!(matcher.host_count(), 0);
        assert!(!matcher.has_host("example.com"));
    }

    #[test]
    fn test_remove_nonexistent_host() {
        let matcher = HostMatcher::new();
        assert!(!matcher.remove_host("nonexistent.com"));
    }

    #[test]
    fn test_clear() {
        let matcher = HostMatcher::new();
        
        matcher.add_host(VHostConfig::new("example.com", "/var/www"));
        matcher.add_host(VHostConfig::new("other.com", "/var/other"));
        matcher.set_default("/var/default");
        
        matcher.clear();
        
        assert_eq!(matcher.host_count(), 0);
        assert!(!matcher.has_default());
    }

    #[test]
    fn test_hostnames() {
        let matcher = HostMatcher::new();
        
        matcher.add_host(VHostConfig::new("example.com", "/var/www"));
        matcher.add_host(VHostConfig::new("other.com", "/var/other"));
        
        let mut names = matcher.hostnames();
        names.sort();
        
        assert_eq!(names, vec!["example.com", "other.com"]);
    }

    #[test]
    fn test_multiple_hosts() {
        let matcher = HostMatcher::new();
        
        matcher.add_host(VHostConfig::new("blog.example.com", "/var/www/blog"));
        matcher.add_host(VHostConfig::new("api.example.com", "/var/www/api"));
        matcher.add_host(VHostConfig::new("www.example.com", "/var/www/main"));
        
        assert_eq!(matcher.host_count(), 3);
        
        let blog = matcher.match_host("blog.example.com").unwrap();
        assert_eq!(blog.root(), &PathBuf::from("/var/www/blog"));
        
        let api = matcher.match_host("api.example.com").unwrap();
        assert_eq!(api.root(), &PathBuf::from("/var/www/api"));
    }

    #[test]
    fn test_default() {
        let matcher = HostMatcher::default();
        assert_eq!(matcher.host_count(), 0);
    }

    #[test]
    fn test_overwrite_host() {
        let matcher = HostMatcher::new();
        
        matcher.add_host(VHostConfig::new("example.com", "/var/www/v1"));
        matcher.add_host(VHostConfig::new("example.com", "/var/www/v2"));
        
        // Should only have one host
        assert_eq!(matcher.host_count(), 1);
        
        // Should use the latest config
        let vhost = matcher.match_host("example.com").unwrap();
        assert_eq!(vhost.root(), &PathBuf::from("/var/www/v2"));
    }
}
