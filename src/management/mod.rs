//! Management API module
//!
//! This module provides management API endpoints for health checks,
//! readiness checks, and runtime statistics.

mod config;
mod handler;
mod stats;

pub use config::ManagementConfig;
pub use handler::{ManagementHandler, ManagementResponse};
pub use stats::{ServerStats, StatsCollector};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;

/// Create a JSON response with the given status and body
pub fn json_response(status: u16, body: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_response_creation() {
        let response = json_response(200, r#"{"test":"value"}"#);
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_json_response_500() {
        let response = json_response(500, r#"{"error":"internal"}"#);
        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_json_response_503() {
        let response = json_response(503, r#"{"status":"not_ready"}"#);
        assert_eq!(response.status(), 503);
    }
}
