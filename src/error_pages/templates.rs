//! Error page templates

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Error page template
#[derive(Debug, Clone)]
pub struct ErrorTemplate {
    /// HTTP status code
    pub status_code: u16,
    /// HTML content
    pub content: String,
    /// Content-Type header
    pub content_type: String,
}

impl ErrorTemplate {
    /// Create a new error template
    pub fn new(status_code: u16, content: impl Into<String>) -> Self {
        Self {
            status_code,
            content: content.into(),
            content_type: "text/html; charset=utf-8".to_string(),
        }
    }

    /// Create with custom content type
    pub fn with_content_type(status_code: u16, content: impl Into<String>, content_type: impl Into<String>) -> Self {
        Self {
            status_code,
            content: content.into(),
            content_type: content_type.into(),
        }
    }

    /// Create default template for a status code
    pub fn default_for(status_code: u16) -> Self {
        let (title, message) = get_default_error_message(status_code);
        let content = generate_default_html(status_code, &title, &message);
        Self::new(status_code, content)
    }
}

/// Get default error message for status code
fn get_default_error_message(status_code: u16) -> (String, String) {
    match status_code {
        400 => ("Bad Request".to_string(), "The request could not be understood by the server.".to_string()),
        401 => ("Unauthorized".to_string(), "Authentication is required to access this resource.".to_string()),
        403 => ("Forbidden".to_string(), "You don't have permission to access this resource.".to_string()),
        404 => ("Not Found".to_string(), "The requested resource could not be found on this server.".to_string()),
        405 => ("Method Not Allowed".to_string(), "The request method is not supported for this resource.".to_string()),
        408 => ("Request Timeout".to_string(), "The server timed out waiting for the request.".to_string()),
        413 => ("Payload Too Large".to_string(), "The request entity is larger than the server is willing to process.".to_string()),
        414 => ("URI Too Long".to_string(), "The URL requested is too long for the server to process.".to_string()),
        429 => ("Too Many Requests".to_string(), "You have sent too many requests in a given amount of time.".to_string()),
        500 => ("Internal Server Error".to_string(), "The server encountered an unexpected condition that prevented it from fulfilling the request.".to_string()),
        501 => ("Not Implemented".to_string(), "The server does not support the functionality required to fulfill the request.".to_string()),
        502 => ("Bad Gateway".to_string(), "The server received an invalid response from an upstream server.".to_string()),
        503 => ("Service Unavailable".to_string(), "The server is currently unable to handle the request due to temporary overload or maintenance.".to_string()),
        504 => ("Gateway Timeout".to_string(), "The server did not receive a timely response from an upstream server.".to_string()),
        _ => ("Error".to_string(), format!("An error occurred (HTTP {}).", status_code)),
    }
}

/// Generate default HTML error page
fn generate_default_html(status_code: u16, title: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Rust Serv</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }}
        .container {{
            text-align: center;
            background: white;
            padding: 60px;
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 500px;
        }}
        .status-code {{
            font-size: 120px;
            font-weight: bold;
            color: #764ba2;
            margin: 0;
            line-height: 1;
        }}
        .title {{
            font-size: 24px;
            color: #333;
            margin: 20px 0 10px;
        }}
        .message {{
            font-size: 16px;
            color: #666;
            margin: 0;
        }}
        .footer {{
            margin-top: 30px;
            font-size: 14px;
            color: #999;
        }}
    </style>
</head>
<body>
    <div class="container">
        <p class="status-code">{}</p>
        <h1 class="title">{}</h1>
        <p class="message">{}</p>
        <p class="footer">Powered by Rust Serv</p>
    </div>
</body>
</html>"#,
        title, status_code, title, message
    )
}

/// Error templates manager
#[derive(Debug, Clone)]
pub struct ErrorTemplates {
    /// Custom templates by status code
    templates: HashMap<u16, ErrorTemplate>,
    /// Custom template files by status code
    template_files: HashMap<u16, PathBuf>,
}

impl ErrorTemplates {
    /// Create a new error templates manager
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            template_files: HashMap::new(),
        }
    }

    /// Set custom template for a status code
    pub fn set_template(&mut self, status_code: u16, template: ErrorTemplate) {
        self.templates.insert(status_code, template);
    }

    /// Set custom template file for a status code
    pub fn set_template_file(&mut self, status_code: u16, path: impl Into<PathBuf>) {
        self.template_files.insert(status_code, path.into());
    }

    /// Get template for a status code
    pub fn get(&self, status_code: u16) -> Option<ErrorTemplate> {
        if let Some(template) = self.templates.get(&status_code) {
            return Some(template.clone());
        }
        
        if let Some(path) = self.template_files.get(&status_code) {
            if let Ok(content) = std::fs::read_to_string(path) {
                return Some(ErrorTemplate::new(status_code, content));
            }
        }
        
        // Return default template
        Some(ErrorTemplate::default_for(status_code))
    }

    /// Check if custom template exists
    pub fn has_custom(&self, status_code: u16) -> bool {
        self.templates.contains_key(&status_code) || self.template_files.contains_key(&status_code)
    }

    /// Remove custom template
    pub fn remove(&mut self, status_code: u16) -> bool {
        self.templates.remove(&status_code).is_some() || self.template_files.remove(&status_code).is_some()
    }

    /// Clear all custom templates
    pub fn clear(&mut self) {
        self.templates.clear();
        self.template_files.clear();
    }

    /// Get count of custom templates
    pub fn custom_count(&self) -> usize {
        self.templates.len() + self.template_files.len()
    }
}

impl Default for ErrorTemplates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_error_template_creation() {
        let template = ErrorTemplate::new(404, "<html>Not Found</html>");
        assert_eq!(template.status_code, 404);
        assert!(template.content.contains("Not Found"));
        assert_eq!(template.content_type, "text/html; charset=utf-8");
    }

    #[test]
    fn test_error_template_custom_content_type() {
        let template = ErrorTemplate::with_content_type(404, "{}", "application/json");
        assert_eq!(template.content_type, "application/json");
    }

    #[test]
    fn test_default_template_404() {
        let template = ErrorTemplate::default_for(404);
        assert_eq!(template.status_code, 404);
        assert!(template.content.contains("404"));
        assert!(template.content.contains("Not Found"));
    }

    #[test]
    fn test_default_template_500() {
        let template = ErrorTemplate::default_for(500);
        assert_eq!(template.status_code, 500);
        assert!(template.content.contains("500"));
        assert!(template.content.contains("Internal Server Error"));
    }

    #[test]
    fn test_default_template_unknown() {
        let template = ErrorTemplate::default_for(999);
        assert_eq!(template.status_code, 999);
        assert!(template.content.contains("999"));
    }

    #[test]
    fn test_templates_creation() {
        let templates = ErrorTemplates::new();
        assert_eq!(templates.custom_count(), 0);
    }

    #[test]
    fn test_templates_set_and_get() {
        let mut templates = ErrorTemplates::new();
        let template = ErrorTemplate::new(404, "<html>Custom 404</html>");
        templates.set_template(404, template);
        
        assert!(templates.has_custom(404));
        assert_eq!(templates.custom_count(), 1);
        
        let retrieved = templates.get(404).unwrap();
        assert!(retrieved.content.contains("Custom 404"));
    }

    #[test]
    fn test_templates_get_default() {
        let templates = ErrorTemplates::new();
        
        // Should return default template even if not set
        let template = templates.get(404).unwrap();
        assert_eq!(template.status_code, 404);
        assert!(!templates.has_custom(404));
    }

    #[test]
    fn test_templates_remove() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(404, ErrorTemplate::new(404, "test"));
        
        assert!(templates.remove(404));
        assert!(!templates.has_custom(404));
        assert_eq!(templates.custom_count(), 0);
    }

    #[test]
    fn test_templates_remove_nonexistent() {
        let mut templates = ErrorTemplates::new();
        assert!(!templates.remove(999));
    }

    #[test]
    fn test_templates_clear() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(404, ErrorTemplate::new(404, "test"));
        templates.set_template(500, ErrorTemplate::new(500, "test"));
        
        templates.clear();
        assert_eq!(templates.custom_count(), 0);
    }

    #[test]
    fn test_templates_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("404.html");
        std::fs::write(&file_path, "<html>Custom File 404</html>").unwrap();
        
        let mut templates = ErrorTemplates::new();
        templates.set_template_file(404, &file_path);
        
        assert!(templates.has_custom(404));
        
        let template = templates.get(404).unwrap();
        assert!(template.content.contains("Custom File 404"));
    }

    #[test]
    fn test_templates_file_not_found() {
        let mut templates = ErrorTemplates::new();
        templates.set_template_file(404, "/nonexistent/404.html");
        
        // Should fall back to default
        let template = templates.get(404).unwrap();
        assert!(template.content.contains("404"));
    }

    #[test]
    fn test_default_error_messages() {
        let (title, msg) = get_default_error_message(400);
        assert_eq!(title, "Bad Request");
        
        let (title, msg) = get_default_error_message(401);
        assert_eq!(title, "Unauthorized");
        
        let (title, msg) = get_default_error_message(403);
        assert_eq!(title, "Forbidden");
        
        let (title, msg) = get_default_error_message(405);
        assert_eq!(title, "Method Not Allowed");
        
        let (title, msg) = get_default_error_message(429);
        assert_eq!(title, "Too Many Requests");
        
        let (title, msg) = get_default_error_message(503);
        assert_eq!(title, "Service Unavailable");
    }

    #[test]
    fn test_generate_default_html() {
        let html = generate_default_html(404, "Not Found", "Test message");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("Test message"));
        assert!(html.contains("Rust Serv"));
    }

    #[test]
    fn test_templates_default() {
        let templates = ErrorTemplates::default();
        assert_eq!(templates.custom_count(), 0);
    }

    #[test]
    fn test_multiple_templates() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(400, ErrorTemplate::new(400, "Bad Request"));
        templates.set_template(404, ErrorTemplate::new(404, "Not Found"));
        templates.set_template(500, ErrorTemplate::new(500, "Server Error"));
        
        assert_eq!(templates.custom_count(), 3);
        assert!(templates.has_custom(400));
        assert!(templates.has_custom(404));
        assert!(templates.has_custom(500));
    }

    #[test]
    fn test_overwrite_template() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(404, ErrorTemplate::new(404, "First"));
        templates.set_template(404, ErrorTemplate::new(404, "Second"));
        
        let template = templates.get(404).unwrap();
        assert!(template.content.contains("Second"));
        assert_eq!(templates.custom_count(), 1);
    }
}
