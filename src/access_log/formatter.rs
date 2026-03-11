//! Access log entry and formatting

use std::net::SocketAddr;
use std::time::SystemTime;

/// Access log entry
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    /// Client IP address
    pub client_ip: String,
    /// Request timestamp
    pub timestamp: SystemTime,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP version
    pub version: String,
    /// Response status code
    pub status: u16,
    /// Response size in bytes
    pub size: usize,
    /// Request duration in milliseconds
    pub duration_ms: u64,
    /// User-Agent header
    pub user_agent: Option<String>,
    /// Referer header
    pub referer: Option<String>,
}

impl AccessLogEntry {
    /// Create a new access log entry
    pub fn new(
        client_ip: impl Into<String>,
        method: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            timestamp: SystemTime::now(),
            method: method.into(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            status: 200,
            size: 0,
            duration_ms: 0,
            user_agent: None,
            referer: None,
        }
    }

    /// Set HTTP version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set response status
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Set response size
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Set request duration
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set User-Agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set Referer
    pub fn with_referer(mut self, referer: impl Into<String>) -> Self {
        self.referer = Some(referer.into());
        self
    }
}

/// Log format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogFormat {
    /// Common Log Format (CLF)
    Common,
    /// Combined Log Format
    Combined,
    /// JSON format
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Combined
    }
}

impl LogFormat {
    /// Format a log entry
    pub fn format(&self, entry: &AccessLogEntry) -> String {
        match self {
            LogFormat::Common => self.format_common(entry),
            LogFormat::Combined => self.format_combined(entry),
            LogFormat::Json => self.format_json(entry),
        }
    }

    /// Common Log Format
    fn format_common(&self, entry: &AccessLogEntry) -> String {
        let timestamp = self.format_timestamp(&entry.timestamp);
        format!(
            "{} - - [{}] \"{} {} {}\" {} {}",
            entry.client_ip,
            timestamp,
            entry.method,
            entry.path,
            entry.version,
            entry.status,
            entry.size
        )
    }

    /// Combined Log Format
    fn format_combined(&self, entry: &AccessLogEntry) -> String {
        let timestamp = self.format_timestamp(&entry.timestamp);
        let referer = entry.referer.as_deref().unwrap_or("-");
        let user_agent = entry.user_agent.as_deref().unwrap_or("-");
        format!(
            "{} - - [{}] \"{} {} {}\" {} {} \"{}\" \"{}\"",
            entry.client_ip,
            timestamp,
            entry.method,
            entry.path,
            entry.version,
            entry.status,
            entry.size,
            referer,
            user_agent
        )
    }

    /// JSON format
    fn format_json(&self, entry: &AccessLogEntry) -> String {
        let timestamp = self.format_timestamp_iso(&entry.timestamp);
        let referer = entry.referer.as_deref().unwrap_or("-");
        let user_agent = entry.user_agent.as_deref().unwrap_or("-");
        
        format!(
            r#"{{"client_ip":"{}","timestamp":"{}","method":"{}","path":"{}","version":"{}","status":{},"size":{},"duration_ms":{},"referer":"{}","user_agent":"{}"}}"#,
            entry.client_ip,
            timestamp,
            entry.method,
            entry.path,
            entry.version,
            entry.status,
            entry.size,
            entry.duration_ms,
            referer,
            user_agent
        )
    }

    /// Format timestamp in CLF format
    fn format_timestamp(&self, time: &SystemTime) -> String {
        use std::time::UNIX_EPOCH;
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        
        // Simple date formatting (without chrono dependency)
        let days = secs / 86400;
        let years = 1970 + days / 365;
        let day_of_year = days % 365;
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                      "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        
        let hour = (secs % 86400) / 3600;
        let min = (secs % 3600) / 60;
        let sec = secs % 60;
        
        let month_idx = (day_of_year / 30) as usize;
        let month = months.get(month_idx).unwrap_or(&"Jan");
        let day = (day_of_year % 30) + 1;
        
        format!("{:02}/{}/{}:{:02}:{:02}:{:02} +0000", day, month, years, hour, min, sec)
    }

    /// Format timestamp in ISO 8601 format
    fn format_timestamp_iso(&self, time: &SystemTime) -> String {
        use std::time::UNIX_EPOCH;
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        
        let days = secs / 86400;
        let years = 1970 + days / 365;
        let day_of_year = days % 365;
        let month = (day_of_year / 30) + 1;
        let day = (day_of_year % 30) + 1;
        let hour = (secs % 86400) / 3600;
        let min = (secs % 3600) / 60;
        let sec = secs % 60;
        
        format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z", years, month, day, hour, min, sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_log_entry_creation() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/index.html");
        assert_eq!(entry.client_ip, "127.0.0.1");
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/index.html");
        assert_eq!(entry.status, 200);
        assert_eq!(entry.size, 0);
    }

    #[test]
    fn test_access_log_entry_with_status() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_status(404);
        assert_eq!(entry.status, 404);
    }

    #[test]
    fn test_access_log_entry_with_size() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_size(1024);
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_access_log_entry_with_duration() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_duration_ms(50);
        assert_eq!(entry.duration_ms, 50);
    }

    #[test]
    fn test_access_log_entry_with_user_agent() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_user_agent("Mozilla/5.0");
        assert_eq!(entry.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_access_log_entry_with_referer() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_referer("https://example.com");
        assert_eq!(entry.referer, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_access_log_entry_with_version() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test")
            .with_version("HTTP/2.0");
        assert_eq!(entry.version, "HTTP/2.0");
    }

    #[test]
    fn test_access_log_entry_chained() {
        let entry = AccessLogEntry::new("127.0.0.1", "POST", "/api")
            .with_status(201)
            .with_size(256)
            .with_duration_ms(10)
            .with_user_agent("curl/7.0");
        
        assert_eq!(entry.client_ip, "127.0.0.1");
        assert_eq!(entry.method, "POST");
        assert_eq!(entry.path, "/api");
        assert_eq!(entry.status, 201);
        assert_eq!(entry.size, 256);
        assert_eq!(entry.duration_ms, 10);
        assert_eq!(entry.user_agent, Some("curl/7.0".to_string()));
    }

    #[test]
    fn test_log_format_default() {
        assert_eq!(LogFormat::default(), LogFormat::Combined);
    }

    #[test]
    fn test_log_format_common() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/index.html")
            .with_status(200)
            .with_size(1234);
        
        let format = LogFormat::Common;
        let output = format.format(&entry);
        
        assert!(output.contains("127.0.0.1"));
        assert!(output.contains("GET /index.html HTTP/1.1"));
        assert!(output.contains("200 1234"));
    }

    #[test]
    fn test_log_format_combined() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/index.html")
            .with_status(200)
            .with_size(1234)
            .with_user_agent("Mozilla/5.0")
            .with_referer("https://example.com");
        
        let format = LogFormat::Combined;
        let output = format.format(&entry);
        
        assert!(output.contains("127.0.0.1"));
        assert!(output.contains("GET /index.html HTTP/1.1"));
        assert!(output.contains("200 1234"));
        assert!(output.contains("Mozilla/5.0"));
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_log_format_combined_without_headers() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/index.html")
            .with_status(200);
        
        let format = LogFormat::Combined;
        let output = format.format(&entry);
        
        // Should have dashes for missing headers
        assert!(output.contains("\"-\" \"-\""));
    }

    #[test]
    fn test_log_format_json() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/api")
            .with_status(200)
            .with_size(512)
            .with_duration_ms(25)
            .with_user_agent("TestAgent");
        
        let format = LogFormat::Json;
        let output = format.format(&entry);
        
        assert!(output.contains("\"client_ip\":\"127.0.0.1\""));
        assert!(output.contains("\"method\":\"GET\""));
        assert!(output.contains("\"path\":\"/api\""));
        assert!(output.contains("\"status\":200"));
        assert!(output.contains("\"size\":512"));
        assert!(output.contains("\"duration_ms\":25"));
        assert!(output.contains("\"user_agent\":\"TestAgent\""));
    }

    #[test]
    fn test_log_format_json_without_optional() {
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test");
        
        let format = LogFormat::Json;
        let output = format.format(&entry);
        
        assert!(output.contains("\"referer\":\"-\""));
        assert!(output.contains("\"user_agent\":\"-\""));
    }

    #[test]
    fn test_format_timestamp() {
        let format = LogFormat::Common;
        let timestamp = SystemTime::now();
        let output = format.format_timestamp(&timestamp);
        
        // Should contain date components
        assert!(output.contains("/"));
        assert!(output.contains(":"));
        assert!(output.contains("+0000"));
    }

    #[test]
    fn test_format_timestamp_iso() {
        let format = LogFormat::Json;
        let timestamp = SystemTime::now();
        let output = format.format_timestamp_iso(&timestamp);
        
        // Should be ISO format
        assert!(output.contains("T"));
        assert!(output.contains("Z"));
        assert!(output.contains("-"));
    }
}
