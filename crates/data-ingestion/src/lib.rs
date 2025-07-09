//! # Data Ingestion
//!
//! Real-time market data ingestion via WebSocket streams.
//! Handles data from Polygon.io and other market data providers.
//!
//! ## Performance Targets
//! - Sub-microsecond data processing latency
//! - Zero-copy operations where possible
//! - Real-time data validation and normalization
//!
//! ## Architecture
//! - WebSocket client for real-time feeds
//! - Data validation and normalization pipeline
//! - Redis publishing for downstream services

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub mod config;
pub mod polygon;
pub mod validation;
pub mod websocket;

/// Market data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketData {
    Trade(TradeData),
    Quote(QuoteData),
    Aggregate(AggregateData),
}

/// Trade data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub symbol: String,
    pub price: f64,
    pub size: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange: String,
    pub conditions: Vec<String>,
}

/// Quote data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub symbol: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: u64,
    pub ask_size: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange: String,
}

/// Aggregate data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateData {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub timespan: String,
}

/// Data ingestion manager
pub struct DataIngestionManager {
    config: config::DataIngestionConfig,
    redis_client: redis::Client,
    market_data_tx: broadcast::Sender<MarketData>,
    websocket_manager: websocket::WebSocketManager,
}

impl DataIngestionManager {
    /// Create new data ingestion manager
    pub async fn new(config: config::DataIngestionConfig) -> Result<Self> {
        let redis_client = redis::Client::open(config.redis_url.clone())?;
        let (market_data_tx, _) = broadcast::channel(10000);
        let websocket_manager = websocket::WebSocketManager::new(&config).await?;

        Ok(Self {
            config,
            redis_client,
            market_data_tx,
            websocket_manager,
        })
    }

    /// Start data ingestion
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting data ingestion manager");
        
        // Start WebSocket connections
        self.websocket_manager.start().await?;
        
        // Start data processing loop
        self.process_data().await?;
        
        Ok(())
    }

    /// Process incoming market data
    async fn process_data(&mut self) -> Result<()> {
        let mut rx = self.websocket_manager.subscribe();
        
        while let Ok(data) = rx.recv().await {
            // Validate data
            if let Err(e) = validation::validate_market_data(&data) {
                warn!("Invalid market data: {:?}", e);
                continue;
            }
            
            // Publish to Redis
            if let Err(e) = self.publish_to_redis(&data).await {
                error!("Failed to publish to Redis: {:?}", e);
            }
            
            // Broadcast to local subscribers
            if let Err(e) = self.market_data_tx.send(data) {
                warn!("Failed to broadcast market data: {:?}", e);
            }
        }
        
        Ok(())
    }

    /// Publish market data to Redis
    async fn publish_to_redis(&self, data: &MarketData) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let channel = match data {
            MarketData::Trade(_) => "market_data:trades",
            MarketData::Quote(_) => "market_data:quotes", 
            MarketData::Aggregate(_) => "market_data:aggregates",
        };
        
        let payload = serde_json::to_string(data)?;
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(payload)
            .query_async(&mut conn)
            .await?;
            
        Ok(())
    }

    /// Subscribe to market data
    pub fn subscribe(&self) -> broadcast::Receiver<MarketData> {
        self.market_data_tx.subscribe()
    }
}