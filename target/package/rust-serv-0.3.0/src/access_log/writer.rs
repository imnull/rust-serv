//! Access log writer with file persistence

use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::formatter::{AccessLogEntry, LogFormat};

/// Access log writer
pub struct AccessLogWriter {
    /// Log file path
    path: PathBuf,
    /// Log format
    format: LogFormat,
    /// Output file
    file: Arc<Mutex<File>>,
}

impl AccessLogWriter {
    /// Create a new access log writer
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Self::with_format(path, LogFormat::default())
    }

    /// Create a new access log writer with custom format
    pub fn with_format<P: AsRef<Path>>(path: P, format: LogFormat) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        
        Ok(Self {
            path,
            format,
            file: Arc::new(Mutex::new(file)),
        })
    }

    /// Write a log entry
    pub fn write(&self, entry: &AccessLogEntry) -> std::io::Result<()> {
        let line = self.format.format(entry);
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", line)?;
        file.flush()?;
        Ok(())
    }

    /// Write multiple log entries
    pub fn write_batch(&self, entries: &[AccessLogEntry]) -> std::io::Result<()> {
        let mut file = self.file.lock().unwrap();
        for entry in entries {
            let line = self.format.format(entry);
            writeln!(file, "{}", line)?;
        }
        file.flush()?;
        Ok(())
    }

    /// Get the log file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the log format
    pub fn format(&self) -> LogFormat {
        self.format
    }

    /// Flush the file buffer
    pub fn flush(&self) -> std::io::Result<()> {
        let mut file = self.file.lock().unwrap();
        file.flush()
    }
}

impl Clone for AccessLogWriter {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            format: self.format,
            file: Arc::clone(&self.file),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_writer() -> (AccessLogWriter, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::new(&path).unwrap();
        (writer, dir)
    }

    #[test]
    fn test_writer_creation() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::new(&path).unwrap();
        
        assert_eq!(writer.path(), path);
        assert_eq!(writer.format(), LogFormat::Combined);
    }

    #[test]
    fn test_writer_with_format() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::with_format(&path, LogFormat::Json).unwrap();
        
        assert_eq!(writer.format(), LogFormat::Json);
    }

    #[test]
    fn test_write_single_entry() {
        let (writer, _dir) = create_test_writer();
        
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/index.html")
            .with_status(200)
            .with_size(1234);
        
        let result = writer.write(&entry);
        assert!(result.is_ok());
        
        // Verify file was created and has content
        let content = std::fs::read_to_string(writer.path()).unwrap();
        assert!(content.contains("127.0.0.1"));
        assert!(content.contains("GET /index.html"));
    }

    #[test]
    fn test_write_multiple_entries() {
        let (writer, _dir) = create_test_writer();
        
        let entries = vec![
            AccessLogEntry::new("127.0.0.1", "GET", "/page1"),
            AccessLogEntry::new("192.168.1.1", "POST", "/api"),
            AccessLogEntry::new("10.0.0.1", "GET", "/page2"),
        ];
        
        let result = writer.write_batch(&entries);
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(writer.path()).unwrap();
        assert!(content.contains("/page1"));
        assert!(content.contains("/api"));
        assert!(content.contains("/page2"));
    }

    #[test]
    fn test_append_mode() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        
        // Write first entry
        {
            let writer = AccessLogWriter::new(&path).unwrap();
            let entry = AccessLogEntry::new("127.0.0.1", "GET", "/first");
            writer.write(&entry).unwrap();
        }
        
        // Write second entry (should append)
        {
            let writer = AccessLogWriter::new(&path).unwrap();
            let entry = AccessLogEntry::new("127.0.0.1", "GET", "/second");
            writer.write(&entry).unwrap();
        }
        
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("/first"));
        assert!(content.contains("/second"));
        // Should have 2 lines
        assert_eq!(content.lines().count(), 2);
    }

    #[test]
    fn test_clone() {
        let (writer, _dir) = create_test_writer();
        let cloned = writer.clone();
        
        // Both should work
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test");
        assert!(writer.write(&entry).is_ok());
        assert!(cloned.write(&entry).is_ok());
    }

    #[test]
    fn test_flush() {
        let (writer, _dir) = create_test_writer();
        
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/test");
        writer.write(&entry).unwrap();
        
        let result = writer.flush();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("logs/app/access.log");
        
        let writer = AccessLogWriter::new(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_json_format_output() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::with_format(&path, LogFormat::Json).unwrap();
        
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/api")
            .with_status(200)
            .with_duration_ms(50);
        
        writer.write(&entry).unwrap();
        
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("{"));
        assert!(content.contains("\"client_ip\""));
        assert!(content.contains("\"duration_ms\":50"));
    }

    #[test]
    fn test_common_format_output() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::with_format(&path, LogFormat::Common).unwrap();
        
        let entry = AccessLogEntry::new("127.0.0.1", "GET", "/page")
            .with_status(200)
            .with_size(100);
        
        writer.write(&entry).unwrap();
        
        let content = std::fs::read_to_string(&path).unwrap();
        // Common format should NOT have user-agent or referer
        assert!(!content.contains("\"-\" \"-\""));
    }

    #[test]
    fn test_concurrent_writes() {
        use std::thread;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");
        let writer = AccessLogWriter::new(&path).unwrap();
        let writer = Arc::new(writer);
        
        let mut handles = vec![];
        for i in 0..5 {
            let w = Arc::clone(&writer);
            handles.push(thread::spawn(move || {
                let entry = AccessLogEntry::new("127.0.0.1", "GET", format!("/page{}", i));
                w.write(&entry).unwrap();
            }));
        }
        
        for h in handles {
            h.join().unwrap();
        }
        
        let content = std::fs::read_to_string(&path).unwrap();
        // Should have 5 lines
        assert_eq!(content.lines().count(), 5);
    }
}
