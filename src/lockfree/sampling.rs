//! Intelligent sampling configuration for lock-free tracking
//!
//! This module defines sampling strategies optimized for high-concurrency
//! scenarios where capturing every allocation would create performance bottlenecks.

/// Sampling configuration for intelligent allocation tracking
/// 
/// Uses dual-dimension sampling (size + frequency) to balance performance
/// with data completeness. Large allocations and high-frequency patterns
/// receive priority sampling.
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Sample rate for large allocations - usually 100% to catch memory leaks
    pub large_allocation_rate: f64,
    /// Sample rate for medium allocations - balanced approach
    pub medium_allocation_rate: f64,
    /// Sample rate for small allocations - low to reduce overhead
    pub small_allocation_rate: f64,
    /// Size threshold for large allocations (bytes)
    pub large_threshold: usize,
    /// Size threshold for medium allocations (bytes) 
    pub medium_threshold: usize,
    /// Frequency threshold for sampling boost
    pub frequency_threshold: u64,
}

impl Default for SamplingConfig {
    /// Default configuration optimized for typical applications
    /// 
    /// Captures all large allocations, moderate sampling of medium allocations,
    /// and light sampling of small allocations to maintain performance.
    fn default() -> Self {
        Self {
            large_allocation_rate: 1.0,   // 100% - catch all potential leaks
            medium_allocation_rate: 0.1,  // 10% - balanced coverage  
            small_allocation_rate: 0.01,  // 1% - minimal overhead
            large_threshold: 10 * 1024,   // 10KB threshold
            medium_threshold: 1 * 1024,   // 1KB threshold
            frequency_threshold: 10,      // Boost after 10 occurrences
        }
    }
}

impl SamplingConfig {
    /// Creates high-precision configuration for debugging scenarios
    /// 
    /// Higher sampling rates for more complete data capture at the cost
    /// of increased performance overhead.
    pub fn high_precision() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.5,   // 50% sampling
            small_allocation_rate: 0.1,    // 10% sampling
            large_threshold: 4 * 1024,     // 4KB threshold
            medium_threshold: 512,         // 512B threshold
            frequency_threshold: 5,        // Earlier boost
        }
    }

    /// Creates performance-optimized configuration for production
    /// 
    /// Minimal sampling to reduce overhead while still capturing
    /// the most critical allocation patterns.
    pub fn performance_optimized() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.05,  // 5% sampling
            small_allocation_rate: 0.001,  // 0.1% sampling
            large_threshold: 50 * 1024,    // 50KB threshold
            medium_threshold: 5 * 1024,    // 5KB threshold
            frequency_threshold: 50,       // Higher boost threshold
        }
    }

    /// Creates configuration for memory leak detection
    /// 
    /// Optimized to catch large allocations and allocation patterns
    /// that might indicate memory leaks.
    pub fn leak_detection() -> Self {
        Self {
            large_allocation_rate: 1.0,
            medium_allocation_rate: 0.8,   // High sampling for leaks
            small_allocation_rate: 0.01,
            large_threshold: 1 * 1024,     // 1KB threshold (lower)
            medium_threshold: 256,         // 256B threshold
            frequency_threshold: 3,        // Quick boost for patterns
        }
    }

    /// Validates configuration parameters
    /// 
    /// Ensures all rates are between 0.0 and 1.0 and thresholds are reasonable.
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.large_allocation_rate) {
            return Err("Large allocation rate must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.medium_allocation_rate) {
            return Err("Medium allocation rate must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.small_allocation_rate) {
            return Err("Small allocation rate must be between 0.0 and 1.0".to_string());
        }
        if self.large_threshold <= self.medium_threshold {
            return Err("Large threshold must be greater than medium threshold".to_string());
        }
        if self.medium_threshold == 0 {
            return Err("Medium threshold must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Calculates expected sampling rate for given allocation size
    /// 
    /// Returns the base sampling rate before frequency adjustments.
    pub fn base_sampling_rate(&self, size: usize) -> f64 {
        if size >= self.large_threshold {
            self.large_allocation_rate
        } else if size >= self.medium_threshold {
            self.medium_allocation_rate
        } else {
            self.small_allocation_rate
        }
    }

    /// Calculates frequency multiplier for sampling boost
    /// 
    /// High-frequency allocations get increased sampling rates to identify
    /// performance hotspots.
    pub fn frequency_multiplier(&self, frequency: u64) -> f64 {
        if frequency > self.frequency_threshold {
            // Logarithmic boost to prevent excessive sampling
            (frequency as f64 / self.frequency_threshold as f64).min(10.0)
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = SamplingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_configs_validation() {
        assert!(SamplingConfig::high_precision().validate().is_ok());
        assert!(SamplingConfig::performance_optimized().validate().is_ok());
        assert!(SamplingConfig::leak_detection().validate().is_ok());
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = SamplingConfig::default();
        
        // Test invalid rate
        config.large_allocation_rate = 1.5;
        assert!(config.validate().is_err());
        
        // Test invalid thresholds
        config.large_allocation_rate = 1.0;
        config.large_threshold = 500;
        config.medium_threshold = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sampling_rate_calculation() {
        let config = SamplingConfig::default();
        
        // Test large allocation
        assert_eq!(config.base_sampling_rate(20 * 1024), 1.0);
        
        // Test medium allocation
        assert_eq!(config.base_sampling_rate(5 * 1024), 0.1);
        
        // Test small allocation
        assert_eq!(config.base_sampling_rate(512), 0.01);
    }

    #[test]
    fn test_frequency_multiplier() {
        let config = SamplingConfig::default();
        
        // Test below threshold
        assert_eq!(config.frequency_multiplier(5), 1.0);
        
        // Test above threshold
        assert!(config.frequency_multiplier(20) > 1.0);
        
        // Test capping at 10x
        assert_eq!(config.frequency_multiplier(1000), 10.0);
    }
}