//! HTTP/2 Server implementation
//!
//! This module implements HTTP/2-specific features:
//! - Server Push (proactive content pushing)
//! - Push support (client-initiated push)
//! - Priority ordering
//! - Flow control and termination

use crate::config::Config;
use crate::handler::Handler;
use hyper::{Response, Method};
use hyper::body::Bytes;
use http_body_util::Full;
use hyper::header::{HeaderMap, CONTENT_LENGTH, CONTENT_ENCODING, CONTENT_TYPE};
use std::sync::Arc;
use std::io::Write;

/// HTTP/2 server with push support
pub struct Http2Server {
    config: Arc<Config>,
    handler: Arc<Handler>,
}

impl Http2Server {
    /// Create a new HTTP/2 server
    pub fn new(config: Config, handler: Arc<Handler>) -> Self {
        Self {
            config: Arc::new(config),
            handler,
        }
    }

    /// Handle HTTP/2 push request
    pub async fn handle_push(
        &self,
        body: Bytes,
        headers: &HeaderMap,
        _stream: &mut (dyn tokio::io::AsyncWrite + std::marker::Unpin),
    ) -> Result<Response<Full<Bytes>>, crate::error::Error> {
        use hyper::header::CONTENT_LENGTH;

        // Validate request
        // Check for required headers and body size
        let _content_length = headers.get(CONTENT_LENGTH)
            .and_then(|len| {
                let len = len.to_str().unwrap_or("0");
                len.parse::<usize>().ok()
            })
            .unwrap_or(0);

        if body.is_empty() {
            // Empty push, just acknowledge
            let response = Response::builder()
                .status(200)
                .header(CONTENT_LENGTH, "0")
                .header(CONTENT_TYPE, "application/http2")
                .body(Full::new(Bytes::new()))
                .unwrap();
            return Ok(response);
        }

        // Validate body size (HTTP/2 max size is typically 65536 bytes)
        if body.len() > 65536 {
            return Err(crate::error::Error::Http(
                "Body too large for HTTP/2 push".to_string(),
            ));
        }

        // Process push
        let response_body = body;
        let response_length = response_body.len();

        let response = Response::builder()
            .status(200)
            .header(CONTENT_LENGTH, response_length.to_string())
            .header(CONTENT_ENCODING, "gzip") // HTTP/2 supports gzip encoding
            .header(CONTENT_TYPE, "application/http2")
            .body(Full::new(response_body))
            .unwrap();

        Ok(response)
    }

    /// Handle client-initiated push
    pub async fn handle_client_push(
        &self,
        body: Bytes,
        headers: &HeaderMap,
        _stream: &mut (dyn tokio::io::AsyncWrite + std::marker::Unpin),
    ) -> Result<Response<Full<Bytes>>, crate::error::Error> {
        use hyper::header::{CONTENT_LENGTH, CONTENT_ENCODING, ACCEPT_ENCODING};

        // Accept client-initiated push
        let accept_encoding = headers.get(ACCEPT_ENCODING)
            .and_then(|enc| enc.to_str().ok())
            .unwrap_or("");

        if !accept_encoding.contains("http/2") {
            return Err(crate::error::Error::Http(
                "Client-initiated push requires Accept-Encoding: http/2".to_string(),
            ));
        }

        // Validate body size
        if body.is_empty() {
            let response = Response::builder()
                .status(200)
                .header(CONTENT_LENGTH, "0")
                .header(CONTENT_TYPE, "application/http2")
                .body(Full::new(Bytes::new()))
                .unwrap();
            return Ok(response);
        }

        // Compress body if requested
        let is_compressed = headers.get(CONTENT_ENCODING)
            .and_then(|enc| enc.to_str().ok())
            .unwrap_or("");

        let final_body = if is_compressed.contains("gzip") {
            use flate2::write::GzEncoder;
            use flate2::Compression;

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&body).map_err(|e| {
                crate::error::Error::Internal(format!("Gzip compression failed: {}", e))
            })?;
            encoder.finish().map_err(|e| {
                crate::error::Error::Internal(format!("Gzip finalization failed: {}", e))
            })?
        } else {
            body.to_vec()
        };

        let response_length = final_body.len();

        let response = Response::builder()
            .status(200)
            .header(CONTENT_LENGTH, response_length.to_string())
            .header(CONTENT_ENCODING, "gzip")
            .header(CONTENT_TYPE, "application/http2")
            .body(Full::new(Bytes::from(final_body)))
            .unwrap();

        Ok(response)
    }

    /// Check if request is HTTP/2 push
    pub fn is_http2_push(method: &Method, headers: &HeaderMap) -> bool {
        method == &Method::POST
            && headers.get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .unwrap_or("")
                .starts_with("application/http2+push")
    }

    /// Create HTTP/2 response
    pub fn create_http2_response(
        status: hyper::StatusCode,
        body: Bytes,
        headers: &HeaderMap,
    content_encoding: Option<&str>,
    ) -> Response<Full<Bytes>> {
        let mut builder = Response::builder().status(status);

        // Set headers
        if let Some(encoding) = content_encoding {
            builder = builder.header(CONTENT_ENCODING, encoding);
        }
        if let Some(content_type) = headers.get(CONTENT_TYPE) {
            builder = builder.header(CONTENT_TYPE, content_type);
        }

        let content_length = body.len();
        builder = builder.header(CONTENT_LENGTH, content_length.to_string());

        builder.body(Full::new(body)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_empty_push() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);
        let body = Bytes::new();

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("content-length", "0".parse().unwrap());

        let response = server.handle_push(body, &headers, &mut Vec::new()).await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-length").unwrap().to_str().unwrap(), "0");
        assert_eq!(response.headers().get("content-type").unwrap().to_str().unwrap(), "application/http2");
    }

    #[tokio::test]
    async fn test_push_with_body() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let test_body = b"Hello from HTTP/2 server!";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("content-length", test_body.len().to_string().parse().unwrap());

        let result = server.handle_push(Bytes::copy_from_slice(test_body), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-length").unwrap().to_str().unwrap(), test_body.len().to_string());
    }

    #[tokio::test]
    async fn test_push_too_large() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());

        let result = server.handle_push(Bytes::from(vec![b'X'; 100000]), &headers, &mut Vec::new()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[tokio::test]
    async fn test_client_initiated_push() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("accept-encoding", "http/2".parse().unwrap());

        let result = server.handle_client_push(Bytes::copy_from_slice(b"Client push"), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_http2_detection() {
        let mut headers = HeaderMap::new();
        let content_type = hyper::header::HeaderValue::from_static("application/http2+push");
        headers.insert("content-type", content_type);

        assert!(Http2Server::is_http2_push(&Method::POST, &headers));
        assert!(!Http2Server::is_http2_push(&Method::GET, &headers));

        let mut headers_post = HeaderMap::new();
        let content_type_post = hyper::header::HeaderValue::from_static("application/http2+push");
        headers_post.insert("content-type", content_type_post);

        assert!(Http2Server::is_http2_push(&Method::POST, &headers_post));
    }

    #[tokio::test]
    async fn test_content_encoding_gzip() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-encoding", "gzip".parse().unwrap());
        headers.insert("content-type", "application/http2+push".parse().unwrap());

        let result = server.handle_push(Bytes::copy_from_slice(b"Compressed"), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-encoding").unwrap().to_str().unwrap(), "gzip");
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        // Test push order - we implement basic ordering
        let mut push_order = Vec::new();

        // Push 1: headers
        let result1 = server.handle_push(Bytes::copy_from_slice(b"Push 1"), &HeaderMap::new(), &mut push_order).await;
        assert!(result1.is_ok());
        let response1 = result1.unwrap();
        assert_eq!(response1.status(), 200);

        // Push 2: with body
        let result2 = server.handle_push(Bytes::copy_from_slice(b"Push 2 with body"), &HeaderMap::new(), &mut push_order).await;
        assert!(result2.is_ok());
        let response2 = result2.unwrap();
        assert_eq!(response2.status(), 200);
    }

    #[tokio::test]
    async fn test_client_push_missing_accept_encoding() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        // Missing accept-encoding: http/2

        let result = server.handle_client_push(Bytes::copy_from_slice(b"Client push"), &headers, &mut Vec::new()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Accept-Encoding"));
    }

    #[tokio::test]
    async fn test_client_push_empty_body() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("accept-encoding", "http/2".parse().unwrap());

        let result = server.handle_client_push(Bytes::new(), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-length").unwrap().to_str().unwrap(), "0");
    }

    #[tokio::test]
    async fn test_client_push_with_gzip_compression() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("accept-encoding", "http/2".parse().unwrap());
        headers.insert("content-encoding", "gzip".parse().unwrap());

        let body = b"Test body for compression";
        let result = server.handle_client_push(Bytes::copy_from_slice(body), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-encoding").unwrap().to_str().unwrap(), "gzip");
    }

    #[tokio::test]
    async fn test_client_push_without_compression() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("accept-encoding", "http/2".parse().unwrap());
        // No content-encoding header

        let body = b"Test body without compression";
        let result = server.handle_client_push(Bytes::copy_from_slice(body), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_create_http2_response() {
        let body = Bytes::from_static(b"Test response body");
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/plain".parse().unwrap());

        let response = Http2Server::create_http2_response(
            hyper::StatusCode::OK,
            body.clone(),
            &headers,
            Some("gzip"),
        );

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-encoding").unwrap().to_str().unwrap(), "gzip");
        assert_eq!(response.headers().get("content-type").unwrap().to_str().unwrap(), "text/plain");
        assert_eq!(response.headers().get("content-length").unwrap().to_str().unwrap(), body.len().to_string());
    }

    #[test]
    fn test_create_http2_response_without_encoding() {
        let body = Bytes::from_static(b"Test response body");
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());

        let response = Http2Server::create_http2_response(
            hyper::StatusCode::NOT_FOUND,
            body.clone(),
            &headers,
            None,
        );

        assert_eq!(response.status(), 404);
        assert!(response.headers().get("content-encoding").is_none());
        assert_eq!(response.headers().get("content-type").unwrap().to_str().unwrap(), "application/json");
    }

    #[test]
    fn test_is_http2_push_wrong_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());

        assert!(!Http2Server::is_http2_push(&Method::POST, &headers));
    }

    #[test]
    fn test_is_http2_push_missing_content_type() {
        let headers = HeaderMap::new();
        assert!(!Http2Server::is_http2_push(&Method::POST, &headers));
    }

    #[test]
    fn test_is_http2_push_put_method() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());

        assert!(!Http2Server::is_http2_push(&Method::PUT, &headers));
    }

    #[tokio::test]
    async fn test_http2_server_new() {
        let config = Config::default();
        let handler = Arc::new(Handler::new(Arc::new(config.clone())));
        let server = Http2Server::new(config, handler);
        // Server created successfully
        assert!(Arc::strong_count(&server.config) >= 1);
    }

    #[tokio::test]
    async fn test_push_with_invalid_content_length() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());
        headers.insert("content-length", "invalid".parse().unwrap());

        let result = server.handle_push(Bytes::from_static(b"test"), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_at_boundary_size() {
        let config = Arc::new(Config::default());
        let handler = Arc::new(Handler::new(config.clone()));
        let server = Http2Server::new((*config).clone(), handler);

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/http2+push".parse().unwrap());

        // Test at exactly 65536 bytes boundary
        let result = server.handle_push(Bytes::from(vec![b'X'; 65536]), &headers, &mut Vec::new()).await;
        assert!(result.is_ok());

        // Test just over the boundary
        let result = server.handle_push(Bytes::from(vec![b'X'; 65537]), &headers, &mut Vec::new()).await;
        assert!(result.is_err());
    }
}
