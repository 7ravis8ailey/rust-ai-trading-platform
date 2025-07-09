//! Market data validation

use crate::MarketData;
use anyhow::{anyhow, Result};
use tracing::warn;

/// Validate market data
pub fn validate_market_data(data: &MarketData) -> Result<()> {
    match data {
        MarketData::Trade(trade) => validate_trade_data(trade),
        MarketData::Quote(quote) => validate_quote_data(quote),
        MarketData::Aggregate(agg) => validate_aggregate_data(agg),
    }
}

/// Validate trade data
fn validate_trade_data(trade: &crate::TradeData) -> Result<()> {
    // Validate symbol
    if trade.symbol.is_empty() {
        return Err(anyhow!("Empty symbol"));
    }
    
    // Validate price
    if trade.price <= 0.0 || trade.price.is_nan() || trade.price.is_infinite() {
        return Err(anyhow!("Invalid price: {}", trade.price));
    }
    
    // Validate size
    if trade.size == 0 {
        return Err(anyhow!("Zero trade size"));
    }
    
    // Validate timestamp (not too old)
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(trade.timestamp);
    if age.num_seconds() > 60 {
        warn!("Old trade data: {} seconds old", age.num_seconds());
    }
    
    Ok(())
}

/// Validate quote data
fn validate_quote_data(quote: &crate::QuoteData) -> Result<()> {
    // Validate symbol
    if quote.symbol.is_empty() {
        return Err(anyhow!("Empty symbol"));
    }
    
    // Validate prices
    if quote.bid_price <= 0.0 || quote.ask_price <= 0.0 {
        return Err(anyhow!("Invalid bid/ask prices"));
    }
    
    // Validate spread
    if quote.ask_price <= quote.bid_price {
        return Err(anyhow!("Invalid spread: ask <= bid"));
    }
    
    // Validate sizes
    if quote.bid_size == 0 || quote.ask_size == 0 {
        warn!("Zero bid/ask size for {}", quote.symbol);
    }
    
    Ok(())
}

/// Validate aggregate data
fn validate_aggregate_data(agg: &crate::AggregateData) -> Result<()> {
    // Validate symbol
    if agg.symbol.is_empty() {
        return Err(anyhow!("Empty symbol"));
    }
    
    // Validate OHLC
    if agg.open <= 0.0 || agg.high <= 0.0 || agg.low <= 0.0 || agg.close <= 0.0 {
        return Err(anyhow!("Invalid OHLC values"));
    }
    
    // Validate OHLC relationships
    if agg.high < agg.low {
        return Err(anyhow!("High < Low"));
    }
    
    if agg.high < agg.open || agg.high < agg.close {
        return Err(anyhow!("High is not the highest"));
    }
    
    if agg.low > agg.open || agg.low > agg.close {
        return Err(anyhow!("Low is not the lowest"));
    }
    
    // Validate volume
    if agg.volume == 0 {
        warn!("Zero volume for {}", agg.symbol);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_valid_trade_data() {
        let trade = crate::TradeData {
            symbol: "AAPL".to_string(),
            price: 150.0,
            size: 100,
            timestamp: Utc::now(),
            exchange: "NASDAQ".to_string(),
            conditions: vec![],
        };
        
        assert!(validate_trade_data(&trade).is_ok());
    }

    #[test]
    fn test_invalid_trade_price() {
        let trade = crate::TradeData {
            symbol: "AAPL".to_string(),
            price: -150.0,
            size: 100,
            timestamp: Utc::now(),
            exchange: "NASDAQ".to_string(),
            conditions: vec![],
        };
        
        assert!(validate_trade_data(&trade).is_err());
    }
}