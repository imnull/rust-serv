use mime_guess::Mime;
use std::path::Path;

/// MIME type detector
pub struct MimeDetector;

impl MimeDetector {
    /// Detect MIME type from file path
    pub fn detect(path: &Path) -> Mime {
        mime_guess::from_path(path).first_or_octet_stream()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_html() {
        let mime = MimeDetector::detect(Path::new("test.html"));
        assert_eq!(mime.type_(), "text");
        assert_eq!(mime.subtype(), "html");
    }

    #[test]
    fn test_detect_css() {
        let mime = MimeDetector::detect(Path::new("style.css"));
        assert_eq!(mime.type_(), "text");
        assert_eq!(mime.subtype(), "css");
    }

    #[test]
    fn test_detect_js() {
        let mime = MimeDetector::detect(Path::new("script.js"));
        // mime_guess may return text/javascript or application/javascript
        assert!(matches!(mime.type_().as_str(), "text" | "application"));
    }

    #[test]
    fn test_detect_png() {
        let mime = MimeDetector::detect(Path::new("image.png"));
        assert_eq!(mime.type_(), "image");
        assert_eq!(mime.subtype(), "png");
    }

    #[test]
    fn test_detect_unknown() {
        let mime = MimeDetector::detect(Path::new("unknown.xyz"));
        // mime_guess returns chemical/x-unknown for unknown extensions
        assert!(matches!(mime.type_().as_str(), "application" | "chemical"));
    }

    #[test]
    fn test_detect_jpg() {
        let mime = MimeDetector::detect(Path::new("image.jpg"));
        assert_eq!(mime.type_(), "image");
        assert!(matches!(mime.subtype().as_str(), "jpeg" | "pjpeg"));
    }

    #[test]
    fn test_detect_svg() {
        let mime = MimeDetector::detect(Path::new("image.svg"));
        assert_eq!(mime.type_(), "image");
        assert_eq!(mime.subtype(), "svg");
    }

    #[test]
    fn test_detect_json() {
        let mime = MimeDetector::detect(Path::new("data.json"));
        assert_eq!(mime.type_(), "application");
        assert_eq!(mime.subtype(), "json");
    }

    #[test]
    fn test_detect_no_extension() {
        let mime = MimeDetector::detect(Path::new("README"));
        // No extension should default to octet-stream or similar
        assert!(matches!(mime.type_().as_str(), "application" | "text"));
    }
}
