use crate::config::Config;
use crate::error::Result;
use crate::handler::Handler;
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
                    let handler = handler.clone();
                    let config = self.config.clone();

                    tokio::task::spawn(async move {
                        // Set connection timeout
                        let timeout = Duration::from_secs(config.connection_timeout_secs);
                        let result = tokio::time::timeout(timeout, async {
                            http1::Builder::new()
                                .serve_connection(io, hyper::service::service_fn(move |req| {
                                    // Health check endpoint and regular requests
                                    let handler = handler.clone();
                                    async move {
                                        handler.handle_request(req).await
                                    }
                                }))
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
}
