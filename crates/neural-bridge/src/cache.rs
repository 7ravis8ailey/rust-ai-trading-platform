//! Model caching system

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Cached model wrapper
#[derive(Debug, Clone)]
pub struct CachedModel {
    pub name: String,
    pub model_data: Arc<Vec<u8>>, // Serialized model data
    pub metadata: crate::models::ModelMetadata,
    pub last_accessed: std::time::Instant,
    pub access_count: u64,
}

/// Model cache implementation
pub struct ModelCache {
    cache: RwLock<HashMap<String, CachedModel>>,
    max_size: usize,
}

impl ModelCache {
    /// Create new model cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
        }
    }

    /// Insert model into cache
    pub fn insert(&self, name: String, model: CachedModel) {
        let mut cache = self.cache.write();
        
        // If cache is full, remove least recently used item
        if cache.len() >= self.max_size {
            if let Some((lru_key, _)) = cache
                .iter()
                .min_by_key(|(_, model)| model.last_accessed)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                cache.remove(&lru_key);
            }
        }
        
        cache.insert(name, model);
    }

    /// Get model from cache
    pub fn get(&self, name: &str) -> Option<CachedModel> {
        let mut cache = self.cache.write();
        
        if let Some(model) = cache.get_mut(name) {
            model.last_accessed = std::time::Instant::now();
            model.access_count += 1;
            Some(model.clone())
        } else {
            None
        }
    }

    /// Check if model exists in cache
    pub fn contains(&self, name: &str) -> bool {
        let cache = self.cache.read();
        cache.contains_key(name)
    }

    /// Remove model from cache
    pub fn remove(&self, name: &str) -> Option<CachedModel> {
        let mut cache = self.cache.write();
        cache.remove(name)
    }

    /// Clear all cached models
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        
        let total_models = cache.len();
        let total_memory = cache
            .values()
            .map(|model| model.model_data.len())
            .sum::<usize>();
        
        let total_accesses = cache
            .values()
            .map(|model| model.access_count)
            .sum::<u64>();
        
        CacheStats {
            total_models,
            total_memory_bytes: total_memory,
            total_accesses,
            hit_ratio: 0.0, // Would need to track misses separately
        }
    }

    /// List all cached model names
    pub fn list_models(&self) -> Vec<String> {
        let cache = self.cache.read();
        cache.keys().cloned().collect()
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        let cache = self.cache.read();
        cache.len()
    }

    /// Get maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_models: usize,
    pub total_memory_bytes: usize,
    pub total_accesses: u64,
    pub hit_ratio: f64,
}

impl CacheStats {
    /// Get memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.total_memory_bytes as f64 / (1024.0 * 1024.0)
    }
}