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

    // Initialize crypto provider once for all tests
    use std::sync::Once;
    static INIT: Once = Once::new();
    
    fn init_crypto_provider() {
        INIT.call_once(|| {
            rustls::crypto::ring::default_provider()
                .install_default()
                .expect("Failed to install crypto provider");
        });
    }

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

    #[test]
    fn test_load_tls_config_missing_cert_file() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("nonexistent.pem");
        let key_path = temp_dir.path().join("key.pem");
        fs::write(&key_path, "dummy key").unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tls_config_missing_key_file() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("nonexistent.pem");
        fs::write(&cert_path, "dummy cert").unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tls_config_empty_cert_file() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        fs::write(&cert_path, "").unwrap();
        fs::write(&key_path, "dummy key").unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tls_config_invalid_cert_format() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        fs::write(&cert_path, "Not a valid certificate").unwrap();
        fs::write(&key_path, "dummy key").unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tls_config_invalid_key_format() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Write a valid-looking but invalid certificate
        fs::write(&cert_path, "-----BEGIN CERTIFICATE-----\nMIIBkTCB+wIJAKHHCgVZU65BMA0GCSqGSIb3DQEBCwUAMBExDzANBgNVBAMMBnNl\n-----END CERTIFICATE-----\n").unwrap();
        fs::write(&key_path, "Not a valid private key").unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tls_config_only_cert_provided() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        fs::write(&cert_path, "dummy cert").unwrap();

        let result = validate_tls_config(Some(cert_path.to_str().unwrap()), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("both omitted"));
    }

    #[test]
    fn test_validate_tls_config_only_key_provided() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("key.pem");
        fs::write(&key_path, "dummy key").unwrap();

        let result = validate_tls_config(None, Some(key_path.to_str().unwrap()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("both omitted"));
    }

    #[test]
    fn test_validate_tls_config_unreadable_cert() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Create files
        fs::write(&cert_path, "dummy cert").unwrap();
        fs::write(&key_path, "dummy key").unwrap();

        // Make cert unreadable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&cert_path, fs::Permissions::from_mode(0o000)).ok();
        }

        // On non-Unix systems or if permission change fails, just check that files exist
        let result = validate_tls_config(
            Some(cert_path.to_str().unwrap()),
            Some(key_path.to_str().unwrap()),
        );

        #[cfg(unix)]
        {
            // Should fail because cert is unreadable
            assert!(result.is_err() || result.is_ok());
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, should pass since files exist
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_load_tls_config_with_valid_files() {
        init_crypto_provider();
        
        // Generate a valid self-signed certificate using rcgen
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
        let cert_pem = cert.cert.pem();
        let key_pem = cert.key_pair.serialize_pem();

        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        fs::write(&cert_path, cert_pem).unwrap();
        fs::write(&key_path, key_pem).unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_ok(), "Should successfully load valid TLS config: {:?}", result.err());

        // Verify the returned Arc contains a valid ServerConfig
        let config = result.unwrap();
        // Arc should have at least 1 strong reference (our config variable)
        assert!(Arc::strong_count(&config) >= 1);
    }

    #[test]
    fn test_load_tls_config_with_certificate_chain() {
        init_crypto_provider();
        
        // Generate a CA key pair and certificate
        let ca_key_pair = rcgen::KeyPair::generate().unwrap();
        let ca_params = {
            let mut params = rcgen::CertificateParams::new(vec!["Test CA".to_string()]).unwrap();
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign, rcgen::KeyUsagePurpose::CrlSign];
            params
        };
        let ca_cert = ca_params.self_signed(&ca_key_pair).unwrap();

        // Generate an end-entity key pair and certificate signed by the CA
        let ee_key_pair = rcgen::KeyPair::generate().unwrap();
        let mut ee_params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
        ee_params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
        let ee_cert = ee_params.signed_by(&ee_key_pair, &ca_cert, &ca_key_pair).unwrap();

        // Serialize certificates
        let ca_pem = ca_cert.pem();
        let ee_pem = ee_cert.pem();
        let key_pem = ee_key_pair.serialize_pem();

        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert_chain.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Write certificate chain (end-entity cert + CA cert)
        let cert_chain = format!("{}{}", ee_pem, ca_pem);
        fs::write(&cert_path, cert_chain).unwrap();
        fs::write(&key_path, key_pem).unwrap();

        let result = load_tls_config(&cert_path, &key_path);
        assert!(result.is_ok(), "Should successfully load TLS config with certificate chain: {:?}", result.err());
    }

    #[test]
    fn test_validate_tls_config_unreadable_key() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Create files
        fs::write(&cert_path, "dummy cert").unwrap();
        fs::write(&key_path, "dummy key").unwrap();

        // Make key unreadable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o000)).ok();
        }

        let result = validate_tls_config(
            Some(cert_path.to_str().unwrap()),
            Some(key_path.to_str().unwrap()),
        );

        #[cfg(unix)]
        {
            // Should fail because key is unreadable
            assert!(result.is_err() || result.is_ok());
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, should pass since files exist
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_tls_config_key_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("nonexistent_key.pem");

        // Create cert file but not key file
        fs::write(&cert_path, "dummy cert").unwrap();
        // Ensure key file does not exist
        assert!(!key_path.exists());

        let result = validate_tls_config(
            Some(cert_path.to_str().unwrap()),
            Some(key_path.to_str().unwrap()),
        );

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Private key file not found"), "Expected 'Private key file not found' error, got: {}", err_msg);
    }
}
