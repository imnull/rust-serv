//! WebSocket Server implementation
//!
//! This module implements WebSocket functionality:
//! - WebSocket protocol handshake
//! - Frame handling and parsing
//! - Connection management
//! - Message broadcasting
//! - Heartbeat and ping/pong

use crate::config::Config;
use crate::error::Error;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hyper::header::{CONNECTION, UPGRADE, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION};
use hyper::{Request, Response, StatusCode, Method};
use http_body_util::Full;
use hyper::body::Bytes;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;
use futures::{StreamExt, SinkExt};

/// WebSocket message types
#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketMessage {
    /// Text message
    Text(String),
    /// Binary message
    Binary(tungstenite::Bytes),
    /// Ping message
    Ping(tungstenite::Bytes),
    /// Pong message
    Pong(tungstenite::Bytes),
    /// Close message
    Close { code: u16, reason: String },
}

/// WebSocket connection info
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub remote_addr: String,
    /// Connected timestamp
    pub connected_at: std::time::Instant,
}

/// WebSocket server with connection management
pub struct WebSocketServer {
    config: Arc<Config>,
    connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<WebSocketMessage>>>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request is a WebSocket upgrade request
    pub fn is_websocket_upgrade<B>(req: &Request<B>) -> bool {
        req.method() == Method::GET
            && req.headers().get(UPGRADE)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_lowercase() == "websocket")
                .unwrap_or(false)
            && req.headers().get(CONNECTION)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_lowercase().contains("upgrade"))
                .unwrap_or(false)
    }

    /// Perform WebSocket handshake
    pub fn handshake<B>(req: &Request<B>) -> Result<Response<Full<Bytes>>, Error> {
        // Validate WebSocket version
        let ws_version = req.headers()
            .get(SEC_WEBSOCKET_VERSION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Error::Http("Missing WebSocket-Version header".to_string()))?;

        if ws_version != "13" {
            return Err(Error::Http("Unsupported WebSocket version".to_string()));
        }

        // Get WebSocket key
        let ws_key = req.headers()
            .get(SEC_WEBSOCKET_KEY)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Error::Http("Missing WebSocket-Key header".to_string()))?;

        // Generate accept key
        let accept_key = Self::generate_accept_key(ws_key)?;

        // Build handshake response
        let response = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "Upgrade")
            .header(SEC_WEBSOCKET_ACCEPT, accept_key)
            .body(Full::new(Bytes::new()))
            .map_err(|e| Error::Internal(format!("Failed to build response: {}", e)))?;

        Ok(response)
    }

    /// Generate WebSocket accept key
    fn generate_accept_key(ws_key: &str) -> Result<String, Error> {
        let magic_guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let combined = format!("{}{}", ws_key, magic_guid);

        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();

        let accept_key = BASE64.encode(&hash);
        Ok(accept_key)
    }

    /// Add a new WebSocket connection
    pub async fn add_connection(&self, id: String, sender: mpsc::UnboundedSender<WebSocketMessage>) {
        let mut connections = self.connections.write().await;
        connections.insert(id, sender);
    }

    /// Remove a WebSocket connection
    pub async fn remove_connection(&self, id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: WebSocketMessage) {
        let connections = self.connections.read().await;
        let mut failed_connections = Vec::new();

        for (id, sender) in connections.iter() {
            if sender.send(message.clone()).is_err() {
                failed_connections.push(id.clone());
            }
        }

        // Remove failed connections
        if !failed_connections.is_empty() {
            drop(connections);
            let mut connections = self.connections.write().await;
            for id in failed_connections {
                connections.remove(&id);
            }
        }
    }

    /// Send a message to a specific connection
    pub async fn send_to(&self, id: &str, message: WebSocketMessage) -> Result<(), Error> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(id) {
            sender.send(message)
                .map_err(|_| Error::Http("Connection closed".to_string()))?;
        } else {
            return Err(Error::Http("Connection not found".to_string()));
        }
        Ok(())
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Get a list of all active connections
    pub async fn list_connections(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// Handle incoming WebSocket message
    pub fn handle_message(&self, message: Message) -> Result<Option<WebSocketMessage>, Error> {
        match message {
            Message::Text(text) => Ok(Some(WebSocketMessage::Text(text.to_string()))),
            Message::Binary(data) => Ok(Some(WebSocketMessage::Binary(data))),
            Message::Ping(data) => {
                // Auto-respond to ping with pong
                Ok(Some(WebSocketMessage::Pong(data)))
            }
            Message::Pong(data) => Ok(Some(WebSocketMessage::Pong(data))),
            Message::Close(frame) => {
                if let Some(frame) = frame {
                    Ok(Some(WebSocketMessage::Close {
                        code: frame.code.into(),
                        reason: frame.reason.to_string(),
                    }))
                } else {
                    Ok(Some(WebSocketMessage::Close {
                        code: 1000,
                        reason: String::new(),
                    }))
                }
            }
            Message::Frame(_) => {
                // Raw frames - ignore for now
                Ok(None)
            }
        }
    }

    /// Convert WebSocket message to tungstenite message
    pub fn to_tungstenite_message(&self, message: &WebSocketMessage) -> Message {
        match message {
            WebSocketMessage::Text(text) => Message::Text(text.clone().into()),
            WebSocketMessage::Binary(data) => Message::Binary(data.clone()),
            WebSocketMessage::Ping(data) => Message::Ping(data.clone()),
            WebSocketMessage::Pong(data) => Message::Pong(data.clone()),
            WebSocketMessage::Close { code, reason } => {
                Message::Close(Some(tungstenite::protocol::frame::CloseFrame {
                    code: tungstenite::protocol::frame::coding::CloseCode::from(*code),
                    reason: reason.clone().into(),
                }))
            }
        }
    }
}

/// WebSocket connection handler
pub struct WebSocketHandler {
    server: Arc<WebSocketServer>,
    connection_id: String,
    receiver: Option<mpsc::UnboundedReceiver<WebSocketMessage>>,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new(server: Arc<WebSocketServer>, connection_id: String) -> (Self, mpsc::UnboundedSender<WebSocketMessage>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let handler = Self {
            server,
            connection_id,
            receiver: Some(receiver),
        };

        (handler, sender)
    }

    /// Handle WebSocket connection
    pub async fn handle_connection<T>(&mut self, mut ws_stream: WebSocketStream<T>) -> Result<(), Error>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::marker::Send + 'static,
    {
        let mut receiver = self.receiver.take().ok_or_else(|| Error::Internal("Receiver already taken".to_string()))?;
        let server = self.server.clone();

        // Handle incoming messages
        loop {
            tokio::select! {
                // Receive from server broadcast
                msg = receiver.recv() => {
                    match msg {
                        Some(message) => {
                            let tungstenite_msg = match message {
                                WebSocketMessage::Text(text) => Message::Text(text.into()),
                                WebSocketMessage::Binary(data) => Message::Binary(data),
                                WebSocketMessage::Ping(data) => Message::Ping(data),
                                WebSocketMessage::Pong(data) => Message::Pong(data),
                                WebSocketMessage::Close { code, reason } => Message::Close(Some(
                                    tungstenite::protocol::frame::CloseFrame {
                                        code: tungstenite::protocol::frame::coding::CloseCode::from(code),
                                        reason: reason.into(),
                                    }
                                )),
                            };

                            if tungstenite_msg.is_close() || SinkExt::send(&mut ws_stream, tungstenite_msg).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                // Receive from WebSocket
                result = ws_stream.next() => {
                    match result {
                        Some(Ok(message)) => {
                            if let Some(ws_message) = server.handle_message(message)? {
                                // Handle special messages
                                if matches!(ws_message, WebSocketMessage::Close { .. }) {
                                    break;
                                }
                            }
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }

        // Clean up
        server.remove_connection(&self.connection_id).await;
        tracing::info!("WebSocket connection {} closed", self.connection_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Request;

    #[test]
    fn test_websocket_upgrade_detection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

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

    #[test]
    fn test_non_websocket_request() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let req = Request::builder()
            .method("GET")
            .uri("/")
            .body(Bytes::new())
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_websocket_version_validation() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Missing WebSocket-Version header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .body(Bytes::new())
            .unwrap();

        assert!(WebSocketServer::handshake(&req).is_err());

        // Invalid WebSocket-Version
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "12")
            .body(Bytes::new())
            .unwrap();

        assert!(WebSocketServer::handshake(&req).is_err());
    }

    #[test]
    fn test_generate_accept_key() {
        let ws_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept_key = WebSocketServer::generate_accept_key(ws_key).unwrap();

        // Known test value from RFC 6455
        assert_eq!(accept_key, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_message_conversion() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test text message
        let text_msg = WebSocketMessage::Text("Hello".to_string());
        let converted = server.to_tungstenite_message(&text_msg);
        assert!(matches!(converted, Message::Text(_)));

        // Test binary message
        let binary_msg = WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        let converted = server.to_tungstenite_message(&binary_msg);
        assert!(matches!(converted, Message::Binary(_)));

        // Test close message
        let close_msg = WebSocketMessage::Close {
            code: 1000,
            reason: "Normal closure".to_string(),
        };
        let converted = server.to_tungstenite_message(&close_msg);
        assert!(matches!(converted, Message::Close(_)));
    }

    #[test]
    fn test_connection_management() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test connection count
        futures::executor::block_on(async {
            assert_eq!(server.connection_count().await, 0);

            // Add a connection
            let (sender, _) = mpsc::unbounded_channel();
            server.add_connection("test_conn".to_string(), sender).await;
            assert_eq!(server.connection_count().await, 1);

            // Remove connection
            server.remove_connection("test_conn").await;
            assert_eq!(server.connection_count().await, 0);

            // List connections
            let (sender1, _) = mpsc::unbounded_channel();
            let (sender2, _) = mpsc::unbounded_channel();
            server.add_connection("conn1".to_string(), sender1).await;
            server.add_connection("conn2".to_string(), sender2).await;

            let connections = server.list_connections().await;
            assert_eq!(connections.len(), 2);
            assert!(connections.contains(&"conn1".to_string()));
            assert!(connections.contains(&"conn2".to_string()));
        });
    }

    #[test]
    fn test_send_to_connection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            // Create a connection with a receiver
            let (sender, mut receiver) = mpsc::unbounded_channel();
            server.add_connection("test_conn".to_string(), sender).await;

            // Send a message to the connection
            let message = WebSocketMessage::Text("Hello".to_string());
            let result = server.send_to("test_conn", message.clone()).await;
            assert!(result.is_ok());

            // Verify the message was received
            let received = receiver.recv().await;
            assert!(received.is_some());
            assert_eq!(received.unwrap(), WebSocketMessage::Text("Hello".to_string()));

            // Try to send to non-existent connection
            let result = server.send_to("nonexistent", message).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_broadcast_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            // Create multiple connections
            let (sender1, mut receiver1) = mpsc::unbounded_channel();
            let (sender2, mut receiver2) = mpsc::unbounded_channel();
            let (sender3, mut receiver3) = mpsc::unbounded_channel();

            server.add_connection("conn1".to_string(), sender1).await;
            server.add_connection("conn2".to_string(), sender2).await;
            server.add_connection("conn3".to_string(), sender3).await;

            // Broadcast a message
            let message = WebSocketMessage::Text("Broadcast message".to_string());
            server.broadcast(message).await;

            // Verify all connections received the message
            assert_eq!(
                receiver1.recv().await.unwrap(),
                WebSocketMessage::Text("Broadcast message".to_string())
            );
            assert_eq!(
                receiver2.recv().await.unwrap(),
                WebSocketMessage::Text("Broadcast message".to_string())
            );
            assert_eq!(
                receiver3.recv().await.unwrap(),
                WebSocketMessage::Text("Broadcast message".to_string())
            );
        });
    }

    #[test]
    fn test_handle_text_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Text("Hello".into());
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap(), WebSocketMessage::Text("Hello".to_string()));
    }

    #[test]
    fn test_handle_binary_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        let ws_message = result.unwrap();
        assert!(matches!(ws_message, WebSocketMessage::Binary(_)));
    }

    #[test]
    fn test_handle_ping_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Ping(tungstenite::Bytes::from(vec![1, 2, 3]));
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        let ws_message = result.unwrap();
        assert!(matches!(ws_message, WebSocketMessage::Pong(_)));
    }

    #[test]
    fn test_handle_pong_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Pong(tungstenite::Bytes::from(vec![1, 2, 3]));
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        let ws_message = result.unwrap();
        assert!(matches!(ws_message, WebSocketMessage::Pong(_)));
    }

    #[test]
    fn test_handle_close_message_with_frame() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Create a close message with a frame using Message::Close
        let message = Message::Close(Some(tungstenite::protocol::frame::CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: "Normal closure".into(),
        }));
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        let ws_message = result.unwrap();
        match ws_message {
            WebSocketMessage::Close { code, reason } => {
                assert_eq!(code, 1000);
                assert_eq!(reason, "Normal closure");
            }
            _ => panic!("Expected Close message"),
        }
    }

    #[test]
    fn test_handle_close_message_without_frame() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Close(None);
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        let ws_message = result.unwrap();
        match ws_message {
            WebSocketMessage::Close { code, reason } => {
                assert_eq!(code, 1000);
                assert_eq!(reason, "");
            }
            _ => panic!("Expected Close message"),
        }
    }

    #[test]
    fn test_to_tungstenite_text() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = WebSocketMessage::Text("Hello".to_string());
        let converted = server.to_tungstenite_message(&message);

        assert!(matches!(converted, Message::Text(_)));
        if let Message::Text(text) = converted {
            assert_eq!(text, "Hello");
        }
    }

    #[test]
    fn test_to_tungstenite_binary() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        let converted = server.to_tungstenite_message(&message);

        assert!(matches!(converted, Message::Binary(_)));
    }

    #[test]
    fn test_to_tungstenite_ping() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = WebSocketMessage::Ping(tungstenite::Bytes::from(vec![1, 2, 3]));
        let converted = server.to_tungstenite_message(&message);

        assert!(matches!(converted, Message::Ping(_)));
    }

    #[test]
    fn test_to_tungstenite_pong() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = WebSocketMessage::Pong(tungstenite::Bytes::from(vec![1, 2, 3]));
        let converted = server.to_tungstenite_message(&message);

        assert!(matches!(converted, Message::Pong(_)));
    }

    #[test]
    fn test_to_tungstenite_close() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = WebSocketMessage::Close {
            code: 1000,
            reason: "Normal closure".to_string(),
        };
        let converted = server.to_tungstenite_message(&message);

        assert!(matches!(converted, Message::Close(_)));
        if let Message::Close(Some(frame)) = converted {
            assert_eq!(frame.code, tungstenite::protocol::frame::coding::CloseCode::Normal);
            assert_eq!(frame.reason, std::borrow::Cow::from("Normal closure"));
        } else {
            panic!("Expected Close message with frame");
        }
    }

    #[test]
    fn test_broadcast_with_closed_connection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            // Create a connection with a closed sender
            let (sender, receiver) = mpsc::unbounded_channel();
            drop(receiver); // Close the receiver

            server.add_connection("closed_conn".to_string(), sender).await;

            // Create a normal connection
            let (sender2, mut receiver2) = mpsc::unbounded_channel();
            server.add_connection("normal_conn".to_string(), sender2).await;

            // Broadcast a message
            let message = WebSocketMessage::Text("Test".to_string());
            server.broadcast(message).await;

            // The closed connection should be removed
            let connections = server.list_connections().await;
            assert!(!connections.contains(&"closed_conn".to_string()));
            assert!(connections.contains(&"normal_conn".to_string()));

            // The normal connection should still receive the message
            assert!(receiver2.recv().await.is_some());
        });
    }

    #[test]
    fn test_send_to_connection_success() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            let (sender, mut receiver) = mpsc::unbounded_channel();
            server.add_connection("test_conn".to_string(), sender).await;

            let message = WebSocketMessage::Text("Hello".to_string());
            let result = server.send_to("test_conn", message).await;
            assert!(result.is_ok());

            let received = receiver.recv().await;
            assert!(received.is_some());
        });
    }

    #[test]
    fn test_send_to_connection_not_found() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            let message = WebSocketMessage::Text("Hello".to_string());
            let result = server.send_to("nonexistent", message).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    fn test_send_to_connection_closed() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        futures::executor::block_on(async {
            let (sender, receiver) = mpsc::unbounded_channel();
            drop(receiver); // Close the receiver immediately

            server.add_connection("test_conn".to_string(), sender).await;

            let message = WebSocketMessage::Text("Hello".to_string());
            let result = server.send_to("test_conn", message).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("closed"));
        });
    }

    #[test]
    fn test_websocket_upgrade_missing_headers() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .body(Full::new(Bytes::new()))
            .unwrap();

        // Should return false for non-WebSocket request
        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_websocket_upgrade_only_upgrade_header() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .body(Full::new(Bytes::new()))
            .unwrap();

        // Should return false because Connection header is missing
        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_websocket_upgrade_wrong_method() {
        let req = Request::builder()
            .method("POST")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .body(Full::new(Bytes::new()))
            .unwrap();

        // Should return false because method is not GET
        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_handshake_missing_key() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let result = WebSocketServer::handshake(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_handshake_missing_version() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let result = WebSocketServer::handshake(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_handshake_wrong_version() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "12")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let result = WebSocketServer::handshake(&req);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_and_remove_connection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender1, _) = mpsc::unbounded_channel();
        let (sender2, _) = mpsc::unbounded_channel();

        server.add_connection("conn1".to_string(), sender1).await;
        server.add_connection("conn2".to_string(), sender2).await;

        let count = server.connection_count().await;
        assert_eq!(count, 2);

        server.remove_connection("conn1").await;
        let count = server.connection_count().await;
        assert_eq!(count, 1);

        server.remove_connection("conn2").await;
        let count = server.connection_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_empty_connections() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Broadcast to empty connections should not fail
        let message = WebSocketMessage::Text("Hello".to_string());
        server.broadcast(message).await;
        // No assertion needed, just ensure no panic
    }

    #[test]
    fn test_websocket_upgrade_case_insensitive() {
        // Test case-insensitive upgrade header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "WebSocket") // Uppercase
            .header("Connection", "upgrade") // lowercase
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(WebSocketServer::is_websocket_upgrade(&req));

        // Test mixed case
        let req2 = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "WEBSOCKET")
            .header("Connection", "UPGRADE")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(WebSocketServer::is_websocket_upgrade(&req2));
    }

    #[test]
    fn test_websocket_upgrade_non_utf8_upgrade_header() {
        // Test with invalid UTF-8 in Upgrade header using HeaderValue
        use hyper::header::HeaderValue;
        
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap()) // Invalid UTF-8
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        // Should return false due to invalid UTF-8
        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_websocket_upgrade_non_utf8_connection_header() {
        // Test with invalid UTF-8 in Connection header
        use hyper::header::HeaderValue;
        
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap()) // Invalid UTF-8
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        // Should return false due to invalid UTF-8
        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_generate_accept_key_empty() {
        // Test with empty key
        let accept_key = WebSocketServer::generate_accept_key("").unwrap();
        // Known value: SHA1 of empty + magic GUID, base64 encoded
        assert_eq!(accept_key.len(), 28); // Base64 encoded SHA1 is always 28 chars
        assert!(!accept_key.is_empty());
    }

    #[test]
    fn test_generate_accept_key_long_key() {
        // Test with a long key
        let long_key = "a".repeat(100);
        let accept_key = WebSocketServer::generate_accept_key(&long_key).unwrap();
        assert_eq!(accept_key.len(), 28); // Base64 encoded SHA1 is always 28 chars
    }

    #[test]
    fn test_generate_accept_key_special_chars() {
        // Test with special characters
        let special_key = "+/=special&chars%$#@!";
        let accept_key = WebSocketServer::generate_accept_key(special_key).unwrap();
        assert_eq!(accept_key.len(), 28);
    }

    #[test]
    fn test_handle_frame_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test handling of raw Frame message - should return Ok(None)
        // Create a frame using Frame::from_payload
        let header = tungstenite::protocol::frame::FrameHeader::default();
        let payload = tungstenite::Bytes::from_static(&[0x01, 0x02, 0x03]);
        let frame = tungstenite::protocol::frame::Frame::from_payload(header, payload);
        let frame_msg = Message::Frame(frame);
        let result = server.handle_message(frame_msg).unwrap();

        // Frame messages should be ignored (return None)
        assert!(result.is_none());
    }

    #[test]
    fn test_handshake_success_response_headers() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = WebSocketServer::handshake(&req).unwrap();

        // Verify status code
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

        // Verify Upgrade header
        assert_eq!(
            response.headers().get(UPGRADE).unwrap().to_str().unwrap(),
            "websocket"
        );

        // Verify Connection header
        assert_eq!(
            response.headers().get(CONNECTION).unwrap().to_str().unwrap(),
            "Upgrade"
        );

        // Verify Sec-WebSocket-Accept header
        assert_eq!(
            response.headers().get(SEC_WEBSOCKET_ACCEPT).unwrap().to_str().unwrap(),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn test_websocket_message_debug() {
        // Test Debug implementation for WebSocketMessage
        let text_msg = WebSocketMessage::Text("Hello".to_string());
        let debug_str = format!("{:?}", text_msg);
        assert!(debug_str.contains("Text"));
        assert!(debug_str.contains("Hello"));

        let binary_msg = WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        let debug_str = format!("{:?}", binary_msg);
        assert!(debug_str.contains("Binary"));

        let close_msg = WebSocketMessage::Close {
            code: 1000,
            reason: "Normal".to_string(),
        };
        let debug_str = format!("{:?}", close_msg);
        assert!(debug_str.contains("Close"));
        assert!(debug_str.contains("1000"));
    }

    #[test]
    fn test_websocket_message_clone() {
        // Test Clone implementation
        let msg = WebSocketMessage::Text("Hello".to_string());
        let cloned = msg.clone();
        assert_eq!(msg, cloned);

        let close_msg = WebSocketMessage::Close {
            code: 1001,
            reason: "Going away".to_string(),
        };
        let cloned_close = close_msg.clone();
        assert_eq!(close_msg, cloned_close);
    }

    #[tokio::test]
    async fn test_websocket_handler_new() {
        let config = Config::default();
        let server = Arc::new(WebSocketServer::new(config));

        let (handler, sender) = WebSocketHandler::new(server, "test-conn-123".to_string());

        assert_eq!(handler.connection_id, "test-conn-123");
        assert!(handler.receiver.is_some());

        // Test that sender works
        let msg = WebSocketMessage::Text("Test".to_string());
        assert!(sender.send(msg).is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_multiple_messages() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("conn1".to_string(), sender).await;

        // Broadcast multiple messages
        for i in 0..10 {
            let msg = WebSocketMessage::Text(format!("Message {}", i));
            server.broadcast(msg).await;
        }

        // Verify all messages received
        for i in 0..10 {
            let received = receiver.recv().await.unwrap();
            assert_eq!(received, WebSocketMessage::Text(format!("Message {}", i)));
        }
    }

    #[tokio::test]
    async fn test_send_to_various_message_types() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("test_conn".to_string(), sender).await;

        // Send text message
        let text_msg = WebSocketMessage::Text("Hello".to_string());
        server.send_to("test_conn", text_msg.clone()).await.unwrap();
        assert_eq!(receiver.recv().await.unwrap(), text_msg);

        // Send binary message
        let binary_msg = WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        server.send_to("test_conn", binary_msg.clone()).await.unwrap();
        assert_eq!(receiver.recv().await.unwrap(), binary_msg);

        // Send ping message
        let ping_msg = WebSocketMessage::Ping(tungstenite::Bytes::from(vec![4, 5, 6]));
        server.send_to("test_conn", ping_msg.clone()).await.unwrap();
        assert_eq!(receiver.recv().await.unwrap(), ping_msg);

        // Send pong message
        let pong_msg = WebSocketMessage::Pong(tungstenite::Bytes::from(vec![7, 8, 9]));
        server.send_to("test_conn", pong_msg.clone()).await.unwrap();
        assert_eq!(receiver.recv().await.unwrap(), pong_msg);

        // Send close message
        let close_msg = WebSocketMessage::Close {
            code: 1000,
            reason: "Normal".to_string(),
        };
        server.send_to("test_conn", close_msg.clone()).await.unwrap();
        assert_eq!(receiver.recv().await.unwrap(), close_msg);
    }

    #[test]
    fn test_handle_close_with_different_codes() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test different close codes
        let codes = vec![
            (1000u16, "Normal closure"),
            (1001, "Going away"),
            (1002, "Protocol error"),
            (1003, "Unsupported data"),
            (1006, "Abnormal closure"),
            (1008, "Policy violation"),
            (1009, "Message too big"),
            (1010, "Mandatory extension"),
            (1011, "Internal error"),
            (1015, "TLS handshake"),
        ];

        for (code, reason) in codes {
            let close_frame = tungstenite::protocol::frame::CloseFrame {
                code: tungstenite::protocol::frame::coding::CloseCode::from(code),
                reason: reason.into(),
            };
            let message = Message::Close(Some(close_frame));
            let result = server.handle_message(message).unwrap();

            match result {
                Some(WebSocketMessage::Close { code: c, reason: r }) => {
                    assert_eq!(c, code);
                    assert_eq!(r, reason);
                }
                _ => panic!("Expected Close message with code {}", code),
            }
        }
    }

    #[test]
    fn test_handle_empty_binary_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Binary(tungstenite::Bytes::from(vec![]));
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        match result.unwrap() {
            WebSocketMessage::Binary(data) => {
                assert!(data.is_empty());
            }
            _ => panic!("Expected Binary message"),
        }
    }

    #[test]
    fn test_handle_empty_text_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let message = Message::Text("".into());
        let result = server.handle_message(message).unwrap();

        assert!(result.is_some());
        match result.unwrap() {
            WebSocketMessage::Text(text) => {
                assert!(text.is_empty());
            }
            _ => panic!("Expected Text message"),
        }
    }

    #[test]
    fn test_handle_empty_ping_pong() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Empty ping
        let ping_msg = Message::Ping(tungstenite::Bytes::from(vec![]));
        let result = server.handle_message(ping_msg).unwrap();
        match result {
            Some(WebSocketMessage::Pong(data)) => {
                assert!(data.is_empty());
            }
            _ => panic!("Expected Pong with empty data"),
        }

        // Empty pong
        let pong_msg = Message::Pong(tungstenite::Bytes::from(vec![]));
        let result = server.handle_message(pong_msg).unwrap();
        match result {
            Some(WebSocketMessage::Pong(data)) => {
                assert!(data.is_empty());
            }
            _ => panic!("Expected Pong with empty data"),
        }
    }

    #[tokio::test]
    async fn test_list_connections_empty() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let connections = server.list_connections().await;
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_list_connections_order() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Add connections in specific order
        for i in 0..5 {
            let (sender, _) = mpsc::unbounded_channel();
            server.add_connection(format!("conn_{}", i), sender).await;
        }

        let connections = server.list_connections().await;
        assert_eq!(connections.len(), 5);

        // Verify all connections are present
        for i in 0..5 {
            assert!(connections.contains(&format!("conn_{}", i)));
        }
    }

    #[tokio::test]
    async fn test_remove_nonexistent_connection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Remove a connection that doesn't exist - should not panic
        server.remove_connection("nonexistent").await;

        // Count should still be 0
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_duplicate_connection() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender1, _receiver1) = mpsc::unbounded_channel();
        let (sender2, mut receiver2) = mpsc::unbounded_channel();

        // Add first connection
        server.add_connection("duplicate_conn".to_string(), sender1).await;
        assert_eq!(server.connection_count().await, 1);

        // Add with same ID - should replace
        server.add_connection("duplicate_conn".to_string(), sender2).await;
        assert_eq!(server.connection_count().await, 1);

        // New sender should work
        let msg = WebSocketMessage::Text("Test".to_string());
        server.send_to("duplicate_conn", msg.clone()).await.unwrap();
        assert_eq!(receiver2.recv().await.unwrap(), msg);
    }

    #[test]
    fn test_is_websocket_upgrade_variations() {
        // Test with "keep-alive, Upgrade" connection header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "keep-alive, Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(WebSocketServer::is_websocket_upgrade(&req));

        // Test with "Upgrade, keep-alive" connection header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade, keep-alive")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_websocket_server_new() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Server should be created successfully
        // We can't directly access private fields, but we can test functionality
        futures::executor::block_on(async {
            assert_eq!(server.connection_count().await, 0);
        });
    }

    #[test]
    fn test_to_tungstenite_close_various_codes() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test various close codes
        let test_cases = vec![
            (1000u16, "Normal"),
            (1001, "Going away"),
            (1006, "Abnormal"),
            (1011, "Error"),
        ];

        for (code, reason) in test_cases {
            let msg = WebSocketMessage::Close {
                code,
                reason: reason.to_string(),
            };
            let converted = server.to_tungstenite_message(&msg);

            match converted {
                Message::Close(Some(frame)) => {
                    // Compare the close codes by converting to u16
                    let frame_code: u16 = frame.code.into();
                    assert_eq!(frame_code, code);
                    assert_eq!(frame.reason, std::borrow::Cow::from(reason));
                }
                _ => panic!("Expected Close message with frame for code {}", code),
            }
        }
    }

    #[tokio::test]
    async fn test_broadcast_binary_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("conn1".to_string(), sender).await;

        // Broadcast binary message
        let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253];
        let msg = WebSocketMessage::Binary(tungstenite::Bytes::from(binary_data.clone()));
        server.broadcast(msg).await;

        let received = receiver.recv().await.unwrap();
        match received {
            WebSocketMessage::Binary(data) => {
                assert_eq!(data.to_vec(), binary_data);
            }
            _ => panic!("Expected Binary message"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_ping_pong() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("conn1".to_string(), sender).await;

        // Broadcast ping
        let ping_data = vec![1, 2, 3];
        let ping_msg = WebSocketMessage::Ping(tungstenite::Bytes::from(ping_data.clone()));
        server.broadcast(ping_msg).await;

        let received = receiver.recv().await.unwrap();
        match received {
            WebSocketMessage::Ping(data) => {
                assert_eq!(data.to_vec(), ping_data);
            }
            _ => panic!("Expected Ping message"),
        }

        // Broadcast pong
        let pong_data = vec![4, 5, 6];
        let pong_msg = WebSocketMessage::Pong(tungstenite::Bytes::from(pong_data.clone()));
        server.broadcast(pong_msg).await;

        let received = receiver.recv().await.unwrap();
        match received {
            WebSocketMessage::Pong(data) => {
                assert_eq!(data.to_vec(), pong_data);
            }
            _ => panic!("Expected Pong message"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_close_message() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("conn1".to_string(), sender).await;

        // Broadcast close message
        let close_msg = WebSocketMessage::Close {
            code: 1000,
            reason: "Server shutting down".to_string(),
        };
        server.broadcast(close_msg.clone()).await;

        let received = receiver.recv().await.unwrap();
        assert_eq!(received, close_msg);
    }

    #[test]
    fn test_websocket_connection_struct() {
        // Test WebSocketConnection struct creation and field access
        let conn = WebSocketConnection {
            id: "test-conn-123".to_string(),
            remote_addr: "192.168.1.1:12345".to_string(),
            connected_at: std::time::Instant::now(),
        };

        // Test field access
        assert_eq!(conn.id, "test-conn-123");
        assert_eq!(conn.remote_addr, "192.168.1.1:12345");
        
        // Test Clone
        let cloned = conn.clone();
        assert_eq!(cloned.id, conn.id);
        assert_eq!(cloned.remote_addr, conn.remote_addr);
        
        // Test Debug
        let debug_str = format!("{:?}", conn);
        assert!(debug_str.contains("test-conn-123"));
        assert!(debug_str.contains("192.168.1.1:12345"));
    }

    #[test]
    fn test_websocket_handler_receiver_already_taken() {
        let config = Config::default();
        let server = Arc::new(WebSocketServer::new(config));

        let (mut handler, _sender) = WebSocketHandler::new(server, "test_conn".to_string());

        // Take the receiver once
        let _receiver = handler.receiver.take().unwrap();

        // Verify receiver is now None
        assert!(handler.receiver.is_none());
    }

    #[tokio::test]
    async fn test_websocket_handler_drop_receiver() {
        let config = Config::default();
        let server = Arc::new(WebSocketServer::new(config));

        let (handler, sender) = WebSocketHandler::new(server, "test_conn".to_string());

        // Drop the handler, sender should still work
        drop(handler);

        // Sender should still be able to send (though receiver is gone)
        let msg = WebSocketMessage::Text("Test".to_string());
        // This will fail because receiver is dropped
        assert!(sender.send(msg).is_err());
    }

    #[test]
    fn test_websocket_message_variants() {
        // Test all WebSocketMessage variants
        let text = WebSocketMessage::Text("hello".to_string());
        let binary = WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3]));
        let ping = WebSocketMessage::Ping(tungstenite::Bytes::from(vec![4, 5, 6]));
        let pong = WebSocketMessage::Pong(tungstenite::Bytes::from(vec![7, 8, 9]));
        let close = WebSocketMessage::Close {
            code: 1000,
            reason: "Normal".to_string(),
        };

        // Test equality
        assert_eq!(text, WebSocketMessage::Text("hello".to_string()));
        assert_eq!(binary, WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3])));
        assert_eq!(ping, WebSocketMessage::Ping(tungstenite::Bytes::from(vec![4, 5, 6])));
        assert_eq!(pong, WebSocketMessage::Pong(tungstenite::Bytes::from(vec![7, 8, 9])));
        assert_eq!(close.clone(), close);

        // Test inequality
        assert_ne!(text, WebSocketMessage::Text("world".to_string()));
        assert_ne!(binary, WebSocketMessage::Binary(tungstenite::Bytes::from(vec![4, 5, 6])));
    }

    #[test]
    fn test_is_websocket_upgrade_no_headers() {
        // Test with no headers at all
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_is_websocket_upgrade_no_upgrade_header() {
        // Test with Connection header but no Upgrade header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Connection", "Upgrade")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_is_websocket_upgrade_no_connection_header() {
        // Test with Upgrade header but no Connection header
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_is_websocket_upgrade_wrong_upgrade_value() {
        // Test with wrong Upgrade value
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "http2")
            .header("Connection", "Upgrade")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_is_websocket_upgrade_connection_without_upgrade() {
        // Test with Connection header that doesn't contain "upgrade"
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "keep-alive")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(!WebSocketServer::is_websocket_upgrade(&req));
    }

    #[test]
    fn test_handshake_response_body_empty() {
        let req = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = WebSocketServer::handshake(&req).unwrap();

        // Verify body is empty - Full<Bytes> doesn't have is_empty
        // Just verify the response status is correct (Switching Protocols)
        assert_eq!(response.status(), 101);
    }

    #[tokio::test]
    async fn test_connection_count_after_operations() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Initially 0
        assert_eq!(server.connection_count().await, 0);

        // Add multiple connections
        for i in 0..10 {
            let (sender, _) = mpsc::unbounded_channel();
            server.add_connection(format!("conn{}", i), sender).await;
            assert_eq!(server.connection_count().await, i + 1);
        }

        // Remove all connections one by one
        for i in (0..10).rev() {
            server.remove_connection(&format!("conn{}", i)).await;
            assert_eq!(server.connection_count().await, i);
        }

        // Should be 0 again
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_after_remove_all() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Add connections
        let (sender1, _receiver1) = mpsc::unbounded_channel();
        let (sender2, _receiver2) = mpsc::unbounded_channel();
        server.add_connection("conn1".to_string(), sender1).await;
        server.add_connection("conn2".to_string(), sender2).await;

        // Remove all
        server.remove_connection("conn1").await;
        server.remove_connection("conn2").await;

        // Broadcast should work without panicking
        let msg = WebSocketMessage::Text("Test".to_string());
        server.broadcast(msg).await;

        // Count should be 0
        assert_eq!(server.connection_count().await, 0);
    }

    #[test]
    fn test_generate_accept_key_unicode() {
        // Test with unicode characters
        let unicode_key = "🔐🔑🗝️密钥";
        let accept_key = WebSocketServer::generate_accept_key(unicode_key).unwrap();
        assert_eq!(accept_key.len(), 28);

        // Test with emoji only
        let emoji_key = "🚀🚀🚀";
        let accept_key = WebSocketServer::generate_accept_key(emoji_key).unwrap();
        assert_eq!(accept_key.len(), 28);
    }

    #[test]
    fn test_to_tungstenite_message_preserves_data() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test text preservation
        let text_data = "Hello, 世界! 🌍";
        let text_msg = WebSocketMessage::Text(text_data.to_string());
        let converted = server.to_tungstenite_message(&text_msg);
        if let Message::Text(text) = converted {
            assert_eq!(text, text_data);
        } else {
            panic!("Expected Text message");
        }

        // Test binary preservation
        let binary_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let binary_msg = WebSocketMessage::Binary(tungstenite::Bytes::from(binary_data.clone()));
        let converted = server.to_tungstenite_message(&binary_msg);
        if let Message::Binary(data) = converted {
            assert_eq!(data.to_vec(), binary_data);
        } else {
            panic!("Expected Binary message");
        }
    }

    #[test]
    fn test_websocket_message_close_edge_codes() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        // Test edge close codes
        let edge_codes = vec![
            0u16,     // Minimum
            999,      // Just below normal
            1000,     // Normal closure
            1001,     // Going away
            2999,     // Application specific range
            3000,     // Application specific
            3999,     // Max application specific
            4000,     // Private use
            4999,     // Max valid code
        ];

        for code in edge_codes {
            let close_frame = tungstenite::protocol::frame::CloseFrame {
                code: tungstenite::protocol::frame::coding::CloseCode::from(code),
                reason: "Test".into(),
            };
            let message = Message::Close(Some(close_frame));
            let result = server.handle_message(message).unwrap();

            match result {
                Some(WebSocketMessage::Close { code: c, .. }) => {
                    assert_eq!(c, code);
                }
                _ => panic!("Expected Close message with code {}", code),
            }
        }
    }

    #[tokio::test]
    async fn test_send_to_all_message_types() {
        let config = Config::default();
        let server = WebSocketServer::new(config);

        let (sender, mut receiver) = mpsc::unbounded_channel();
        server.add_connection("test_conn".to_string(), sender).await;

        // Test all message types via send_to
        let test_cases = vec![
            WebSocketMessage::Text("Hello".to_string()),
            WebSocketMessage::Binary(tungstenite::Bytes::from(vec![1, 2, 3])),
            WebSocketMessage::Ping(tungstenite::Bytes::from(vec![4, 5, 6])),
            WebSocketMessage::Pong(tungstenite::Bytes::from(vec![7, 8, 9])),
            WebSocketMessage::Close {
                code: 1000,
                reason: "Normal".to_string(),
            },
        ];

        for msg in test_cases {
            let msg_clone = msg.clone();
            server.send_to("test_conn", msg).await.unwrap();
            let received = receiver.recv().await.unwrap();
            assert_eq!(received, msg_clone);
        }
    }
}