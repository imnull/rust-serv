use crate::error::{Error, Result};
use hyper::HeaderMap;

/// Compression encoding type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    Gzip,
    Brotli,
    None,
}

/// Extract compression preference from Accept-Encoding header
pub fn parse_accept_encoding(headers: &HeaderMap) -> CompressionType {
    let accept_encoding = match headers.get("Accept-Encoding")
        .and_then(|v| v.to_str().ok()) {
        Some(h) => h,
        None => return CompressionType::None,
    };

    // Check for brotli first (higher preference)
    if accept_encoding.contains("br") {
        return CompressionType::Brotli;
    }

    // Check for gzip
    if accept_encoding.contains("gzip") {
        return CompressionType::Gzip;
    }

    CompressionType::None
}

/// Check if content type should be skipped for compression
pub fn should_skip_compression(content_type: &str) -> bool {
    // Skip compression for already compressed formats, images, and small files
    content_type.starts_with("image/")
        || content_type.starts_with("video/")
        || content_type.starts_with("audio/")
        || content_type == "application/gzip"
        || content_type == "application/x-gzip"
        || content_type == "application/x-brotli"
        || content_type == "application/zip"
        || content_type == "application/x-rar-compressed"
        || content_type == "application/x-7z-compressed"
        || content_type == "application/x-tar"
        || content_type == "application/x-tar-gz"
}

/// Compress data using gzip
pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
        .map_err(|e| Error::Internal(format!("Gzip compression failed: {}", e)))
}

/// Compress data using brotli
pub fn compress_brotli(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;

    let mut compressed = Vec::new();

    // Use brotli compress with default settings
    {
        let quality = 11u32;
        let lgwin = 22u32;
        let mut encoder = brotli::CompressorWriter::new(&mut compressed, 4096, quality, lgwin);
        encoder.write_all(data)
            .map_err(|e| Error::Internal(format!("Brotli compression failed: {}", e)))?;
        encoder.flush()
            .map_err(|e| Error::Internal(format!("Brotli flush failed: {}", e)))?;
        // encoder is dropped here, releasing the borrow on compressed
    }

    Ok(compressed)
}

/// Compress data based on the specified compression type
pub fn compress(data: &[u8], compression_type: CompressionType) -> Result<Vec<u8>> {
    // Don't compress very small data (overhead might be larger than benefit)
    if data.len() < 512 {
        return Ok(data.to_vec());
    }

    match compression_type {
        CompressionType::Gzip => compress_gzip(data),
        CompressionType::Brotli => compress_brotli(data),
        CompressionType::None => Ok(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::HeaderMap;

    #[test]
    fn test_parse_accept_encoding_gzip() {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Encoding", "gzip, deflate".parse().unwrap());
        assert_eq!(parse_accept_encoding(&headers), CompressionType::Gzip);
    }

    #[test]
    fn test_parse_accept_encoding_brotli() {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Encoding", "br, gzip".parse().unwrap());
        assert_eq!(parse_accept_encoding(&headers), CompressionType::Brotli);
    }

    #[test]
    fn test_parse_accept_encoding_none() {
        let headers = HeaderMap::new();
        assert_eq!(parse_accept_encoding(&headers), CompressionType::None);
    }

    #[test]
    fn test_parse_accept_encoding_identity() {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Encoding", "identity".parse().unwrap());
        assert_eq!(parse_accept_encoding(&headers), CompressionType::None);
    }

    #[test]
    fn test_should_skip_compression_images() {
        assert!(should_skip_compression("image/jpeg"));
        assert!(should_skip_compression("image/png"));
    }

    #[test]
    fn test_should_skip_compression_videos() {
        assert!(should_skip_compression("video/mp4"));
        assert!(should_skip_compression("video/webm"));
    }

    #[test]
    fn test_should_skip_compression_audio() {
        assert!(should_skip_compression("audio/mpeg"));
        assert!(should_skip_compression("audio/ogg"));
    }

    #[test]
    fn test_should_skip_compression_compressed() {
        assert!(should_skip_compression("application/gzip"));
        assert!(should_skip_compression("application/zip"));
    }

    #[test]
    fn test_should_compress_text() {
        assert!(!should_skip_compression("text/html"));
        assert!(!should_skip_compression("application/json"));
        assert!(!should_skip_compression("text/css"));
        assert!(!should_skip_compression("text/javascript"));
    }

    #[test]
    fn test_compress_gzip() {
        let data = b"Hello, World! Hello, World! Hello, World!";
        let compressed = compress_gzip(data).unwrap();

        // Compressed data should be smaller
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_gzip_large() {
        let data = b"Hello, World! ".repeat(100);
        let compressed = compress_gzip(&data).unwrap();

        // Compressed data should be significantly smaller for repetitive data
        assert!(compressed.len() < data.len() / 2);
    }

    #[test]
    fn test_compress_brotli() {
        let data = b"Hello, World! Hello, World! Hello, World!";
        let compressed = compress_brotli(data).unwrap();

        // Compressed data should be smaller
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_none() {
        let data = b"Hello, World!";
        let compressed = compress(data, CompressionType::None).unwrap();

        // Should return original data unchanged
        assert_eq!(compressed, data.to_vec());
    }

    #[test]
    fn test_compress_small_data() {
        let data = b"Hi!";
        let compressed = compress(data, CompressionType::Gzip).unwrap();

        // Small data should not be compressed
        assert_eq!(compressed, data.to_vec());
    }

    #[test]
    fn test_compress_repetitive_data() {
        let data = b"ABC ".repeat(1000);
        let compressed = compress_gzip(&data).unwrap();

        // Highly repetitive data compresses very well
        assert!(compressed.len() < data.len() / 10);
    }

    #[test]
    fn test_compress_with_none_type() {
        let data = b"Hello, World! This is a test.";
        let result = compress(data, CompressionType::None);
        assert!(result.is_ok());
        // With None type, data should be returned as-is
        let compressed = result.unwrap();
        assert_eq!(compressed, data.to_vec());
    }
}
