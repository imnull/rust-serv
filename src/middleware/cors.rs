//! CORS (Cross-Origin Resource Sharing) Middleware
//!
//! This module implements CORS support:
//! - Origin header validation
//! - Preflight request handling
//! - CORS headers configuration
//! - Credentials management
//! - Request method validation

use hyper::{HeaderMap, Method};
use hyper::header::{HeaderValue, ORIGIN, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_METHOD, ACCESS_CONTROL_REQUEST_HEADERS};
use std::sync::Arc;

/// CORS configuration options
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// List of allowed origins ( "*" for all origins)
    pub allowed_origins: Vec<String>,

    /// Allowed HTTP methods
    pub allowed_methods: Vec<Method>,

    /// Allowed request headers
    pub allowed_headers: Vec<String>,

    /// Exposed response headers
    pub exposed_headers: Vec<String>,

    /// Whether credentials are allowed
    pub allow_credentials: bool,

    /// Maximum age for preflight results (seconds)
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: None,
        }
    }
}

/// CORS layer that handles cross-origin requests
pub struct CorsLayer {
    config: Arc<CorsConfig>,
}

impl CorsLayer {
    /// Create a new CORS layer
    pub fn new(config: CorsConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Check if origin is allowed
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allowed_origins.is_empty() {
            return true;
        }

        self.config.allowed_origins.iter().any(|allowed| {
            allowed == "*" || origin == allowed
        })
    }

    /// Check if method is allowed
    fn is_method_allowed(&self, method: &Method) -> bool {
        self.config.allowed_methods.contains(method)
    }

    /// Check if header is allowed
    fn is_header_allowed(&self, header_name: &str) -> bool {
        if self.config.allowed_headers.is_empty() {
            return true;
        }

        self.config.allowed_headers.iter().any(|allowed| {
            header_name == allowed
        })
    }

    /// Add CORS headers to response
    pub fn add_cors_headers(&self, response_headers: &mut HeaderMap, origin: Option<&str>) {
        // Allow-Origin
        let allow_origin_value = if self.config.allowed_origins.contains(&"*".to_string()) {
            HeaderValue::from_static("*")
        } else if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*"))
            } else {
                return; // Invalid origin, don't add headers
            }
        } else {
            return; // No origin provided, don't add headers
        };

        response_headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, allow_origin_value);

        // Allow-Methods
        let methods_value = self.config.allowed_methods
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        response_headers.insert(ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_bytes(methods_value.as_bytes()).unwrap());

        // Allow-Headers
        if !self.config.allowed_headers.is_empty() {
            let headers_value = self.config.allowed_headers.join(", ");
            response_headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_bytes(headers_value.as_bytes()).unwrap());
        }

        // Allow-Credentials
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
            if self.config.allow_credentials {
                HeaderValue::from_static("true")
            } else {
                HeaderValue::from_static("false")
            }
        );

        // Expose-Headers
        if !self.config.exposed_headers.is_empty() {
            let exposed_value = self.config.exposed_headers.join(", ");
            response_headers.insert(ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_bytes(exposed_value.as_bytes()).unwrap());
        }

        // Max-Age
        if let Some(max_age) = self.config.max_age {
            response_headers.insert(ACCESS_CONTROL_MAX_AGE, HeaderValue::from_bytes(max_age.to_string().as_bytes()).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CorsConfig::default();

        assert_eq!(config.allowed_origins.len(), 0);
        assert_eq!(config.allowed_methods.len(), 0);
        assert_eq!(config.allowed_headers.len(), 0);
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age, None);
    }

    #[test]
    fn test_allow_all_origins() {
        let config = CorsConfig {
            allowed_origins: vec![],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://any-origin.com"));
        assert!(cors.is_origin_allowed("https://localhost:8080"));
    }

    #[test]
    fn test_restrict_origins() {
        let config = CorsConfig {
            allowed_origins: vec!["https://trusted.com".to_string()],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(cors.is_origin_allowed("https://trusted.com"));
        assert!(!cors.is_origin_allowed("https://untrusted.com"));
        assert!(!cors.is_origin_allowed("https://malicious.com"));
    }

    #[test]
    fn test_method_validation() {
        let config = CorsConfig {
            allowed_methods: vec![Method::GET, Method::POST],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(cors.is_method_allowed(&Method::GET));
        assert!(cors.is_method_allowed(&Method::POST));
        assert!(!cors.is_method_allowed(&Method::DELETE));
        assert!(!cors.is_method_allowed(&Method::PUT));
    }

    #[test]
    fn test_header_validation() {
        let config = CorsConfig {
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(cors.is_header_allowed("Content-Type"));
        assert!(cors.is_header_allowed("Authorization"));
        assert!(!cors.is_header_allowed("X-Custom-Header"));
    }

    #[test]
    fn test_credentials_config() {
        let config_with_creds = CorsConfig {
            allow_credentials: true,
            ..Default::default()
        };
        let cors = CorsLayer::new(config_with_creds);

        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert_eq!(
            response_headers.get(ACCESS_CONTROL_ALLOW_CREDENTIALS).unwrap(),
            HeaderValue::from_static("true")
        );

        let config_without_creds = CorsConfig::default();
        let cors = CorsLayer::new(config_without_creds);
        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert_eq!(
            response_headers.get(ACCESS_CONTROL_ALLOW_CREDENTIALS).unwrap(),
            HeaderValue::from_static("false")
        );
    }

    #[test]
    fn test_max_age_config() {
        let config = CorsConfig {
            max_age: Some(3600), // 1 hour
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert_eq!(
            response_headers.get(ACCESS_CONTROL_MAX_AGE).unwrap(),
            HeaderValue::from_static("3600")
        );

        let config_no_max_age = CorsConfig::default();
        let cors = CorsLayer::new(config_no_max_age);
        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert!(response_headers.get(ACCESS_CONTROL_MAX_AGE).is_none());
    }

    #[test]
    fn test_exposed_headers_config() {
        let config = CorsConfig {
            exposed_headers: vec!["X-Custom-Header".to_string(), "X-Another-Header".to_string()],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert_eq!(
            response_headers.get(ACCESS_CONTROL_EXPOSE_HEADERS).unwrap(),
            HeaderValue::from_bytes("X-Custom-Header, X-Another-Header".as_bytes()).unwrap()
        );
    }

    #[test]
    fn test_preflight_invalid_method() {
        let config = CorsConfig {
            allowed_methods: vec![Method::GET],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(!cors.is_method_allowed(&Method::DELETE));
    }

    #[test]
    fn test_preflight_invalid_header() {
        let config = CorsConfig {
            allowed_headers: vec!["Content-Type".to_string()],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(!cors.is_header_allowed("X-Forbidden-Header"));
    }

    #[test]
    fn test_cors_headers_addition() {
        let config = CorsConfig::default();
        let cors = CorsLayer::new(config);

        let mut response_headers = HeaderMap::new();
        cors.add_cors_headers(&mut response_headers, Some("https://example.com"));

        assert!(response_headers.contains_key(ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(response_headers.contains_key(ACCESS_CONTROL_ALLOW_METHODS));
        assert!(response_headers.contains_key(ACCESS_CONTROL_ALLOW_CREDENTIALS));
    }

    #[test]
    fn test_wildcard_origin() {
        let config = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            ..Default::default()
        };
        let cors = CorsLayer::new(config);

        assert!(cors.is_origin_allowed("https://any-origin.com"));
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://localhost:8080"));
    }
}