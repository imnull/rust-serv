//! ACME account management
//!
//! This module handles ACME account creation and management.

use super::{AutoTlsError, AutoTlsResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ACME account information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcmeAccount {
    /// Account email
    pub email: String,
    /// Account private key (PEM format)
    pub private_key: String,
    /// Account URL (from ACME server)
    pub account_url: Option<String>,
}

impl AcmeAccount {
    /// Create a new ACME account
    pub fn new(email: String, private_key: String) -> Self {
        Self {
            email,
            private_key,
            account_url: None,
        }
    }

    /// Create an ACME account with URL
    pub fn with_url(email: String, private_key: String, account_url: String) -> Self {
        Self {
            email,
            private_key,
            account_url: Some(account_url),
        }
    }

    /// Save the account to a file
    pub fn save_to_file(&self, path: &Path) -> AutoTlsResult<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AutoTlsError::SerializationError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load an account from a file
    pub fn load_from_file(path: &Path) -> AutoTlsResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let account: AcmeAccount = serde_json::from_str(&content)
            .map_err(|e| AutoTlsError::SerializationError(e.to_string()))?;
        Ok(account)
    }

    /// Check if the account has a URL
    pub fn has_url(&self) -> bool {
        self.account_url.is_some()
    }

    /// Set the account URL
    pub fn set_url(&mut self, url: String) {
        self.account_url = Some(url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_acme_account_creation() {
        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.private_key, "private_key_pem");
        assert!(account.account_url.is_none());
        assert!(!account.has_url());
    }

    #[test]
    fn test_acme_account_with_url() {
        let account = AcmeAccount::with_url(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
            "https://acme.example.com/account/123".to_string(),
        );
        assert!(account.has_url());
        assert_eq!(
            account.account_url,
            Some("https://acme.example.com/account/123".to_string())
        );
    }

    #[test]
    fn test_acme_account_set_url() {
        let mut account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        assert!(!account.has_url());
        account.set_url("https://acme.example.com/account/456".to_string());
        assert!(account.has_url());
    }

    #[test]
    fn test_acme_account_serialization() {
        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("private_key_pem"));
    }

    #[test]
    fn test_acme_account_deserialization() {
        let json = r#"{"email":"test@example.com","private_key":"key","account_url":"https://example.com"}"#;
        let account: AcmeAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.private_key, "key");
        assert_eq!(account.account_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_acme_account_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("account.json");

        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );

        account.save_to_file(&path).unwrap();
        assert!(path.exists());

        let loaded = AcmeAccount::load_from_file(&path).unwrap();
        assert_eq!(account, loaded);
    }

    #[test]
    fn test_acme_account_clone() {
        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        let cloned = account.clone();
        assert_eq!(account, cloned);
    }

    #[test]
    fn test_acme_account_debug() {
        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        let debug_str = format!("{:?}", account);
        assert!(debug_str.contains("AcmeAccount"));
        assert!(debug_str.contains("test@example.com"));
    }

    #[test]
    fn test_acme_account_equality() {
        let account1 = AcmeAccount::new(
            "test@example.com".to_string(),
            "key".to_string(),
        );
        let account2 = AcmeAccount::new(
            "test@example.com".to_string(),
            "key".to_string(),
        );
        assert_eq!(account1, account2);
    }

    #[test]
    fn test_acme_account_load_nonexistent_file() {
        let result = AcmeAccount::load_from_file(Path::new("/nonexistent/path/account.json"));
        assert!(result.is_err());
    }
}
