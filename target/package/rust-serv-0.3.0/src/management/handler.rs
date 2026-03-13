//! Management API handler
//!
//! This module provides handlers for management API endpoints.

use super::config::ManagementConfig;
use super::stats::StatsCollector;
use super::json_response;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use serde::{Deserialize, Serialize};

/// Management API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagementResponse {
    /// Response status
    pub status: String,
}

/// Management API handler
#[derive(Debug, Clone)]
pub struct ManagementHandler {
    config: ManagementConfig,
    stats: StatsCollector,
}

impl ManagementHandler {
    /// Create a new management handler
    pub fn new(config: ManagementConfig, stats: StatsCollector) -> Self {
        Self { config, stats }
    }

    /// Create a new management handler with default stats collector
    pub fn with_config(config: ManagementConfig) -> Self {
        Self {
            config,
            stats: StatsCollector::new(),
        }
    }

    /// Get the stats collector
    pub fn stats(&self) -> &StatsCollector {
        &self.stats
    }

    /// Get a clone of the stats collector
    pub fn stats_collector(&self) -> StatsCollector {
        self.stats.clone()
    }

    /// Check if a path matches a management endpoint
    pub fn is_management_path(&self, path: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        path == self.config.health_path
            || path == self.config.ready_path
            || path == self.config.stats_path
    }

    /// Handle a management API request
    pub fn handle_request(
        &self,
        req: &Request<hyper::body::Incoming>,
    ) -> Option<Response<Full<Bytes>>> {
        if !self.config.enabled {
            return None;
        }

        let path = req.uri().path();

        if path == self.config.health_path {
            Some(self.handle_health())
        } else if path == self.config.ready_path {
            Some(self.handle_ready())
        } else if path == self.config.stats_path {
            Some(self.handle_stats())
        } else {
            None
        }
    }

    /// Handle health check request
    /// GET /health -> 200 OK {"status":"healthy"}
    pub fn handle_health(&self) -> Response<Full<Bytes>> {
        let response = ManagementResponse {
            status: "healthy".to_string(),
        };
        json_response(200, &serde_json::to_string(&response).unwrap())
    }

    /// Handle readiness check request
    /// GET /ready -> 200 OK {"status":"ready"} or 503 {"status":"not_ready"}
    pub fn handle_ready(&self) -> Response<Full<Bytes>> {
        if self.stats.is_ready() {
            let response = ManagementResponse {
                status: "ready".to_string(),
            };
            json_response(200, &serde_json::to_string(&response).unwrap())
        } else {
            let response = ManagementResponse {
                status: "not_ready".to_string(),
            };
            json_response(503, &serde_json::to_string(&response).unwrap())
        }
    }

    /// Handle stats request
    /// GET /stats -> JSON statistics
    pub fn handle_stats(&self) -> Response<Full<Bytes>> {
        let stats = self.stats.get_stats();
        json_response(200, &serde_json::to_string(&stats).unwrap())
    }

    /// Get the health path
    pub fn health_path(&self) -> &str {
        &self.config.health_path
    }

    /// Get the ready path
    pub fn ready_path(&self) -> &str {
        &self.config.ready_path
    }

    /// Get the stats path
    pub fn stats_path(&self) -> &str {
        &self.config.stats_path
    }

    /// Check if management API is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    // Skip request-based tests - test handler methods directly
    // Creating hyper::body::Incoming for tests is complex
    // The handle_request method is tested via integration tests

    #[test]
    fn test_management_handler_creation() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config.clone());
        assert!(handler.is_enabled());
        assert_eq!(handler.health_path(), "/health");
        assert_eq!(handler.ready_path(), "/ready");
        assert_eq!(handler.stats_path(), "/stats");
    }

    #[test]
    fn test_handle_health() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        let response = handler.handle_health();
        assert_eq!(response.status(), 200);

        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("healthy"));
    }

    #[test]
    fn test_handle_ready_when_ready() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().set_ready(true);

        let response = handler.handle_ready();
        assert_eq!(response.status(), 200);

        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("ready"));
    }

    #[test]
    fn test_handle_ready_when_not_ready() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().set_ready(false);

        let response = handler.handle_ready();
        assert_eq!(response.status(), 503);

        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("not_ready"));
    }

    #[test]
    fn test_handle_stats() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().increment_requests();
        handler.stats().increment_connections();

        let response = handler.handle_stats();
        assert_eq!(response.status(), 200);

        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("total_requests"));
        assert!(body_str.contains("active_connections"));
    }

    #[test]
    fn test_is_management_path_enabled() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        assert!(handler.is_management_path("/health"));
        assert!(handler.is_management_path("/ready"));
        assert!(handler.is_management_path("/stats"));
        assert!(!handler.is_management_path("/other"));
    }

    #[test]
    fn test_is_management_path_disabled() {
        let config = ManagementConfig {
            enabled: false,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        assert!(!handler.is_management_path("/health"));
        assert!(!handler.is_management_path("/ready"));
        assert!(!handler.is_management_path("/stats"));
    }

    // Note: handle_request tests require creating hyper::body::Incoming
    // which is complex in unit tests. These are covered by integration tests.
    // The individual handler methods (handle_health, handle_ready, handle_stats)
    // are tested above.

    #[test]
    fn test_custom_paths() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/api/health".to_string(),
            ready_path: "/api/ready".to_string(),
            stats_path: "/api/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        assert!(handler.is_management_path("/api/health"));
        assert!(handler.is_management_path("/api/ready"));
        assert!(handler.is_management_path("/api/stats"));
        assert!(!handler.is_management_path("/health"));
    }

    #[test]
    fn test_handler_clone() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().increment_requests();

        let cloned = handler.clone();
        assert_eq!(cloned.stats().get_stats().total_requests, 1);
    }

    #[test]
    fn test_stats_collector_from_handler() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().increment_requests();
        handler.stats().add_bytes_sent(1000);

        let collector = handler.stats_collector();
        let stats = collector.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.bytes_sent, 1000);
    }

    #[test]
    fn test_management_response_serialization() {
        let response = ManagementResponse {
            status: "healthy".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"status":"healthy"}"#);
    }

    #[test]
    fn test_management_response_deserialization() {
        let json = r#"{"status":"ready"}"#;
        let response: ManagementResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ready");
    }

    #[test]
    fn test_management_response_equality() {
        let response1 = ManagementResponse {
            status: "healthy".to_string(),
        };
        let response2 = ManagementResponse {
            status: "healthy".to_string(),
        };
        assert_eq!(response1, response2);
    }

    #[test]
    fn test_new_with_config_and_stats() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let stats = StatsCollector::new();
        stats.increment_requests();
        stats.increment_requests();

        let handler = ManagementHandler::new(config, stats);
        assert_eq!(handler.stats().get_stats().total_requests, 2);
    }

    #[test]
    fn test_stats_json_format() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        handler.stats().increment_requests();
        handler.stats().record_cache_hit();
        handler.stats().record_cache_miss();

        let response = handler.handle_stats();
        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();

        // Verify JSON structure
        assert!(body_str.contains("\"active_connections\":0"));
        assert!(body_str.contains("\"total_requests\":1"));
        assert!(body_str.contains("\"cache_hit_rate\":0.5"));
    }

    #[test]
    fn test_health_response_content_type() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        let response = handler.handle_health();
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_ready_response_content_type() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        let response = handler.handle_ready();
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_stats_response_content_type() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        let response = handler.handle_stats();
        let content_type = response.headers().get("Content-Type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_handler_debug() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("ManagementHandler"));
    }

    #[test]
    fn test_multiple_requests_increment_counter() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        // Simulate multiple requests by incrementing stats
        for _ in 0..10 {
            handler.stats().increment_requests();
        }

        let response = handler.handle_stats();
        let body = response.into_body();
        let bytes = futures::executor::block_on(body.collect()).unwrap().to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("\"total_requests\":10"));
    }

    #[test]
    fn test_health_not_ready_state() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let handler = ManagementHandler::with_config(config);

        // Health check should still return 200 even if server is not ready
        handler.stats().set_ready(false);
        let response = handler.handle_health();
        assert_eq!(response.status(), 200);
    }
}
