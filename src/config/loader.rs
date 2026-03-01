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
}
