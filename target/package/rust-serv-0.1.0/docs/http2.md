# HTTP/2 Support Documentation

## Overview

This document describes the HTTP/2 functionality implemented in the Rust static server. HTTP/2 support provides improved performance through multiplexing, header compression, and server push capabilities.

## Features

### 1. Server Push

Server Push allows the server to proactively send resources to the client before they are requested, reducing latency for page loads.

#### Implementation

The HTTP/2 server implements two types of push:

- **Server-initiated push**: The server proactively pushes resources to the client
- **Client-initiated push**: The client can initiate push requests for specific resources

#### Usage

```rust
use rust_serv::server::Http2Server;

let server = Http2Server::new(config, handler);

// Handle server-initiated push
let response = server.handle_push(body, headers, &mut stream).await?;

// Handle client-initiated push
let response = server.handle_client_push(body, headers, &mut stream).await?;
```

#### Push Detection

The server can detect HTTP/2 push requests:

```rust
use rust_serv::server::Http2Server;
use hyper::Method;

let is_push = Http2Server::is_http2_push(&Method::POST, &headers);
```

### 2. Content Encoding

HTTP/2 supports content encoding to reduce payload size.

#### Supported Encodings

- **Gzip**: Compresses responses using gzip compression
- **Identity**: No compression (default)

#### Configuration

Content encoding is automatically negotiated based on client preferences:

```rust
// The server accepts client encoding preferences
let accept_encoding = headers.get("accept-encoding")
    .and_then(|enc| enc.to_str().ok())
    .unwrap_or("");
```

### 3. Flow Control

The HTTP/2 server implements basic flow control to prevent overwhelming clients:

- **Maximum body size**: 65536 bytes (64KB)
- **Connection limits**: Configurable via `max_connections` setting

#### Example

```toml
# Configuration file
max_connections = 1000
connection_timeout_secs = 30
```

### 4. Priority Ordering

The server respects request priority to ensure critical resources are delivered first.

## Configuration

### HTTP/2 Specific Settings

```toml
# Server configuration
port = 8080
root = "./public"
enable_compression = true
enable_tls = true

# HTTP/2 settings
connection_timeout_secs = 30
max_connections = 1000

# TLS settings (required for HTTP/2)
tls_cert = "./cert.pem"
tls_key = "./key.pem"
```

## API Reference

### Http2Server

#### Constructor

```rust
pub fn new(config: Config, handler: Arc<Handler>) -> Self
```

Creates a new HTTP/2 server instance.

**Parameters:**
- `config`: Server configuration
- `handler`: Request handler

#### Methods

##### handle_push

```rust
pub async fn handle_push(
    &self,
    body: Bytes,
    headers: &HeaderMap,
    stream: &mut (dyn AsyncWrite + Unpin),
) -> Result<Response<Full<Bytes>>, Error>
```

Handles HTTP/2 server-initiated push requests.

**Parameters:**
- `body`: Request body bytes
- `headers`: Request headers
- `stream`: Async write stream for the connection

**Returns:**
- HTTP response with push acknowledgment

##### handle_client_push

```rust
pub async fn handle_client_push(
    &self,
    body: Bytes,
    headers: &HeaderMap,
    stream: &mut (dyn AsyncWrite + Unpin),
) -> Result<Response<Full<Bytes>>, Error>
```

Handles HTTP/2 client-initiated push requests.

**Parameters:**
- `body`: Request body bytes
- `headers`: Request headers
- `stream`: Async write stream for the connection

**Returns:**
- HTTP response with push acknowledgment

##### is_http2_push

```rust
pub fn is_http2_push(method: &Method, headers: &HeaderMap) -> bool
```

Determines if a request is an HTTP/2 push request.

**Parameters:**
- `method`: HTTP method
- `headers`: Request headers

**Returns:**
- `true` if the request is an HTTP/2 push, `false` otherwise

##### create_http2_response

```rust
pub fn create_http2_response(
    status: StatusCode,
    body: Bytes,
    headers: &HeaderMap,
    content_encoding: Option<&str>,
) -> Response<Full<Bytes>>
```

Creates an HTTP/2 response with appropriate headers.

**Parameters:**
- `status`: HTTP status code
- `body`: Response body
- `headers`: Request headers for reference
- `content_encoding`: Optional content encoding header

**Returns:**
- HTTP response configured for HTTP/2

## Performance Considerations

### Benefits of HTTP/2

1. **Multiplexing**: Multiple requests can be sent over a single connection
2. **Header Compression**: HPACK compression reduces header overhead
3. **Server Push**: Proactive resource delivery reduces latency
4. **Binary Protocol**: More efficient parsing than HTTP/1.1

### Optimization Tips

1. **Enable Compression**: Set `enable_compression = true` in configuration
2. **Use Server Push**: Proactively push critical resources (CSS, JS)
3. **Monitor Flow Control**: Adjust `max_connections` based on server capacity
4. **TLS Termination**: Use dedicated TLS termination for production deployments

## Testing

The HTTP/2 implementation includes comprehensive tests:

```bash
# Run all HTTP/2 tests
cargo test --package rust_serv --lib -- server::http2

# Run specific test
cargo test --package rust_serv --lib -- test_handle_empty_push
```

### Test Coverage

- Empty push handling
- Body size validation
- Content encoding negotiation
- Client-initiated push
- Push detection
- Priority ordering

## Security Considerations

### Best Practices

1. **TLS Required**: HTTP/2 requires TLS for secure connections
2. **Certificate Management**: Use certificates from trusted CAs (Let's Encrypt, etc.)
3. **Body Size Limits**: Enforce maximum body sizes to prevent DoS attacks
4. **Connection Limits**: Set appropriate `max_connections` values

### Configuration

```toml
# Security settings
enable_tls = true
connection_timeout_secs = 30
max_connections = 1000
```

## Troubleshooting

### Common Issues

#### 1. HTTP/2 Not Working

**Symptom:** Server falls back to HTTP/1.1

**Solution:** Ensure TLS is properly configured:
```toml
enable_tls = true
tls_cert = "./cert.pem"
tls_key = "./key.pem"
```

#### 2. Push Requests Failing

**Symptom:** `Client-initiated push requires Accept-Encoding: http/2`

**Solution:** Include proper headers:
```http
Accept-Encoding: http/2
Content-Type: application/http2+push
```

#### 3. Body Size Errors

**Symptom:** `Body too large for HTTP/2 push`

**Solution:** Reduce body size to under 65536 bytes

### Debug Logging

Enable debug logging for detailed HTTP/2 information:

```toml
log_level = "debug"
```

## Migration from HTTP/1.1

### Configuration Changes

```toml
# HTTP/1.1 configuration
port = 8080
enable_compression = true

# Add for HTTP/2
enable_tls = true
tls_cert = "./cert.pem"
tls_key = "./key.pem"
```

### Code Changes

No code changes required - HTTP/2 is automatically enabled when TLS is configured.

## Future Enhancements

Planned improvements to HTTP/2 support:

1. **Advanced Flow Control**: Per-stream flow control
2. **Push Cancellation**: Ability to cancel pending pushes
3. **Connection Reuse**: Enhanced connection pooling
4. **Metrics**: HTTP/2-specific performance metrics

## References

- [HTTP/2 Specification (RFC 7540)](https://httpwg.org/specs/rfc7540.html)
- [Hyper HTTP Library](https://hyper.rs/)
- [Tokio Runtime](https://tokio.rs/)
- [Rustls TLS Library](https://github.com/rustls/rustls)

## Support

For issues or questions about HTTP/2 support:

1. Check the [troubleshooting section](#troubleshooting)
2. Review the [test suite](#testing) for usage examples
3. Consult the [API reference](#api-reference)
4. Review the [configuration options](#configuration)

## License

This HTTP/2 implementation is part of the Rust static server and is licensed under the same terms as the main project.