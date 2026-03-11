//! Basic Authenticator

use super::credentials::Credentials;
use super::validator::AuthValidator;

/// Basic authenticator
pub struct BasicAuthenticator {
    validator: AuthValidator,
    realm: String,
}

impl BasicAuthenticator {
    /// Create a new basic authenticator
    pub fn new(validator: AuthValidator) -> Self {
        Self {
            validator,
            realm: "Protected Area".to_string(),
        }
    }

    /// Create a new basic authenticator with custom realm
    pub fn with_realm(validator: AuthValidator, realm: impl Into<String>) -> Self {
        Self {
            validator,
            realm: realm.into(),
        }
    }

    /// Get the realm
    pub fn realm(&self) -> &str {
        &self.realm
    }

    /// Check if path requires authentication
    pub fn requires_auth(&self, path: &str) -> bool {
        self.validator.requires_auth(path)
    }

    /// Authenticate a request using Basic Auth header
    pub fn authenticate(&self, auth_header: Option<&str>) -> AuthResult {
        match auth_header {
            Some(header) => {
                match Credentials::from_header(header) {
                    Some(creds) => {
                        if self.validator.validate(&creds) {
                            AuthResult::Success(creds.username)
                        } else {
                            AuthResult::InvalidCredentials
                        }
                    }
                    None => AuthResult::InvalidHeader,
                }
            }
            None => AuthResult::MissingHeader,
        }
    }

    /// Get the WWW-Authenticate header value for 401 responses
    pub fn www_authenticate_header(&self) -> String {
        format!("Basic realm=\"{}\"", self.realm)
    }

    /// Get the validator
    pub fn validator(&self) -> &AuthValidator {
        &self.validator
    }

    /// Get mutable validator
    pub fn validator_mut(&mut self) -> &mut AuthValidator {
        &mut self.validator
    }
}

/// Authentication result
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResult {
    /// Authentication successful, contains username
    Success(String),
    /// Missing Authorization header
    MissingHeader,
    /// Invalid Authorization header format
    InvalidHeader,
    /// Invalid credentials
    InvalidCredentials,
}

impl AuthResult {
    /// Check if authentication was successful
    pub fn is_success(&self) -> bool {
        matches!(self, AuthResult::Success(_))
    }

    /// Get username if successful
    pub fn username(&self) -> Option<&str> {
        match self {
            AuthResult::Success(username) => Some(username),
            _ => None,
        }
    }

    /// Get the HTTP status code for this result
    pub fn status_code(&self) -> u16 {
        match self {
            AuthResult::Success(_) => 200,
            _ => 401,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_authenticator() -> BasicAuthenticator {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "secret");
        validator.add_protected_path("/admin");
        BasicAuthenticator::new(validator)
    }

    #[test]
    fn test_authenticator_creation() {
        let auth = create_test_authenticator();
        assert_eq!(auth.realm(), "Protected Area");
    }

    #[test]
    fn test_authenticator_custom_realm() {
        let validator = AuthValidator::new();
        let auth = BasicAuthenticator::with_realm(validator, "My App");
        assert_eq!(auth.realm(), "My App");
    }

    #[test]
    fn test_requires_auth() {
        let auth = create_test_authenticator();
        
        assert!(auth.requires_auth("/admin"));
        assert!(auth.requires_auth("/admin/settings"));
        assert!(!auth.requires_auth("/public"));
    }

    #[test]
    fn test_authenticate_success() {
        let auth = create_test_authenticator();
        let header = "Basic YWRtaW46c2VjcmV0"; // admin:secret
        
        let result = auth.authenticate(Some(header));
        
        assert!(result.is_success());
        assert_eq!(result.username(), Some("admin"));
    }

    #[test]
    fn test_authenticate_wrong_password() {
        let auth = create_test_authenticator();
        let header = "Basic YWRtaW46d3Jvbmc="; // admin:wrong
        
        let result = auth.authenticate(Some(header));
        
        assert_eq!(result, AuthResult::InvalidCredentials);
        assert!(!result.is_success());
    }

    #[test]
    fn test_authenticate_wrong_user() {
        let auth = create_test_authenticator();
        let header = "Basic dXNlcjpwYXNz"; // user:pass
        
        let result = auth.authenticate(Some(header));
        
        assert_eq!(result, AuthResult::InvalidCredentials);
    }

    #[test]
    fn test_authenticate_missing_header() {
        let auth = create_test_authenticator();
        
        let result = auth.authenticate(None);
        
        assert_eq!(result, AuthResult::MissingHeader);
    }

    #[test]
    fn test_authenticate_invalid_header_format() {
        let auth = create_test_authenticator();
        
        let result = auth.authenticate(Some("Bearer token"));
        assert_eq!(result, AuthResult::InvalidHeader);
        
        let result = auth.authenticate(Some("Basic invalid!!!"));
        assert_eq!(result, AuthResult::InvalidHeader);
    }

    #[test]
    fn test_www_authenticate_header() {
        let auth = create_test_authenticator();
        let header = auth.www_authenticate_header();
        
        assert_eq!(header, "Basic realm=\"Protected Area\"");
    }

    #[test]
    fn test_www_authenticate_custom_realm() {
        let validator = AuthValidator::new();
        let auth = BasicAuthenticator::with_realm(validator, "My App");
        let header = auth.www_authenticate_header();
        
        assert_eq!(header, "Basic realm=\"My App\"");
    }

    #[test]
    fn test_auth_result_is_success() {
        let success = AuthResult::Success("user".to_string());
        assert!(success.is_success());
        
        let missing = AuthResult::MissingHeader;
        assert!(!missing.is_success());
        
        let invalid_header = AuthResult::InvalidHeader;
        assert!(!invalid_header.is_success());
        
        let invalid_creds = AuthResult::InvalidCredentials;
        assert!(!invalid_creds.is_success());
    }

    #[test]
    fn test_auth_result_username() {
        let success = AuthResult::Success("admin".to_string());
        assert_eq!(success.username(), Some("admin"));
        
        let missing = AuthResult::MissingHeader;
        assert!(missing.username().is_none());
    }

    #[test]
    fn test_auth_result_status_code() {
        let success = AuthResult::Success("user".to_string());
        assert_eq!(success.status_code(), 200);
        
        let missing = AuthResult::MissingHeader;
        assert_eq!(missing.status_code(), 401);
        
        let invalid = AuthResult::InvalidCredentials;
        assert_eq!(invalid.status_code(), 401);
    }

    #[test]
    fn test_validator_access() {
        let mut auth = create_test_authenticator();
        
        // Read access
        assert_eq!(auth.validator().user_count(), 1);
        
        // Mutable access
        auth.validator_mut().add_user("newuser", "pass");
        assert_eq!(auth.validator().user_count(), 2);
    }

    #[test]
    fn test_authenticate_empty_password() {
        let mut validator = AuthValidator::new();
        validator.add_user("guest", "");
        let auth = BasicAuthenticator::new(validator);
        
        let header = "Basic Z3Vlc3Q6"; // guest:
        let result = auth.authenticate(Some(header));
        
        assert!(result.is_success());
    }

    #[test]
    fn test_authenticate_empty_username() {
        let mut validator = AuthValidator::new();
        validator.add_user("", "password");
        let auth = BasicAuthenticator::new(validator);
        
        let header = "Basic OnBhc3N3b3Jk"; // :password
        let result = auth.authenticate(Some(header));
        
        assert!(result.is_success());
    }
}
