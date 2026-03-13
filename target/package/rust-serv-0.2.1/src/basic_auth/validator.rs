//! Authentication validator

use std::collections::HashMap;

use super::credentials::Credentials;

/// Authentication validator
#[derive(Debug, Clone)]
pub struct AuthValidator {
    /// User database
    users: HashMap<String, String>,
    /// Protected paths
    protected_paths: Vec<String>,
}

impl AuthValidator {
    /// Create a new auth validator
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            protected_paths: Vec::new(),
        }
    }

    /// Add a user
    pub fn add_user(&mut self, username: impl Into<String>, password: impl Into<String>) {
        self.users.insert(username.into(), password.into());
    }

    /// Add a protected path
    pub fn add_protected_path(&mut self, path: impl Into<String>) {
        self.protected_paths.push(path.into());
    }

    /// Check if path requires authentication
    pub fn requires_auth(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }

    /// Validate credentials
    pub fn validate(&self, credentials: &Credentials) -> bool {
        if let Some(stored_password) = self.users.get(&credentials.username) {
            // Simple password comparison (in production, use secure comparison)
            stored_password == &credentials.password
        } else {
            false
        }
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Get protected paths count
    pub fn protected_path_count(&self) -> usize {
        self.protected_paths.len()
    }

    /// Clear all users
    pub fn clear_users(&mut self) {
        self.users.clear();
    }

    /// Clear all protected paths
    pub fn clear_paths(&mut self) {
        self.protected_paths.clear();
    }

    /// Check if user exists
    pub fn has_user(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }

    /// Remove a user
    pub fn remove_user(&mut self, username: &str) -> bool {
        self.users.remove(username).is_some()
    }
}

impl Default for AuthValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = AuthValidator::new();
        assert_eq!(validator.user_count(), 0);
        assert_eq!(validator.protected_path_count(), 0);
    }

    #[test]
    fn test_add_user() {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "secret");
        
        assert_eq!(validator.user_count(), 1);
        assert!(validator.has_user("admin"));
    }

    #[test]
    fn test_add_multiple_users() {
        let mut validator = AuthValidator::new();
        validator.add_user("user1", "pass1");
        validator.add_user("user2", "pass2");
        validator.add_user("user3", "pass3");
        
        assert_eq!(validator.user_count(), 3);
    }

    #[test]
    fn test_remove_user() {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "secret");
        
        assert!(validator.remove_user("admin"));
        assert_eq!(validator.user_count(), 0);
        assert!(!validator.has_user("admin"));
    }

    #[test]
    fn test_remove_nonexistent_user() {
        let mut validator = AuthValidator::new();
        assert!(!validator.remove_user("nonexistent"));
    }

    #[test]
    fn test_add_protected_path() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        
        assert_eq!(validator.protected_path_count(), 1);
    }

    #[test]
    fn test_requires_auth_exact() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        
        assert!(validator.requires_auth("/admin"));
    }

    #[test]
    fn test_requires_auth_prefix() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        
        assert!(validator.requires_auth("/admin/settings"));
        assert!(validator.requires_auth("/admin/users/list"));
    }

    #[test]
    fn test_requires_auth_no_match() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        
        assert!(!validator.requires_auth("/public"));
        assert!(!validator.requires_auth("/api/data"));
    }

    #[test]
    fn test_requires_auth_multiple_paths() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        validator.add_protected_path("/api/private");
        
        assert!(validator.requires_auth("/admin"));
        assert!(validator.requires_auth("/api/private/secret"));
        assert!(!validator.requires_auth("/api/public"));
    }

    #[test]
    fn test_validate_correct_credentials() {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "secret");
        
        let creds = Credentials::new("admin", "secret");
        assert!(validator.validate(&creds));
    }

    #[test]
    fn test_validate_wrong_password() {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "secret");
        
        let creds = Credentials::new("admin", "wrong");
        assert!(!validator.validate(&creds));
    }

    #[test]
    fn test_validate_nonexistent_user() {
        let validator = AuthValidator::new();
        
        let creds = Credentials::new("hacker", "password");
        assert!(!validator.validate(&creds));
    }

    #[test]
    fn test_validate_empty_password() {
        let mut validator = AuthValidator::new();
        validator.add_user("guest", "");
        
        let creds = Credentials::new("guest", "");
        assert!(validator.validate(&creds));
    }

    #[test]
    fn test_clear_users() {
        let mut validator = AuthValidator::new();
        validator.add_user("user1", "pass1");
        validator.add_user("user2", "pass2");
        
        validator.clear_users();
        assert_eq!(validator.user_count(), 0);
    }

    #[test]
    fn test_clear_paths() {
        let mut validator = AuthValidator::new();
        validator.add_protected_path("/admin");
        validator.add_protected_path("/api");
        
        validator.clear_paths();
        assert_eq!(validator.protected_path_count(), 0);
    }

    #[test]
    fn test_default() {
        let validator = AuthValidator::default();
        assert_eq!(validator.user_count(), 0);
    }

    #[test]
    fn test_overwrite_user() {
        let mut validator = AuthValidator::new();
        validator.add_user("admin", "old_pass");
        validator.add_user("admin", "new_pass");
        
        assert_eq!(validator.user_count(), 1);
        
        let old_creds = Credentials::new("admin", "old_pass");
        let new_creds = Credentials::new("admin", "new_pass");
        
        assert!(!validator.validate(&old_creds));
        assert!(validator.validate(&new_creds));
    }
}
