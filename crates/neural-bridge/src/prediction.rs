//! Prediction utilities and helpers

use crate::{PredictionInput, PredictionResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prediction request with additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub input: PredictionInput,
    pub model_preference: Option<String>,
    pub priority: PredictionPriority,
    pub callback_url: Option<String>,
    pub request_id: String,
}

/// Prediction priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Batch prediction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionRequest {
    pub requests: Vec<PredictionRequest>,
    pub batch_id: String,
    pub max_parallel: Option<usize>,
}

/// Prediction response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResponse {
    pub result: PredictionResult,
    pub request_id: String,
    pub processing_time_ms: u64,
    pub queue_time_ms: u64,
    pub status: PredictionStatus,
}

/// Prediction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionStatus {
    Success,
    Failed { error: String },
    Timeout,
    Queued,
    Processing,
}

/// Prediction quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionQuality {
    pub confidence_score: f64,
    pub uncertainty_bounds: (f64, f64),
    pub feature_importance: HashMap<String, f64>,
    pub model_agreement: f64, // If ensemble is used
}

/// Prediction validator
pub struct PredictionValidator;

impl PredictionValidator {
    /// Validate prediction input
    pub fn validate_input(input: &PredictionInput) -> Result<()> {
        // Check symbol
        if input.symbol.is_empty() {
            return Err(anyhow::anyhow!("Empty symbol"));
        }
        
        // Check historical data
        if input.historical_data.is_empty() {
            return Err(anyhow::anyhow!("Empty historical data"));
        }
        
        if input.historical_data.len() < 20 {
            return Err(anyhow::anyhow!("Insufficient historical data (minimum 20 points)"));
        }
        
        // Check for invalid values
        for &value in &input.historical_data {
            if value.is_nan() || value.is_infinite() || value < 0.0 {
                return Err(anyhow::anyhow!("Invalid data point: {}", value));
            }
        }
        
        // Check timestamps
        if input.timestamps.len() != input.historical_data.len() {
            return Err(anyhow::anyhow!("Timestamp and data length mismatch"));
        }
        
        // Check horizon
        if input.horizon == 0 || input.horizon > 100 {
            return Err(anyhow::anyhow!("Invalid prediction horizon: {}", input.horizon));
        }
        
        Ok(())
    }
    
    /// Validate prediction result
    pub fn validate_result(result: &PredictionResult) -> Result<()> {
        // Check prediction values
        if result.prediction.is_empty() {
            return Err(anyhow::anyhow!("Empty prediction"));
        }
        
        // Check for invalid predictions
        for &value in &result.prediction {
            if value.is_nan() || value.is_infinite() {
                return Err(anyhow::anyhow!("Invalid prediction value: {}", value));
            }
        }
        
        // Check confidence
        if result.confidence < 0.0 || result.confidence > 1.0 {
            return Err(anyhow::anyhow!("Invalid confidence: {}", result.confidence));
        }
        
        // Check horizon matches prediction length
        if result.prediction.len() != result.horizon {
            return Err(anyhow::anyhow!(
                "Prediction length ({}) doesn't match horizon ({})",
                result.prediction.len(),
                result.horizon
            ));
        }
        
        Ok(())
    }
}

/// Prediction formatter for different output formats
pub struct PredictionFormatter;

impl PredictionFormatter {
    /// Format prediction as JSON
    pub fn to_json(result: &PredictionResult) -> Result<String> {
        serde_json::to_string_pretty(result).map_err(Into::into)
    }
    
    /// Format prediction as CSV
    pub fn to_csv(result: &PredictionResult) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("timestamp,symbol,value,confidence\n");
        
        let base_time = result.timestamp;
        for (i, &value) in result.prediction.iter().enumerate() {
            let timestamp = base_time + chrono::Duration::minutes(i as i64);
            csv.push_str(&format!(
                "{},{},{:.6},{:.4}\n",
                timestamp.format("%Y-%m-%d %H:%M:%S"),
                result.symbol,
                value,
                result.confidence
            ));
        }
        
        Ok(csv)
    }
    
    /// Format prediction as summary
    pub fn to_summary(result: &PredictionResult) -> String {
        let avg_prediction = result.prediction.iter().sum::<f64>() / result.prediction.len() as f64;
        let min_prediction = result.prediction.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_prediction = result.prediction.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Prediction Summary for {}:\n\
            Model: {}\n\
            Horizon: {} periods\n\
            Confidence: {:.2}%\n\
            Average: {:.6}\n\
            Range: {:.6} - {:.6}\n\
            Generated: {}",
            result.symbol,
            result.model_name,
            result.horizon,
            result.confidence * 100.0,
            avg_prediction,
            min_prediction,
            max_prediction,
            result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_validate_valid_input() {
        let input = PredictionInput {
            symbol: "AAPL".to_string(),
            historical_data: (0..50).map(|i| 100.0 + i as f64).collect(),
            timestamps: (0..50).map(|i| Utc::now() - chrono::Duration::minutes(50 - i)).collect(),
            features: HashMap::new(),
            horizon: 10,
        };
        
        assert!(PredictionValidator::validate_input(&input).is_ok());
    }
    
    #[test]
    fn test_validate_invalid_input() {
        let input = PredictionInput {
            symbol: "".to_string(), // Empty symbol
            historical_data: vec![],
            timestamps: vec![],
            features: HashMap::new(),
            horizon: 10,
        };
        
        assert!(PredictionValidator::validate_input(&input).is_err());
    }
}