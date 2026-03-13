//! Custom error pages module
//!
//! This module provides customizable error page templates.

mod templates;
mod handler;

pub use templates::ErrorTemplates;
pub use handler::ErrorPageHandler;
