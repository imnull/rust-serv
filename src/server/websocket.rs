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
}