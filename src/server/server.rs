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
}