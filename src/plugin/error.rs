//! Plugin system errors

use thiserror::Error;

/// Plugin error type
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),
    
    #[error("Invalid plugin configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),
    
    #[error("Plugin timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Wasm compilation failed: {0}")]
    WasmCompilation(String),
    
    #[error("Wasm instantiation failed: {0}")]
    WasmInstantiation(String),
    
    #[error("Host function error: {0}")]
    HostFunction(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("{0}")]
    Other(String),
}

/// Plugin result type
pub type PluginResult<T> = Result<T, PluginError>;

impl PluginError {
    /// Create an other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
    
    /// Check if error is timeout
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::Timeout(_) | 
            Self::ExecutionError(_) |
            Self::HostFunction(_)
        )
    }
    
    /// Get error code for C interop
    pub fn error_code(&self) -> i32 {
        match self {
            Self::InitFailed(_) => 1000,
            Self::InvalidConfig(_) => 1001,
            Self::NotFound(_) => 1002,
            Self::AlreadyLoaded(_) => 1003,
            Self::ExecutionError(_) => 2000,
            Self::Timeout(_) => 2001,
            Self::PermissionDenied(_) => 3000,
            Self::InvalidInput(_) => 4000,
            Self::Serialization(_) => 4001,
            Self::WasmCompilation(_) => 5000,
            Self::WasmInstantiation(_) => 5001,
            Self::HostFunction(_) => 5002,
            Self::Io(_) => 6000,
            Self::Json(_) => 6001,
            Self::Other(_) => 9999,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_code() {
        let err = PluginError::Timeout(100);
        assert_eq!(err.error_code(), 2001);
        assert!(err.is_timeout());
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_error_recovery() {
        let err = PluginError::NotFound("test".to_string());
        assert!(!err.is_recoverable());
    }
}
