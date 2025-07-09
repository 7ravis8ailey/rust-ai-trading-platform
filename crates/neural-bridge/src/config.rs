//! Configuration for neural bridge

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Neural bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralBridgeConfig {
    /// Python environment path
    pub python_path: Option<String>,
    
    /// NeuralForecast configuration
    pub neuralforecast: NeuralForecastConfig,
    
    /// Model cache settings
    pub cache_size: usize,
    
    /// Prediction cache TTL in seconds
    pub cache_ttl_seconds: u64,
    
    /// Maximum cache entries
    pub max_cache_entries: usize,
    
    /// Models to preload on startup
    pub preload_models: Vec<String>,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// NeuralForecast specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralForecastConfig {
    /// Model repository path
    pub model_repo_path: String,
    
    /// Available models configuration
    pub models: HashMap<String, ModelConfig>,
    
    /// Default prediction horizon
    pub default_horizon: usize,
    
    /// Maximum batch size
    pub max_batch_size: usize,
}

/// Individual model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type (TFT, NBEATS, LSTM, etc.)
    pub model_type: String,
    
    /// Model file path
    pub model_path: String,
    
    /// Expected accuracy
    pub accuracy: f64,
    
    /// Optimal prediction horizons
    pub optimal_horizons: Vec<usize>,
    
    /// Required features
    pub required_features: Vec<String>,
    
    /// Model-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target inference time in milliseconds
    pub target_inference_ms: u64,
    
    /// Enable GPU acceleration
    pub enable_gpu: bool,
    
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Enable model compilation
    pub enable_compilation: bool,
}

impl Default for NeuralBridgeConfig {
    fn default() -> Self {
        let mut models = HashMap::new();
        
        // TFT configuration
        models.insert("TFT".to_string(), ModelConfig {
            model_type: "TemporalFusionTransformer".to_string(),
            model_path: "models/tft_model.pkl".to_string(),
            accuracy: 0.80, // 78-82% accuracy
            optimal_horizons: vec![5, 10, 15, 20],
            required_features: vec!["price".to_string(), "volume".to_string()],
            parameters: [(
                "input_size".to_string(),
                serde_json::Value::Number(serde_json::Number::from(168))
            )].iter().cloned().collect(),
        });
        
        // N-BEATS configuration
        models.insert("NBEATS".to_string(), ModelConfig {
            model_type: "NBEATS".to_string(),
            model_path: "models/nbeats_model.pkl".to_string(),
            accuracy: 0.735, // 72-75% accuracy
            optimal_horizons: vec![1, 2, 3, 4, 5],
            required_features: vec!["price".to_string()],
            parameters: [(
                "stack_types".to_string(),
                serde_json::Value::Array(vec![
                    serde_json::Value::String("trend".to_string()),
                    serde_json::Value::String("seasonality".to_string())
                ])
            )].iter().cloned().collect(),
        });
        
        // LSTM configuration
        models.insert("LSTM".to_string(), ModelConfig {
            model_type: "LSTM".to_string(),
            model_path: "models/lstm_model.pkl".to_string(),
            accuracy: 0.70,
            optimal_horizons: vec![10, 20, 30, 50],
            required_features: vec!["price".to_string(), "volume".to_string()],
            parameters: [(
                "hidden_size".to_string(),
                serde_json::Value::Number(serde_json::Number::from(128))
            )].iter().cloned().collect(),
        });
        
        Self {
            python_path: None,
            neuralforecast: NeuralForecastConfig {
                model_repo_path: "./models".to_string(),
                models,
                default_horizon: 10,
                max_batch_size: 32,
            },
            cache_size: 1000,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_entries: 10000,
            preload_models: vec![
                "TFT".to_string(),
                "NBEATS".to_string(),
                "LSTM".to_string(),
            ],
            performance: PerformanceConfig {
                target_inference_ms: 10,
                enable_gpu: true,
                worker_threads: 4,
                enable_compilation: true,
            },
        }
    }
}