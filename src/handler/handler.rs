use crate::error::Error;
use crate::file_service::{FileService, FileMetadata};
use crate::handler::RangeRequest;
use crate::mime_types::MimeDetector;
use crate::path_security::PathValidator;
use crate::utils::format_bytes;
use crate::Config;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use std::time::SystemTime;

/// HTTP request handler
#[derive(Clone)]
pub struct Handler {
    config: Arc<Config>,
    path_validator: PathValidator,
}

impl Handler {
    pub fn new(config: Arc<Config>) -> Self {
        let path_validator = PathValidator::new(config.root.clone());
        Self {
            config,
            path_validator,
        }
    }

    pub async fn handle_request(&self, req: Request<Incoming>) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let path = req.uri().path();

        // Remove leading slash and URL decode
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        let decoded_path = urlencoding::decode(clean_path).unwrap_or_else(|_| clean_path.to_string().into());

        let full_path = self.config.root.join(decoded_path.as_ref());

        // Check if it's a directory - if so, try index.html or serve directory listing
        if FileService::is_directory(&full_path) {
            let index_path = full_path.join("index.html");
            if index_path.exists() {
                return self.serve_file(&index_path, req).await;
            }
            // Serve directory listing if enabled
            if self.config.enable_indexing {
                return Ok(self.serve_directory_index(&full_path));
            }
            return Ok(self.serve_not_found());
        }

        // Validate path security
        match self.path_validator.validate(&full_path) {
            Ok(_) => self.serve_file(&full_path, req).await,
            Err(Error::NotFound(_)) => Ok(self.serve_not_found()),
            Err(Error::Forbidden(_)) => Ok(self.serve_forbidden()),
            Err(_) => Ok(self.serve_internal_error()),
        }
    }

    async fn serve_file(&self, path: &PathBuf, req: Request<Incoming>) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        // Check for Range header
        let range_header = req.headers().get("Range")
            .and_then(|v| v.to_str().ok());

        // Check for If-None-Match header (for ETag cache validation)
        let if_none_match = req.headers().get("If-None-Match")
            .and_then(|v| v.to_str().ok());

        match FileService::read_file(path) {
            Ok(content) => {
                let file_size = content.len() as u64;

                // Generate ETag and Last-Modified based on file size and modification time
                let etag = self.generate_etag(path, file_size);

                // If-None-Match validation
                if let Some(if_none_match) = if_none_match {
                    if if_none_match == etag {
                        // ETag matches, return 304 Not Modified
                        let last_modified = self.generate_last_modified(path);
                        return Ok(Response::builder()
                            .status(304)
                            .header("Content-Type", MimeDetector::detect(path).to_string())
                            .header("ETag", etag)
                            .header("Last-Modified", last_modified)
                            .body(Full::new(Bytes::new()))
                            .unwrap());
                    }
                }

                // Handle range request
                if let Some(range_header) = range_header {
                    match RangeRequest::parse(range_header, file_size) {
                        Ok(Some(range)) => {
                            // Return 206 Partial Content
                            let byte_range = range.to_range();
                            let range_content = content[byte_range.clone()].to_vec();
                            let range_size = range_content.len() as u64;

                            let content_range_str = format!("bytes {}-{}/{}",
                                range.start, range.end.unwrap_or(file_size - 1), file_size);
                            let last_modified = self.generate_last_modified(path);

                            return Ok(Response::builder()
                                .status(206)
                                .header("Content-Type", MimeDetector::detect(path).to_string())
                                .header("Content-Range", content_range_str)
                                .header("Content-Length", range_size.to_string())
                                .header("Accept-Ranges", "bytes")
                                .header("ETag", etag)
                                .header("Last-Modified", last_modified)
                                .header("Cache-Control", "public, max-age=86400")
                                .body(Full::new(Bytes::from(range_content)))
                                .unwrap());
                        }
                        Ok(None) => {
                            // Invalid range, return full content
                            return Ok(self.serve_file_with_etag(path, content, etag));
                        }
                        Err(_) => {
                            // Parse error, return full content
                            return Ok(self.serve_file_with_etag(path, content, etag));
                        }
                    }
                } else {
                    // No range header, return full content
                    Ok(self.serve_file_with_etag(path, content, etag))
                }
            }
            Err(Error::NotFound(_)) => Ok(self.serve_not_found()),
            Err(Error::Forbidden(_)) => Ok(self.serve_forbidden()),
            Err(_) => Ok(self.serve_internal_error()),
        }
    }

    /// Generate ETag for a file based on size and modification time
    fn generate_etag(&self, path: &PathBuf, file_size: u64) -> String {
        let modified = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let modified_secs = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!(r#""{}-{}""#, modified_secs, file_size)
    }

    /// Generate Last-Modified header value
    fn generate_last_modified(&self, path: &PathBuf) -> String {
        let modified = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let duration = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        let datetime = time::OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64)
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

        // Manual formatting for RFC 2822
        let format = time::format_description::parse("[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT")
            .expect("Invalid format description");
        datetime
            .format(&format)
            .unwrap_or_else(|_| "Thu, 01 Jan 1970 00:00:00 GMT".to_string())
    }

    fn serve_file_with_etag(&self, path: &PathBuf, content: Vec<u8>, etag: String) -> Response<Full<Bytes>> {
        let mime = MimeDetector::detect(path);
        let last_modified = self.generate_last_modified(path);
        Response::builder()
            .status(200)
            .header("Content-Type", mime.to_string())
            .header("Content-Length", content.len().to_string())
            .header("Accept-Ranges", "bytes")
            .header("ETag", etag)
            .header("Last-Modified", last_modified)
            .header("Cache-Control", "public, max-age=86400")
            .body(Full::new(Bytes::from(content)))
            .unwrap()
    }

    fn serve_not_found(&self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from("404 Not Found")))
            .unwrap()
    }

    fn serve_forbidden(&self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(403)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from("403 Forbidden")))
            .unwrap()
    }

    fn serve_internal_error(&self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(500)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from("500 Internal Server Error")))
            .unwrap()
    }

    fn serve_directory_index(&self, path: &PathBuf) -> Response<Full<Bytes>> {
        let files = match FileService::list_directory(path) {
            Ok(files) => files,
            Err(_) => return self.serve_internal_error(),
        };

        let html = self.generate_directory_html(path, &files);
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap()
    }

    fn generate_directory_html(&self, path: &PathBuf, files: &[FileMetadata]) -> String {
        let path_str = path.display();
        let mut html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Index of {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ margin-bottom: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #f2f2f2; }}
        a {{ color: #0066cc; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Index of {}</h1>
    <table>
        <thead>
            <tr>
                <th>Name</th>
                <th>Type</th>
                <th>Size</th>
            </tr>
        </thead>
        <tbody>
"#, path_str, path_str);

        // Add parent directory link if not at root
        if path != &self.config.root {
            html.push_str(&format!(
                r#"<tr>
                <td><a href="../">../</a></td>
                <td>Directory</td>
                <td>-</td>
            </tr>"#
            ));
        }

        for file in files {
            let name = &file.name;
            let file_type = if file.is_dir { "Directory" } else { "File" };
            let size = if file.is_dir { "-" } else { &format_bytes(file.size) };

            let link = if file.is_dir {
                format!("{}/", name)
            } else {
                name.clone()
            };

            html.push_str(&format!(
                r#"<tr>
                <td><a href="{}">{}</a></td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
                link, name, file_type, size
            ));
        }

        html.push_str(r#"
        </tbody>
    </table>
</body>
</html>"#);

        html
    }
}

/// Handle incoming HTTP request (backward compatible function)
pub async fn handle_request(
    req: Request<Incoming>,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    let config = Arc::new(Config::default());
    let handler = Handler::new(config);
    handler.handle_request(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);
        // Handler should be created successfully
        assert_eq!(handler.config.port, 8080);
    }

    #[test]
    fn test_serve_not_found() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let response = handler.serve_not_found();
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_serve_forbidden() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let response = handler.serve_forbidden();
        assert_eq!(response.status(), 403);
    }

    #[test]
    fn test_serve_internal_error() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let response = handler.serve_internal_error();
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_generate_etag() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let file_size = std::fs::metadata(&test_file).unwrap().len();
        let etag = handler.generate_etag(&test_file, file_size);

        // ETag should be in format "timestamp-filesize"
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
    }

    #[test]
    fn test_handler_with_custom_config() {
        let config = Arc::new(Config {
            port: 3000,
            root: "/tmp".into(),
            enable_indexing: true,
            enable_compression: false,
            log_level: "info".into(),
            enable_tls: false,
            tls_cert: None,
            tls_key: None,
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
        });
        let handler = Handler::new(config);

        assert_eq!(handler.config.port, 3000);
        assert_eq!(handler.config.root, PathBuf::from("/tmp"));
        assert!(handler.config.enable_indexing);
        assert!(!handler.config.enable_compression);
    }

    #[test]
    fn test_serve_file_with_etag() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let content = std::fs::read(&test_file).unwrap();
        let etag = handler.generate_etag(&test_file, content.len() as u64);
        let etag_clone = etag.clone();
        let response = handler.serve_file_with_etag(&test_file, content, etag);

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("ETag").unwrap().to_str().unwrap(), etag_clone);
    }

    #[test]
    fn test_generate_etag_nonexistent_file() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        // Should still work even for nonexistent files (returns default time)
        let etag = handler.generate_etag(&nonexistent_file, 100);

        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
    }

    #[test]
    fn test_generate_last_modified() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "content").unwrap();

        let last_modified = handler.generate_last_modified(&test_file);

        assert!(last_modified.contains("GMT"));
        assert!(last_modified.len() > 0);
    }

    #[test]
    fn test_generate_directory_html_empty() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let files: Vec<FileMetadata> = vec![];

        let html = handler.generate_directory_html(&temp_dir.path().to_path_buf(), &files);

        assert!(html.contains("Index of"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_generate_directory_html_with_files() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let files = vec![
            FileMetadata {
                path: temp_dir.path().join("file1.txt").display().to_string(),
                name: "file1.txt".to_string(),
                size: 100,
                is_dir: false,
            },
            FileMetadata {
                path: temp_dir.path().join("dir1").display().to_string(),
                name: "dir1".to_string(),
                size: 0,
                is_dir: true,
            },
        ];

        let html = handler.generate_directory_html(&temp_dir.path().to_path_buf(), &files);

        assert!(html.contains("file1.txt"));
        assert!(html.contains("dir1"));
        assert!(html.contains("File"));
        assert!(html.contains("Directory"));
        assert!(html.contains("100.00 B"));
    }

    #[test]
    fn test_handler_root_path() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        assert_eq!(handler.config.root, PathBuf::from("."));
    }

    #[test]
    fn test_serve_file_with_etag_includes_headers() {
        let config = Arc::new(Config::default());
        let handler = Handler::new(config);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.html");
        std::fs::write(&test_file, "<html>test</html>").unwrap();

        let content = std::fs::read(&test_file).unwrap();
        let etag = handler.generate_etag(&test_file, content.len() as u64);
        let response = handler.serve_file_with_etag(&test_file, content, etag);

        assert!(response.headers().get("Content-Type").is_some());
        assert!(response.headers().get("Content-Length").is_some());
        assert!(response.headers().get("ETag").is_some());
        assert!(response.headers().get("Accept-Ranges").is_some());
    }
}
