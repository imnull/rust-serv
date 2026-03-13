//! Upload handler

use std::path::PathBuf;
use std::io::Write;

use super::config::UploadConfig;
use super::multipart::MultipartParser;

/// Upload result
#[derive(Debug, Clone, PartialEq)]
pub enum UploadResult {
    /// Upload successful
    Success {
        filename: String,
        path: PathBuf,
        size: usize,
    },
    /// File too large
    TooLarge {
        filename: String,
        size: usize,
        max_size: usize,
    },
    /// Extension not allowed
    ExtensionNotAllowed {
        filename: String,
    },
    /// File already exists
    AlreadyExists {
        filename: String,
    },
    /// Write error
    WriteError {
        filename: String,
        error: String,
    },
    /// Invalid request
    InvalidRequest(String),
}

impl UploadResult {
    /// Check if upload was successful
    pub fn is_success(&self) -> bool {
        matches!(self, UploadResult::Success { .. })
    }
}

/// Upload handler
pub struct UploadHandler {
    config: UploadConfig,
}

impl UploadHandler {
    /// Create a new upload handler
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &UploadConfig {
        &self.config
    }

    /// Handle file upload
    pub fn handle_upload(&self, filename: &str, data: &[u8]) -> UploadResult {
        // Check file size
        if !self.config.is_size_allowed(data.len()) {
            return UploadResult::TooLarge {
                filename: filename.to_string(),
                size: data.len(),
                max_size: self.config.max_file_size,
            };
        }

        // Check extension
        if !self.config.is_extension_allowed(filename) {
            return UploadResult::ExtensionNotAllowed {
                filename: filename.to_string(),
            };
        }

        // Generate final filename
        let final_filename = self.config.generate_unique_filename(filename);
        let file_path = self.config.upload_dir.join(&final_filename);

        // Check if file exists
        if !self.config.overwrite && file_path.exists() {
            return UploadResult::AlreadyExists {
                filename: final_filename,
            };
        }

        // Create upload directory if needed
        if let Some(parent) = file_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return UploadResult::WriteError {
                    filename: final_filename,
                    error: e.to_string(),
                };
            }
        }

        // Write file
        match std::fs::File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(data) {
                    return UploadResult::WriteError {
                        filename: final_filename,
                        error: e.to_string(),
                    };
                }
                UploadResult::Success {
                    filename: final_filename,
                    path: file_path,
                    size: data.len(),
                }
            }
            Err(e) => UploadResult::WriteError {
                filename: final_filename,
                error: e.to_string(),
            },
        }
    }

    /// Handle multipart upload
    pub fn handle_multipart(&self, content_type: &str, data: &[u8]) -> Vec<(String, UploadResult)> {
        let parser = match MultipartParser::from_content_type(content_type) {
            Some(p) => p,
            None => return vec![("".to_string(), UploadResult::InvalidRequest("Invalid Content-Type".to_string()))],
        };

        let parts = match parser.parse(data) {
            Ok(p) => p,
            Err(e) => return vec![("".to_string(), UploadResult::InvalidRequest(e))],
        };

        parts
            .into_iter()
            .filter(|p| p.is_file())
            .map(|part| {
                let filename = part.filename.unwrap_or_default();
                let result = self.handle_upload(&filename, &part.data);
                (filename, result)
            })
            .collect()
    }

    /// Delete an uploaded file
    pub fn delete(&self, filename: &str) -> Result<(), String> {
        let file_path = self.config.upload_dir.join(filename);
        
        if !file_path.exists() {
            return Err("File not found".to_string());
        }

        std::fs::remove_file(&file_path)
            .map_err(|e| e.to_string())
    }

    /// List uploaded files
    pub fn list(&self) -> Vec<String> {
        let mut files = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.config.upload_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(name.to_string());
                    }
                }
            }
        }
        
        files.sort();
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_handler() -> (UploadHandler, TempDir) {
        let dir = TempDir::new().unwrap();
        let config = UploadConfig::new(dir.path());
        (UploadHandler::new(config), dir)
    }

    #[test]
    fn test_handler_creation() {
        let (handler, _dir) = create_test_handler();
        assert_eq!(handler.config().max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_upload_success() {
        let (handler, dir) = create_test_handler();
        
        let result = handler.handle_upload("test.txt", b"Hello World");
        
        if let UploadResult::Success { filename, path, size } = result {
            assert_eq!(filename, "test.txt");
            assert!(path.starts_with(dir.path()));
            assert_eq!(size, 11);
        } else {
            panic!("Expected Success result");
        }
    }

    #[test]
    fn test_upload_too_large() {
        let dir = TempDir::new().unwrap();
        let config = UploadConfig::new(dir.path()).with_max_size(10);
        let handler = UploadHandler::new(config);
        
        let result = handler.handle_upload("test.txt", b"Hello World!");
        
        if let UploadResult::TooLarge { size, max_size, .. } = result {
            assert_eq!(size, 12);
            assert_eq!(max_size, 10);
        } else {
            panic!("Expected TooLarge result");
        }
    }

    #[test]
    fn test_upload_extension_not_allowed() {
        let dir = TempDir::new().unwrap();
        let config = UploadConfig::new(dir.path())
            .with_extensions(vec!["txt".to_string()]);
        let handler = UploadHandler::new(config);
        
        let result = handler.handle_upload("test.exe", b"data");
        
        assert!(matches!(result, UploadResult::ExtensionNotAllowed { .. }));
    }

    #[test]
    fn test_upload_already_exists() {
        let (handler, _dir) = create_test_handler();
        
        // First upload
        handler.handle_upload("test.txt", b"data1");
        
        // Second upload with same name
        let result = handler.handle_upload("test.txt", b"data2");
        
        assert!(matches!(result, UploadResult::AlreadyExists { .. }));
    }

    #[test]
    fn test_upload_overwrite() {
        let dir = TempDir::new().unwrap();
        let config = UploadConfig::new(dir.path()).with_overwrite(true);
        let handler = UploadHandler::new(config);
        
        // First upload
        handler.handle_upload("test.txt", b"data1");
        
        // Second upload with same name (should overwrite)
        let result = handler.handle_upload("test.txt", b"data2");
        
        assert!(result.is_success());
        
        // Verify content was overwritten
        let content = std::fs::read_to_string(dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "data2");
    }

    #[test]
    fn test_upload_unique_names() {
        let dir = TempDir::new().unwrap();
        let config = UploadConfig::new(dir.path()).with_unique_names(true);
        let handler = UploadHandler::new(config);
        
        let result1 = handler.handle_upload("test.txt", b"data1");
        let result2 = handler.handle_upload("test.txt", b"data2");
        
        if let (UploadResult::Success { filename: f1, .. }, UploadResult::Success { filename: f2, .. }) = (result1, result2) {
            assert_ne!(f1, f2);
            assert!(f1.starts_with("test_"));
            assert!(f2.starts_with("test_"));
        } else {
            panic!("Expected Success results");
        }
    }

    #[test]
    fn test_upload_create_directory() {
        let dir = TempDir::new().unwrap();
        let upload_path = dir.path().join("uploads").join("nested");
        let config = UploadConfig::new(upload_path);
        let handler = UploadHandler::new(config);
        
        let result = handler.handle_upload("test.txt", b"data");
        
        assert!(result.is_success());
    }

    #[test]
    fn test_delete() {
        let (handler, _dir) = create_test_handler();
        
        handler.handle_upload("test.txt", b"data");
        
        let result = handler.delete("test.txt");
        assert!(result.is_ok());
        
        // File should be gone
        let result = handler.delete("test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent() {
        let (handler, _dir) = create_test_handler();
        
        let result = handler.delete("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let (handler, _dir) = create_test_handler();
        
        handler.handle_upload("a.txt", b"data");
        handler.handle_upload("b.txt", b"data");
        handler.handle_upload("c.txt", b"data");
        
        let files = handler.list();
        
        assert_eq!(files.len(), 3);
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_list_empty() {
        let (handler, _dir) = create_test_handler();
        
        let files = handler.list();
        assert!(files.is_empty());
    }

    #[test]
    fn test_upload_result_is_success() {
        let success = UploadResult::Success {
            filename: "test.txt".to_string(),
            path: PathBuf::from("/uploads/test.txt"),
            size: 100,
        };
        assert!(success.is_success());
        
        let too_large = UploadResult::TooLarge {
            filename: "test.txt".to_string(),
            size: 200,
            max_size: 100,
        };
        assert!(!too_large.is_success());
    }

    #[test]
    fn test_handle_multipart() {
        let (handler, _dir) = create_test_handler();
        
        let content_type = "multipart/form-data; boundary=boundary";
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello World\r\n\
            --boundary--";
        
        let results = handler.handle_multipart(content_type, data);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_success());
    }

    #[test]
    fn test_handle_multipart_multiple() {
        let (handler, _dir) = create_test_handler();
        
        let content_type = "multipart/form-data; boundary=boundary";
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"file1\"; filename=\"a.txt\"\r\n\
            \r\n\
            content1\r\n\
            --boundary\r\n\
            Content-Disposition: form-data; name=\"file2\"; filename=\"b.txt\"\r\n\
            \r\n\
            content2\r\n\
            --boundary--";
        
        let results = handler.handle_multipart(content_type, data);
        
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_success());
        assert!(results[1].1.is_success());
    }

    #[test]
    fn test_handle_multipart_invalid_content_type() {
        let (handler, _dir) = create_test_handler();
        
        let results = handler.handle_multipart("application/json", b"data");
        
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].1, UploadResult::InvalidRequest(_)));
    }
}
