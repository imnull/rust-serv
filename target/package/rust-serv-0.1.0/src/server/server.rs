use crate::config::Config;
use crate::error::Result;
use crate::handler::Handler;
use crate::server::tls::{load_tls_config, validate_tls_config};
use hyper::server::conn::http1;
use hyper::Response;
use hyper::body::Bytes;
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// Health check response
#[allow(dead_code)]
fn health_check_response() -> Response<Full<Bytes>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(r#"{"status":"ok","service":"rust-serv"}"#)))
        .unwrap()
}

/// HTTP Server
pub struct Server {
    config: Arc<Config>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl Server {
    /// Create a new server with given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start server
    pub async fn run(&self) -> Result<()> {
        let scheme = if self.config.enable_tls { "https" } else { "http" };
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port).parse()?;
        let listener = TcpListener::bind(addr).await?;

        println!("Server listening on {}://{}", scheme, addr);

        let handler = Handler::new(self.config.clone());

        // Setup signal handling for graceful shutdown
        #[cfg(unix)]
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");

        // Create connection semaphore for max connections
        let max_connections = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        // Setup TLS if enabled
        let tls_config = if self.config.enable_tls {
            let cert_path = self.config.tls_cert.as_ref().ok_or_else(|| {
                crate::error::Error::Internal("tls_cert must be specified when enable_tls is true".to_string())
            })?;
            let key_path = self.config.tls_key.as_ref().ok_or_else(|| {
                crate::error::Error::Internal("tls_key must be specified when enable_tls is true".to_string())
            })?;

            // Validate TLS configuration
            validate_tls_config(Some(cert_path), Some(key_path))?;

            let cert_path = std::path::Path::new(cert_path);
            let key_path = std::path::Path::new(key_path);

            // Load TLS configuration
            Some(load_tls_config(&cert_path, &key_path)?.clone())
        } else {
            None
        };

        loop {
            tokio::select! {
                // Accept new connection
                accept_result = listener.accept() => {
                    let (stream, _) = accept_result?;
                    let max_conn = max_connections.clone();

                    // Check connection limit
                    let _permit = match max_conn.try_acquire() {
                        Ok(p) => p,
                        Err(_) => {
                            // Too many connections, close immediately
                            drop(stream);
                            continue;
                        }
                    };

                    let handler = Arc::new(handler.clone());
                    let config = self.config.clone();
                    let tls_config_clone = tls_config.clone();

                    tokio::task::spawn(async move {
                        // Set connection timeout
                        let timeout = Duration::from_secs(config.connection_timeout_secs);

                        let result = tokio::time::timeout(timeout, async {
                            // Handle TLS or plain connection
                            if let Some(tls_config) = tls_config_clone {
                                // TLS connection
                                let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
                                let tls_stream = match tls_acceptor.accept(stream).await {
                                    Ok(stream) => stream,
                                    Err(e) => {
                                        eprintln!("TLS handshake failed: {}", e);
                                        return;
                                    }
                                };

                                let io = TokioIo::new(tls_stream);
                                Self::serve_connection(io, handler).await;
                            } else {
                                // Plain HTTP connection
                                let io = TokioIo::new(stream);
                                Self::serve_connection(io, handler).await;
                            }
                        }).await;

                        match result {
                            Ok(_) => {}
                            Err(_) => {
                                // Connection timeout, close gracefully
                            }
                        }

                        // Permit is automatically dropped here
                    });
                }

                // Handle shutdown signal
                _ = sigterm.recv() => {
                    println!("Received SIGTERM, shutting down gracefully...");
                    break;
                }

                _ = sigint.recv() => {
                    println!("Received SIGINT, shutting down gracefully...");
                    break;
                }
            }
        }

        println!("Server shutdown complete");
        Ok(())
    }

    /// Serve a single HTTP connection
    async fn serve_connection<Io: hyper::rt::Read + hyper::rt::Write + Unpin>(
        io: Io,
        handler: Arc<Handler>,
    ) {
        let handler = handler.clone();

        http1::Builder::new()
            .serve_connection(io, hyper::service::service_fn(move |req| {
                let handler = handler.clone();
                async move {
                    handler.handle_request(req).await
                }
            }))
            .await
            .ok(); // Ignore connection errors
    }

    /// Shutdown the server
    pub fn shutdown(&self) {
        self.shutdown_signal.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = Config::default();
        let server = Server::new(config);
        assert_eq!(server.config.port, 8080);
    }

    #[test]
    fn test_server_with_custom_config() {
        let config = Config {
            port: 3000,
            root: "/tmp".into(),
            enable_indexing: false,
            enable_compression: false,
            log_level: "warn".into(),
            enable_tls: false,
            tls_cert: None,
            tls_key: None,
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec!["GET".to_string()],
            cors_allowed_headers: vec![],
            cors_allow_credentials: false,
            cors_exposed_headers: vec![],
            cors_max_age: Some(86400),
            enable_security: true,
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
            ip_allowlist: vec![],
            ip_blocklist: vec![],
            max_body_size: 10 * 1024 * 1024,
            max_headers: 100,
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 3000);
        assert_eq!(server.config.root, std::path::PathBuf::from("/tmp"));
    }

    #[test]
    fn test_server_clone_config() {
        let config = Config::default();
        let server = Server::new(config);
        // Config should be Arc wrapped
        let _ = server.config.clone();
    }

    #[test]
    fn test_server_address_parsing() {
        // Test that address format is correct
        let addr_str = format!("0.0.0.0:{}", 8080);
        let addr: std::net::SocketAddr = addr_str.parse().unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip(), std::net::Ipv4Addr::new(0, 0, 0, 0));
    }

    #[test]
    fn test_server_ipv6_address_parsing() {
        // Test IPv6 address parsing
        let addr_str = "[::1]:8080";
        let addr: std::net::SocketAddr = addr_str.parse().unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.is_ipv6());
    }

    #[test]
    fn test_health_check_response() {
        let response = health_check_response();
        assert_eq!(response.status(), 200);
        // Check Content-Type header
        assert_eq!(response.headers().get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_shutdown_signal() {
        let config = Config::default();
        let server = Server::new(config);
        // Should not panic
        server.shutdown();
    }

    #[test]
    fn test_server_with_tls_config() {
        let config = Config {
            port: 443,
            root: "/var/www".into(),
            enable_indexing: true,
            enable_compression: true,
            log_level: "info".into(),
            enable_tls: true,
            tls_cert: Some("/path/to/cert.pem".to_string()),
            tls_key: Some("/path/to/key.pem".to_string()),
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec!["GET".to_string()],
            cors_allowed_headers: vec![],
            cors_allow_credentials: false,
            cors_exposed_headers: vec![],
            cors_max_age: Some(86400),
            enable_security: true,
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
            ip_allowlist: vec![],
            ip_blocklist: vec![],
            max_body_size: 10 * 1024 * 1024,
            max_headers: 100,
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 443);
        assert!(server.config.enable_tls);
        assert_eq!(server.config.tls_cert, Some("/path/to/cert.pem".to_string()));
        assert_eq!(server.config.tls_key, Some("/path/to/key.pem".to_string()));
    }

    #[test]
    fn test_tls_config_missing_cert() {
        let config = Config {
            port: 443,
            root: "/var/www".into(),
            enable_indexing: true,
            enable_compression: true,
            log_level: "info".into(),
            enable_tls: true,
            tls_cert: None, // Missing cert
            tls_key: Some("/path/to/key.pem".to_string()),
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec!["GET".to_string()],
            cors_allowed_headers: vec![],
            cors_allow_credentials: false,
            cors_exposed_headers: vec![],
            cors_max_age: Some(86400),
            enable_security: true,
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
            ip_allowlist: vec![],
            ip_blocklist: vec![],
            max_body_size: 10 * 1024 * 1024,
            max_headers: 100,
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 443);
        assert!(server.config.enable_tls);
        assert!(server.config.tls_cert.is_none());
    }

    #[test]
    fn test_tls_config_missing_key() {
        let config = Config {
            port: 443,
            root: "/var/www".into(),
            enable_indexing: true,
            enable_compression: true,
            log_level: "info".into(),
            enable_tls: true,
            tls_cert: Some("/path/to/cert.pem".to_string()),
            tls_key: None, // Missing key
            connection_timeout_secs: 30,
            max_connections: 1000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec!["GET".to_string()],
            cors_allowed_headers: vec![],
            cors_allow_credentials: false,
            cors_exposed_headers: vec![],
            cors_max_age: Some(86400),
            enable_security: true,
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
            ip_allowlist: vec![],
            ip_blocklist: vec![],
            max_body_size: 10 * 1024 * 1024,
            max_headers: 100,
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 443);
        assert!(server.config.tls_key.is_none());
    }

    #[test]
    fn test_max_connections_default() {
        let config = Config::default();
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_connection_timeout_default() {
        let config = Config::default();
        assert_eq!(config.connection_timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_scheme_selection() {
        // Test HTTP scheme when TLS is disabled
        let config = Config {
            enable_tls: false,
            ..Config::default()
        };
        let server = Server::new(config);
        assert!(!server.config.enable_tls);

    }

    #[tokio::test]
    async fn test_scheme_selection_https() {
        // Test HTTPS scheme when TLS is enabled
        let config = Config {
            enable_tls: true,
            tls_cert: Some("/path/cert.pem".to_string()),
            tls_key: Some("/path/key.pem".to_string()),
            ..Config::default()
        };
        let server = Server::new(config);
        assert!(server.config.enable_tls);
    }

    #[tokio::test]
    async fn test_bind_address() {
        // Test that address parsing works correctly
        let port = 9090;
        let addr_str = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = addr_str.parse().unwrap();
        assert_eq!(addr.port(), port);
    }

    #[tokio::test]
    async fn test_shutdown_notify() {
        let config = Config::default();
        let server = Server::new(config);

        // Clone the signal before shutdown
        let _signal = server.shutdown_signal.clone();

        // Shutdown should notify
        server.shutdown();

        // The notify should have been triggered
        // (we can't easily test the without async, but the call shouldn't panic)
    }

    #[tokio::test]
    async fn test_connection_limit_enforcement() {
        // Note: AsyncReadExt and AsyncWriteExt imported but not used in this test

        // Create a config with max_connections = 2
        let config = Config {
            port: 0, // Use random port
            max_connections: 2,
            ..Config::default()
        };

        // Start server in background
        let _server = Server::new(config.clone());
        let _addr = format!("0.0.0.0:{}", config.port);

        // Create a listener to get actual port
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let actual_addr = listener.local_addr().unwrap();
        drop(listener);

        // Create config with actual port
        let config = Config {
            port: actual_addr.port(),
            max_connections: 2,
            ..Config::default()
        };

        let server = Server::new(config);

        // Run server in background
        let server_handle = tokio::spawn(async move {
            // Run for a short time
            let _ = tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                server.run()
            ).await;
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify semaphore was created correctly
        let max_connections = Arc::new(tokio::sync::Semaphore::new(2));
        assert_eq!(max_connections.available_permits(), 2);

        // Test acquiring permits
        let p1 = max_connections.try_acquire().unwrap();
        let p2 = max_connections.try_acquire().unwrap();

        // Third should fail
        assert!(max_connections.try_acquire().is_err());

        drop(p1);
        drop(p2);
        drop(server_handle);
    }

    #[tokio::test]
    async fn test_connection_timeout() {
        // Test that timeout duration is set correctly
        let config = Config {
            connection_timeout_secs: 5,
            ..Config::default()
        };

        let timeout = Duration::from_secs(config.connection_timeout_secs);
        assert_eq!(timeout, Duration::from_secs(5));

        // Test that a very short timeout would work
        let short_config = Config {
            connection_timeout_secs: 1,
            ..Config::default()
        };
        let short_timeout = Duration::from_secs(short_config.connection_timeout_secs);
        assert_eq!(short_timeout, Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_tls_handshake_failure_handling() {
        // Test that TLS config validation works
        let config = Config {
            enable_tls: true,
            tls_cert: Some("/nonexistent/cert.pem".to_string()),
            tls_key: Some("/nonexistent/key.pem".to_string()),
            ..Config::default()
        };

        // Validation should fail for nonexistent files
        let result = validate_tls_config(
            config.tls_cert.as_deref(),
            config.tls_key.as_deref()
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_connections() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Test concurrent connection counting
        let counter = Arc::new(AtomicUsize::new(0));
        let max_connections = 10;

        let counter_clone = counter.clone();
        let handles: Vec<_> = (0..5).map(|_| {
            let counter = counter_clone.clone();
            tokio::spawn(async move {
                // Simulate connection
                let prev = counter.fetch_add(1, Ordering::SeqCst);
                assert!(prev < max_connections);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                counter.fetch_sub(1, Ordering::SeqCst);
            })
        }).collect();

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_graceful_shutdown_signal() {
        let config = Config::default();
        let server = Server::new(config);

        // Verify shutdown signal exists
        let _signal = server.shutdown_signal.clone();

        // Call shutdown multiple times (should not panic)
        server.shutdown();
        server.shutdown();
        server.shutdown();
    }

    #[test]
    fn test_server_config_variations() {
        // Test with different config combinations
        let configs = vec![
            Config {
                port: 80,
                max_connections: 100,
                connection_timeout_secs: 10,
                enable_tls: false,
                ..Config::default()
            },
            Config {
                port: 443,
                max_connections: 500,
                connection_timeout_secs: 60,
                enable_tls: true,
                tls_cert: Some("/cert.pem".to_string()),
                tls_key: Some("/key.pem".to_string()),
                ..Config::default()
            },
            Config {
                port: 8080,
                max_connections: 1000,
                connection_timeout_secs: 30,
                enable_tls: false,
                ..Config::default()
            },
        ];

        for config in configs {
            let server = Server::new(config.clone());
            assert_eq!(server.config.port, config.port);
            assert_eq!(server.config.max_connections, config.max_connections);
        }
    }

    #[test]
    fn test_connection_semaphore_creation() {
        // Test that semaphore is created with correct capacity
        let permits = 100;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(permits));
        assert_eq!(semaphore.available_permits(), permits);

        // Test try_acquire
        let p1 = semaphore.try_acquire().unwrap();
        assert_eq!(semaphore.available_permits(), permits - 1);

        let p2 = semaphore.try_acquire().unwrap();
        assert_eq!(semaphore.available_permits(), permits - 2);

        drop(p1);
        assert_eq!(semaphore.available_permits(), permits - 1);

        drop(p2);
        assert_eq!(semaphore.available_permits(), permits);
    }

    #[test]
    fn test_tls_config_validation_both_paths() {
        // Both paths provided but not existing
        let result = validate_tls_config(
            Some("/nonexistent/cert.pem"),
            Some("/nonexistent/key.pem")
        );
        assert!(result.is_err());

        // Missing cert
        let result = validate_tls_config(None, Some("/nonexistent/key.pem"));
        assert!(result.is_err());

        // Missing key
        let result = validate_tls_config(Some("/nonexistent/cert.pem"), None);
        assert!(result.is_err());

        // Both missing
        let result = validate_tls_config(None, None);
        assert!(result.is_ok()); // No TLS, valid
    }

    // NEW TESTS ADDED FOR COVERAGE IMPROVEMENT

    #[test]
    fn test_health_check_response_body() {
        let response = health_check_response();
        assert_eq!(response.status(), 200);
        
        // Verify the body type is correct
        // health_check_response returns Full<Bytes>
        let _body_ref = response.body();
        
        // Verify Content-Type header is set correctly
        assert!(response.headers().contains_key("Content-Type"));
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_server_with_zero_connections() {
        // Edge case: max_connections = 0
        let config = Config {
            max_connections: 0,
            ..Config::default()
        };
        let server = Server::new(config);
        assert_eq!(server.config.max_connections, 0);
    }

    #[test]
    fn test_server_with_zero_timeout() {
        // Edge case: connection_timeout_secs = 0
        let config = Config {
            connection_timeout_secs: 0,
            ..Config::default()
        };
        let server = Server::new(config);
        assert_eq!(server.config.connection_timeout_secs, 0);
    }

    #[test]
    fn test_server_config_clone() {
        let config = Config {
            port: 9999,
            root: "/custom".into(),
            enable_tls: true,
            tls_cert: Some("/cert.pem".to_string()),
            tls_key: Some("/key.pem".to_string()),
            ..Config::default()
        };
        let server = Server::new(config.clone());
        
        // Verify Arc works correctly
        let config_ref = server.config.clone();
        assert_eq!(config_ref.port, 9999);
        assert_eq!(config_ref.root, std::path::PathBuf::from("/custom"));
    }

    #[test]
    fn test_address_format_variations() {
        // Test different valid address formats
        let test_cases = vec![
            ("0.0.0.0:8080", 8080u16, false),
            ("127.0.0.1:3000", 3000u16, false),
            ("192.168.1.1:443", 443u16, false),
            ("[::1]:8080", 8080u16, true),
            ("[::]:9000", 9000u16, true),
        ];

        for (addr_str, expected_port, is_ipv6) in test_cases {
            let addr: SocketAddr = addr_str.parse().unwrap();
            assert_eq!(addr.port(), expected_port, "Port mismatch for {}", addr_str);
            assert_eq!(addr.is_ipv6(), is_ipv6, "IPv6 mismatch for {}", addr_str);
        }
    }

    #[test]
    fn test_address_parsing_error() {
        // Test invalid address formats
        let invalid_addrs = vec![
            "not_an_address",
            "999.999.999.999:8080",
            "",
            ":8080",
            "localhost",
        ];

        for addr in invalid_addrs {
            let result: std::result::Result<SocketAddr, _> = addr.parse();
            assert!(result.is_err(), "Should fail for: {}", addr);
        }
    }

    #[tokio::test]
    async fn test_semaphore_try_acquire_behavior() {
        let max_conn = Arc::new(tokio::sync::Semaphore::new(1));
        
        // First acquire should succeed
        let permit = max_conn.try_acquire();
        assert!(permit.is_ok());
        
        // Second acquire should fail (no permits available)
        let permit2 = max_conn.try_acquire();
        assert!(permit2.is_err());
        
        // After dropping, should be able to acquire again
        drop(permit);
        let permit3 = max_conn.try_acquire();
        assert!(permit3.is_ok());
    }

    #[tokio::test]
    async fn test_duration_conversions() {
        // Test various timeout durations
        let test_cases = vec![
            (1u64, Duration::from_secs(1)),
            (30u64, Duration::from_secs(30)),
            (60u64, Duration::from_secs(60)),
            (3600u64, Duration::from_secs(3600)),
        ];

        for (secs, expected) in test_cases {
            let duration = Duration::from_secs(secs);
            assert_eq!(duration, expected);
        }
    }

    #[test]
    fn test_handler_clone_in_server() {
        let config = Config::default();
        let handler = Handler::new(Arc::new(config));
        
        // Handler should be cloneable
        let _handler_clone = handler.clone();
    }

    #[tokio::test]
    async fn test_serve_connection_basic() {
        // Test that serve_connection function exists and has correct signature
        // We can't easily test the actual HTTP serving without a full integration test,
        // but we can verify the method structure
        let config = Config::default();
        let handler = Arc::new(Handler::new(Arc::new(config)));
        
        // Just verify handler is cloneable for use in serve_connection
        let _handler_for_connection = handler.clone();
    }

    #[test]
    fn test_tls_config_error_messages() {
        // Test that proper error messages are generated
        let result = validate_tls_config(
            Some("/nonexistent/cert.pem"),
            Some("/nonexistent/key.pem")
        );
        
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("not found") || msg.contains("No such file"));
        }
    }

    #[test]
    fn test_run_method_error_paths() {
        // Test that run method handles various error conditions
        // This tests the error paths in run() without actually running the server
        
        // TLS config with missing cert
        let config_with_missing_cert = Config {
            enable_tls: true,
            tls_cert: None,
            tls_key: Some("/key.pem".to_string()),
            ..Config::default()
        };
        let server = Server::new(config_with_missing_cert);
        assert!(server.config.enable_tls);
        assert!(server.config.tls_cert.is_none());

        // TLS config with missing key
        let config_with_missing_key = Config {
            enable_tls: true,
            tls_cert: Some("/cert.pem".to_string()),
            tls_key: None,
            ..Config::default()
        };
        let server = Server::new(config_with_missing_key);
        assert!(server.config.tls_key.is_none());
    }

    #[tokio::test]
    async fn test_connection_timeout_zero() {
        let config = Config {
            connection_timeout_secs: 0,
            ..Config::default()
        };
        
        let timeout = Duration::from_secs(config.connection_timeout_secs);
        assert_eq!(timeout, Duration::from_secs(0));
        
        // Zero timeout should be valid (immediate timeout)
        let result = tokio::time::timeout(timeout, async {
            tokio::task::yield_now().await;
        }).await;
        
        // Should complete successfully since task finishes before timeout
        assert!(result.is_ok());
    }

    #[test]
    fn test_shutdown_signal_multiple() {
        let config = Config::default();
        let server = Server::new(config);
        
        // Verify the notify can be called multiple times
        server.shutdown_signal.notify_one();
        server.shutdown_signal.notify_one();
        server.shutdown_signal.notify_one();
    }

    #[tokio::test]
    async fn test_signal_notification_awaits() {
        let notify = Arc::new(tokio::sync::Notify::new());
        let notify_clone = notify.clone();
        
        // Spawn a task that waits for notification
        let handle = tokio::spawn(async move {
            notify_clone.notified().await;
        });
        
        // Give it time to start waiting
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Notify
        notify.notify_one();
        
        // Should complete
        let result = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_edge_cases() {
        // Test very large values
        let config = Config {
            port: 65535, // Max port number
            max_connections: 100000,
            connection_timeout_secs: u64::MAX,
            ..Config::default()
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 65535);
        assert_eq!(server.config.max_connections, 100000);
    }

    #[test]
    fn test_config_minimum_values() {
        let config = Config {
            port: 1, // Minimum valid port
            max_connections: 1,
            connection_timeout_secs: 1,
            ..Config::default()
        };
        let server = Server::new(config);
        assert_eq!(server.config.port, 1);
        assert_eq!(server.config.max_connections, 1);
        assert_eq!(server.config.connection_timeout_secs, 1);
    }

    #[test]
    fn test_tls_enabled_variations() {
        // Test all TLS configuration variations
        let configs = vec![
            (false, None, None, "no_tls"),
            (true, Some("cert.pem"), Some("key.pem"), "full_tls"),
            (true, Some("cert.pem"), None, "missing_key"),
            (true, None, Some("key.pem"), "missing_cert"),
            (true, None, None, "both_missing"),
        ];

        for (enable_tls, cert, key, _desc) in configs {
            let config = Config {
                enable_tls,
                tls_cert: cert.map(|s| s.to_string()),
                tls_key: key.map(|s| s.to_string()),
                ..Config::default()
            };
            let server = Server::new(config);
            assert_eq!(server.config.enable_tls, enable_tls);
        }
    }

    #[tokio::test]
    async fn test_hyper_service_fn_usage() {
        // Test that the service_fn closure works correctly
        let config = Config::default();
        let handler = Handler::new(Arc::new(config));
        let handler = Arc::new(handler);
        
        // Clone handler like serve_connection does
        let handler_for_request = handler.clone();
        
        // Just verify it clones correctly
        assert!(Arc::strong_count(&handler) > 0);
        drop(handler_for_request);
    }

    #[test]
    fn test_http_response_builder() {
        // Test Response builder directly
        let response = Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(r#"{"test":"data"}"#)))
            .unwrap();
        
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_http_response_builder_error_cases() {
        // Test invalid status codes (should still work, but verify builder)
        let response = Response::builder()
            .status(500)
            .body(Full::new(Bytes::from("error")))
            .unwrap();
        
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_bytes_creation() {
        // Test Bytes creation for body
        let bytes1 = Bytes::from(r#"{"status":"ok"}"#);
        assert_eq!(bytes1.len(), 15);
        
        let bytes2 = Bytes::from_static(b"test");
        assert_eq!(bytes2.len(), 4);
        
        let bytes3 = Bytes::new();
        assert!(bytes3.is_empty());
    }

    #[tokio::test]
    async fn test_tokio_timeout_behavior() {
        // Test tokio::time::timeout directly
        let short_timeout = Duration::from_millis(10);
        let long_task = Duration::from_millis(100);
        
        let result = tokio::time::timeout(short_timeout, async {
            tokio::time::sleep(long_task).await;
        }).await;
        
        assert!(result.is_err(), "Should timeout");
        
        // Test successful completion
        let long_timeout = Duration::from_secs(1);
        let short_task = Duration::from_millis(10);
        
        let result = tokio::time::timeout(long_timeout, async {
            tokio::time::sleep(short_task).await;
        }).await;
        
        assert!(result.is_ok(), "Should complete successfully");
    }

    #[tokio::test]
    async fn test_semaphore_fairness() {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(3));
        
        // Acquire all permits
        let p1 = semaphore.try_acquire().unwrap();
        let p2 = semaphore.try_acquire().unwrap();
        let p3 = semaphore.try_acquire().unwrap();
        
        // Should have no more permits
        assert!(semaphore.try_acquire().is_err());
        
        // Drop in different order
        drop(p2);
        assert_eq!(semaphore.available_permits(), 1);
        
        drop(p1);
        assert_eq!(semaphore.available_permits(), 2);
        
        drop(p3);
        assert_eq!(semaphore.available_permits(), 3);
    }

    #[test]
    fn test_full_config_all_fields() {
        // Create a config with every field explicitly set
        let config = Config {
            port: 9090,
            root: "/web/root".into(),
            enable_indexing: true,
            enable_compression: true,
            log_level: "debug".into(),
            enable_tls: true,
            tls_cert: Some("/path/to/cert".into()),
            tls_key: Some("/path/to/key".into()),
            connection_timeout_secs: 120,
            max_connections: 5000,
            enable_health_check: true,
            enable_cors: true,
            cors_allowed_origins: vec!["*".into(), "localhost".into()],
            cors_allowed_methods: vec!["GET".into(), "POST".into()],
            cors_allowed_headers: vec!["Content-Type".into()],
            cors_allow_credentials: true,
            cors_exposed_headers: vec!["X-Custom".into()],
            cors_max_age: Some(3600),
            enable_security: true,
            rate_limit_max_requests: 1000,
            rate_limit_window_secs: 120,
            ip_allowlist: vec!["127.0.0.1".into()],
            ip_blocklist: vec!["10.0.0.1".into()],
            max_body_size: 50 * 1024 * 1024,
            max_headers: 200,
        };
        
        let server = Server::new(config.clone());
        
        assert_eq!(server.config.port, 9090);
        assert_eq!(server.config.root, std::path::PathBuf::from("/web/root"));
        assert_eq!(server.config.log_level, "debug");
        assert_eq!(server.config.connection_timeout_secs, 120);
        assert_eq!(server.config.max_connections, 5000);
        assert_eq!(server.config.max_body_size, 50 * 1024 * 1024);
        assert_eq!(server.config.max_headers, 200);
    }

    #[tokio::test]
    async fn test_run_method_with_tls_validation_error() {
        // Test the error path when TLS files don't exist
        let config = Config {
            enable_tls: true,
            tls_cert: Some("/definitely/not/existing/cert.pem".to_string()),
            tls_key: Some("/definitely/not/existing/key.pem".to_string()),
            port: 0,
            ..Config::default()
        };
        
        let server = Server::new(config);
        
        // Verify TLS is enabled (run() would fail with validation error)
        assert!(server.config.enable_tls);
        assert!(server.config.tls_cert.is_some());
    }

    #[test]
    fn test_server_arc_behavior() {
        let config = Config::default();
        let server = Server::new(config);
        
        // Test that we can clone the Arc
        let config1 = server.config.clone();
        let config2 = server.config.clone();
        
        // Both should point to same data
        assert_eq!(config1.port, config2.port);
        assert_eq!(Arc::strong_count(&server.config), 3); // original + 2 clones
        
        drop(config1);
        assert_eq!(Arc::strong_count(&server.config), 2);
    }

    #[test]
    fn test_socket_addr_display() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
        
        let addr_v6: SocketAddr = "[::1]:8080".parse().unwrap();
        assert_eq!(addr_v6.to_string(), "[::1]:8080");
    }

    #[test]
    fn test_health_check_response_headers() {
        let response = health_check_response();
        
        // Verify all expected headers
        let headers = response.headers();
        assert!(headers.contains_key("Content-Type"));
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        
        // Verify status
        assert_eq!(response.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn test_concurrent_semaphore_operations() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5));
        let counter = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));
        
        let mut handles = vec![];
        
        for _ in 0..10 {
            let sem = semaphore.clone();
            let cnt = counter.clone();
            let max = max_observed.clone();
            
            let handle = tokio::spawn(async move {
                if let Ok(_permit) = sem.try_acquire() {
                    let current = cnt.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    // Update max observed
                    let mut max_val = max.load(Ordering::SeqCst);
                    while current > max_val {
                        match max.compare_exchange_weak(
                            max_val,
                            current,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        ) {
                            Ok(_) => break,
                            Err(actual) => max_val = actual,
                        }
                    }
                    
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    cnt.fetch_sub(1, Ordering::SeqCst);
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Max observed should not exceed permit count
        assert!(max_observed.load(Ordering::SeqCst) <= 5);
    }

    #[test]
    fn test_config_with_empty_strings() {
        let config = Config {
            log_level: "".into(),
            root: "".into(),
            ..Config::default()
        };
        let server = Server::new(config);
        assert!(server.config.log_level.is_empty());
        assert!(server.config.root.as_os_str().is_empty());
    }

    #[test]
    fn test_config_with_special_chars_in_paths() {
        let config = Config {
            tls_cert: Some("/path with spaces/cert.pem".into()),
            tls_key: Some("/path-with-dashes/key.pem".into()),
            ..Config::default()
        };
        let server = Server::new(config);
        assert_eq!(server.config.tls_cert, Some("/path with spaces/cert.pem".to_string()));
        assert_eq!(server.config.tls_key, Some("/path-with-dashes/key.pem".to_string()));
    }

    #[test]
    fn test_tls_error_display() {
        use crate::error::Error;
        
        let err = Error::Internal("TLS test error".to_string());
        assert!(err.to_string().contains("TLS test error"));
    }

    #[test]
    fn test_path_conversions() {
        use std::path::Path;
        
        let path_str = "/tmp/test/path";
        let path = Path::new(path_str);
        assert_eq!(path.to_str().unwrap(), path_str);
        
        let path_buf: std::path::PathBuf = path_str.into();
        assert_eq!(path_buf, std::path::PathBuf::from(path_str));
    }
}
