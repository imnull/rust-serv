// This module re-exports compression utilities from the handler module
// The actual compression implementation is in src/handler/compress.rs
// This maintains backward compatibility with existing imports

pub use crate::handler::compress::{CompressionType, compress, parse_accept_encoding, should_skip_compression};
