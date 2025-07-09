//! Configuration for data ingestion

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data ingestion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIngestionConfig {
    /// Redis connection URL
    pub redis_url: String,
    
    /// Polygon.io API key
    pub polygon_api_key: String,
    
    /// WebSocket connection settings
    pub websocket: WebSocketConfig,
    
    /// Subscribed symbols
    pub symbols: Vec<String>,
    
    /// Data validation settings
    pub validation: ValidationConfig,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    
    /// Reconnection attempts
    pub max_reconnect_attempts: u32,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    
    /// Buffer size for incoming messages
    pub buffer_size: usize,
}

/// Data validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Maximum price deviation from last known price (percentage)
    pub max_price_deviation: f64,
    
    /// Maximum timestamp lag in seconds
    pub max_timestamp_lag: i64,
    
    /// Enable strict validation
    pub strict_validation: bool,
}

impl Default for DataIngestionConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            polygon_api_key: String::new(),
            websocket: WebSocketConfig::default(),
            symbols: vec!["SPY".to_string(), "QQQ".to_string()],
            validation: ValidationConfig::default(),
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 30,
            max_reconnect_attempts: 5,
            heartbeat_interval: 30,
            buffer_size: 10000,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_price_deviation: 10.0, // 10%
            max_timestamp_lag: 5, // 5 seconds
            strict_validation: true,
        }
    }
}