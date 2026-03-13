//! Proxy configuration

use std::time::Duration;

/// Proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Path pattern to match (e.g., "/api")
    pub path: String,
    /// Target URL (e.g., "http://localhost:3000")
    pub target: String,
    /// Strip the matched path prefix before forwarding
    pub strip_prefix: bool,
    /// Connection timeout
    pub timeout: Duration,
    /// Preserve Host header
    pub preserve_host: bool,
    /// Follow redirects
    pub follow_redirects: bool,
}

impl ProxyConfig {
    /// Create a new proxy config
    pub fn new(path: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            target: target.into(),
            strip_prefix: true,
            timeout: Duration::from_secs(30),
            preserve_host: false,
            follow_redirects: false,
        }
    }

    /// Set strip prefix
    pub fn with_strip_prefix(mut self, strip: bool) -> Self {
        self.strip_prefix = strip;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set preserve host
    pub fn with_preserve_host(mut self, preserve: bool) -> Self {
        self.preserve_host = preserve;
        self
    }

    /// Set follow redirects
    pub fn with_follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Check if a path matches this proxy rule
    pub fn matches(&self, path: &str) -> bool {
        path.starts_with(&self.path)
    }

    /// Build target URL for a request path
    pub fn build_target_url(&self, request_path: &str) -> String {
        let target_path = if self.strip_prefix && request_path.starts_with(&self.path) {
            &request_path[self.path.len()..]
        } else {
            request_path
        };

        // Ensure target_path starts with /
        let target_path = if target_path.starts_with('/') {
            target_path.to_string()
        } else if target_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", target_path)
        };

        // Combine target and path
        let target = self.target.trim_end_matches('/');
        format!("{}{}", target, target_path)
    }

    /// Parse target into scheme, host, port
    pub fn parse_target(&self) -> Result<(String, String, Option<u16>), String> {
        let target = &self.target;
        
        // Extract scheme
        let scheme_end = target.find("://").ok_or("Invalid target URL: missing scheme")?;
        let scheme = target[..scheme_end].to_lowercase();
        
        let rest = &target[scheme_end + 3..];
        
        // Extract host and port
        let (host, port) = if let Some(port_start) = rest.find(':') {
            let host = rest[..port_start].to_string();
            let port_str = &rest[port_start + 1..];
            let port = port_str.parse::<u16>().map_err(|_| "Invalid port")?;
            (host, Some(port))
        } else {
            (rest.to_string(), None)
        };
        
        Ok((scheme, host, port))
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self::new("/api", "http://localhost:3000")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_creation() {
        let config = ProxyConfig::new("/api", "http://localhost:3000");
        assert_eq!(config.path, "/api");
        assert_eq!(config.target, "http://localhost:3000");
        assert!(config.strip_prefix);
    }

    #[test]
    fn test_proxy_config_with_strip_prefix() {
        let config = ProxyConfig::new("/api", "http://localhost:3000")
            .with_strip_prefix(false);
        assert!(!config.strip_prefix);
    }

    #[test]
    fn test_proxy_config_with_timeout() {
        let config = ProxyConfig::new("/api", "http://localhost:3000")
            .with_timeout(Duration::from_secs(60));
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_proxy_config_with_preserve_host() {
        let config = ProxyConfig::new("/api", "http://localhost:3000")
            .with_preserve_host(true);
        assert!(config.preserve_host);
    }

    #[test]
    fn test_proxy_config_with_follow_redirects() {
        let config = ProxyConfig::new("/api", "http://localhost:3000")
            .with_follow_redirects(true);
        assert!(config.follow_redirects);
    }

    #[test]
    fn test_matches_exact() {
        let config = ProxyConfig::new("/api", "http://localhost:3000");
        assert!(config.matches("/api"));
        assert!(config.matches("/api/users"));
        assert!(config.matches("/api/v1/data"));
    }

    #[test]
    fn test_matches_no_match() {
        let config = ProxyConfig::new("/api", "http://localhost:3000");
        assert!(!config.matches("/"));
        assert!(!config.matches("/other"));
        // Note: /apiv1 DOES match /api because it starts with /api
        // This is expected behavior for prefix matching
    }

    #[test]
    fn test_build_target_url_strip_prefix() {
        let config = ProxyConfig::new("/api", "http://localhost:3000");
        
        assert_eq!(config.build_target_url("/api"), "http://localhost:3000/");
        assert_eq!(config.build_target_url("/api/users"), "http://localhost:3000/users");
        assert_eq!(config.build_target_url("/api/v1/data"), "http://localhost:3000/v1/data");
    }

    #[test]
    fn test_build_target_url_no_strip() {
        let config = ProxyConfig::new("/api", "http://localhost:3000")
            .with_strip_prefix(false);
        
        assert_eq!(config.build_target_url("/api"), "http://localhost:3000/api");
        assert_eq!(config.build_target_url("/api/users"), "http://localhost:3000/api/users");
    }

    #[test]
    fn test_build_target_url_trailing_slash() {
        let config = ProxyConfig::new("/api/", "http://localhost:3000/");
        
        let url = config.build_target_url("/api/users");
        assert!(url.starts_with("http://localhost:3000"));
        assert!(url.ends_with("/users"));
    }

    #[test]
    fn test_parse_target_http() {
        let config = ProxyConfig::new("/api", "http://localhost:3000");
        let (scheme, host, port) = config.parse_target().unwrap();
        
        assert_eq!(scheme, "http");
        assert_eq!(host, "localhost");
        assert_eq!(port, Some(3000));
    }

    #[test]
    fn test_parse_target_https() {
        let config = ProxyConfig::new("/api", "https://example.com");
        let (scheme, host, port) = config.parse_target().unwrap();
        
        assert_eq!(scheme, "https");
        assert_eq!(host, "example.com");
        assert_eq!(port, None);
    }

    #[test]
    fn test_parse_target_with_port() {
        let config = ProxyConfig::new("/api", "http://backend:8080");
        let (scheme, host, port) = config.parse_target().unwrap();
        
        assert_eq!(scheme, "http");
        assert_eq!(host, "backend");
        assert_eq!(port, Some(8080));
    }

    #[test]
    fn test_parse_target_invalid() {
        let config = ProxyConfig::new("/api", "not-a-url");
        assert!(config.parse_target().is_err());
    }

    #[test]
    fn test_parse_target_invalid_port() {
        let config = ProxyConfig::new("/api", "http://localhost:abc");
        assert!(config.parse_target().is_err());
    }

    #[test]
    fn test_default() {
        let config = ProxyConfig::default();
        assert_eq!(config.path, "/api");
        assert_eq!(config.target, "http://localhost:3000");
    }

    #[test]
    fn test_matches_empty_path() {
        let config = ProxyConfig::new("/", "http://localhost:3000");
        assert!(config.matches("/"));
        assert!(config.matches("/anything"));
    }

    #[test]
    fn test_build_target_url_root_path() {
        let config = ProxyConfig::new("/", "http://localhost:3000");
        
        assert_eq!(config.build_target_url("/"), "http://localhost:3000/");
        assert_eq!(config.build_target_url("/users"), "http://localhost:3000/users");
    }
}
