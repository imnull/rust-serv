//! Error page handler

use super::templates::ErrorTemplates;

/// Handler for generating error responses
#[derive(Debug, Clone)]
pub struct ErrorPageHandler {
    templates: ErrorTemplates,
}

impl ErrorPageHandler {
    /// Create a new error page handler
    pub fn new() -> Self {
        Self {
            templates: ErrorTemplates::new(),
        }
    }

    /// Create with custom templates
    pub fn with_templates(templates: ErrorTemplates) -> Self {
        Self { templates }
    }

    /// Get mutable templates
    pub fn templates_mut(&mut self) -> &mut ErrorTemplates {
        &mut self.templates
    }

    /// Get templates reference
    pub fn templates(&self) -> &ErrorTemplates {
        &self.templates
    }

    /// Generate error page HTML
    pub fn render(&self, status_code: u16) -> String {
        self.templates
            .get(status_code)
            .map(|t| t.content)
            .unwrap_or_else(|| format!("Error {}", status_code))
    }

    /// Check if custom template exists
    pub fn has_custom(&self, status_code: u16) -> bool {
        self.templates.has_custom(status_code)
    }
}

impl Default for ErrorPageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = ErrorPageHandler::new();
        assert!(!handler.has_custom(404));
    }

    #[test]
    fn test_handler_with_templates() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(404, super::super::templates::ErrorTemplate::new(404, "Custom"));
        
        let handler = ErrorPageHandler::with_templates(templates);
        assert!(handler.has_custom(404));
    }

    #[test]
    fn test_handler_render_default() {
        let handler = ErrorPageHandler::new();
        let html = handler.render(404);
        
        assert!(html.contains("404"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_handler_render_custom() {
        let mut templates = ErrorTemplates::new();
        templates.set_template(404, super::super::templates::ErrorTemplate::new(404, "<html>Custom 404</html>"));
        
        let handler = ErrorPageHandler::with_templates(templates);
        let html = handler.render(404);
        
        assert!(html.contains("Custom 404"));
    }

    #[test]
    fn test_handler_templates_access() {
        let handler = ErrorPageHandler::new();
        assert_eq!(handler.templates().custom_count(), 0);
    }

    #[test]
    fn test_handler_templates_mut_access() {
        let mut handler = ErrorPageHandler::new();
        handler.templates_mut().set_template(500, super::super::templates::ErrorTemplate::new(500, "test"));
        
        assert!(handler.has_custom(500));
    }

    #[test]
    fn test_handler_default() {
        let handler = ErrorPageHandler::default();
        let html = handler.render(500);
        assert!(html.contains("500"));
    }
}
