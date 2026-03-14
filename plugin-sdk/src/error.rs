//! Plugin SDK 错误类型

use thiserror::Error;

/// 插件错误类型
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PluginError {
    /// 初始化失败
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    /// 配置无效
    #[error("Invalid plugin configuration: {0}")]
    InvalidConfig(String),

    /// 执行错误
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    /// 超时
    #[error("Plugin timeout after {0}ms")]
    Timeout(u64),

    /// 权限被拒绝
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// 输入无效
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 未找到
    #[error("Not found: {0}")]
    NotFound(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

/// 插件结果类型
pub type PluginResult<T> = Result<T, PluginError>;

impl PluginError {
    /// 创建其他错误
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// 创建初始化失败错误
    pub fn init_failed(msg: impl Into<String>) -> Self {
        Self::InitFailed(msg.into())
    }

    /// 创建配置错误
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// 创建执行错误
    pub fn execution_error(msg: impl Into<String>) -> Self {
        Self::ExecutionError(msg.into())
    }

    /// 创建超时错误
    pub fn timeout(ms: u64) -> Self {
        Self::Timeout(ms)
    }

    /// 创建权限错误
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// 创建序列化错误
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// 创建输入错误
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 检查是否为超时错误
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// 检查是否为权限错误
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// 检查是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(self, Self::InvalidConfig(_) | Self::InitFailed(_))
    }

    /// 检查是否可恢复
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_) | Self::ExecutionError(_) | Self::PermissionDenied(_)
        )
    }

    /// 获取错误代码
    pub fn error_code(&self) -> u32 {
        match self {
            Self::InitFailed(_) => 1000,
            Self::InvalidConfig(_) => 1001,
            Self::NotFound(_) => 1002,
            Self::ExecutionError(_) => 2000,
            Self::Timeout(_) => 2001,
            Self::PermissionDenied(_) => 3000,
            Self::InvalidInput(_) => 4000,
            Self::Serialization(_) => 4001,
            Self::Other(_) => 9999,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_init_failed() {
        let err = PluginError::init_failed("initialization failed");
        assert!(matches!(err, PluginError::InitFailed(_)));
        assert_eq!(err.error_code(), 1000);
        assert!(!err.is_recoverable());
        assert!(err.is_config_error());
    }

    #[test]
    fn test_error_invalid_config() {
        let err = PluginError::invalid_config("missing required field");
        assert!(matches!(err, PluginError::InvalidConfig(_)));
        assert_eq!(err.error_code(), 1001);
        assert!(!err.is_recoverable());
        assert!(err.is_config_error());
    }

    #[test]
    fn test_error_not_found() {
        let err = PluginError::not_found("plugin not found");
        assert!(matches!(err, PluginError::NotFound(_)));
        assert_eq!(err.error_code(), 1002);
        assert!(!err.is_recoverable());
        assert!(!err.is_config_error());
    }

    #[test]
    fn test_error_execution_error() {
        let err = PluginError::execution_error("runtime error");
        assert!(matches!(err, PluginError::ExecutionError(_)));
        assert_eq!(err.error_code(), 2000);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_timeout() {
        let err = PluginError::timeout(5000);
        assert!(matches!(err, PluginError::Timeout(5000)));
        assert_eq!(err.error_code(), 2001);
        assert!(err.is_timeout());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_permission_denied() {
        let err = PluginError::permission_denied("access denied");
        assert!(matches!(err, PluginError::PermissionDenied(_)));
        assert_eq!(err.error_code(), 3000);
        assert!(err.is_permission_denied());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_invalid_input() {
        let err = PluginError::invalid_input("invalid parameter");
        assert!(matches!(err, PluginError::InvalidInput(_)));
        assert_eq!(err.error_code(), 4000);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_serialization() {
        let err = PluginError::serialization("json parse error");
        assert!(matches!(err, PluginError::Serialization(_)));
        assert_eq!(err.error_code(), 4001);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_other() {
        let err = PluginError::other("unknown error");
        assert!(matches!(err, PluginError::Other(_)));
        assert_eq!(err.error_code(), 9999);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = PluginError::timeout(100);
        let msg = format!("{}", err);
        assert!(msg.contains("100"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_error_debug() {
        let err = PluginError::not_found("test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
    }

    #[test]
    fn test_error_codes_unique() {
        use std::collections::HashSet;

        let codes = vec![
            PluginError::init_failed("").error_code(),
            PluginError::invalid_config("").error_code(),
            PluginError::not_found("").error_code(),
            PluginError::execution_error("").error_code(),
            PluginError::timeout(0).error_code(),
            PluginError::permission_denied("").error_code(),
            PluginError::invalid_input("").error_code(),
            PluginError::serialization("").error_code(),
            PluginError::other("").error_code(),
        ];

        let unique: HashSet<_> = codes.iter().cloned().collect();
        assert_eq!(codes.len(), unique.len(), "Error codes should be unique");
    }

    #[test]
    fn test_recoverable_errors() {
        // 可恢复的错误
        assert!(PluginError::timeout(100).is_recoverable());
        assert!(PluginError::execution_error("").is_recoverable());
        assert!(PluginError::permission_denied("").is_recoverable());

        // 不可恢复的错误
        assert!(!PluginError::init_failed("").is_recoverable());
        assert!(!PluginError::invalid_config("").is_recoverable());
        assert!(!PluginError::not_found("").is_recoverable());
        assert!(!PluginError::invalid_input("").is_recoverable());
        assert!(!PluginError::serialization("").is_recoverable());
        assert!(!PluginError::other("").is_recoverable());
    }

    #[test]
    fn test_plugin_result_ok() {
        let result: PluginResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_plugin_result_err() {
        let result: PluginResult<i32> = Err(PluginError::not_found("item"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }
}
