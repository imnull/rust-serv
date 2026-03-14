//! Host 函数接口
//!
//! 提供插件调用宿主（rust-serv）功能的接口

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// 调试级别
    Debug = 0,
    /// 信息级别
    Info = 1,
    /// 警告级别
    Warn = 2,
    /// 错误级别
    Error = 3,
}

impl LogLevel {
    /// 从 i32 转换
    pub fn from_i32(level: i32) -> Self {
        match level {
            0 => Self::Debug,
            1 => Self::Info,
            2 => Self::Warn,
            3 => Self::Error,
            _ => Self::Info, // 默认 Info
        }
    }

    /// 转换为 i32
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// 获取级别名称
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Host 函数集合
///
/// 这些函数在 Wasm 插件中通过 `host_log` 等名称导入
pub struct HostFunctions;

impl HostFunctions {
    /// 输出日志
    ///
    /// 在真实插件中，这会调用宿主的 host_log 函数
    pub fn log(level: LogLevel, message: &str) {
        eprintln!("[{}] {}", level.as_str(), message);
    }

    /// 调试日志
    pub fn debug(message: &str) {
        Self::log(LogLevel::Debug, message);
    }

    /// 信息日志
    pub fn info(message: &str) {
        Self::log(LogLevel::Info, message);
    }

    /// 警告日志
    pub fn warn(message: &str) {
        Self::log(LogLevel::Warn, message);
    }

    /// 错误日志
    pub fn error(message: &str) {
        Self::log(LogLevel::Error, message);
    }

    /// 获取配置值（桩实现）
    ///
    /// 在真实插件中，这会调用宿主的 host_get_config 函数
    pub fn get_config(_key: &str) -> Option<String> {
        // 在 Wasm 环境中，这会调用 host 函数
        // 当前为桩实现
        None
    }

    /// 设置响应头（桩实现）
    ///
    /// 在真实插件中，这会调用宿主的 host_set_header 函数
    pub fn set_header(_name: &str, _value: &str) {
        // 在 Wasm 环境中，这会调用 host 函数
        // 当前为桩实现
    }

    /// 上报计数器指标（桩实现）
    pub fn metrics_counter(_name: &str, _value: f64) {
        // 桩实现
    }

    /// 上报仪表盘指标（桩实现）
    pub fn metrics_gauge(_name: &str, _value: f64) {
        // 桩实现
    }

    /// 上报直方图指标（桩实现）
    pub fn metrics_histogram(_name: &str, _value: f64) {
        // 桩实现
    }
}

/// 便捷宏：输出调试日志
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::host::HostFunctions::debug(&format!($($arg)*))
    };
}

/// 便捷宏：输出信息日志
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::host::HostFunctions::info(&format!($($arg)*))
    };
}

/// 便捷宏：输出警告日志
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::host::HostFunctions::warn(&format!($($arg)*))
    };
}

/// 便捷宏：输出错误日志
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::host::HostFunctions::error(&format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_i32() {
        assert_eq!(LogLevel::from_i32(0), LogLevel::Debug);
        assert_eq!(LogLevel::from_i32(1), LogLevel::Info);
        assert_eq!(LogLevel::from_i32(2), LogLevel::Warn);
        assert_eq!(LogLevel::from_i32(3), LogLevel::Error);
        assert_eq!(LogLevel::from_i32(99), LogLevel::Info); // 默认
    }

    #[test]
    fn test_log_level_as_i32() {
        assert_eq!(LogLevel::Debug.as_i32(), 0);
        assert_eq!(LogLevel::Info.as_i32(), 1);
        assert_eq!(LogLevel::Warn.as_i32(), 2);
        assert_eq!(LogLevel::Error.as_i32(), 3);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_level_default() {
        let level: LogLevel = Default::default();
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_debug_fmt() {
        let level = LogLevel::Info;
        let debug = format!("{:?}", level);
        assert_eq!(debug, "Info");
    }

    #[test]
    fn test_host_functions_log() {
        // 只测试不 panic
        HostFunctions::log(LogLevel::Debug, "debug message");
        HostFunctions::log(LogLevel::Info, "info message");
        HostFunctions::log(LogLevel::Warn, "warn message");
        HostFunctions::log(LogLevel::Error, "error message");
    }

    #[test]
    fn test_host_functions_debug() {
        HostFunctions::debug("test debug");
    }

    #[test]
    fn test_host_functions_info() {
        HostFunctions::info("test info");
    }

    #[test]
    fn test_host_functions_warn() {
        HostFunctions::warn("test warn");
    }

    #[test]
    fn test_host_functions_error() {
        HostFunctions::error("test error");
    }

    #[test]
    fn test_host_functions_get_config() {
        let result = HostFunctions::get_config("some_key");
        assert!(result.is_none());
    }

    #[test]
    fn test_host_functions_set_header() {
        // 桩实现，只测试不 panic
        HostFunctions::set_header("X-Custom", "value");
    }

    #[test]
    fn test_host_functions_metrics_counter() {
        HostFunctions::metrics_counter("requests_total", 1.0);
        HostFunctions::metrics_counter("requests_total", 0.0);
        HostFunctions::metrics_counter("requests_total", -1.0);
    }

    #[test]
    fn test_host_functions_metrics_gauge() {
        HostFunctions::metrics_gauge("active_connections", 42.5);
        HostFunctions::metrics_gauge("active_connections", 0.0);
    }

    #[test]
    fn test_host_functions_metrics_histogram() {
        HostFunctions::metrics_histogram("request_duration", 0.123);
        HostFunctions::metrics_histogram("request_duration", 1.5);
    }

    #[test]
    fn test_log_macros() {
        // 测试宏展开不 panic
        log_debug!("test {} {}", "debug", 1);
        log_info!("test {} {}", "info", 2);
        log_warn!("test {} {}", "warn", 3);
        log_error!("test {} {}", "error", 4);
    }

    #[test]
    fn test_log_level_clone() {
        let level = LogLevel::Info;
        let cloned = level.clone();
        assert_eq!(level, cloned);
    }

    #[test]
    fn test_log_level_copy() {
        let level = LogLevel::Debug;
        let copied = level; // Copy trait
        assert_eq!(level, copied);
    }

    #[test]
    fn test_log_level_partial_eq() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Debug);
    }

    #[test]
    fn test_log_level_eq() {
        // 测试 Eq trait
        fn assert_eq_trait<T: Eq>(_: T) {}
        assert_eq_trait(LogLevel::Info);
    }

    #[test]
    fn test_log_level_all_variants() {
        let levels = vec![
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];

        for level in levels {
            let i32_val = level.as_i32();
            let from_i32 = LogLevel::from_i32(i32_val);
            assert_eq!(level, from_i32);
        }
    }

    #[test]
    fn test_host_functions_log_with_empty_message() {
        HostFunctions::log(LogLevel::Info, "");
        HostFunctions::log(LogLevel::Debug, "");
    }

    #[test]
    fn test_host_functions_log_with_unicode() {
        HostFunctions::log(LogLevel::Info, "你好世界 🌍🎉");
        HostFunctions::log(LogLevel::Debug, "Special: αβγ δεζ ηθι");
    }

    #[test]
    fn test_host_functions_log_with_long_message() {
        let long_message = "x".repeat(10000);
        HostFunctions::log(LogLevel::Info, &long_message);
    }

    #[test]
    fn test_host_functions_get_config_with_various_keys() {
        assert!(HostFunctions::get_config("").is_none());
        assert!(HostFunctions::get_config("key").is_none());
        assert!(HostFunctions::get_config("very.long.key.name").is_none());
        assert!(HostFunctions::get_config("key-with-dashes").is_none());
        assert!(HostFunctions::get_config("key_with_underscores").is_none());
    }

    #[test]
    fn test_host_functions_set_header_various() {
        HostFunctions::set_header("", "");
        HostFunctions::set_header("X-Custom", "");
        HostFunctions::set_header("", "value");
        HostFunctions::set_header("X-Very-Long-Header-Name-Here", "value".repeat(100));
    }

    #[test]
    fn test_host_functions_metrics_various_values() {
        // Counter
        HostFunctions::metrics_counter("counter", f64::MAX);
        HostFunctions::metrics_counter("counter", f64::MIN);
        HostFunctions::metrics_counter("counter", f64::EPSILON);

        // Gauge
        HostFunctions::metrics_gauge("gauge", f64::MAX);
        HostFunctions::metrics_gauge("gauge", f64::MIN);
        HostFunctions::metrics_gauge("gauge", -42.5);

        // Histogram
        HostFunctions::metrics_histogram("histogram", 0.0);
        HostFunctions::metrics_histogram("histogram", 0.001);
        HostFunctions::metrics_histogram("histogram", 1000.5);
    }

    #[test]
    fn test_log_macros_with_empty() {
        log_debug!("");
        log_info!("");
        log_warn!("");
        log_error!("");
    }

    #[test]
    fn test_log_macros_with_unicode() {
        log_debug!("Unicode: 你好 🌍");
        log_info!("Special chars: αβγ");
        log_warn!("Emoji: 🚨⚠️🔥");
        log_error!("Mixed: test 测试 テスト");
    }

    #[test]
    fn test_log_macros_complex_formatting() {
        log_debug!("Number: {}, String: {}", 42, "hello");
        log_info!("Float: {:.2}, Hex: {:x}", 3.14159, 255);
        log_warn!("Array: {:?}", vec![1, 2, 3]);
        log_error!("Debug: {:?}", ("tuple", 42, true));
    }
}
