//! Configuration hot reloader

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::config::Config;
use crate::config_reloader::diff::ConfigDiff;
use crate::config_reloader::watcher::{ConfigEvent, ConfigWatcher};

/// Configuration reload result
#[derive(Debug, Clone, PartialEq)]
pub enum ReloadResult {
    /// Configuration reloaded successfully
    Success(ConfigDiff),
    /// No changes detected
    NoChanges,
    /// Configuration file not found
    FileNotFound,
    /// Parse error
    ParseError(String),
    /// Reload requires restart
    RequiresRestart(ConfigDiff),
}

/// Configuration hot reloader
pub struct ConfigReloader {
    /// Current configuration
    config: Arc<RwLock<Config>>,
    /// Configuration file path
    config_path: PathBuf,
    /// File watcher
    watcher: Option<ConfigWatcher>,
    /// Auto-reload enabled
    auto_reload: bool,
    /// Debounce duration
    debounce_ms: u64,
}

impl ConfigReloader {
    /// Create a new config reloader
    pub fn new(config: Config, config_path: impl AsRef<Path>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: config_path.as_ref().to_path_buf(),
            watcher: None,
            auto_reload: false,
            debounce_ms: 500,
        }
    }
    
    /// Enable auto-reload with file watching
    pub fn enable_auto_reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut watcher = ConfigWatcher::new(&self.config_path)?;
        watcher.watch(&self.config_path)?;
        self.watcher = Some(watcher);
        self.auto_reload = true;
        Ok(())
    }
    
    /// Disable auto-reload
    pub fn disable_auto_reload(&mut self) {
        self.watcher = None;
        self.auto_reload = false;
    }
    
    /// Check if auto-reload is enabled
    pub fn is_auto_reload_enabled(&self) -> bool {
        self.auto_reload
    }
    
    /// Set debounce duration
    pub fn set_debounce_ms(&mut self, ms: u64) {
        self.debounce_ms = ms;
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> Config {
        self.config.read().unwrap().clone()
    }
    
    /// Manually reload configuration
    pub fn reload(&self) -> ReloadResult {
        // Check if file exists
        if !self.config_path.exists() {
            return ReloadResult::FileNotFound;
        }
        
        // Load new configuration
        let content = match std::fs::read_to_string(&self.config_path) {
            Ok(c) => c,
            Err(e) => return ReloadResult::ParseError(e.to_string()),
        };
        
        let new_config: Config = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => return ReloadResult::ParseError(e.to_string()),
        };
        
        // Get current config
        let current_config = self.get_config();
        
        // Compare configurations
        let diff = ConfigDiff::compare(&current_config, &new_config);
        
        if !diff.has_changes() {
            return ReloadResult::NoChanges;
        }
        
        // Check if restart is required
        if diff.requires_restart {
            return ReloadResult::RequiresRestart(diff);
        }
        
        // Update configuration
        if let Ok(mut config) = self.config.write() {
            *config = new_config;
        }
        
        ReloadResult::Success(diff)
    }
    
    /// Check for file changes and reload if necessary
    pub fn check_and_reload(&self) -> Option<ReloadResult> {
        if !self.auto_reload {
            return None;
        }
        
        if let Some(ref watcher) = self.watcher {
            if watcher.try_recv().is_some() {
                // Debounce
                std::thread::sleep(Duration::from_millis(self.debounce_ms));
                return Some(self.reload());
            }
        }
        
        None
    }
    
    /// Get configuration path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
    
    /// Get debounce duration
    pub fn debounce_ms(&self) -> u64 {
        self.debounce_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use tempfile::TempDir;
    
    fn create_test_config() -> Config {
        Config::default()
    }
    
    #[test]
    fn test_reloader_creation() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        assert_eq!(reloader.config_path(), config_path);
        assert!(!reloader.is_auto_reload_enabled());
    }
    
    #[test]
    fn test_reload_no_changes() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        let result = reloader.reload();
        assert!(matches!(result, ReloadResult::NoChanges));
    }
    
    #[test]
    fn test_reload_with_changes() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "log_level = \"debug\"").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        let result = reloader.reload();
        
        if let ReloadResult::Success(diff) = result {
            assert!(diff.has_changes());
            assert!(diff.field_changed("log_level"));
        } else {
            panic!("Expected Success result");
        }
    }
    
    #[test]
    fn test_reload_requires_restart() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 9090").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        let result = reloader.reload();
        
        if let ReloadResult::RequiresRestart(diff) = result {
            assert!(diff.has_changes());
            assert!(diff.field_changed("port"));
            assert!(diff.requires_restart);
        } else {
            panic!("Expected RequiresRestart result");
        }
    }
    
    #[test]
    fn test_reload_file_not_found() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("nonexistent.toml");
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        let result = reloader.reload();
        assert!(matches!(result, ReloadResult::FileNotFound));
    }
    
    #[test]
    fn test_reload_parse_error() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "invalid toml content {{{").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        let result = reloader.reload();
        assert!(matches!(result, ReloadResult::ParseError(_)));
    }
    
    #[test]
    fn test_get_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config.clone(), &config_path);
        
        let retrieved = reloader.get_config();
        assert_eq!(retrieved.port, config.port);
    }
    
    #[test]
    fn test_set_debounce() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "").unwrap();
        
        let config = create_test_config();
        let mut reloader = ConfigReloader::new(config, &config_path);
        
        reloader.set_debounce_ms(1000);
        assert_eq!(reloader.debounce_ms(), 1000);
    }

    #[test]
    fn test_enable_disable_auto_reload() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let config = create_test_config();
        let mut reloader = ConfigReloader::new(config, &config_path);
        
        // Auto-reload disabled by default
        assert!(!reloader.is_auto_reload_enabled());
        
        // Enable auto-reload
        assert!(reloader.enable_auto_reload().is_ok());
        assert!(reloader.is_auto_reload_enabled());
        
        // Disable auto-reload
        reloader.disable_auto_reload();
        assert!(!reloader.is_auto_reload_enabled());
    }

    #[test]
    fn test_check_and_reload_without_auto_reload() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let config = create_test_config();
        let reloader = ConfigReloader::new(config, &config_path);
        
        // Should return None when auto-reload is disabled
        let result = reloader.check_and_reload();
        assert!(result.is_none());
    }
}
