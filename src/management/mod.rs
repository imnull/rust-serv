//! Management API module
//!
//! This module provides management API endpoints for health checks,
//! readiness checks, runtime statistics, and plugin management.

mod config;
mod handler;
mod plugins;
mod stats;

pub use config::ManagementConfig;
pub use handler::{ManagementHandler, ManagementResponse};
pub use plugins::{
    PluginManagementHandler,
    PluginListResponse,
    PluginInfo,
    LoadPluginRequest,
    UpdatePluginRequest,
};
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

    #[test]
    fn test_json_response_404() {
        let response = json_response(404, r#"{"error":"not_found"}"#);
        assert_eq!(response.status(), 404);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_json_response_400() {
        let response = json_response(400, r#"{"error":"bad_request"}"#);
        assert_eq!(response.status(), 400);
    }

    #[test]
    fn test_json_response_401() {
        let response = json_response(401, r#"{"error":"unauthorized"}"#);
        assert_eq!(response.status(), 401);
    }

    #[test]
    fn test_json_response_403() {
        let response = json_response(403, r#"{"error":"forbidden"}"#);
        assert_eq!(response.status(), 403);
    }

    #[test]
    fn test_json_response_empty_body() {
        let response = json_response(200, "");
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_json_response_unicode_body() {
        let response = json_response(200, r#"{"message":"你好世界"}"#);
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_json_response_large_body() {
        let large_body = "x".repeat(10000);
        let json_body = format!("{{\"data\":\"{}\"}}", large_body);
        let response = json_response(200, &json_body);
        assert_eq!(response.status(), 200);
    }
}
