//! TLS/HTTPS support for the server
//!
//! This module provides TLS configuration and utilities for HTTPS connections.

use crate::error::{Error, Result};
use rustls::ServerConfig;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

/// Load TLS configuration from certificate and key files
pub fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<Arc<ServerConfig>> {
    // Load certificate chain
    let cert_file = File::open(cert_path)
        .map_err(|e| Error::Internal(format!("Failed to open certificate file: {}", e)))?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Vec<_> = certs(&mut cert_reader)
        .filter_map(|c| c.ok())
        .collect();

    if cert_chain.is_empty() {
        return Err(Error::Internal("No certificates found in certificate file".to_string()));
    }

    // Load private key
    let key_file = File::open(key_path)
        .map_err(|e| Error::Internal(format!("Failed to open private key file: {}", e)))?;
    let mut key_reader = BufReader::new(key_file);
    let private_key_result = private_key(&mut key_reader);

    let private_key = private_key_result
        .map_err(|e| Error::Internal(format!("Failed to read private key: {}", e)))?
        .ok_or_else(|| Error::Internal("No private key found in key file".to_string()))?;

    // Build server config
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .map_err(|e| Error::Internal(format!("Failed to build TLS config: {}", e)))?;

    Ok(Arc::new(config))
}

/// Validate TLS configuration paths
pub fn validate_tls_config(cert_path: Option<&str>, key_path: Option<&str>) -> Result<()> {
    match (cert_path, key_path) {
        (Some(cert), Some(key)) => {
            let cert_path = Path::new(cert);
            let key_path = Path::new(key);

            // Check if certificate file exists
            if !cert_path.exists() {
                return Err(Error::Internal(format!(
                    "Certificate file not found: {}",
                    cert_path.display()
                )));
            }

            // Check if key file exists
            if !key_path.exists() {
                return Err(Error::Internal(format!(
                    "Private key file not found: {}",
                    key_path.display()
                )));
            }

            // Check if both files are readable
            if let Err(e) = File::open(cert_path) {
                return Err(Error::Internal(format!(
                    "Cannot read certificate file: {}",
                    e
                )));
            }

            if let Err(e) = File::open(key_path) {
                return Err(Error::Internal(format!(
                    "Cannot read private key file: {}",
                    e
                )));
            }

            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(Error::Internal(
            "Both tls_cert and tls_key must be specified together or both omitted".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_tls_config_missing_both() {
        let result = validate_tls_config(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tls_config_missing_cert() {
        let result = validate_tls_config(Some("nonexistent.pem"), Some("key.pem"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tls_config_missing_key() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        fs::write(&cert_path, "dummy cert").unwrap();

        let result = validate_tls_config(Some(cert_path.to_str().unwrap()), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tls_config_both_provided() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        fs::write(&cert_path, "dummy cert").unwrap();
        fs::write(&key_path, "dummy key").unwrap();

        let result = validate_tls_config(
            Some(cert_path.to_str().unwrap()),
            Some(key_path.to_str().unwrap()),
        );
        // Files exist but are not valid TLS files
        // This should pass validation (only checks existence)
        assert!(result.is_ok());
    }
}
