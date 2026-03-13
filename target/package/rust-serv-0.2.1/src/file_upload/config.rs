//! Upload configuration

use std::path::PathBuf;

/// Upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Upload directory (where files will be stored)
    pub upload_dir: PathBuf,
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: usize,
    /// Allowed file extensions (empty = all allowed)
    pub allowed_extensions: Vec<String>,
    /// Overwrite existing files
    pub overwrite: bool,
    /// Generate unique filenames
    pub unique_names: bool,
}

impl UploadConfig {
    /// Create a new upload config
    pub fn new(upload_dir: impl Into<PathBuf>) -> Self {
        Self {
            upload_dir: upload_dir.into(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec![],
            overwrite: false,
            unique_names: false,
        }
    }

    /// Set max file size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set allowed extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = extensions;
        self
    }

    /// Set overwrite mode
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Set unique names mode
    pub fn with_unique_names(mut self, unique: bool) -> Self {
        self.unique_names = unique;
        self
    }

    /// Check if file extension is allowed
    pub fn is_extension_allowed(&self, filename: &str) -> bool {
        if self.allowed_extensions.is_empty() {
            return true;
        }
        
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        match ext {
            Some(e) => self.allowed_extensions.iter().any(|a| a.to_lowercase() == e),
            None => false,
        }
    }

    /// Check if file size is within limit
    pub fn is_size_allowed(&self, size: usize) -> bool {
        size <= self.max_file_size
    }

    /// Generate unique filename
    pub fn generate_unique_filename(&self, original: &str) -> String {
        if !self.unique_names {
            return original.to_string();
        }
        
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        
        let path = std::path::Path::new(original);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        
        if ext.is_empty() {
            format!("{}_{}", stem, timestamp)
        } else {
            format!("{}_{}.{}", stem, timestamp, ext)
        }
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self::new("./uploads")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = UploadConfig::new("/var/uploads");
        assert_eq!(config.upload_dir, PathBuf::from("/var/uploads"));
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_config_with_max_size() {
        let config = UploadConfig::new("/uploads").with_max_size(1024);
        assert_eq!(config.max_file_size, 1024);
    }

    #[test]
    fn test_config_with_extensions() {
        let config = UploadConfig::new("/uploads")
            .with_extensions(vec!["txt".to_string(), "pdf".to_string()]);
        
        assert_eq!(config.allowed_extensions.len(), 2);
    }

    #[test]
    fn test_config_with_overwrite() {
        let config = UploadConfig::new("/uploads").with_overwrite(true);
        assert!(config.overwrite);
    }

    #[test]
    fn test_config_with_unique_names() {
        let config = UploadConfig::new("/uploads").with_unique_names(true);
        assert!(config.unique_names);
    }

    #[test]
    fn test_is_extension_allowed_all() {
        let config = UploadConfig::new("/uploads");
        
        assert!(config.is_extension_allowed("test.txt"));
        assert!(config.is_extension_allowed("test.pdf"));
        assert!(config.is_extension_allowed("test.jpg"));
    }

    #[test]
    fn test_is_extension_allowed_specific() {
        let config = UploadConfig::new("/uploads")
            .with_extensions(vec!["txt".to_string(), "PDF".to_string()]);
        
        assert!(config.is_extension_allowed("test.txt"));
        assert!(config.is_extension_allowed("test.pdf"));
        assert!(!config.is_extension_allowed("test.jpg"));
        assert!(!config.is_extension_allowed("noextension"));
    }

    #[test]
    fn test_is_extension_allowed_case_insensitive() {
        let config = UploadConfig::new("/uploads")
            .with_extensions(vec!["TXT".to_string()]);
        
        assert!(config.is_extension_allowed("test.TXT"));
        assert!(config.is_extension_allowed("test.txt"));
    }

    #[test]
    fn test_is_size_allowed() {
        let config = UploadConfig::new("/uploads").with_max_size(1000);
        
        assert!(config.is_size_allowed(500));
        assert!(config.is_size_allowed(1000));
        assert!(!config.is_size_allowed(1001));
    }

    #[test]
    fn test_generate_unique_filename_disabled() {
        let config = UploadConfig::new("/uploads");
        
        let result = config.generate_unique_filename("test.txt");
        assert_eq!(result, "test.txt");
    }

    #[test]
    fn test_generate_unique_filename_enabled() {
        let config = UploadConfig::new("/uploads").with_unique_names(true);
        
        let result = config.generate_unique_filename("test.txt");
        
        // Should have format: test_{timestamp}.txt
        assert!(result.starts_with("test_"));
        assert!(result.ends_with(".txt"));
        assert_ne!(result, "test.txt");
        // Should contain a number (timestamp)
        assert!(result.chars().any(|c| c.is_numeric()));
    }

    #[test]
    fn test_generate_unique_filename_no_ext() {
        let config = UploadConfig::new("/uploads").with_unique_names(true);
        
        let result = config.generate_unique_filename("README");
        
        // Should have format: README_{timestamp}
        assert!(result.starts_with("README_"));
        assert!(!result.contains('.'));
    }

    #[test]
    fn test_default() {
        let config = UploadConfig::default();
        assert_eq!(config.upload_dir, PathBuf::from("./uploads"));
    }
}
