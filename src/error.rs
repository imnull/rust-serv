use std::io;
use std::net::AddrParseError;
use thiserror::Error;

/// Error type for the static file server
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Path security error: {0}")]
    PathSecurity(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Address parse error: {0}")]
    AddrParse(#[from] AddrParseError),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
