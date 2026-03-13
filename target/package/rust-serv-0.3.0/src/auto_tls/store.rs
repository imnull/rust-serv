//! Certificate storage

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use super::{AutoTlsError, AutoTlsResult};

/// Stored certificate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCertificate {
    /// Domain name
    pub domain: String,
    /// Certificate chain (PEM format)
    pub certificate: String,
    /// Private key (PEM format)
    pub private_key: String,
    /// When the certificate was issued (Unix timestamp)
    pub issued_at: u64,
    /// When the certificate expires (Unix timestamp)
    pub expires_at: u64,
}

impl StoredCertificate {
    /// Create a new stored certificate
    pub fn new(domain: String, certificate: String, private_key: String, expires_at: u64) -> Self {
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            domain,
            certificate,
            private_key,
            issued_at,
            expires_at,
        }
    }

    /// Check if certificate is expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    /// Get days until expiration
    pub fn days_until_expiry(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let secs_until = self.expires_at.saturating_sub(now);
        (secs_until / 86400) as i64
    }
}

/// Certificate store for persistence
#[derive(Debug, Clone)]
pub struct CertificateStore {
    cache_dir: PathBuf,
}

impl CertificateStore {
    /// Create a new certificate store
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    /// Get path for account credentials
    pub fn account_path(&self) -> PathBuf {
        self.cache_dir.join("account.json")
    }

    /// Get path for certificate file
    pub fn certificate_path(&self, domain: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", domain))
    }

    /// Get path for certificate PEM file
    pub fn cert_pem_path(&self, domain: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.crt", domain))
    }

    /// Get path for private key PEM file
    pub fn key_pem_path(&self, domain: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.key", domain))
    }

    /// Initialize storage directory
    pub async fn init(&self) -> AutoTlsResult<()> {
        fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to create cache dir: {}", e)))?;

        // Set restrictive permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            fs::set_permissions(&self.cache_dir, perms)
                .await
                .ok();
        }

        Ok(())
    }

    /// Save certificate to storage
    pub async fn save_certificate(
        &self,
        domain: &str,
        certificate: &str,
        private_key: &str,
    ) -> AutoTlsResult<()> {
        self.save_certificate_with_expiry(domain, certificate, private_key, 0).await
    }

    /// Save certificate with custom expiry time
    pub async fn save_certificate_with_expiry(
        &self,
        domain: &str,
        certificate: &str,
        private_key: &str,
        expires_at: u64,
    ) -> AutoTlsResult<()> {
        self.init().await?;

        // Use provided expiry or default to 90 days from now
        let expires_at = if expires_at == 0 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + (90 * 24 * 60 * 60)
        } else {
            expires_at
        };

        let stored = StoredCertificate::new(
            domain.to_string(),
            certificate.to_string(),
            private_key.to_string(),
            expires_at,
        );

        // Save JSON metadata
        let json_path = self.certificate_path(domain);
        let content = serde_json::to_string_pretty(&stored)?;
        fs::write(&json_path, content)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to write certificate: {}", e)))?;

        // Save certificate PEM
        let cert_path = self.cert_pem_path(domain);
        fs::write(&cert_path, certificate)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to write cert PEM: {}", e)))?;

        // Save private key PEM
        let key_path = self.key_pem_path(domain);
        fs::write(&key_path, private_key)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to write key PEM: {}", e)))?;

        // Set restrictive permissions on key file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&key_path, perms)
                .await
                .ok();
        }

        tracing::info!(
            "Saved certificate for {} to {}",
            domain,
            self.cache_dir.display()
        );

        Ok(())
    }

    /// Load certificate from storage
    pub async fn load_certificate(&self, domain: &str) -> AutoTlsResult<Option<StoredCertificate>> {
        let path = self.certificate_path(domain);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to read certificate: {}", e)))?;

        let cert: StoredCertificate = serde_json::from_str(&content)?;

        Ok(Some(cert))
    }

    /// Delete certificate from storage
    pub async fn delete_certificate(&self, domain: &str) -> AutoTlsResult<()> {
        let json_path = self.certificate_path(domain);
        let cert_path = self.cert_pem_path(domain);
        let key_path = self.key_pem_path(domain);

        if json_path.exists() {
            fs::remove_file(&json_path).await.ok();
        }
        if cert_path.exists() {
            fs::remove_file(&cert_path).await.ok();
        }
        if key_path.exists() {
            fs::remove_file(&key_path).await.ok();
        }

        Ok(())
    }

    /// List all stored certificates
    pub async fn list_certificates(&self) -> AutoTlsResult<Vec<String>> {
        if !self.cache_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| AutoTlsError::StoreError(format!("Failed to read cache dir: {}", e)))?;

        let mut domains = vec![];

        while let Some(entry) = entries.next_entry().await.ok().flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    if let Some(stem) = path.file_stem() {
                        if stem != "account" {
                            domains.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(domains)
    }

    /// Clean up expired certificates
    pub async fn cleanup_expired(&self) -> AutoTlsResult<Vec<String>> {
        let domains = self.list_certificates().await?;
        let mut cleaned = vec![];

        for domain in domains {
            if let Some(cert) = self.load_certificate(&domain).await? {
                if cert.is_expired() {
                    self.delete_certificate(&domain).await?;
                    cleaned.push(domain);
                }
            }
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_stored_certificate_creation() {
        let cert = StoredCertificate::new(
            "example.com".to_string(),
            "cert_pem".to_string(),
            "key_pem".to_string(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 86400 * 90,
        );

        assert_eq!(cert.domain, "example.com");
        assert!(!cert.is_expired());
        assert!(cert.days_until_expiry() > 80);
    }

    #[test]
    fn test_stored_certificate_expired() {
        let cert = StoredCertificate::new(
            "example.com".to_string(),
            "cert_pem".to_string(),
            "key_pem".to_string(),
            0, // Expired at Unix epoch
        );

        assert!(cert.is_expired());
        assert!(cert.days_until_expiry() <= 0);
    }

    #[test]
    fn test_stored_certificate_serialization() {
        let cert = StoredCertificate::new(
            "example.com".to_string(),
            "cert_pem".to_string(),
            "key_pem".to_string(),
            1234567890,
        );

        let json = serde_json::to_string(&cert).unwrap();
        let parsed: StoredCertificate = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.domain, cert.domain);
        assert_eq!(parsed.certificate, cert.certificate);
        assert_eq!(parsed.expires_at, cert.expires_at);
    }

    #[tokio::test]
    async fn test_certificate_store_paths() {
        let dir = tempdir().unwrap();
        let store = CertificateStore::new(dir.path());

        assert_eq!(store.account_path(), dir.path().join("account.json"));
        assert_eq!(
            store.certificate_path("example.com"),
            dir.path().join("example.com.json")
        );
        assert_eq!(
            store.cert_pem_path("example.com"),
            dir.path().join("example.com.crt")
        );
        assert_eq!(
            store.key_pem_path("example.com"),
            dir.path().join("example.com.key")
        );
    }

    #[tokio::test]
    async fn test_certificate_store_save_load() {
        let dir = tempdir().unwrap();
        let store = CertificateStore::new(dir.path());

        store.save_certificate("example.com", "cert", "key").await.unwrap();

        let loaded = store.load_certificate("example.com").await.unwrap();
        assert!(loaded.is_some());

        let cert = loaded.unwrap();
        assert_eq!(cert.domain, "example.com");
        assert_eq!(cert.certificate, "cert");
        assert_eq!(cert.private_key, "key");
    }

    #[tokio::test]
    async fn test_certificate_store_delete() {
        let dir = tempdir().unwrap();
        let store = CertificateStore::new(dir.path());

        store.save_certificate("example.com", "cert", "key").await.unwrap();
        store.delete_certificate("example.com").await.unwrap();

        let loaded = store.load_certificate("example.com").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_certificate_store_list() {
        let dir = tempdir().unwrap();
        let store = CertificateStore::new(dir.path());

        store.save_certificate("example.com", "cert1", "key1").await.unwrap();
        store.save_certificate("test.com", "cert2", "key2").await.unwrap();

        let domains = store.list_certificates().await.unwrap();
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"example.com".to_string()));
        assert!(domains.contains(&"test.com".to_string()));
    }

    #[tokio::test]
    async fn test_certificate_store_init() {
        let dir = tempdir().unwrap();
        let new_dir = dir.path().join("new_cache");
        let store = CertificateStore::new(&new_dir);

        assert!(!new_dir.exists());
        store.init().await.unwrap();
        assert!(new_dir.exists());
    }
}
