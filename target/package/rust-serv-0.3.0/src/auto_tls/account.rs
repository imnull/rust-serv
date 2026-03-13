//! ACME account management
//!
//! This module handles ACME account creation and management.

use serde::{Deserialize, Serialize};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acme_account_creation() {
        let account = AcmeAccount::new(
            "test@example.com".to_string(),
            "private_key_pem".to_string(),
        );
        assert_eq!(account.email, "test@example.com");
        assert!(account.account_url.is_none());
    }
}
