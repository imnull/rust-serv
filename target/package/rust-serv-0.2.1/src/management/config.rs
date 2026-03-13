//! Management API configuration
//!
//! Configuration for the management API endpoints.

use serde::{Deserialize, Serialize};

/// Re-export ManagementConfig from the main config module
pub use crate::config::ManagementConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_management_config_creation() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        assert!(config.enabled);
        assert_eq!(config.health_path, "/health");
        assert_eq!(config.ready_path, "/ready");
        assert_eq!(config.stats_path, "/stats");
    }

    #[test]
    fn test_management_config_clone() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_management_config_debug() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ManagementConfig"));
        assert!(debug_str.contains("enabled: true"));
    }

    #[test]
    fn test_management_config_serialization() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"health_path\":\"/health\""));
    }

    #[test]
    fn test_management_config_deserialization() {
        let json = r#"{"enabled":true,"health_path":"/health","ready_path":"/ready","stats_path":"/stats"}"#;
        let config: ManagementConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.health_path, "/health");
    }

    #[test]
    fn test_management_config_custom_paths() {
        let config = ManagementConfig {
            enabled: true,
            health_path: "/api/v1/health".to_string(),
            ready_path: "/api/v1/ready".to_string(),
            stats_path: "/api/v1/stats".to_string(),
        };
        assert_eq!(config.health_path, "/api/v1/health");
        assert_eq!(config.ready_path, "/api/v1/ready");
        assert_eq!(config.stats_path, "/api/v1/stats");
    }

    #[test]
    fn test_management_config_disabled() {
        let config = ManagementConfig {
            enabled: false,
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
            stats_path: "/stats".to_string(),
        };
        assert!(!config.enabled);
    }
}
