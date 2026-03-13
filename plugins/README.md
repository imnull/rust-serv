# rust-serv Plugins

Example WebAssembly plugins for rust-serv.

## Prerequisites

Install the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
```

## Building

```bash
./build.sh
```

Or manually:

```bash
# Build a single plugin
cd examples/add-header
cargo build --target wasm32-unknown-unknown --release
```

## Examples

### 1. add-header

Simple plugin that adds custom headers to responses.

**Use case:** Add security headers, CORS headers, etc.

```bash
cd examples/add-header
cargo build --target wasm32-unknown-unknown --release
```

### 2. rate-limiter

Advanced plugin that implements rate limiting.

**Use case:** Protect your server from abuse.

**Features:**
- Request counting
- Configurable rate limit
- Returns 429 when exceeded

```bash
cd examples/rate-limiter
cargo build --target wasm32-unknown-unknown --release
```

## Plugin Structure

A rust-serv plugin must export:

```rust
// Required exports
#[no_mangle]
pub extern "C" fn plugin_init(config_ptr: i32, config_len: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_request(req_ptr: i32, req_len: i32, result_ptr: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_response(res_ptr: i32, res_len: i32, result_ptr: i32) -> i32;

#[no_mangle]
pub extern "C" fn plugin_unload() -> i32;

// Memory export
#[no_mangle]
pub extern "C" fn memory() -> *mut u8;
```

## Communication Protocol

### Request Format (JSON)

```json
{
  "method": "GET",
  "path": "/api/users",
  "query": {"page": "1"},
  "headers": {"content-type": "application/json"},
  "body": null,
  "client_ip": "127.0.0.1",
  "request_id": "req-123",
  "version": "HTTP/1.1",
  "host": "example.com"
}
```

### Response Format (JSON)

```json
{
  "status": 200,
  "headers": {"x-custom": "value"},
  "body": "base64-encoded-content"
}
```

### Action Format (JSON)

```rust
enum PluginAction {
    Continue,                          // Pass to next plugin
    Intercept(Response),               // Return this response immediately
    ModifyRequest(Request),            // Modify and continue
    ModifyResponse(Response),          // Modify and continue
    Error { message: String },         // Return error
}
```

## Testing

Load a plugin in rust-serv:

```rust
use rust_serv::plugin::{PluginManager, PluginConfig};
use std::path::Path;

let mut manager = PluginManager::new()?;

let id = manager.load(
    Path::new("./plugins/examples/add-header/target/wasm32-unknown-unknown/release/plugin_add_header.wasm"),
    PluginConfig::default()
)?;
```

## Directory Structure

```
plugins/
├── README.md
├── build.sh
└── examples/
    ├── add-header/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    └── rate-limiter/
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

## License

MIT OR Apache-2.0
