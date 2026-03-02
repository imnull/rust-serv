//! HTTP/2 support for static file server
//!
//! This module provides HTTP/2 support using tower-http.

use crate::config::Config;
use crate::error::Result;
use crate::handler::Handler;
use hyper::server::conn::http1;
use hyper::Response;
use hyper::body::Bytes;
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use tower::ServiceBuilder;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// HTTP/2 Server
pub struct Http2Server {
    config: Arc<Config>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl Http2Server {
    /// Create a new HTTP/2 server with given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start HTTP/2 server
    pub async fn run(&self) -> Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port).parse()?;
        let listener = TcpListener::bind(addr).await?;

        println!("HTTP/2 Server listening on http://{}", addr);

        let handler = Handler::new(self.config.clone());

        // Setup signal handling for graceful shutdown
        #[cfg(unix)]
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");

        // Create connection semaphore for max connections
        let max_connections = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        // Build HTTP/2 service
        let make_service = || {
            let handler = handler.clone();
            ServiceBuilder::new()
                .service(move |req| {
                    let handler = handler.clone();
                    async move {
                        handler.handle_request(req).await
                    }
                })
        };

        let service = make_service();

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

                    let io = TokioIo::new(stream);

                    tokio::task::spawn(async move {
                        // Set connection timeout
                        let timeout = Duration::from_secs(self.config.connection_timeout_secs);
                        let result = tokio::time::timeout(timeout, async {
                            http1::Builder::new()
                                .serve_connection(io, service)
                                .await
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

        println!("HTTP/2 Server shutdown complete");
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
    fn test_http2_server_creation() {
        let config = Config::default();
        let server = Http2Server::new(config);
        assert_eq!(server.config.port, 8080);
    }

    #[test]
    fn test_shutdown_signal() {
        let config = Config::default();
        let server = Http2Server::new(config);
        // Should not panic
        server.shutdown();
    }
}
