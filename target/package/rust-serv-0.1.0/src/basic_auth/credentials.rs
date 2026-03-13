//! Credentials for Basic Auth

use base64::{engine::general_purpose::STANDARD, Engine};

/// Username and password credentials
#[derive(Debug, Clone, PartialEq)]
pub struct Credentials {
    /// Username
    pub username: String,
    /// Password (plain text or hash depending on context)
    pub password: String,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Parse from Basic Auth header value (base64 encoded)
    pub fn from_header(header_value: &str) -> Option<Self> {
        // Basic Auth format: "Basic <base64(username:password)>"
        let encoded = header_value.strip_prefix("Basic ")?;
        let decoded = STANDARD.decode(encoded).ok()?;
        let decoded_str = String::from_utf8(decoded).ok()?;
        
        let (username, password) = decoded_str.split_once(':')?;
        
        Some(Self {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    /// Convert to Basic Auth header value
    pub fn to_header(&self) -> String {
        let combined = format!("{}:{}", self.username, self.password);
        let encoded = STANDARD.encode(combined.as_bytes());
        format!("Basic {}", encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::new("admin", "password123");
        assert_eq!(creds.username, "admin");
        assert_eq!(creds.password, "password123");
    }

    #[test]
    fn test_credentials_to_header() {
        let creds = Credentials::new("admin", "secret");
        let header = creds.to_header();
        assert!(header.starts_with("Basic "));
        assert_eq!(header, "Basic YWRtaW46c2VjcmV0");
    }

    #[test]
    fn test_credentials_from_header() {
        let creds = Credentials::from_header("Basic YWRtaW46c2VjcmV0").unwrap();
        assert_eq!(creds.username, "admin");
        assert_eq!(creds.password, "secret");
    }

    #[test]
    fn test_credentials_from_header_invalid_prefix() {
        let result = Credentials::from_header("Bearer token");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_from_header_no_colon() {
        // Valid base64 but no colon separator
        let result = Credentials::from_header("Basic dXNlcm5hbWVvbmx5");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_roundtrip() {
        let original = Credentials::new("user@test.com", "p@ss:word");
        let header = original.to_header();
        let parsed = Credentials::from_header(&header).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_credentials_empty_username() {
        let creds = Credentials::new("", "password");
        let header = creds.to_header();
        let parsed = Credentials::from_header(&header).unwrap();
        assert_eq!(parsed.username, "");
        assert_eq!(parsed.password, "password");
    }

    #[test]
    fn test_credentials_empty_password() {
        let creds = Credentials::new("username", "");
        let header = creds.to_header();
        let parsed = Credentials::from_header(&header).unwrap();
        assert_eq!(parsed.username, "username");
        assert_eq!(parsed.password, "");
    }

    #[test]
    fn test_credentials_special_characters() {
        let creds = Credentials::new("用户名", "密码123");
        let header = creds.to_header();
        let parsed = Credentials::from_header(&header).unwrap();
        assert_eq!(parsed.username, "用户名");
        assert_eq!(parsed.password, "密码123");
    }

    #[test]
    fn test_credentials_from_header_missing_basic() {
        let result = Credentials::from_header("YWRtaW46c2VjcmV0");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_from_header_empty() {
        let result = Credentials::from_header("");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_from_header_invalid_base64() {
        let result = Credentials::from_header("Basic !!!invalid!!!");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_clone() {
        let creds = Credentials::new("admin", "secret");
        let cloned = creds.clone();
        assert_eq!(creds, cloned);
    }

    #[test]
    fn test_credentials_debug() {
        let creds = Credentials::new("admin", "secret");
        let debug = format!("{:?}", creds);
        assert!(debug.contains("admin"));
        assert!(debug.contains("secret"));
    }
}
