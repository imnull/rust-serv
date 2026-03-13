use crate::config::Config;
use crate::error::Result;
use std::fs;
use std::path::Path;

/// Load configuration from a TOML file
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)
        .map_err(|e| crate::error::Error::Config(format!("Failed to parse config: {}", e)))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
            port = 9000
            root = "/test"
        "#).unwrap();
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn test_load_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid = [").unwrap();
        assert!(load_config(file.path()).is_err());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_config("/nonexistent/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_empty_config() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "").unwrap();
        let config = load_config(file.path()).unwrap();
        // Should use default values
        assert_eq!(config.port, 8080); // default port
    }

    #[test]
    fn test_load_partial_config() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
            port = 3000
            enable_indexing = false
        "#).unwrap();
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.port, 3000);
        assert_eq!(config.enable_indexing, false);
    }

    #[test]
    fn test_load_config_with_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "this is not valid toml [[[").unwrap();
        let result = load_config(file.path());
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse config"));
    }

    #[test]
    fn test_load_config_with_all_fields() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
            port = 9000
            root = "/var/www"
            enable_indexing = true
            enable_compression = true
            log_level = "debug"
            enable_tls = false
            connection_timeout_secs = 60
            max_connections = 500
            enable_health_check = true
        "#).unwrap();
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.root, std::path::PathBuf::from("/var/www"));
        assert_eq!(config.enable_indexing, true);
        assert_eq!(config.enable_compression, true);
        assert_eq!(config.log_level, "debug");
    }
}
