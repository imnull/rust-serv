# WebSocket Support Documentation

## Overview

This document describes the WebSocket functionality implemented in the Rust static server. WebSocket support enables real-time bidirectional communication between clients and the server over a single persistent connection.

## Features

### 1. WebSocket Handshake

The server implements the WebSocket handshake protocol defined in RFC 6455, allowing clients to upgrade from HTTP to WebSocket connections.

#### Implementation

The WebSocket handshake validates:
- WebSocket version (must be version 13)
- Required headers (`Upgrade`, `Connection`, `Sec-WebSocket-Key`)
- Generates proper `Sec-WebSocket-Accept` response header

#### Usage

```rust
use rust_serv::server::WebSocketServer;
use hyper::Request;

let server = WebSocketServer::new(config);

// Check if request is WebSocket upgrade
if WebSocketServer::is_websocket_upgrade(&req) {
    // Perform handshake
    let response = server.handshake(&req)?;
    // Send 101 Switching Protocols response
}
```

### 2. Message Types

The server supports various WebSocket message types:

#### WebSocketMessage Enum

```rust
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
```

### 3. Connection Management

The WebSocket server provides comprehensive connection management capabilities:

#### Features

- **Connection Tracking**: Track all active connections with unique IDs
- **Message Broadcasting**: Send messages to all connected clients
- **Individual Messaging**: Send messages to specific connections
- **Automatic Cleanup**: Remove disconnected clients automatically
- **Connection Info**: Track remote address and connection time

#### API

```rust
// Add a connection
server.add_connection(connection_id, sender).await;

// Remove a connection
server.remove_connection(&connection_id).await;

// Broadcast to all clients
server.broadcast(WebSocketMessage::Text("Hello all!")).await;

// Send to specific client
server.send_to(&connection_id, WebSocketMessage::Text("Hello!")).await?;

// Get connection count
let count = server.connection_count().await;

// List all connections
let connections = server.list_connections().await;
```

### 4. Message Broadcasting

The server supports efficient message broadcasting to all connected clients:

#### Features

- **Automatic Cleanup**: Failed connections are removed automatically
- **Error Handling**: Handles connection failures gracefully
- **Type Safety**: Strongly typed message system

#### Example

```rust
// Broadcast a text message to all clients
server.broadcast(WebSocketMessage::Text("Server announcement")).await;

// Broadcast binary data
server.broadcast(WebSocketMessage::Binary(tungstenite::Bytes::from(data))).await;
```

### 5. Auto-Ping/Pong

The server automatically responds to ping messages with pong messages to maintain connection health:

```rust
// Ping is handled automatically - no manual intervention needed
Message::Ping(data) => {
    // Auto-respond to ping with pong
    Ok(Some(WebSocketMessage::Pong(data)))
}
```

## Configuration

### WebSocket-Specific Settings

The WebSocket server uses the same configuration as the main HTTP server:

```toml
# Server configuration
port = 8080
root = "./public"
enable_compression = true

# Connection settings
connection_timeout_secs = 30
max_connections = 1000

# Optional: Enable WebSocket support
# Note: WebSocket is always enabled, but these settings affect connection management
```

## API Reference

### WebSocketServer

#### Constructor

```rust
pub fn new(config: Config) -> Self
```

Creates a new WebSocket server instance.

**Parameters:**
- `config`: Server configuration

#### Methods

##### is_websocket_upgrade

```rust
pub fn is_websocket_upgrade<B>(req: &Request<B>) -> bool
```

Determines if a request is a WebSocket upgrade request.

**Parameters:**
- `req`: HTTP request

**Returns:**
- `true` if the request is a WebSocket upgrade, `false` otherwise

##### handshake

```rust
pub fn handshake<B>(req: &Request<B>) -> Result<Response<Full<Bytes>>, Error>
```

Performs WebSocket handshake and returns the upgrade response.

**Parameters:**
- `req`: HTTP upgrade request

**Returns:**
- HTTP response with status 101 (Switching Protocols) and WebSocket accept headers

##### generate_accept_key

```rust
pub fn generate_accept_key(ws_key: &str) -> Result<String, Error>
```

Generates the WebSocket accept key from the WebSocket key.

**Parameters:**
- `ws_key`: WebSocket key from client request

**Returns:**
- Base64-encoded accept key

##### add_connection

```rust
pub async fn add_connection(&self, id: String, sender: mpsc::UnboundedSender<WebSocketMessage>)
```

Adds a new WebSocket connection to the connection pool.

**Parameters:**
- `id`: Unique connection identifier
- `sender`: Message sender for this connection

##### remove_connection

```rust
pub async fn remove_connection(&self, id: &str)
```

Removes a WebSocket connection from the connection pool.

**Parameters:**
- `id`: Connection identifier to remove

##### broadcast

```rust
pub async fn broadcast(&self, message: WebSocketMessage)
```

Broadcasts a message to all connected clients.

**Parameters:**
- `message`: Message to broadcast

**Behavior:**
- Removes failed connections automatically
- Handles connection errors gracefully

##### send_to

```rust
pub async fn send_to(&self, id: &str, message: WebSocketMessage) -> Result<(), Error>
```

Sends a message to a specific connection.

**Parameters:**
- `id`: Target connection identifier
- `message`: Message to send

**Returns:**
- `Ok(())` if successful, `Err` if connection not found or sending failed

##### connection_count

```rust
pub async fn connection_count(&self) -> usize
```

Gets the number of active WebSocket connections.

**Returns:**
- Count of active connections

##### list_connections

```rust
pub async fn list_connections(&self) -> Vec<String>
```

Gets a list of all active connection IDs.

**Returns:**
- Vector of connection identifiers

##### handle_message

```rust
pub fn handle_message(&self, message: Message) -> Result<Option<WebSocketMessage>, Error>
```

Handles incoming WebSocket messages and converts them to internal message types.

**Parameters:**
- `message`: Incoming WebSocket message

**Returns:**
- `Some(WebSocketMessage)` if the message should be processed, `None` otherwise

**Behavior:**
- Automatically responds to ping messages with pong
- Handles close messages
- Ignores raw frames

##### to_tungstenite_message

```rust
pub fn to_tungstenite_message(&self, message: &WebSocketMessage) -> Message
```

Converts internal WebSocket message to tungstenite message format.

**Parameters:**
- `message`: Internal WebSocket message

**Returns:**
- Tungstenite message ready for sending

### WebSocketHandler

#### Constructor

```rust
pub fn new(server: Arc<WebSocketServer>, connection_id: String) -> (Self, mpsc::UnboundedSender<WebSocketMessage>)
```

Creates a new WebSocket handler with message channel.

**Parameters:**
- `server`: WebSocket server instance
- `connection_id`: Unique connection identifier

**Returns:**
- Tuple of (handler, sender) where sender can be used to send messages to this handler

#### Methods

##### handle_connection

```rust
pub async fn handle_connection<T>(&mut self, ws_stream: WebSocketStream<T>) -> Result<(), Error>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::marker::Send + 'static,
```

Handles a WebSocket connection, processing incoming and outgoing messages.

**Parameters:**
- `ws_stream`: WebSocket stream for the connection

**Behavior:**
- Uses `tokio::select!` to handle both incoming and outgoing messages concurrently
- Automatically cleans up connection on close or error
- Handles ping/pong automatically
- Removes connection from server pool on completion

## Usage Examples

### Basic WebSocket Server

```rust
use rust_serv::server::{WebSocketServer, WebSocketHandler, WebSocketMessage};
use hyper::{Request, Response, StatusCode};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

async fn websocket_handler(req: Request<impl hyper::body::Body>) -> Result<Response<impl hyper::body::Body>, Error> {
    let server = WebSocketServer::new(config);

    // Check for WebSocket upgrade
    if WebSocketServer::is_websocket_upgrade(&req) {
        // Perform handshake
        let response = server.handshake(&req)?;

        // Here you would normally upgrade the connection
        // For actual implementation, you need access to the underlying TCP stream
        // This is a simplified example

        Ok(response)
    } else {
        // Handle regular HTTP request
        Ok(Response::builder().status(StatusCode::OK).body("Hello World").unwrap())
    }
}
```

### Broadcasting Messages

```rust
// Broadcast text message to all connected clients
let message = WebSocketMessage::Text("Server announcement: New features available!");
server.broadcast(message).await;

// Broadcast binary data
let data = tungstenite::Bytes::from(vec![1, 2, 3, 4]);
let binary_message = WebSocketMessage::Binary(data);
server.broadcast(binary_message).await;
```

### Individual Messaging

```rust
// Send message to specific client
let target_client_id = "client_123";
let message = WebSocketMessage::Text("Hello, client!");
server.send_to(&target_client_id, message).await?;
```

### Connection Management

```rust
// Check active connections
let count = server.connection_count().await;
println!("Active connections: {}", count);

// List all connection IDs
let connections = server.list_connections().await;
for (i, id) in connections.iter().enumerate() {
    println!("Connection {}: {}", i + 1, id);
}
```

## Testing

The WebSocket implementation includes comprehensive tests:

```bash
# Run all WebSocket tests
cargo test --package rust_serv --lib -- server::websocket

# Run specific test
cargo test --package rust_serv --lib -- test_websocket_upgrade_detection
```

### Test Coverage

- ✅ WebSocket upgrade detection
- ✅ Handshake protocol
- ✅ Message type conversion
- ✅ Connection management
- ✅ Message broadcasting
- ✅ Version validation

### Example Test

```rust
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
```

## Performance Considerations

### Benefits of WebSocket

1. **Persistent Connection**: Single connection for multiple message exchanges
2. **Lower Latency**: No HTTP header overhead for each message
3. **Full Duplex**: Bidirectional communication without request/response cycle
4. **Efficient**: Binary data transmission without encoding overhead

### Optimization Tips

1. **Connection Pooling**: Reuse connections when possible
2. **Message Batching**: Send multiple messages together when possible
3. **Heartbeat**: Use ping/pong to detect dead connections
4. **Error Handling**: Handle network errors gracefully
5. **Resource Cleanup**: Remove disconnected connections promptly

## Security Considerations

### Best Practices

1. **Origin Validation**: Validate request origin headers
2. **Rate Limiting**: Implement per-connection message rate limits
3. **Message Size Limits**: Enforce maximum message sizes
4. **Connection Limits**: Set appropriate `max_connections` values
5. **Input Validation**: Validate all incoming messages
6. **TLS Encryption**: Use WebSocket over HTTPS (wss://)

### Configuration

```toml
# Security settings
enable_tls = true
connection_timeout_secs = 30
max_connections = 1000

# Add custom validation as needed
```

### WebSocket-Specific Security

```rust
// Example: Validate origin header
let origin = req.headers()
    .get("origin")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

if !is_origin_allowed(&origin) {
    return Err(Error::Http("Origin not allowed".to_string()));
}

// Example: Rate limiting per connection
const MAX_MESSAGES_PER_SECOND: usize = 10;
// Implement rate limiting logic...
```

## Troubleshooting

### Common Issues

#### 1. WebSocket Handshake Fails

**Symptom:** Handshake returns error or wrong status code

**Solutions:**
- Ensure all required headers are present: `Upgrade`, `Connection`, `Sec-WebSocket-Key`, `Sec-WebSocket-Version`
- Verify WebSocket version is "13"
- Check that client supports the WebSocket protocol

#### 2. Connection Drops Immediately

**Symptom:** Connection closes after handshake

**Solutions:**
- Check for TLS issues (if using wss://)
- Verify firewall settings
- Ensure proper timeout configuration
- Check server logs for specific error messages

#### 3. Messages Not Received

**Symptom:** Client sends messages but server doesn't receive them

**Solutions:**
- Verify message format is correct
- Check for proper framing
- Ensure message size limits aren't being exceeded
- Review handler implementation

#### 4. Broadcast Not Working

**Symptom:** Some clients don't receive broadcast messages

**Solutions:**
- Check for failed connections in the connection pool
- Verify connection cleanup is working
- Monitor for connection errors
- Review broadcaster error handling

### Debug Logging

Enable debug logging for detailed WebSocket information:

```toml
log_level = "debug"
```

## Integration with HTTP Server

### WebSocket Upgrade Detection

The WebSocket server can be integrated into the main HTTP server by detecting upgrade requests:

```rust
use rust_serv::server::{Server, WebSocketServer};

let server = Server::new(config);

// In your request handler
if WebSocketServer::is_websocket_upgrade(&req) {
    // Handle WebSocket upgrade
    let ws_server = WebSocketServer::new(config.clone());
    let response = ws_server.handshake(&req)?;

    // Upgrade connection to WebSocket
    // Implementation depends on your server architecture

    Ok(response)
} else {
    // Handle regular HTTP request
    server.handle_request(req).await
}
```

## Future Enhancements

Planned improvements to WebSocket support:

1. **Subprotocol Support**: Support for custom WebSocket subprotocols
2. **Message Queues**: Persistent message queues for offline clients
3. **Room Support**: Channel/group-based message routing
4. **Compression**: WebSocket message compression (permessage-deflate)
5. **Authentication**: Token-based authentication for WebSocket connections
6. **Metrics**: WebSocket-specific performance metrics

## References

- [WebSocket Protocol RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [Tungstenite](https://github.com/snapview/tungstenite-rs)
- [Hyper](https://hyper.rs/)

## Comparison with Alternatives

### WebSocket vs HTTP Polling

| Feature | WebSocket | HTTP Polling |
|---------|-----------|---------------|
| Connection | Persistent | New per request |
| Latency | Low | High |
| Server Load | Low | High |
| Bidirectional | Yes | No |
| Overhead | Minimal | HTTP headers per request |

### WebSocket vs Server-Sent Events (SSE)

| Feature | WebSocket | SSE |
|---------|-----------|-----|
| Bidirectional | Yes | No |
| Binary Support | Yes | No |
| Protocol | WebSocket | HTTP |
| Use Case | Interactive apps | Updates/notifications |

## Support

For issues or questions about WebSocket support:

1. Check the [troubleshooting section](#troubleshooting)
2. Review the [testing section](#testing) for usage examples
3. Consult the [API reference](#api-reference)
4. Review the [configuration options](#configuration)

## License

This WebSocket implementation is part of the Rust static server and is licensed under the same terms as the main project.