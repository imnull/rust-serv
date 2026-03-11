//! Multipart form-data parser

/// A single part from multipart form data
#[derive(Debug, Clone)]
pub struct MultipartPart {
    /// Field name
    pub name: String,
    /// Filename (if file upload)
    pub filename: Option<String>,
    /// Content type
    pub content_type: String,
    /// Content data
    pub data: Vec<u8>,
}

impl MultipartPart {
    /// Create a new multipart part
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            filename: None,
            content_type: "application/octet-stream".to_string(),
            data,
        }
    }

    /// Set filename
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Check if this is a file upload
    pub fn is_file(&self) -> bool {
        self.filename.is_some()
    }

    /// Get data size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Multipart parser
#[derive(Debug, Clone)]
pub struct MultipartParser {
    /// Boundary string
    boundary: String,
}

impl MultipartParser {
    /// Create a new multipart parser
    pub fn new(boundary: impl Into<String>) -> Self {
        Self {
            boundary: boundary.into(),
        }
    }

    /// Extract boundary from Content-Type header
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        // Content-Type: multipart/form-data; boundary=----WebKitFormBoundaryXXX
        let boundary_start = content_type.find("boundary=")?;
        let boundary_str = &content_type[boundary_start + 9..];
        
        // Extract boundary (may be quoted)
        let boundary = if boundary_str.starts_with('"') {
            // Quoted boundary - find matching end quote
            let rest = &boundary_str[1..]; // Skip opening quote
            if let Some(end_pos) = rest.find('"') {
                rest[..end_pos].to_string()
            } else {
                // No closing quote found, use trimmed value
                rest.trim().trim_end_matches('"').to_string()
            }
        } else {
            // Unquoted boundary - take until semicolon or end
            boundary_str.split(';').next()?.trim().to_string()
        };
        
        Some(Self { boundary })
    }

    /// Get the boundary
    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    /// Parse multipart data
    pub fn parse(&self, data: &[u8]) -> Result<Vec<MultipartPart>, String> {
        let mut parts = Vec::new();
        
        // Find all boundaries
        let boundary_bytes = format!("--{}", self.boundary);
        let mut positions = Vec::new();
        
        for i in 0..data.len().saturating_sub(boundary_bytes.len()) {
            if &data[i..i + boundary_bytes.len()] == boundary_bytes.as_bytes() {
                positions.push(i);
            }
        }
        
        // Parse segments between boundaries
        for i in 0..positions.len() {
            let start = positions[i] + boundary_bytes.len();
            let end = if i + 1 < positions.len() {
                positions[i + 1]
            } else {
                // Last segment - find end marker or use data end
                data.len()
            };
            
            // Skip if this is the end marker
            let segment_data = &data[start..end];
            if segment_data.starts_with(b"--") {
                continue;
            }
            
            if let Some(part) = self.parse_part_bytes(segment_data) {
                parts.push(part);
            }
        }
        
        Ok(parts)
    }
    
    /// Parse a single part from bytes
    fn parse_part_bytes(&self, segment: &[u8]) -> Option<MultipartPart> {
        // Remove leading/trailing whitespace (CRLF)
        let segment = if segment.starts_with(b"\r\n") {
            &segment[2..]
        } else {
            segment
        };
        
        // Find header end (double CRLF)
        let header_end_marker = b"\r\n\r\n";
        let header_end = segment.windows(header_end_marker.len())
            .position(|w| w == header_end_marker)?;
        
        let headers = &segment[..header_end];
        let data_start = header_end + 4;
        let data = &segment[data_start..];
        
        // Remove trailing \r\n from data
        let data = if data.ends_with(b"\r\n") {
            &data[..data.len() - 2]
        } else {
            data
        };
        
        let headers_str = std::str::from_utf8(headers).ok()?;
        let name = self.extract_header_value(headers_str, "name")?;
        let filename = self.extract_header_value(headers_str, "filename");
        let content_type = self.extract_content_type(headers_str);
        
        let mut part = MultipartPart::new(name, data.to_vec());
        
        if let Some(fname) = filename {
            part = part.with_filename(fname);
        }
        
        if let Some(ct) = content_type {
            part = part.with_content_type(ct);
        }
        
        Some(part)
    }

    /// Extract a value from Content-Disposition header
    fn extract_header_value(&self, headers: &str, key: &str) -> Option<String> {
        let search = format!("{}=\"", key);
        
        for line in headers.lines() {
            if line.contains("Content-Disposition") {
                if let Some(start) = line.find(&search) {
                    let value_start = start + search.len();
                    if let Some(end) = line[value_start..].find('"') {
                        return Some(line[value_start..value_start + end].to_string());
                    }
                }
            }
        }
        
        None
    }

    /// Extract Content-Type from headers
    fn extract_content_type(&self, headers: &str) -> Option<String> {
        for line in headers.lines() {
            if line.starts_with("Content-Type:") {
                return Some(line[13..].trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_part_creation() {
        let part = MultipartPart::new("field", vec![1, 2, 3]);
        assert_eq!(part.name, "field");
        assert_eq!(part.data, vec![1, 2, 3]);
        assert_eq!(part.content_type, "application/octet-stream");
    }

    #[test]
    fn test_multipart_part_with_filename() {
        let part = MultipartPart::new("file", vec![])
            .with_filename("test.txt");
        
        assert_eq!(part.filename, Some("test.txt".to_string()));
        assert!(part.is_file());
    }

    #[test]
    fn test_multipart_part_with_content_type() {
        let part = MultipartPart::new("text", vec![])
            .with_content_type("text/plain");
        
        assert_eq!(part.content_type, "text/plain");
    }

    #[test]
    fn test_multipart_part_is_file() {
        let file_part = MultipartPart::new("file", vec![])
            .with_filename("test.txt");
        assert!(file_part.is_file());
        
        let field_part = MultipartPart::new("field", vec![]);
        assert!(!field_part.is_file());
    }

    #[test]
    fn test_multipart_part_size() {
        let part = MultipartPart::new("data", vec![1, 2, 3, 4, 5]);
        assert_eq!(part.size(), 5);
    }

    #[test]
    fn test_multipart_parser_creation() {
        let parser = MultipartParser::new("----WebKitFormBoundary");
        assert_eq!(parser.boundary(), "----WebKitFormBoundary");
    }

    #[test]
    fn test_from_content_type() {
        let content_type = "multipart/form-data; boundary=----WebKitFormBoundary";
        let parser = MultipartParser::from_content_type(content_type);
        
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().boundary(), "----WebKitFormBoundary");
    }

    #[test]
    fn test_from_content_type_quoted() {
        let content_type = "multipart/form-data; boundary=\"----WebKitFormBoundary\"";
        let parser = MultipartParser::from_content_type(content_type);
        
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().boundary(), "----WebKitFormBoundary");
    }

    #[test]
    fn test_from_content_type_no_boundary() {
        let content_type = "multipart/form-data";
        let parser = MultipartParser::from_content_type(content_type);
        
        assert!(parser.is_none());
    }

    #[test]
    fn test_from_content_type_invalid() {
        let content_type = "application/json";
        let parser = MultipartParser::from_content_type(content_type);
        
        assert!(parser.is_none());
    }

    #[test]
    fn test_parse_simple() {
        let parser = MultipartParser::new("boundary");
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"field\"\r\n\
            \r\n\
            value\r\n\
            --boundary--";
        
        let parts = parser.parse(data).unwrap();
        
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "field");
        assert_eq!(parts[0].data, b"value");
    }

    #[test]
    fn test_parse_file() {
        let parser = MultipartParser::new("boundary");
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            Hello World\r\n\
            --boundary--";
        
        let parts = parser.parse(data).unwrap();
        
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "file");
        assert_eq!(parts[0].filename, Some("test.txt".to_string()));
        assert_eq!(parts[0].content_type, "text/plain");
        assert!(parts[0].is_file());
    }

    #[test]
    fn test_parse_multiple_parts() {
        let parser = MultipartParser::new("boundary");
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"field1\"\r\n\
            \r\n\
            value1\r\n\
            --boundary\r\n\
            Content-Disposition: form-data; name=\"field2\"\r\n\
            \r\n\
            value2\r\n\
            --boundary--";
        
        let parts = parser.parse(data).unwrap();
        
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "field1");
        assert_eq!(parts[1].name, "field2");
    }

    #[test]
    fn test_parse_binary_data() {
        let parser = MultipartParser::new("boundary");
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"file\"; filename=\"test.bin\"\r\n\
            Content-Type: application/octet-stream\r\n\
            \r\n\
            \x00\x01\x02\x03\xff\r\n\
            --boundary--";
        
        let parts = parser.parse(data).unwrap();
        
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].data, b"\x00\x01\x02\x03\xff");
    }

    #[test]
    fn test_parse_empty_field() {
        let parser = MultipartParser::new("boundary");
        let data = b"--boundary\r\n\
            Content-Disposition: form-data; name=\"empty\"\r\n\
            \r\n\
            \r\n\
            --boundary--";
        
        let parts = parser.parse(data).unwrap();
        
        assert_eq!(parts.len(), 1);
        assert!(parts[0].data.is_empty());
    }
}
