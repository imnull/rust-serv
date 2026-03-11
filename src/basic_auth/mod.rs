//! Basic Authentication module
//!
//! This module provides HTTP Basic Authentication for protecting routes.

mod authenticator;
mod credentials;
mod validator;

pub use authenticator::BasicAuthenticator;
pub use credentials::Credentials;
pub use validator::AuthValidator;
