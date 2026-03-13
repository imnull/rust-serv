//! Proxy handler

use super::config::ProxyConfig;

/// Proxy handler manages multiple proxy configurations
#[derive(Debug, Clone)]
pub struct ProxyHandler {
    /// List of proxy configurations (in order of priority)
    configs: Vec<ProxyConfig>,
    /// Default timeout for all proxies
    default_timeout: std::time::Duration,
}

impl ProxyHandler {
    /// Create a new proxy handler
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            default_timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Add a proxy configuration
    pub fn add_proxy(&mut self, config: ProxyConfig) {
        self.configs.push(config);
    }

    /// Remove a proxy by path
    pub fn remove_proxy(&mut self, path: &str) -> bool {
        let len_before = self.configs.len();
        self.configs.retain(|c| c.path != path);
        self.configs.len() < len_before
    }

    /// Find matching proxy for a path
    pub fn find_match(&self, path: &str) -> Option<&ProxyConfig> {
        // Return first match (configs are checked in order)
        self.configs.iter().find(|c| c.matches(path))
    }

    /// Check if a path should be proxied
    pub fn should_proxy(&self, path: &str) -> bool {
        self.find_match(path).is_some()
    }

    /// Get target URL for a path
    pub fn get_target_url(&self, path: &str) -> Option<String> {
        self.find_match(path).map(|c| c.build_target_url(path))
    }

    /// Get proxy count
    pub fn proxy_count(&self) -> usize {
        self.configs.len()
    }

    /// Clear all proxies
    pub fn clear(&mut self) {
        self.configs.clear();
    }

    /// Get all paths
    pub fn paths(&self) -> Vec<String> {
        self.configs.iter().map(|c| c.path.clone()).collect()
    }

    /// Check if a proxy exists for a path
    pub fn has_proxy(&self, path: &str) -> bool {
        self.configs.iter().any(|c| c.path == path)
    }

    /// Set default timeout
    pub fn set_default_timeout(&mut self, timeout: std::time::Duration) {
        self.default_timeout = timeout;
    }

    /// Get default timeout
    pub fn default_timeout(&self) -> std::time::Duration {
        self.default_timeout
    }

    /// Get mutable configs
    pub fn configs_mut(&mut self) -> &mut Vec<ProxyConfig> {
        &mut self.configs
    }

    /// Get configs reference
    pub fn configs(&self) -> &[ProxyConfig] {
        &self.configs
    }
}

impl Default for ProxyHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = ProxyHandler::new();
        assert_eq!(handler.proxy_count(), 0);
    }

    #[test]
    fn test_add_proxy() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        assert_eq!(handler.proxy_count(), 1);
        assert!(handler.has_proxy("/api"));
    }

    #[test]
    fn test_remove_proxy() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        assert!(handler.remove_proxy("/api"));
        assert_eq!(handler.proxy_count(), 0);
        assert!(!handler.has_proxy("/api"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut handler = ProxyHandler::new();
        assert!(!handler.remove_proxy("/nonexistent"));
    }

    #[test]
    fn test_find_match() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        let config = handler.find_match("/api/users").unwrap();
        assert_eq!(config.path, "/api");
        assert_eq!(config.target, "http://localhost:3000");
    }

    #[test]
    fn test_find_match_no_match() {
        let handler = ProxyHandler::new();
        assert!(handler.find_match("/api").is_none());
    }

    #[test]
    fn test_should_proxy() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        assert!(handler.should_proxy("/api"));
        assert!(handler.should_proxy("/api/users"));
        assert!(!handler.should_proxy("/other"));
    }

    #[test]
    fn test_get_target_url() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        let url = handler.get_target_url("/api/users").unwrap();
        assert_eq!(url, "http://localhost:3000/users");
    }

    #[test]
    fn test_get_target_url_no_match() {
        let handler = ProxyHandler::new();
        assert!(handler.get_target_url("/api").is_none());
    }

    #[test]
    fn test_multiple_proxies() {
        let mut handler = ProxyHandler::new();
        
        handler.add_proxy(ProxyConfig::new("/api", "http://api:3000"));
        handler.add_proxy(ProxyConfig::new("/web", "http://web:8080"));
        handler.add_proxy(ProxyConfig::new("/admin", "http://admin:9000"));
        
        assert_eq!(handler.proxy_count(), 3);
        
        assert!(handler.should_proxy("/api"));
        assert!(handler.should_proxy("/web"));
        assert!(handler.should_proxy("/admin"));
        assert!(!handler.should_proxy("/other"));
    }

    #[test]
    fn test_proxy_priority() {
        let mut handler = ProxyHandler::new();
        
        // More specific path should be added first for priority
        handler.add_proxy(ProxyConfig::new("/api/v1", "http://api-v1:3000"));
        handler.add_proxy(ProxyConfig::new("/api", "http://api:3000"));
        
        // /api/v1/users should match /api/v1 (first)
        let config = handler.find_match("/api/v1/users").unwrap();
        assert_eq!(config.path, "/api/v1");
        
        // /api/other should match /api (second)
        let config = handler.find_match("/api/other").unwrap();
        assert_eq!(config.path, "/api");
    }

    #[test]
    fn test_clear() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        handler.add_proxy(ProxyConfig::new("/web", "http://localhost:8080"));
        
        handler.clear();
        
        assert_eq!(handler.proxy_count(), 0);
        assert!(!handler.should_proxy("/api"));
    }

    #[test]
    fn test_paths() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        handler.add_proxy(ProxyConfig::new("/web", "http://localhost:8080"));
        
        let mut paths = handler.paths();
        paths.sort();
        
        assert_eq!(paths, vec!["/api", "/web"]);
    }

    #[test]
    fn test_default_timeout() {
        let mut handler = ProxyHandler::new();
        assert_eq!(handler.default_timeout(), std::time::Duration::from_secs(30));
        
        handler.set_default_timeout(std::time::Duration::from_secs(60));
        assert_eq!(handler.default_timeout(), std::time::Duration::from_secs(60));
    }

    #[test]
    fn test_default() {
        let handler = ProxyHandler::default();
        assert_eq!(handler.proxy_count(), 0);
    }

    #[test]
    fn test_configs_access() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/api", "http://localhost:3000"));
        
        // Immutable access
        assert_eq!(handler.configs().len(), 1);
        
        // Mutable access
        handler.configs_mut().push(ProxyConfig::new("/web", "http://localhost:8080"));
        assert_eq!(handler.proxy_count(), 2);
    }

    #[test]
    fn test_empty_path() {
        let mut handler = ProxyHandler::new();
        handler.add_proxy(ProxyConfig::new("/", "http://localhost:3000"));
        
        assert!(handler.should_proxy("/"));
        assert!(handler.should_proxy("/anything"));
        
        let url = handler.get_target_url("/api").unwrap();
        assert_eq!(url, "http://localhost:3000/api");
    }
}
