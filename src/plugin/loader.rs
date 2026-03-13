//! Plugin loader for WebAssembly modules

use crate::plugin::{
    error::{PluginError, PluginResult},
    traits::PluginMetadata,
};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use wasmtime::*;

/// Compiled Wasm module cache
pub struct ModuleCache {
    modules: HashMap<PathBuf, Module>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
    
    pub fn get(&self, path: &Path) -> Option<&Module> {
        self.modules.get(path)
    }
    
    pub fn insert(&mut self, path: PathBuf, module: Module) {
        self.modules.insert(path, module);
    }
    
    pub fn remove(&mut self, path: &Path) {
        self.modules.remove(path);
    }
    
    pub fn clear(&mut self) {
        self.modules.clear();
    }
    
    pub fn len(&self) -> usize {
        self.modules.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

/// Plugin loader for Wasm modules
pub struct PluginLoader {
    engine: Engine,
    cache: ModuleCache,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> PluginResult<Self> {
        let mut config = Config::new();
        config
            .cranelift_opt_level(OptLevel::Speed)
            .wasm_bulk_memory(true)
            .wasm_reference_types(true);
        
        let engine = Engine::new(&config)
            .map_err(|e| PluginError::WasmCompilation(e.to_string()))?;
        
        Ok(Self {
            engine,
            cache: ModuleCache::new(),
        })
    }
    
    /// Compile a Wasm module from file
    pub fn compile(&mut self, path: &Path) -> PluginResult<Module> {
        // Check cache first
        if let Some(module) = self.cache.get(path) {
            return Ok(module.clone());
        }
        
        // Compile module
        let module = Module::from_file(&self.engine, path)
            .map_err(|e| PluginError::WasmCompilation(e.to_string()))?;
        
        // Cache it
        self.cache.insert(path.to_path_buf(), module.clone());
        
        Ok(module)
    }
    
    /// Compile from bytes
    pub fn compile_bytes(&mut self, bytes: &[u8], key: PathBuf) -> PluginResult<Module> {
        let module = Module::from_binary(&self.engine, bytes)
            .map_err(|e| PluginError::WasmCompilation(e.to_string()))?;
        
        self.cache.insert(key, module.clone());
        
        Ok(module)
    }
    
    /// Extract metadata from Wasm module
    pub fn extract_metadata(&self, module: &Module) -> PluginResult<PluginMetadata> {
        // Return default metadata
        // TODO: Implement metadata extraction from custom sections
        Ok(PluginMetadata {
            id: "unknown".to_string(),
            name: "Unknown Plugin".to_string(),
            version: "0.0.0".to_string(),
            description: "Plugin without metadata".to_string(),
            author: "Unknown".to_string(),
            homepage: None,
            license: "MIT".to_string(),
            min_server_version: "0.1.0".to_string(),
            priority: 100,
            capabilities: vec![],
            permissions: vec![],
        })
    }
    
    /// Get engine reference
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
    
    /// Get cache reference
    pub fn cache(&self) -> &ModuleCache {
        &self.cache
    }
    
    /// Get mutable cache reference
    pub fn cache_mut(&mut self) -> &mut ModuleCache {
        &mut self.cache
    }
    
    /// Clear module cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create PluginLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_module_cache() {
        let mut cache = ModuleCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();

        cache.insert(PathBuf::from("/test.wasm"), module);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_module_cache_get() {
        let mut cache = ModuleCache::new();
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();

        let path = PathBuf::from("/test.wasm");
        cache.insert(path.clone(), module);

        assert!(cache.get(&path).is_some());
        assert!(cache.get(&PathBuf::from("/nonexistent.wasm")).is_none());
    }

    #[test]
    fn test_module_cache_remove() {
        let mut cache = ModuleCache::new();
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();

        let path = PathBuf::from("/test.wasm");
        cache.insert(path.clone(), module);
        assert_eq!(cache.len(), 1);

        cache.remove(&path);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_loader_engine() {
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        // Just verify we can get the engine
        let _ = engine.config().clone();
    }

    #[test]
    fn test_loader_cache() {
        let loader = PluginLoader::new().unwrap();
        let cache = loader.cache();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_loader_cache_mut() {
        let mut loader = PluginLoader::new().unwrap();
        let cache = loader.cache_mut();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_loader_clear_cache() {
        let mut loader = PluginLoader::new().unwrap();
        let engine = loader.engine().clone();
        let module = Module::new(&engine, "(module)").unwrap();
        
        loader.cache_mut().insert(PathBuf::from("/test.wasm"), module);
        assert_eq!(loader.cache_size(), 1);

        loader.clear_cache();
        assert_eq!(loader.cache_size(), 0);
    }

    #[test]
    fn test_loader_cache_size() {
        let mut loader = PluginLoader::new().unwrap();
        assert_eq!(loader.cache_size(), 0);

        let engine = loader.engine().clone();
        let module = Module::new(&engine, "(module)").unwrap();
        loader.cache_mut().insert(PathBuf::from("/test.wasm"), module);

        assert_eq!(loader.cache_size(), 1);
    }

    #[test]
    fn test_extract_metadata() {
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();

        let metadata = loader.extract_metadata(&module).unwrap();
        assert_eq!(metadata.id, "unknown");
        assert_eq!(metadata.name, "Unknown Plugin");
    }

    #[test]
    fn test_loader_default() {
        let loader = PluginLoader::default();
        // Should not panic
        let _ = loader.engine();
    }

    #[test]
    fn test_compile_bytes() {
        let mut loader = PluginLoader::new().unwrap();
        
        // Valid Wasm bytes (minimal module: magic + version)
        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6D, // magic
            0x01, 0x00, 0x00, 0x00, // version
        ];
        let key = PathBuf::from("/memory.wasm");
        
        let result = loader.compile_bytes(&wasm_bytes, key);
        assert!(result.is_ok());
        assert_eq!(loader.cache_size(), 1);
    }

    #[test]
    fn test_compile_bytes_invalid() {
        let mut loader = PluginLoader::new().unwrap();
        
        // Invalid Wasm bytes
        let invalid_bytes = vec![0x00, 0x01, 0x02, 0x03];
        let key = PathBuf::from("/invalid.wasm");
        
        let result = loader.compile_bytes(&invalid_bytes, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_nonexistent_file() {
        let mut loader = PluginLoader::new().unwrap();
        
        let path = PathBuf::from("/nonexistent/path/to/plugin.wasm");
        let result = loader.compile(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_cached_module() {
        let mut loader = PluginLoader::new().unwrap();
        
        // Create a minimal valid Wasm file
        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6D, // magic
            0x01, 0x00, 0x00, 0x00, // version
        ];
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&wasm_bytes).unwrap();
        let path = temp_file.path().to_path_buf();
        
        // First compile
        let result1 = loader.compile(&path);
        assert!(result1.is_ok());
        assert_eq!(loader.cache_size(), 1);
        
        // Second compile should use cache
        let result2 = loader.compile(&path);
        assert!(result2.is_ok());
        assert_eq!(loader.cache_size(), 1); // Still 1, not 2
    }

    #[test]
    fn test_module_cache_multiple_insert() {
        let mut cache = ModuleCache::new();
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        
        for i in 0..10 {
            let module = Module::new(engine, "(module)").unwrap();
            let path = PathBuf::from(format!("/test{}.wasm", i));
            cache.insert(path, module);
        }
        
        assert_eq!(cache.len(), 10);
    }

    #[test]
    fn test_loader_multiple_caches() {
        let mut loader = PluginLoader::new().unwrap();
        let engine = loader.engine().clone();
        
        // Insert multiple modules
        for i in 0..5 {
            let module = Module::new(&engine, "(module)").unwrap();
            let path = PathBuf::from(format!("/plugin{}.wasm", i));
            loader.cache_mut().insert(path, module);
        }
        
        assert_eq!(loader.cache_size(), 5);
        
        // Clear and verify
        loader.clear_cache();
        assert_eq!(loader.cache_size(), 0);
    }

    #[test]
    fn test_extract_metadata_default_values() {
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();

        let metadata = loader.extract_metadata(&module).unwrap();
        
        // Verify all default fields
        assert_eq!(metadata.id, "unknown");
        assert_eq!(metadata.name, "Unknown Plugin");
        assert_eq!(metadata.version, "0.0.0");
        assert_eq!(metadata.description, "Plugin without metadata");
        assert_eq!(metadata.author, "Unknown");
        assert_eq!(metadata.homepage, None);
        assert_eq!(metadata.license, "MIT");
        assert_eq!(metadata.min_server_version, "0.1.0");
        assert_eq!(metadata.priority, 100);
        assert!(metadata.capabilities.is_empty());
        assert!(metadata.permissions.is_empty());
    }

    #[test]
    fn test_module_cache_is_empty() {
        let mut cache = ModuleCache::new();
        assert!(cache.is_empty());
        
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();
        
        cache.insert(PathBuf::from("/test.wasm"), module);
        assert!(!cache.is_empty());
        
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_loader_engine_config() {
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let _config = engine.config();
        
        // Engine was successfully created with custom config
        assert!(true);
    }

    #[test]
    fn test_compile_bytes_and_cache() {
        let mut loader = PluginLoader::new().unwrap();
        
        // Compile from bytes
        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6D, // magic
            0x01, 0x00, 0x00, 0x00, // version
        ];
        let key = PathBuf::from("/func.wasm");
        
        let module = loader.compile_bytes(&wasm_bytes, key.clone()).unwrap();
        
        // Verify it's in cache
        let cached = loader.cache().get(&key);
        assert!(cached.is_some());
    }
}
