//! Configuration module for Porta server
//!
//! This module handles loading and parsing TOML configuration files.

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Node configuration
    #[serde(default)]
    pub node: NodeConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// P2P network configuration
    #[serde(default)]
    pub p2p: P2pConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address for the HTTP API
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    /// Listen port for the HTTP API
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            port: default_port(),
        }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8090
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node display name
    #[serde(default = "default_node_name")]
    pub name: String,

    /// Node role: "edge" or "community"
    #[serde(default = "default_role")]
    pub role: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: default_node_name(),
            role: default_role(),
        }
    }
}

fn default_node_name() -> String {
    "Porta Server".to_string()
}

fn default_role() -> String {
    "edge".to_string()
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

fn default_db_path() -> String {
    "porta.db".to_string()
}

/// P2P network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// TCP listen port for P2P connections
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,

    /// QUIC listen port for P2P connections
    #[serde(default = "default_quic_port")]
    pub quic_port: u16,

    /// Enable mDNS for local peer discovery
    #[serde(default = "default_true")]
    pub mdns_enable: bool,

    /// Enable DHT for distributed peer discovery
    #[serde(default = "default_true")]
    pub dht_enable: bool,

    /// External addresses to advertise
    #[serde(default)]
    pub external_addrs: Vec<String>,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            tcp_port: default_tcp_port(),
            quic_port: default_quic_port(),
            mdns_enable: true,
            dht_enable: true,
            external_addrs: Vec::new(),
        }
    }
}

fn default_tcp_port() -> u16 {
    9000
}

fn default_quic_port() -> u16 {
    9001
}

fn default_true() -> bool {
    true
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format: "compact", "pretty", or "json"
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "compact".to_string()
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }

    /// Load configuration from a TOML file, or return defaults if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if path.exists() {
            Self::load(path)
        } else {
            tracing::info!("Config file not found, using defaults: {}", path.display());
            Ok(Self::default())
        }
    }

    /// Get the full server bind address
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.listen_addr, self.server.port)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate role
        if self.node.role != "edge" && self.node.role != "community" {
            anyhow::bail!("Invalid node role: '{}'. Must be 'edge' or 'community'", self.node.role);
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            anyhow::bail!("Invalid log level: '{}'. Must be one of: {:?}", self.logging.level, valid_levels);
        }

        // Validate log format
        let valid_formats = ["compact", "pretty", "json"];
        if !valid_formats.contains(&self.logging.format.to_lowercase().as_str()) {
            anyhow::bail!("Invalid log format: '{}'. Must be one of: {:?}", self.logging.format, valid_formats);
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            node: NodeConfig::default(),
            database: DatabaseConfig::default(),
            p2p: P2pConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8090);
        assert_eq!(config.node.role, "edge");
        assert!(config.p2p.mdns_enable);
    }

    #[test]
    fn test_parse_minimal_toml() {
        let toml_str = r#"
[server]
port = 9090
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.listen_addr, "0.0.0.0");
    }

    #[test]
    fn test_parse_full_toml() {
        let toml_str = r#"
[server]
listen_addr = "127.0.0.1"
port = 8080

[node]
name = "Test Node"
role = "community"

[database]
path = "/var/lib/porta/data.db"

[p2p]
tcp_port = 9000
quic_port = 9001
mdns_enable = false
dht_enable = true

[logging]
level = "debug"
format = "pretty"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.listen_addr, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.node.name, "Test Node");
        assert_eq!(config.node.role, "community");
        assert!(!config.p2p.mdns_enable);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_role() {
        let mut config = Config::default();
        config.node.role = "invalid".to_string();
        assert!(config.validate().is_err());
    }
}
