//! Polygon.io specific implementations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Polygon.io WebSocket message types
#[derive(Debug, Deserialize)]
#[serde(tag = "ev")]
pub enum PolygonMessage {
    #[serde(rename = "T")]
    Trade(PolygonTrade),
    #[serde(rename = "Q")]
    Quote(PolygonQuote),
    #[serde(rename = "A")]
    Aggregate(PolygonAggregate),
    #[serde(rename = "status")]
    Status(PolygonStatus),
}

/// Polygon.io trade message
#[derive(Debug, Deserialize)]
pub struct PolygonTrade {
    pub sym: String,     // Symbol
    pub x: i32,          // Exchange ID
    pub p: f64,          // Price
    pub s: u64,          // Size
    pub c: Vec<i32>,     // Conditions
    pub t: u64,          // Timestamp (nanoseconds)
}

/// Polygon.io quote message
#[derive(Debug, Deserialize)]
pub struct PolygonQuote {
    pub sym: String,     // Symbol
    pub bx: i32,         // Bid exchange ID
    pub ax: i32,         // Ask exchange ID
    pub bp: f64,         // Bid price
    pub ap: f64,         // Ask price
    pub bs: u64,         // Bid size
    pub as_: u64,        // Ask size
    pub t: u64,          // Timestamp (nanoseconds)
}

/// Polygon.io aggregate message
#[derive(Debug, Deserialize)]
pub struct PolygonAggregate {
    pub sym: String,     // Symbol
    pub o: f64,          // Open
    pub h: f64,          // High
    pub l: f64,          // Low
    pub c: f64,          // Close
    pub v: u64,          // Volume
    pub s: u64,          // Start timestamp
    pub e: u64,          // End timestamp
}

/// Polygon.io status message
#[derive(Debug, Deserialize)]
pub struct PolygonStatus {
    pub status: String,
    pub message: String,
}

/// Exchange ID to name mapping
pub fn exchange_id_to_name(id: i32) -> &'static str {
    match id {
        1 => "NYSE",
        2 => "NASDAQ",
        3 => "NYSE_ARCA",
        4 => "NASDAQ_OMX_BX",
        5 => "NYSE_NATIONAL",
        6 => "CBOE_EDGX",
        7 => "CBOE_EDGA",
        8 => "CBOE_BZX",
        9 => "CBOE_BYX",
        10 => "IEX",
        11 => "NYSE_CHICAGO",
        12 => "NASDAQ_PSX",
        _ => "UNKNOWN",
    }
}

/// Convert Polygon timestamp to chrono DateTime
pub fn polygon_timestamp_to_datetime(timestamp_ns: u64) -> chrono::DateTime<chrono::Utc> {
    let timestamp_ms = timestamp_ns / 1_000_000;
    chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(chrono::Utc::now)
}

/// Convert Polygon trade to our TradeData
impl From<PolygonTrade> for crate::TradeData {
    fn from(trade: PolygonTrade) -> Self {
        Self {
            symbol: trade.sym,
            price: trade.p,
            size: trade.s,
            timestamp: polygon_timestamp_to_datetime(trade.t),
            exchange: exchange_id_to_name(trade.x).to_string(),
            conditions: trade.c.into_iter().map(|c| c.to_string()).collect(),
        }
    }
}

/// Convert Polygon quote to our QuoteData
impl From<PolygonQuote> for crate::QuoteData {
    fn from(quote: PolygonQuote) -> Self {
        Self {
            symbol: quote.sym,
            bid_price: quote.bp,
            ask_price: quote.ap,
            bid_size: quote.bs,
            ask_size: quote.as_,
            timestamp: polygon_timestamp_to_datetime(quote.t),
            exchange: format!(
                "{}|{}",
                exchange_id_to_name(quote.bx),
                exchange_id_to_name(quote.ax)
            ),
        }
    }
}

/// Convert Polygon aggregate to our AggregateData
impl From<PolygonAggregate> for crate::AggregateData {
    fn from(agg: PolygonAggregate) -> Self {
        Self {
            symbol: agg.sym,
            open: agg.o,
            high: agg.h,
            low: agg.l,
            close: agg.c,
            volume: agg.v,
            timestamp: polygon_timestamp_to_datetime(agg.s),
            timespan: "1m".to_string(), // Default to 1 minute
        }
    }
}