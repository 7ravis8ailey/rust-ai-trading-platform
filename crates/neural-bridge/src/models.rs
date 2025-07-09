//! Model management and statistics

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub model_name: String,
    pub accuracy: f64,
    pub average_inference_time_ms: f64,
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub failed_predictions: u64,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub memory_usage_mb: f64,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub model_type: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub trained_on: String,
    pub features: Vec<String>,
    pub hyperparameters: HashMap<String, serde_json::Value>,
}

/// Model performance tracker
pub struct ModelPerformanceTracker {
    stats: HashMap<String, ModelStats>,
}

impl ModelPerformanceTracker {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Record a prediction
    pub fn record_prediction(
        &mut self,
        model_name: &str,
        inference_time_ms: f64,
        success: bool,
    ) {
        let stats = self.stats.entry(model_name.to_string()).or_insert_with(|| ModelStats {
            model_name: model_name.to_string(),
            accuracy: 0.0,
            average_inference_time_ms: 0.0,
            total_predictions: 0,
            successful_predictions: 0,
            failed_predictions: 0,
            last_used: chrono::Utc::now(),
            memory_usage_mb: 0.0,
        });

        stats.total_predictions += 1;
        if success {
            stats.successful_predictions += 1;
        } else {
            stats.failed_predictions += 1;
        }

        // Update average inference time
        stats.average_inference_time_ms = (
            stats.average_inference_time_ms * (stats.total_predictions - 1) as f64
            + inference_time_ms
        ) / stats.total_predictions as f64;

        stats.last_used = chrono::Utc::now();
    }

    /// Get stats for a model
    pub fn get_stats(&self, model_name: &str) -> Option<&ModelStats> {
        self.stats.get(model_name)
    }

    /// Get all model stats
    pub fn get_all_stats(&self) -> &HashMap<String, ModelStats> {
        &self.stats
    }

    /// Update model accuracy
    pub fn update_accuracy(&mut self, model_name: &str, accuracy: f64) {
        if let Some(stats) = self.stats.get_mut(model_name) {
            stats.accuracy = accuracy;
        }
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, model_name: &str, memory_mb: f64) {
        if let Some(stats) = self.stats.get_mut(model_name) {
            stats.memory_usage_mb = memory_mb;
        }
    }
}

/// Model selector based on performance and context
pub struct ModelSelector {
    performance_tracker: ModelPerformanceTracker,
}

impl ModelSelector {
    pub fn new() -> Self {
        Self {
            performance_tracker: ModelPerformanceTracker::new(),
        }
    }

    /// Select best model for given criteria
    pub fn select_model(
        &self,
        horizon: usize,
        symbol_type: &str,
        features: &[String],
    ) -> Result<String> {
        // Model selection logic based on:
        // 1. Prediction horizon
        // 2. Symbol characteristics
        // 3. Available features
        // 4. Historical performance

        match horizon {
            1..=5 => {
                // Short-term: prefer N-BEATS
                if self.is_model_performing_well("NBEATS") {
                    Ok("NBEATS".to_string())
                } else {
                    Ok("TFT".to_string())
                }
            }
            6..=20 => {
                // Medium-term: prefer TFT
                if self.is_model_performing_well("TFT") {
                    Ok("TFT".to_string())
                } else {
                    Ok("LSTM".to_string())
                }
            }
            21.. => {
                // Long-term: prefer LSTM
                if self.is_model_performing_well("LSTM") {
                    Ok("LSTM".to_string())
                } else {
                    Ok("TFT".to_string())
                }
            }
            _ => Err(anyhow::anyhow!("Invalid horizon: {}", horizon)),
        }
    }

    /// Check if model is performing well
    fn is_model_performing_well(&self, model_name: &str) -> bool {
        if let Some(stats) = self.performance_tracker.get_stats(model_name) {
            // Consider model performing well if:
            // 1. Success rate > 95%
            // 2. Average inference time < 10ms
            // 3. Has been used recently
            let success_rate = if stats.total_predictions > 0 {
                stats.successful_predictions as f64 / stats.total_predictions as f64
            } else {
                0.0
            };

            let recent = chrono::Utc::now()
                .signed_duration_since(stats.last_used)
                .num_hours() < 24;

            success_rate > 0.95 && stats.average_inference_time_ms < 10.0 && recent
        } else {
            false
        }
    }

    /// Get performance tracker
    pub fn get_performance_tracker(&self) -> &ModelPerformanceTracker {
        &self.performance_tracker
    }

    /// Get mutable performance tracker
    pub fn get_performance_tracker_mut(&mut self) -> &mut ModelPerformanceTracker {
        &mut self.performance_tracker
    }
}