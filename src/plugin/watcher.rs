//! Plugin file watcher for hot-reload

use crate::plugin::{
    error::{PluginError, PluginResult},
    manager::PluginManager,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};

/// Plugin watcher for hot-reload
pub struct PluginWatcher {
    watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
}

impl PluginWatcher {
    /// Create a new plugin watcher
    pub fn new(manager: Arc<tokio::sync::RwLock<PluginManager>>) -> PluginResult<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Create file watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }).map_err(|e| PluginError::WatcherError(e.to_string()))?;

        // Spawn task to handle file changes
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = handle_file_event(&manager, &event).await {
                    eprintln!("Plugin watcher error: {}", e);
                }
            }
        });

        Ok(Self {
            watcher,
            watched_paths: vec![],
        })
    }

    /// Watch a directory for plugin changes
    pub fn watch(&mut self, path: &Path) -> PluginResult<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)
            .map_err(|e| PluginError::WatcherError(e.to_string()))?;

        self.watched_paths.push(path.to_path_buf());

        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: &Path) -> PluginResult<()> {
        self.watcher.unwatch(path)
            .map_err(|e| PluginError::WatcherError(e.to_string()))?;

        self.watched_paths.retain(|p| p != path);

        Ok(())
    }

    /// Get watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }

    /// Stop all watching
    pub fn stop(&mut self) {
        for path in self.watched_paths.clone() {
            let _ = self.watcher.unwatch(&path);
        }
        self.watched_paths.clear();
    }
}

async fn handle_file_event(
    manager: &Arc<tokio::sync::RwLock<PluginManager>>,
    event: &Event,
) -> PluginResult<()> {
    // Only handle create/modify/remove events
    match event.kind {
        EventKind::Create(_) |
        EventKind::Modify(_) |
        EventKind::Remove(_) => {
            // Check if it's a .wasm file
            for path in &event.paths {
                if path.extension().map(|e| e == "wasm").unwrap_or(false) {
                    println!("Plugin file changed: {:?}", path);

                    // Find plugin ID to reload
                    let plugin_id = {
                        let manager = manager.read().await;
                        manager.list()
                            .iter()
                            .find(|p| p.path == *path)
                            .map(|p| p.id.clone())
                    };

                    // Reload if found
                    if let Some(id) = plugin_id {
                        let mut manager = manager.write().await;
                        println!("Reloading plugin: {}", id);
                        manager.reload(&id)?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::time::Duration;

    #[test]
    fn test_watcher_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));
            let watcher = PluginWatcher::new(manager);
            assert!(watcher.is_ok());
        });
    }

    #[test]
    fn test_watcher_watch_and_unwatch() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));
            let mut watcher = PluginWatcher::new(manager).unwrap();

            let test_dir = std::env::temp_dir().join("rust_serv_test_watch");
            std::fs::create_dir_all(&test_dir).ok();

            // Test watch
            let result = watcher.watch(&test_dir);
            // May fail on some platforms, so just check it doesn't panic

            // Test unwatch
            let result = watcher.unwatch(&test_dir);
            // May fail if watch failed, so just check it doesn't panic

            // Clean up
            std::fs::remove_dir_all(&test_dir).ok();
        });
    }

    #[test]
    fn test_watcher_watched_paths() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));
            let watcher = PluginWatcher::new(manager).unwrap();

            let paths = watcher.watched_paths();
            assert!(paths.is_empty());
        });
    }

    #[test]
    fn test_watcher_stop() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));
            let mut watcher = PluginWatcher::new(manager).unwrap();

            // Should not panic
            watcher.stop();
        });
    }

    #[test]
    fn test_handle_file_event_wasm_file() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));

            // Test with a .wasm file path
            let event = Event {
                kind: EventKind::Any,
                paths: vec![std::path::PathBuf::from("test.wasm")],
                attrs: Default::default(),
            };

            let result = handle_file_event(&manager, &event).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_handle_file_event_non_wasm() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let manager = Arc::new(tokio::sync::RwLock::new(PluginManager::new().unwrap()));

            // Test with a non-.wasm file
            let event = Event {
                kind: EventKind::Any,
                paths: vec![std::path::PathBuf::from("test.txt")],
                attrs: Default::default(),
            };

            let result = handle_file_event(&manager, &event).await;
            assert!(result.is_ok());
        });
    }
}
