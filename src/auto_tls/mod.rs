//! Auto TLS (Let's Encrypt) module
//!
//! This module provides automatic TLS certificate management using Let's Encrypt.

mod config;
mod client;
mod challenge;
mod renewer;

pub use config::AutoTlsConfig;
pub use client::AcmeClient;
pub use challenge::ChallengeHandler;
pub use renewer::CertificateRenewer;

#[cfg(test)]
mod tests {
    // Tests will be added when implementing the full module
}
