//! ACME Challenge handling

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Challenge token and key authorization
#[derive(Debug, Clone)]
pub struct ChallengeToken {
    /// Token for the challenge
    pub token: String,
    /// Key authorization (token + thumbprint)
    pub key_authorization: String,
}

/// HTTP-01 challenge handler
/// 
/// Stores challenge tokens that need to be served at:
/// GET /.well-known/acme-challenge/{token}
#[derive(Debug, Clone, Default)]
pub struct ChallengeHandler {
    /// Map of token -> key_authorization
    challenges: Arc<RwLock<HashMap<String, String>>>,
}

impl ChallengeHandler {
    /// Create a new challenge handler
    pub fn new() -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a challenge token
    pub async fn add_challenge(&self, token: String, key_authorization: String) {
        let mut challenges = self.challenges.write().await;
        challenges.insert(token, key_authorization);
    }

    /// Remove a challenge token
    pub async fn remove_challenge(&self, token: &str) {
        let mut challenges = self.challenges.write().await;
        challenges.remove(token);
    }

    /// Get key authorization for a token
    pub async fn get_key_authorization(&self, token: &str) -> Option<String> {
        let challenges = self.challenges.read().await;
        challenges.get(token).cloned()
    }

    /// Check if path is an ACME challenge path
    pub fn is_challenge_path(path: &str) -> bool {
        path.starts_with("/.well-known/acme-challenge/")
    }

    /// Extract token from challenge path
    /// Returns None if path is not a valid challenge path
    pub fn extract_token(path: &str) -> Option<&str> {
        path.strip_prefix("/.well-known/acme-challenge/")
    }

    /// Handle an ACME challenge request
    /// Returns the key authorization if found
    pub async fn handle_challenge(&self, path: &str) -> Option<String> {
        if !Self::is_challenge_path(path) {
            return None;
        }

        let token = Self::extract_token(path)?;
        self.get_key_authorization(token).await
    }

    /// Clear all challenges
    pub async fn clear(&self) {
        let mut challenges = self.challenges.write().await;
        challenges.clear();
    }

    /// Get number of active challenges
    pub async fn challenge_count(&self) -> usize {
        let challenges = self.challenges.read().await;
        challenges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_challenge_handler_basic() {
        let handler = ChallengeHandler::new();
        
        handler.add_challenge("token123".to_string(), "key_auth_456".to_string()).await;
        
        let result = handler.get_key_authorization("token123").await;
        assert_eq!(result, Some("key_auth_456".to_string()));
        
        handler.remove_challenge("token123").await;
        let result = handler.get_key_authorization("token123").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_is_challenge_path() {
        assert!(ChallengeHandler::is_challenge_path("/.well-known/acme-challenge/abc123"));
        assert!(!ChallengeHandler::is_challenge_path("/health"));
        assert!(!ChallengeHandler::is_challenge_path("/.well-known/acme-challenge"));
        assert!(!ChallengeHandler::is_challenge_path("/other/path"));
    }

    #[test]
    fn test_extract_token() {
        assert_eq!(
            ChallengeHandler::extract_token("/.well-known/acme-challenge/abc123"),
            Some("abc123")
        );
        assert_eq!(
            ChallengeHandler::extract_token("/.well-known/acme-challenge/"),
            Some("")
        );
        assert_eq!(
            ChallengeHandler::extract_token("/other/path"),
            None
        );
    }

    #[tokio::test]
    async fn test_handle_challenge() {
        let handler = ChallengeHandler::new();
        handler.add_challenge("abc123".to_string(), "xyz789".to_string()).await;

        let result = handler.handle_challenge("/.well-known/acme-challenge/abc123").await;
        assert_eq!(result, Some("xyz789".to_string()));

        let result = handler.handle_challenge("/.well-known/acme-challenge/notfound").await;
        assert_eq!(result, None);

        let result = handler.handle_challenge("/health").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_clear_challenges() {
        let handler = ChallengeHandler::new();
        handler.add_challenge("token1".to_string(), "auth1".to_string()).await;
        handler.add_challenge("token2".to_string(), "auth2".to_string()).await;
        
        assert_eq!(handler.challenge_count().await, 2);
        
        handler.clear().await;
        assert_eq!(handler.challenge_count().await, 0);
    }

    #[tokio::test]
    async fn test_challenge_handler_clone() {
        let handler1 = ChallengeHandler::new();
        handler1.add_challenge("token".to_string(), "auth".to_string()).await;
        
        let handler2 = handler1.clone();
        let result = handler2.get_key_authorization("token").await;
        assert_eq!(result, Some("auth".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let handler = Arc::new(ChallengeHandler::new());
        let mut handles = vec![];

        for i in 0..10 {
            let h = handler.clone();
            handles.push(tokio::spawn(async move {
                h.add_challenge(format!("token{}", i), format!("auth{}", i)).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(handler.challenge_count().await, 10);
    }
}
