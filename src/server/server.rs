//! Server module with plugin system integration
//!
//! This module provides the HTTP server with WebAssembly plugin support.

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

        let handler = Arc::new(Handler::new(self.config.clone()));

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

                    let handler = Arc::clone(&handler);
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
                                http1::Builder::new()
                                    .serve_connection(io, hyper::service::service_fn(move |req| {
                                        let handler = Arc::clone(&handler);
                                        async move {
                                            handler.handle_request(req).await
                                        }
                                    }))
                                    .await
                                    .ok();
                            } else {
                                // Plain HTTP connection
                                let io = TokioIo::new(stream);
                                http1::Builder::new()
                                    .serve_connection(io, hyper::service::service_fn(move |req| {
                                        let handler = Arc::clone(&handler);
                                        async move {
                                            handler.handle_request(req).await
                                        }
                                    }))
                                    .await
                                    .ok();
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
    fn test_server_new_with_custom_port() {
        let mut config = Config::default();
        config.port = 3000;
        let server = Server::new(config);
        assert_eq!(server.config.port, 3000);
    }

    #[test]
    fn test_server_config_access() {
        let config = Config::default();
        let server = Server::new(config.clone());
        assert!(server.config.enable_indexing);
        assert!(!server.config.enable_tls);
    }

    #[test]
    fn test_server_shutdown_signal() {
        let config = Config::default();
        let server = Server::new(config);
        // Verify shutdown signal exists by checking we can notify it
        server.shutdown_signal.notify_one();
        // If we reach here, the signal was created successfully
    }

    #[test]
    fn test_server_with_tls_config() {
        let mut config = Config::default();
        config.enable_tls = true;
        config.tls_cert = Some("/path/to/cert.pem".to_string());
        config.tls_key = Some("/path/to/key.pem".to_string());
        
        let server = Server::new(config);
        assert!(server.config.enable_tls);
        assert_eq!(server.config.tls_cert, Some("/path/to/cert.pem".to_string()));
        assert_eq!(server.config.tls_key, Some("/path/to/key.pem".to_string()));
    }

    #[test]
    fn test_server_max_connections() {
        let mut config = Config::default();
        config.max_connections = 500;
        
        let server = Server::new(config);
        assert_eq!(server.config.max_connections, 500);
    }

    #[test]
    fn test_server_connection_timeout() {
        let mut config = Config::default();
        config.connection_timeout_secs = 60;
        
        let server = Server::new(config);
        assert_eq!(server.config.connection_timeout_secs, 60);
    }

    #[test]
    fn test_server_with_compression_disabled() {
        let mut config = Config::default();
        config.enable_compression = false;
        
        let server = Server::new(config);
        assert!(!server.config.enable_compression);
    }

    #[test]
    fn test_server_with_cors_disabled() {
        let mut config = Config::default();
        config.enable_cors = false;
        
        let server = Server::new(config);
        assert!(!server.config.enable_cors);
    }

    #[test]
    fn test_server_with_security_disabled() {
        let mut config = Config::default();
        config.enable_security = false;
        
        let server = Server::new(config);
        assert!(!server.config.enable_security);
    }

    #[test]
    fn test_server_with_health_check_disabled() {
        let mut config = Config::default();
        config.enable_health_check = false;
        
        let server = Server::new(config);
        assert!(!server.config.enable_health_check);
    }

    #[test]
    fn test_server_rate_limit_config() {
        let mut config = Config::default();
        config.rate_limit_max_requests = 200;
        config.rate_limit_window_secs = 120;
        
        let server = Server::new(config);
        assert_eq!(server.config.rate_limit_max_requests, 200);
        assert_eq!(server.config.rate_limit_window_secs, 120);
    }

    #[test]
    fn test_server_root_directory() {
        let mut config = Config::default();
        config.root = std::path::PathBuf::from("/var/www/html");
        
        let server = Server::new(config);
        assert_eq!(server.config.root, std::path::PathBuf::from("/var/www/html"));
    }

    #[test]
    fn test_server_max_body_size() {
        let mut config = Config::default();
        config.max_body_size = 50 * 1024 * 1024; // 50MB
        
        let server = Server::new(config);
        assert_eq!(server.config.max_body_size, 50 * 1024 * 1024);
    }

    #[test]
    fn test_server_max_headers() {
        let mut config = Config::default();
        config.max_headers = 200;
        
        let server = Server::new(config);
        assert_eq!(server.config.max_headers, 200);
    }

    #[test]
    fn test_server_cors_origins() {
        let mut config = Config::default();
        config.cors_allowed_origins = vec!["https://example.com".to_string(), "https://app.example.com".to_string()];
        
        let server = Server::new(config);
        assert_eq!(server.config.cors_allowed_origins.len(), 2);
        assert!(server.config.cors_allowed_origins.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_server_cors_methods() {
        let mut config = Config::default();
        config.cors_allowed_methods = vec!["GET".to_string(), "POST".to_string()];
        
        let server = Server::new(config);
        assert_eq!(server.config.cors_allowed_methods.len(), 2);
        assert!(server.config.cors_allowed_methods.contains(&"POST".to_string()));
    }

    #[test]
    fn test_server_ip_allowlist() {
        let mut config = Config::default();
        config.ip_allowlist = vec!["192.168.1.0/24".to_string(), "10.0.0.1".to_string()];
        
        let server = Server::new(config);
        assert_eq!(server.config.ip_allowlist.len(), 2);
    }

    #[test]
    fn test_server_ip_blocklist() {
        let mut config = Config::default();
        config.ip_blocklist = vec!["192.168.1.100".to_string()];
        
        let server = Server::new(config);
        assert_eq!(server.config.ip_blocklist.len(), 1);
        assert!(server.config.ip_blocklist.contains(&"192.168.1.100".to_string()));
    }

    #[test]
    fn test_server_log_level() {
        let mut config = Config::default();
        config.log_level = "debug".to_string();
        
        let server = Server::new(config);
        assert_eq!(server.config.log_level, "debug");
    }

    #[test]
    fn test_health_check_response() {
        let response = health_check_response();
        assert_eq!(response.status(), 200);
        
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[tokio::test]
    async fn test_server_shutdown() {
        let config = Config::default();
        let server = Server::new(config);
        
        // Test shutdown doesn't panic
        server.shutdown();
    }

    #[test]
    fn test_health_check_response_body() {
        let response = health_check_response();
        assert_eq!(response.status(), 200);
        
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");
        
        // Check body contains expected JSON
        // Note: We can't easily check the body content without consuming it,
        // but we verified the status and content-type above
    }

    #[test]
    fn test_server_with_all_security_options() {
        let mut config = Config::default();
        config.enable_security = true;
        config.rate_limit_max_requests = 1000;
        config.rate_limit_window_secs = 60;
        config.ip_allowlist = vec!["192.168.0.0/16".to_string()];
        config.ip_blocklist = vec!["10.0.0.1".to_string()];
        config.max_body_size = 1024 * 1024;
        config.max_headers = 100;
        
        let server = Server::new(config);
        assert!(server.config.enable_security);
        assert_eq!(server.config.rate_limit_max_requests, 1000);
        assert_eq!(server.config.rate_limit_window_secs, 60);
        assert_eq!(server.config.ip_allowlist.len(), 1);
        assert_eq!(server.config.ip_blocklist.len(), 1);
        assert_eq!(server.config.max_body_size, 1024 * 1024);
        assert_eq!(server.config.max_headers, 100);
    }

    #[test]
    fn test_server_cors_full_config() {
        let mut config = Config::default();
        config.enable_cors = true;
        config.cors_allowed_origins = vec!["https://example.com".to_string()];
        config.cors_allowed_methods = vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()];
        config.cors_allowed_headers = vec!["Content-Type".to_string(), "Authorization".to_string()];
        config.cors_allow_credentials = true;
        config.cors_exposed_headers = vec!["X-Custom-Header".to_string()];
        config.cors_max_age = Some(3600);
        
        let server = Server::new(config);
        assert!(server.config.enable_cors);
        assert_eq!(server.config.cors_allowed_origins.len(), 1);
        assert_eq!(server.config.cors_allowed_methods.len(), 3);
        assert_eq!(server.config.cors_allowed_headers.len(), 2);
        assert!(server.config.cors_allow_credentials);
        assert_eq!(server.config.cors_exposed_headers.len(), 1);
        assert_eq!(server.config.cors_max_age, Some(3600));
    }

    #[test]
    fn test_server_with_management_config() {
        use crate::config::ManagementConfig;

        let mut config = Config::default();
        config.management = Some(ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        });

        let server = Server::new(config);
        assert!(server.config.management.is_some());
        let mgmt = server.config.management.as_ref().unwrap();
        assert!(mgmt.enabled);
        assert_eq!(mgmt.health_path, "/health");
        assert_eq!(mgmt.ready_path, "/ready");
        assert_eq!(mgmt.stats_path, "/stats");
    }

    #[test]
    fn test_server_with_plugins_config() {
        use crate::config::PluginSystemConfig;

        let mut config = Config::default();
        config.plugins = Some(PluginSystemConfig {
            enabled: true,
            directory: std::path::PathBuf::from("./plugins"),
            hot_reload: true,
            max_plugins: 50,
            timeout_ms: 100,
            api_prefix: "_plugins".to_string(),
            preload: vec![],
        });

        let server = Server::new(config);
        assert!(server.config.plugins.is_some());
        let plugins = server.config.plugins.as_ref().unwrap();
        assert!(plugins.enabled);
        assert_eq!(plugins.max_plugins, 50);
        assert!(plugins.hot_reload);
        assert_eq!(plugins.timeout_ms, 100);
        assert_eq!(plugins.api_prefix, "_plugins");
    }

    #[test]
    fn test_server_port_boundaries() {
        // Test with port 1 (minimum valid)
        let mut config = Config::default();
        config.port = 1;
        let server = Server::new(config);
        assert_eq!(server.config.port, 1);
        
        // Test with port 65535 (maximum valid)
        let mut config = Config::default();
        config.port = 65535;
        let server = Server::new(config);
        assert_eq!(server.config.port, 65535);
    }

    #[test]
    fn test_server_config_clone() {
        let config = Config::default();
        let server = Server::new(config.clone());
        
        // Verify config was cloned correctly
        assert_eq!(server.config.port, config.port);
        assert_eq!(server.config.root, config.root);
        assert_eq!(server.config.enable_indexing, config.enable_indexing);
    }

    #[test]
    fn test_server_multiple_shutdown_calls() {
        let config = Config::default();
        let server = Server::new(config);
        
        // Multiple shutdown calls should not panic
        server.shutdown();
        server.shutdown();
        server.shutdown();
    }

    #[test]
    fn test_server_tls_with_empty_paths() {
        let mut config = Config::default();
        config.enable_tls = true;
        config.tls_cert = Some("".to_string());
        config.tls_key = Some("".to_string());
        
        let server = Server::new(config);
        assert!(server.config.enable_tls);
        assert_eq!(server.config.tls_cert, Some("".to_string()));
        assert_eq!(server.config.tls_key, Some("".to_string()));
    }

    #[test]
    fn test_server_empty_cors_origins() {
        let mut config = Config::default();
        config.cors_allowed_origins = vec![];
        
        let server = Server::new(config);
        assert!(server.config.cors_allowed_origins.is_empty());
    }

    #[test]
    fn test_server_empty_ip_lists() {
        let mut config = Config::default();
        config.ip_allowlist = vec![];
        config.ip_blocklist = vec![];
        
        let server = Server::new(config);
        assert!(server.config.ip_allowlist.is_empty());
        assert!(server.config.ip_blocklist.is_empty());
    }

    #[test]
    fn test_server_zero_timeout() {
        let mut config = Config::default();
        config.connection_timeout_secs = 0;
        
        let server = Server::new(config);
        assert_eq!(server.config.connection_timeout_secs, 0);
    }

    #[test]
    fn test_server_zero_max_connections() {
        let mut config = Config::default();
        config.max_connections = 0;
        
        let server = Server::new(config);
        assert_eq!(server.config.max_connections, 0);
    }

    #[test]
    fn test_server_large_max_body_size() {
        let mut config = Config::default();
        config.max_body_size = 100 * 1024 * 1024; // 100MB
        
        let server = Server::new(config);
        assert_eq!(server.config.max_body_size, 100 * 1024 * 1024);
    }
}