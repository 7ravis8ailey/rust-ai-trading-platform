//! WebSocket client for real-time market data

use crate::{config::DataIngestionConfig, MarketData};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};

/// WebSocket manager for market data streams
pub struct WebSocketManager {
    config: DataIngestionConfig,
    data_tx: broadcast::Sender<MarketData>,
}

impl WebSocketManager {
    /// Create new WebSocket manager
    pub async fn new(config: &DataIngestionConfig) -> Result<Self> {
        let (data_tx, _) = broadcast::channel(config.websocket.buffer_size);
        
        Ok(Self {
            config: config.clone(),
            data_tx,
        })
    }

    /// Start WebSocket connections
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting WebSocket connections");
        
        // Connect to Polygon.io WebSocket
        self.connect_polygon().await?;
        
        Ok(())
    }

    /// Connect to Polygon.io WebSocket
    async fn connect_polygon(&self) -> Result<()> {
        let url = format!(
            "wss://socket.polygon.io/stocks?apikey={}",
            self.config.polygon_api_key
        );
        
        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        // Subscribe to symbols
        let subscribe_msg = serde_json::json!({
            "action": "subscribe",
            "params": format!("T.{}", self.config.symbols.join(",T."))
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        
        let data_tx = self.data_tx.clone();
        
        // Handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(market_data) = Self::parse_polygon_message(&text) {
                            if let Err(_) = data_tx.send(market_data) {
                                warn!("No subscribers for market data");
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("WebSocket connection closed");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {:?}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }

    /// Parse Polygon.io message
    fn parse_polygon_message(text: &str) -> Result<MarketData> {
        // Simplified parser - implement full Polygon.io protocol
        let value: serde_json::Value = serde_json::from_str(text)?;
        
        // This is a simplified implementation
        // In production, implement full Polygon.io message parsing
        if let Some(trades) = value.as_array() {
            for trade in trades {
                if let Some(event_type) = trade.get("ev").and_then(|v| v.as_str()) {
                    match event_type {
                        "T" => {
                            // Trade message
                            let trade_data = crate::TradeData {
                                symbol: trade.get("sym").unwrap().as_str().unwrap().to_string(),
                                price: trade.get("p").unwrap().as_f64().unwrap(),
                                size: trade.get("s").unwrap().as_u64().unwrap(),
                                timestamp: chrono::Utc::now(),
                                exchange: trade.get("x").unwrap_or(&serde_json::Value::String("UNKNOWN".to_string())).as_str().unwrap().to_string(),
                                conditions: vec![],
                            };
                            return Ok(MarketData::Trade(trade_data));
                        }
                        _ => continue,
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Unable to parse message"))
    }

    /// Subscribe to market data stream
    pub fn subscribe(&self) -> broadcast::Receiver<MarketData> {
        self.data_tx.subscribe()
    }
}