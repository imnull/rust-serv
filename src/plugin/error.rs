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
    
    #[error("File watcher error: {0}")]
    WatcherError(String),
    
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
            Self::WatcherError(_) => 5003,
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

    #[test]
    fn test_error_init_failed() {
        let err = PluginError::InitFailed("init failed".to_string());
        assert_eq!(err.error_code(), 1000);
        assert!(!err.is_recoverable());
        assert!(!err.is_timeout());
    }

    #[test]
    fn test_error_invalid_config() {
        let err = PluginError::InvalidConfig("bad config".to_string());
        assert_eq!(err.error_code(), 1001);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_not_found() {
        let err = PluginError::NotFound("plugin not found".to_string());
        assert_eq!(err.error_code(), 1002);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_already_loaded() {
        let err = PluginError::AlreadyLoaded("plugin1".to_string());
        assert_eq!(err.error_code(), 1003);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_execution_error() {
        let err = PluginError::ExecutionError("execution failed".to_string());
        assert_eq!(err.error_code(), 2000);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_timeout() {
        let err = PluginError::Timeout(5000);
        assert_eq!(err.error_code(), 2001);
        assert!(err.is_timeout());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_permission_denied() {
        let err = PluginError::PermissionDenied("no access".to_string());
        assert_eq!(err.error_code(), 3000);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_invalid_input() {
        let err = PluginError::InvalidInput("bad input".to_string());
        assert_eq!(err.error_code(), 4000);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_serialization() {
        let err = PluginError::Serialization("json error".to_string());
        assert_eq!(err.error_code(), 4001);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_wasm_compilation() {
        let err = PluginError::WasmCompilation("compile error".to_string());
        assert_eq!(err.error_code(), 5000);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_wasm_instantiation() {
        let err = PluginError::WasmInstantiation("instantiate error".to_string());
        assert_eq!(err.error_code(), 5001);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_host_function() {
        let err = PluginError::HostFunction("host error".to_string());
        assert_eq!(err.error_code(), 5002);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_watcher_error() {
        let err = PluginError::WatcherError("watch error".to_string());
        assert_eq!(err.error_code(), 5003);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = PluginError::Io(io_err);
        assert_eq!(err.error_code(), 6000);
    }

    #[test]
    fn test_error_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = PluginError::Json(json_err);
        assert_eq!(err.error_code(), 6001);
    }

    #[test]
    fn test_error_other() {
        let err = PluginError::Other("some error".to_string());
        assert_eq!(err.error_code(), 9999);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_all_error_codes_unique() {
        let codes = vec![
            PluginError::InitFailed("".into()).error_code(),
            PluginError::InvalidConfig("".into()).error_code(),
            PluginError::NotFound("".into()).error_code(),
            PluginError::AlreadyLoaded("".into()).error_code(),
            PluginError::ExecutionError("".into()).error_code(),
            PluginError::Timeout(0).error_code(),
            PluginError::PermissionDenied("".into()).error_code(),
            PluginError::InvalidInput("".into()).error_code(),
            PluginError::Serialization("".into()).error_code(),
            PluginError::WasmCompilation("".into()).error_code(),
            PluginError::WasmInstantiation("".into()).error_code(),
            PluginError::HostFunction("".into()).error_code(),
            PluginError::WatcherError("".into()).error_code(),
            PluginError::Other("".into()).error_code(),
        ];

        // Check all codes are unique
        let unique_codes: std::collections::HashSet<_> = codes.iter().cloned().collect();
        assert_eq!(codes.len(), unique_codes.len());
    }

    #[test]
    fn test_is_recoverable_variants() {
        // Recoverable errors
        assert!(PluginError::Timeout(100).is_recoverable());
        assert!(PluginError::ExecutionError("".into()).is_recoverable());
        assert!(PluginError::HostFunction("".into()).is_recoverable());

        // Non-recoverable errors
        assert!(!PluginError::NotFound("".into()).is_recoverable());
        assert!(!PluginError::InitFailed("".into()).is_recoverable());
        assert!(!PluginError::InvalidConfig("".into()).is_recoverable());
    }

    #[test]
    fn test_is_timeout_only_timeout() {
        assert!(PluginError::Timeout(100).is_timeout());
        assert!(!PluginError::NotFound("".into()).is_timeout());
        assert!(!PluginError::ExecutionError("".into()).is_timeout());
    }
}
