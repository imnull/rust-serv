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
    
    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.is_ok());
    }
    
    #[test]
    fn test_module_cache() {
        let mut cache = ModuleCache::new();
        assert_eq!(cache.len(), 0);
        
        let loader = PluginLoader::new().unwrap();
        let engine = loader.engine();
        let module = Module::new(engine, "(module)").unwrap();
        
        cache.insert(PathBuf::from("/test.wasm"), module);
        assert_eq!(cache.len(), 1);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
