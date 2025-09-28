// Hybrid Memory Tracking Strategy
// Combines thread-local and async tracking for complex applications
// Handles mixed thread and async environments

use crate::unified::tracking_dispatcher::{MemoryTracker, TrackerConfig, TrackerStatistics, TrackerType, TrackerError};
use crate::unified::strategies::{ThreadLocalStrategy, AsyncStrategy};
use tracing::{debug, info};

/// Hybrid memory tracking strategy
/// Combines both thread-local and async tracking capabilities
/// Optimized for applications mixing traditional threads with async tasks
pub struct HybridStrategy {
    /// Thread-local tracking component
    thread_tracker: ThreadLocalStrategy,
    /// Async tracking component
    async_tracker: AsyncStrategy,
    /// Configuration for hybrid operation
    config: Option<TrackerConfig>,
    /// Strategy coordination state
    coordination_state: HybridCoordinationState,
}

/// Coordination state for hybrid tracking
/// Manages interaction between thread-local and async components
#[derive(Debug, Clone)]
struct HybridCoordinationState {
    /// Whether hybrid tracking is active
    is_active: bool,
    /// Primary tracking mode
    _primary_mode: TrackingMode,
    /// Fallback mode when primary fails
    _fallback_mode: TrackingMode,
}

/// Tracking mode for hybrid strategy
#[derive(Debug, Clone, PartialEq)]
enum TrackingMode {
    /// Both modes active simultaneously
    DualMode,
    /// Automatic mode selection based on context
    Adaptive,
}

impl Default for HybridCoordinationState {
    /// Initialize with adaptive mode
    fn default() -> Self {
        Self {
            is_active: false,
            _primary_mode: TrackingMode::Adaptive,
            _fallback_mode: TrackingMode::DualMode,
        }
    }
}

impl HybridStrategy {
    /// Create new hybrid strategy instance
    pub fn new() -> Self {
        debug!("Creating new hybrid strategy");
        
        Self {
            thread_tracker: ThreadLocalStrategy::new(),
            async_tracker: AsyncStrategy::new(),
            config: None,
            coordination_state: HybridCoordinationState::default(),
        }
    }
}

impl MemoryTracker for HybridStrategy {
    /// Initialize both tracking components
    fn initialize(&mut self, config: TrackerConfig) -> Result<(), TrackerError> {
        debug!("Initializing hybrid strategy with config: {:?}", config);
        
        // Initialize both components with same config
        self.thread_tracker.initialize(config.clone())?;
        self.async_tracker.initialize(config.clone())?;
        
        self.config = Some(config);
        
        info!("Hybrid strategy initialized successfully");
        Ok(())
    }

    /// Start both tracking components
    fn start_tracking(&mut self) -> Result<(), TrackerError> {
        debug!("Starting hybrid tracking");
        
        // Start both components
        self.thread_tracker.start_tracking()?;
        self.async_tracker.start_tracking()?;
        
        self.coordination_state.is_active = true;
        
        info!("Hybrid tracking started successfully");
        Ok(())
    }

    /// Stop both components and merge data
    fn stop_tracking(&mut self) -> Result<Vec<u8>, TrackerError> {
        debug!("Stopping hybrid tracking");
        
        // Stop both components
        let thread_data = self.thread_tracker.stop_tracking()?;
        let async_data = self.async_tracker.stop_tracking()?;
        
        // Merge data (simplified - would need sophisticated merging in real implementation)
        let thread_json: serde_json::Value = serde_json::from_slice(&thread_data)?;
        let async_json: serde_json::Value = serde_json::from_slice(&async_data)?;
        
        let mut merged = serde_json::Map::new();
        merged.insert("thread_data".to_string(), thread_json);
        merged.insert("async_data".to_string(), async_json);
        merged.insert("strategy_type".to_string(), serde_json::Value::String("hybrid".to_string()));
        
        let merged_json = serde_json::to_string_pretty(&merged)
            .map_err(|e| TrackerError::DataCollectionFailed {
                reason: format!("Failed to merge hybrid data: {e}"),
            })?;
        
        self.coordination_state.is_active = false;
        
        info!("Hybrid tracking stopped and data merged");
        Ok(merged_json.into_bytes())
    }

    /// Get combined statistics from both components
    fn get_statistics(&self) -> TrackerStatistics {
        let thread_stats = self.thread_tracker.get_statistics();
        let async_stats = self.async_tracker.get_statistics();
        
        TrackerStatistics {
            allocations_tracked: thread_stats.allocations_tracked + async_stats.allocations_tracked,
            memory_tracked_bytes: thread_stats.memory_tracked_bytes + async_stats.memory_tracked_bytes,
            overhead_bytes: thread_stats.overhead_bytes + async_stats.overhead_bytes,
            tracking_duration_ms: thread_stats.tracking_duration_ms.max(async_stats.tracking_duration_ms),
        }
    }

    /// Check if either component is active
    fn is_active(&self) -> bool {
        self.coordination_state.is_active && 
        (self.thread_tracker.is_active() || self.async_tracker.is_active())
    }

    /// Return hybrid tracker type
    fn tracker_type(&self) -> TrackerType {
        TrackerType::HybridTracker
    }
}

impl Default for HybridStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_strategy_creation() {
        let strategy = HybridStrategy::new();
        assert!(!strategy.is_active());
        assert_eq!(strategy.tracker_type(), TrackerType::HybridTracker);
    }

    #[test]
    fn test_hybrid_initialization() {
        let mut strategy = HybridStrategy::new();
        let config = TrackerConfig::default();
        
        let result = strategy.initialize(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hybrid_tracking_lifecycle() {
        let mut strategy = HybridStrategy::new();
        strategy.initialize(TrackerConfig::default()).unwrap();
        
        // Start tracking
        assert!(strategy.start_tracking().is_ok());
        assert!(strategy.is_active());
        
        // Stop tracking
        let data = strategy.stop_tracking();
        assert!(data.is_ok());
        assert!(!strategy.is_active());
    }
}