//! Configuration file watcher

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

/// Watches configuration files for changes
pub struct ConfigWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<ConfigEvent>,
}

/// Configuration change events
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    /// Configuration file changed
    Changed(String),
    /// Configuration file removed
    Removed(String),
    /// Watcher error
    Error(String),
}

impl ConfigWatcher {
    /// Create a new config watcher for a file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        
        let path = path.as_ref().to_path_buf();
        
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    for path in event.paths {
                        if let Some(path_str) = path.to_str() {
                            let event_type = match event.kind {
                                notify::EventKind::Modify(_) => ConfigEvent::Changed(path_str.to_string()),
                                notify::EventKind::Remove(_) => ConfigEvent::Removed(path_str.to_string()),
                                _ => continue,
                            };
                            let _ = tx.send(event_type);
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(ConfigEvent::Error(e.to_string()));
                }
            }
        })?;
        
        Ok(Self { watcher, rx })
    }
    
    /// Start watching a path
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        Ok(())
    }
    
    /// Stop watching a path
    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.watcher.unwatch(path.as_ref())?;
        Ok(())
    }
    
    /// Check for events (non-blocking)
    pub fn try_recv(&self) -> Option<ConfigEvent> {
        match self.rx.try_recv() {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
    
    /// Wait for next event (blocking)
    pub fn recv(&self) -> Option<ConfigEvent> {
        match self.rx.recv() {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_watcher_creation() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let result = ConfigWatcher::new(&config_path);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_watcher_watch_unwatch() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let mut watcher = ConfigWatcher::new(&config_path).unwrap();
        
        // Should be able to watch the file
        assert!(watcher.watch(&config_path).is_ok());
        
        // Should be able to unwatch
        assert!(watcher.unwatch(&config_path).is_ok());
    }

    #[test]
    fn test_watcher_try_recv_no_event() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let mut watcher = ConfigWatcher::new(&config_path).unwrap();
        watcher.watch(&config_path).unwrap();
        
        // No events yet
        let result = watcher.try_recv();
        assert!(result.is_none());
    }

    #[test]
    fn test_watcher_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("nonexistent.toml");
        
        // Should still be able to create watcher even if file doesn't exist
        let result = ConfigWatcher::new(&config_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_watcher_recv_timeout() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "port = 8080").unwrap();
        
        let mut watcher = ConfigWatcher::new(&config_path).unwrap();
        watcher.watch(&config_path).unwrap();
        
        // Use try_recv which is non-blocking
        let result = watcher.try_recv();
        // Initially no events
        assert!(result.is_none());
    }

    #[test] 
    fn test_config_event_debug() {
        let event = ConfigEvent::Changed("/test/path.toml".to_string());
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Changed"));
        
        let event = ConfigEvent::Removed("/test/path.toml".to_string());
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Removed"));
        
        let event = ConfigEvent::Error("test error".to_string());
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Error"));
    }
}
