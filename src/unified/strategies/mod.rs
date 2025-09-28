// Strategy Implementations for Unified Memory Tracking
// Provides concrete implementations for different runtime environments
// Maintains zero-lock architecture and high performance standards

//! # Memory Tracking Strategies
//! 
//! This module provides concrete implementations of memory tracking strategies
//! optimized for different runtime environments.
//! 
//! ## Available Strategies
//! 
//! - [`SingleThreadStrategy`] - Optimized for single-threaded applications
//! - [`ThreadLocalStrategy`] - Thread-local storage for multi-threaded apps  
//! - [`AsyncStrategy`] - Task-local tracking for async applications
//! - [`HybridStrategy`] - Combined approach for complex environments
//! 
//! ## Design Principles
//! 
//! All strategies implement the unified `MemoryTracker` trait and follow:
//! - **Zero-lock architecture**: No mutex or rwlock usage
//! - **Minimal overhead**: <3% performance impact target
//! - **Data compatibility**: Consistent export formats
//! - **Graceful fallback**: Error recovery without data loss

pub mod single_thread;
pub mod thread_local;
pub mod async_strategy;
pub mod hybrid_strategy;

// Re-export strategy implementations
pub use single_thread::SingleThreadStrategy;
pub use thread_local::ThreadLocalStrategy; 
pub use async_strategy::AsyncStrategy;
pub use hybrid_strategy::HybridStrategy;

use crate::unified::tracking_dispatcher::{MemoryTracker, TrackerConfig, TrackerError};
use std::collections::HashMap;
use tracing::{debug, info};

/// Strategy factory for creating appropriate tracker implementations
/// Provides centralized strategy creation with proper configuration
pub struct StrategyFactory {
    /// Configuration templates for each strategy type
    strategy_configs: HashMap<String, TrackerConfig>,
    /// Performance monitoring for strategy selection
    performance_history: PerformanceHistory,
}

/// Performance history tracking for strategy optimization
/// Maintains historical performance data to guide strategy selection
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    /// Strategy performance records
    pub strategy_performance: HashMap<String, StrategyPerformance>,
    /// Total tracking sessions recorded
    pub total_sessions: u64,
    /// Average overhead across all strategies
    pub average_overhead_percent: f64,
}

/// Performance metrics for individual strategy
/// Tracks key performance indicators for strategy evaluation
#[derive(Debug, Clone)]
pub struct StrategyPerformance {
    /// Total sessions using this strategy
    pub session_count: u64,
    /// Average memory overhead percentage
    pub avg_overhead_percent: f64,
    /// Average initialization time (microseconds)
    pub avg_init_time_us: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// User satisfaction score (0.0 to 1.0)
    pub satisfaction_score: f64,
}

impl Default for PerformanceHistory {
    /// Initialize performance history with empty records
    fn default() -> Self {
        Self {
            strategy_performance: HashMap::new(),
            total_sessions: 0,
            average_overhead_percent: 0.0,
        }
    }
}

impl Default for StrategyPerformance {
    /// Initialize strategy performance with neutral metrics
    fn default() -> Self {
        Self {
            session_count: 0,
            avg_overhead_percent: 2.0, // Conservative estimate
            avg_init_time_us: 100.0,   // 100 microseconds baseline
            success_rate: 1.0,         // Optimistic default
            satisfaction_score: 0.8,   // Good default score
        }
    }
}

impl StrategyFactory {
    /// Create new strategy factory with default configurations
    /// Initializes templates optimized for different use cases
    pub fn new() -> Self {
        let mut strategy_configs = HashMap::new();
        
        // Single-thread optimized configuration
        strategy_configs.insert("single_thread".to_string(), TrackerConfig {
            sample_rate: 1.0,           // Full sampling for single-thread
            max_overhead_mb: 32,        // Conservative memory limit
            thread_affinity: None,      // No affinity needed
            custom_params: HashMap::new(),
        });
        
        // Multi-thread optimized configuration
        strategy_configs.insert("thread_local".to_string(), TrackerConfig {
            sample_rate: 0.8,           // Reduced sampling for performance
            max_overhead_mb: 64,        // Higher limit for multi-thread
            thread_affinity: None,      // Dynamic affinity
            custom_params: HashMap::new(),
        });
        
        // Async optimized configuration
        strategy_configs.insert("async".to_string(), TrackerConfig {
            sample_rate: 0.9,           // High sampling for async tracking
            max_overhead_mb: 48,        // Moderate memory limit
            thread_affinity: None,      // No affinity for async
            custom_params: HashMap::new(),
        });
        
        // Hybrid configuration balancing all needs
        strategy_configs.insert("hybrid".to_string(), TrackerConfig {
            sample_rate: 0.7,           // Balanced sampling
            max_overhead_mb: 96,        // Higher limit for complexity
            thread_affinity: None,      // Complex affinity patterns
            custom_params: HashMap::new(),
        });

        Self {
            strategy_configs,
            performance_history: PerformanceHistory::default(),
        }
    }

    /// Create single-thread strategy instance with optimized configuration
    /// Best for simple, sequential applications
    pub fn create_single_thread_strategy(&self) -> Result<Box<dyn MemoryTracker>, TrackerError> {
        debug!("Creating single-thread strategy");
        
        let config = self.strategy_configs.get("single_thread")
            .cloned()
            .unwrap_or_default();
            
        let mut strategy = SingleThreadStrategy::new();
        strategy.initialize(config)?;
        
        info!("Single-thread strategy created successfully");
        Ok(Box::new(strategy))
    }

    /// Create thread-local strategy instance for multi-threaded applications
    /// Optimized for applications with multiple worker threads
    pub fn create_thread_local_strategy(&self) -> Result<Box<dyn MemoryTracker>, TrackerError> {
        debug!("Creating thread-local strategy");
        
        let config = self.strategy_configs.get("thread_local")
            .cloned()
            .unwrap_or_default();
            
        let mut strategy = ThreadLocalStrategy::new();
        strategy.initialize(config)?;
        
        info!("Thread-local strategy created successfully");
        Ok(Box::new(strategy))
    }

    /// Create async strategy instance for async/await applications
    /// Specialized for futures and task-based concurrency
    pub fn create_async_strategy(&self) -> Result<Box<dyn MemoryTracker>, TrackerError> {
        debug!("Creating async strategy");
        
        let config = self.strategy_configs.get("async")
            .cloned()
            .unwrap_or_default();
            
        let mut strategy = AsyncStrategy::new();
        strategy.initialize(config)?;
        
        info!("Async strategy created successfully");
        Ok(Box::new(strategy))
    }

    /// Create hybrid strategy instance for complex applications
    /// Handles mixed thread and async environments
    pub fn create_hybrid_strategy(&self) -> Result<Box<dyn MemoryTracker>, TrackerError> {
        debug!("Creating hybrid strategy");
        
        let config = self.strategy_configs.get("hybrid")
            .cloned()
            .unwrap_or_default();
            
        let mut strategy = HybridStrategy::new();
        strategy.initialize(config)?;
        
        info!("Hybrid strategy created successfully");
        Ok(Box::new(strategy))
    }

    /// Record strategy performance for future optimization
    /// Updates performance history with session metrics
    pub fn record_performance(&mut self, strategy_name: &str, performance: StrategyPerformance) {
        debug!("Recording performance for strategy: {}", strategy_name);
        
        let (avg_overhead_percent, success_rate) = {
            let entry = self.performance_history.strategy_performance
                .entry(strategy_name.to_string())
                .or_insert_with(StrategyPerformance::default);
            
            // Update with exponential moving average
            let weight = 0.1; // 10% weight for new data
            entry.avg_overhead_percent = (1.0 - weight) * entry.avg_overhead_percent + 
                                       weight * performance.avg_overhead_percent;
            entry.avg_init_time_us = (1.0 - weight) * entry.avg_init_time_us + 
                                    weight * performance.avg_init_time_us;
            entry.success_rate = (1.0 - weight) * entry.success_rate + 
                               weight * performance.success_rate;
            entry.satisfaction_score = (1.0 - weight) * entry.satisfaction_score + 
                                      weight * performance.satisfaction_score;
            
            entry.session_count += 1;
            
            (entry.avg_overhead_percent, entry.success_rate)
        };
        
        self.performance_history.total_sessions += 1;
        
        // Update average overhead across all strategies
        self.update_average_overhead();
        
        info!("Performance recorded for {}: overhead={:.2}%, success={:.2}%", 
              strategy_name, avg_overhead_percent, success_rate * 100.0);
    }

    /// Update average overhead calculation across all strategies
    fn update_average_overhead(&mut self) {
        if self.performance_history.strategy_performance.is_empty() {
            return;
        }
        
        let total_overhead: f64 = self.performance_history.strategy_performance
            .values()
            .map(|perf| perf.avg_overhead_percent)
            .sum();
        
        self.performance_history.average_overhead_percent = 
            total_overhead / self.performance_history.strategy_performance.len() as f64;
    }

    /// Get performance history for analysis
    /// Provides read-only access to historical performance data
    pub fn get_performance_history(&self) -> &PerformanceHistory {
        &self.performance_history
    }

    /// Recommend optimal strategy based on performance history
    /// Uses historical data to suggest best strategy for given requirements
    pub fn recommend_strategy(&self, requirements: &StrategyRequirements) -> String {
        debug!("Recommending strategy for requirements: {:?}", requirements);
        
        let mut best_strategy = "single_thread".to_string();
        let mut best_score = 0.0;
        
        for (strategy_name, performance) in &self.performance_history.strategy_performance {
            let score = self.calculate_strategy_score(strategy_name, performance, requirements);
            
            if score > best_score {
                best_score = score;
                best_strategy = strategy_name.clone();
            }
        }
        
        info!("Recommended strategy: {} (score: {:.2})", best_strategy, best_score);
        best_strategy
    }

    /// Calculate strategy fitness score for given requirements
    fn calculate_strategy_score(
        &self, 
        _strategy_name: &str, 
        performance: &StrategyPerformance, 
        requirements: &StrategyRequirements
    ) -> f64 {
        let mut score = 0.0;
        
        // Weight factors for different requirements
        let overhead_weight = 0.3;
        let speed_weight = 0.3;
        let reliability_weight = 0.4;
        
        // Overhead score (lower is better)
        let overhead_score = if requirements.max_overhead_percent > 0.0 {
            (requirements.max_overhead_percent - performance.avg_overhead_percent)
                .max(0.0) / requirements.max_overhead_percent
        } else {
            1.0 - (performance.avg_overhead_percent / 10.0) // Assume 10% is maximum acceptable
        };
        
        // Speed score (lower init time is better)
        let speed_score = 1.0 - (performance.avg_init_time_us / 10000.0).min(1.0); // 10ms max
        
        // Reliability score
        let reliability_score = performance.success_rate * performance.satisfaction_score;
        
        score = overhead_weight * overhead_score + 
                speed_weight * speed_score + 
                reliability_weight * reliability_score;
        
        score.max(0.0).min(1.0) // Clamp to [0, 1]
    }
}

/// Strategy selection requirements
/// Defines criteria for optimal strategy selection
#[derive(Debug, Clone)]
pub struct StrategyRequirements {
    /// Maximum acceptable overhead percentage
    pub max_overhead_percent: f64,
    /// Preferred initialization time (microseconds)
    pub preferred_init_time_us: f64,
    /// Minimum required success rate
    pub min_success_rate: f64,
    /// Environment constraints
    pub environment_constraints: Vec<String>,
}

impl Default for StrategyRequirements {
    /// Default requirements for typical use cases
    fn default() -> Self {
        Self {
            max_overhead_percent: 5.0,
            preferred_init_time_us: 1000.0, // 1ms
            min_success_rate: 0.95,
            environment_constraints: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_factory_creation() {
        let factory = StrategyFactory::new();
        assert_eq!(factory.strategy_configs.len(), 4);
        assert!(factory.strategy_configs.contains_key("single_thread"));
        assert!(factory.strategy_configs.contains_key("thread_local"));
        assert!(factory.strategy_configs.contains_key("async"));
        assert!(factory.strategy_configs.contains_key("hybrid"));
    }

    #[test]
    fn test_single_thread_strategy_creation() {
        let factory = StrategyFactory::new();
        let result = factory.create_single_thread_strategy();
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_strategy_creation() {
        let factory = StrategyFactory::new();
        
        // Test all strategy types
        assert!(factory.create_single_thread_strategy().is_ok());
        assert!(factory.create_thread_local_strategy().is_ok());
        assert!(factory.create_async_strategy().is_ok());
        assert!(factory.create_hybrid_strategy().is_ok());
    }

    #[test]
    fn test_performance_recording() {
        let mut factory = StrategyFactory::new();
        let performance = StrategyPerformance {
            session_count: 1,
            avg_overhead_percent: 2.5,
            avg_init_time_us: 150.0,
            success_rate: 0.98,
            satisfaction_score: 0.9,
        };
        
        factory.record_performance("single_thread", performance);
        
        let history = factory.get_performance_history();
        assert_eq!(history.total_sessions, 1);
        assert!(history.strategy_performance.contains_key("single_thread"));
    }

    #[test]
    fn test_strategy_recommendation() {
        let mut factory = StrategyFactory::new();
        
        // Record some performance data
        factory.record_performance("single_thread", StrategyPerformance {
            avg_overhead_percent: 1.0,
            success_rate: 0.99,
            satisfaction_score: 0.95,
            ..Default::default()
        });
        
        let requirements = StrategyRequirements::default();
        let recommendation = factory.recommend_strategy(&requirements);
        
        assert_eq!(recommendation, "single_thread");
    }

    #[test]
    fn test_performance_history_update() {
        let mut factory = StrategyFactory::new();
        
        // Record multiple performance entries
        for i in 0..5 {
            factory.record_performance("test_strategy", StrategyPerformance {
                avg_overhead_percent: 2.0 + i as f64 * 0.1,
                ..Default::default()
            });
        }
        
        let history = factory.get_performance_history();
        assert_eq!(history.total_sessions, 5);
        assert!(history.average_overhead_percent > 0.0);
    }
}