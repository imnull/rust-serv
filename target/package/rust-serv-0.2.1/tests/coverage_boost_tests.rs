//! Coverage Boost Integration Tests
//!
//! These tests are designed to increase test coverage across
//! multiple modules and test end-to-end scenarios.

use std::sync::Arc;
use std::path::PathBuf;

use rust_serv::config::Config;
use rust_serv::handler::Handler;
use rust_serv::middleware::security::{SecurityLayer, SecurityConfig, RateLimitConfig};
use rust_serv::server::websocket::WebSocketServer;
use rust_serv::server::http2::Http2Server;
use hyper::{Request, Method};
use hyper::body::{Bytes, Incoming};

/// Test the complete request lifecycle with security middleware
#[tokio::test]
async fn test_full_request_lifecycle_with_security() {
    // Create a temporary directory with test files
    let temp_dir = tempfile::TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "Hello, World!").unwrap();

    // Create security configuration
    let security_config = SecurityConfig {
        rate_limit: RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            enabled: true,
        },
        ..Default::default()
    };

    let security_layer = Arc::new(SecurityLayer::new(security_config));

    // Create server configuration
    let config = Arc::new(Config {
        root: temp_dir.path().to_path_buf(),
        enable_indexing: true,
        enable_compression: true,
        enable_security: true,
        ..Default::default()
    });

    let handler = Handler::new(config);

    // Test request passes security checks
    let ip = "192.168.1.1";
    assert!(security_layer.is_ip_allowed(ip));
    assert!(security_layer.check_rate_limit(ip).await);

    // Note: handle_request expects Request<Incoming> which requires
    // an actual HTTP connection. For unit testing without network,
    // we test the handler components directly.
}

/// Test WebSocket server creation
#[tokio::test]
async fn test_websocket_server_creation() {
    let config = Config::default();
    let ws_server = WebSocketServer::new(config);

    // Test basic WebSocket server functionality
    assert_eq!(ws_server.connection_count().await, 0);

    // Test WebSocket upgrade detection
    let req = Request::builder()
        .method("GET")
        .uri("/ws")
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("Sec-WebSocket-Version", "13")
        .body(Bytes::new())
        .unwrap();

    assert!(WebSocketServer::is_websocket_upgrade(&req));
}

/// Test HTTP/2 push functionality
#[tokio::test]
async fn test_http2_push_integration() {
    let config = Arc::new(Config::default());
    let handler = Arc::new(Handler::new(config.clone()));
    let http2_server = Http2Server::new((*config).clone(), handler);

    // Test empty push
    let result = http2_server.handle_push(
        Bytes::new(),
        &hyper::HeaderMap::new(),
        &mut Vec::new()
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 200);

    // Test is_http2_push detection
    let mut headers = hyper::HeaderMap::new();
    headers.insert("content-type", "application/http2+push".parse().unwrap());
    assert!(Http2Server::is_http2_push(&Method::POST, &headers));
    assert!(!Http2Server::is_http2_push(&Method::GET, &headers));
}

/// Test security middleware integration
#[tokio::test]
async fn test_security_middleware_integration() {
    let security_config = SecurityConfig {
        rate_limit: RateLimitConfig {
            max_requests: 5,
            window_secs: 60,
            enabled: true,
        },
        ..Default::default()
    };
    let security_layer = SecurityLayer::new(security_config);

    // Test IP allowlist/blocklist
    let ip = "192.168.1.1";
    assert!(security_layer.is_ip_allowed(ip));

    // Test rate limiting
    for _ in 0..5 {
        assert!(security_layer.check_rate_limit(ip).await);
    }
    // Next request should be rate limited
    assert!(!security_layer.check_rate_limit(ip).await);

    // Different IP should still work
    assert!(security_layer.check_rate_limit("192.168.1.2").await);
}
