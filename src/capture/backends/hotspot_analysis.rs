//! Enhanced hotspot analysis for performance optimization
//!
//! This module provides advanced hotspot detection capabilities,
//! including call stack analysis, frequency pattern detection,
//! and memory peak detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hot call stack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackHotspot {
    /// Hash of the call stack
    pub call_stack_hash: u64,
    /// Total frequency across all allocations
    pub total_frequency: u64,
    /// Total memory allocated by this call stack
    pub total_size: usize,
    /// Impact score (frequency * size)
    pub impact_score: u64,
    /// Tasks that use this call stack
    pub tasks: Vec<u64>,
    /// Average allocation size
    pub average_size: f64,
    /// Peak memory usage from this call stack
    pub peak_memory: usize,
}

impl CallStackHotspot {
    /// Create new hot call stack
    pub fn new(call_stack_hash: u64) -> Self {
        Self {
            call_stack_hash,
            total_frequency: 0,
            total_size: 0,
            impact_score: 0,
            tasks: Vec::new(),
            average_size: 0.0,
            peak_memory: 0,
        }
    }

    /// Add allocation to this call stack
    pub fn add_allocation(&mut self, size: usize, task_id: u64) {
        self.total_frequency += 1;
        self.total_size += size;
        self.impact_score = self.total_frequency.saturating_mul(self.total_size as u64);
        self.average_size = self.total_size as f64 / self.total_frequency as f64;
        self.peak_memory = self.peak_memory.max(size);

        if !self.tasks.contains(&task_id) {
            self.tasks.push(task_id);
        }
    }

    /// Get impact score
    pub fn impact_score(&self) -> u64 {
        self.impact_score
    }
}

/// Frequency pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAnalysis {
    /// Call stack hash
    pub call_stack_hash: u64,
    /// Allocation frequency (allocations per second)
    pub frequency_per_sec: f64,
    /// Pattern type
    pub pattern: AllocationFrequencyPattern,
    /// Time window analyzed
    pub time_window_ms: u64,
    /// Total allocations in window
    pub total_allocations: u64,
}

/// Frequency pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationFrequencyPattern {
    /// Constant allocation rate
    Constant,
    /// Increasing allocation rate (potential leak)
    Increasing,
    /// Decreasing allocation rate
    Decreasing,
    /// Bursty allocation pattern
    Bursty,
    /// Sporadic allocation pattern
    Sporadic,
}

impl AllocationFrequencyPattern {
    /// Get description of pattern
    pub fn description(&self) -> &'static str {
        match self {
            Self::Constant => "Constant allocation rate",
            Self::Increasing => "Increasing allocation rate (potential memory leak)",
            Self::Decreasing => "Decreasing allocation rate",
            Self::Bursty => "Bursty allocation pattern",
            Self::Sporadic => "Sporadic allocation pattern",
        }
    }
}

/// Memory peak detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsagePeak {
    /// Timestamp of the peak
    pub timestamp_ms: u64,
    /// Task ID that caused the peak
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Memory usage at peak (bytes)
    pub memory_usage: usize,
    /// Number of active allocations at peak
    pub active_allocations: u64,
    /// Call stack that triggered the peak
    pub triggering_call_stack: u64,
    /// Peak duration in milliseconds
    pub duration_ms: u64,
}

/// Enhanced hotspot analyzer
pub struct HotspotAnalyzer {
    /// Hot call stacks detected
    hot_call_stacks: HashMap<u64, CallStackHotspot>,
    /// Frequency data for call stacks
    frequency_data: HashMap<u64, FrequencyAnalysis>,
    /// Memory peaks detected
    memory_peaks: Vec<MemoryUsagePeak>,
    /// Configuration
    config: HotspotConfig,
}

/// Hotspot analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotConfig {
    /// Minimum frequency to consider a call stack as hot
    pub min_hot_frequency: u64,
    /// Minimum impact score to consider a call stack as hot
    pub min_impact_score: u64,
    /// Maximum number of hot call stacks to track
    pub max_hot_call_stacks: usize,
    /// Enable frequency pattern analysis
    pub enable_frequency_analysis: bool,
    /// Enable memory peak detection
    pub enable_peak_detection: bool,
    /// Peak detection threshold (percentage of peak memory)
    pub peak_threshold_percent: f64,
}

impl Default for HotspotConfig {
    fn default() -> Self {
        Self {
            min_hot_frequency: 10,
            min_impact_score: 1000,
            max_hot_call_stacks: 100,
            enable_frequency_analysis: true,
            enable_peak_detection: true,
            peak_threshold_percent: 90.0,
        }
    }
}

impl HotspotAnalyzer {
    /// Create new hotspot analyzer with default configuration
    pub fn new() -> Self {
        Self {
            hot_call_stacks: HashMap::new(),
            frequency_data: HashMap::new(),
            memory_peaks: Vec::new(),
            config: HotspotConfig::default(),
        }
    }

    /// Create new hotspot analyzer with custom configuration
    pub fn with_config(config: HotspotConfig) -> Self {
        Self {
            hot_call_stacks: HashMap::new(),
            frequency_data: HashMap::new(),
            memory_peaks: Vec::new(),
            config,
        }
    }

    /// Analyze allocation for hotspot detection
    pub fn analyze_allocation(
        &mut self,
        call_stack_hash: u64,
        size: usize,
        task_id: u64,
        timestamp_ms: u64,
    ) {
        let hot_stack = self
            .hot_call_stacks
            .entry(call_stack_hash)
            .or_insert_with(|| CallStackHotspot::new(call_stack_hash));
        hot_stack.add_allocation(size, task_id);

        if self.config.enable_peak_detection {
            self.detect_memory_peak(task_id, size, call_stack_hash, timestamp_ms);
        }
    }

    /// Analyze frequency pattern for a call stack
    pub fn analyze_frequency_pattern(
        &mut self,
        call_stack_hash: u64,
        allocations_in_window: u64,
        time_window_ms: u64,
    ) {
        if !self.config.enable_frequency_analysis {
            return;
        }

        let frequency_per_sec = if time_window_ms > 0 {
            (allocations_in_window as f64 * 1000.0) / time_window_ms as f64
        } else {
            0.0
        };

        let pattern = self.detect_pattern(allocations_in_window, time_window_ms);

        let frequency_data = FrequencyAnalysis {
            call_stack_hash,
            frequency_per_sec,
            pattern,
            time_window_ms,
            total_allocations: allocations_in_window,
        };

        self.frequency_data.insert(call_stack_hash, frequency_data);
    }

    /// Detect frequency pattern
    fn detect_pattern(&self, allocations: u64, time_window_ms: u64) -> AllocationFrequencyPattern {
        let frequency_per_sec = if time_window_ms > 0 {
            (allocations as f64 * 1000.0) / time_window_ms as f64
        } else {
            0.0
        };

        if frequency_per_sec < 1.0 {
            AllocationFrequencyPattern::Sporadic
        } else if frequency_per_sec > 100.0 {
            AllocationFrequencyPattern::Bursty
        } else if allocations > 1000 && time_window_ms > 10000 {
            AllocationFrequencyPattern::Increasing
        } else {
            AllocationFrequencyPattern::Constant
        }
    }

    /// Detect memory peak
    fn detect_memory_peak(
        &mut self,
        task_id: u64,
        memory_usage: usize,
        call_stack_hash: u64,
        timestamp_ms: u64,
    ) {
        let is_peak = if self.memory_peaks.is_empty() {
            true
        } else {
            let max_peak = self
                .memory_peaks
                .iter()
                .map(|p| p.memory_usage)
                .max()
                .unwrap_or(0);
            memory_usage as f64 > max_peak as f64 * (self.config.peak_threshold_percent / 100.0)
        };

        if is_peak {
            let peak = MemoryUsagePeak {
                timestamp_ms,
                task_id,
                task_name: String::from("unknown"),
                memory_usage,
                active_allocations: 0,
                triggering_call_stack: call_stack_hash,
                duration_ms: 0,
            };
            self.memory_peaks.push(peak);
        }
    }

    /// Get hot call stacks sorted by impact score
    pub fn get_hot_call_stacks(&self) -> Vec<&CallStackHotspot> {
        let mut stacks: Vec<&CallStackHotspot> = self
            .hot_call_stacks
            .values()
            .filter(|s| {
                s.total_frequency >= self.config.min_hot_frequency
                    || s.impact_score >= self.config.min_impact_score
            })
            .collect();

        stacks.sort_by_key(|b| std::cmp::Reverse(b.impact_score));
        stacks.truncate(self.config.max_hot_call_stacks);
        stacks
    }

    /// Get frequency data for all call stacks
    pub fn get_frequency_data(&self) -> Vec<&FrequencyAnalysis> {
        self.frequency_data.values().collect()
    }

    /// Get memory peaks sorted by memory usage
    pub fn get_memory_peaks(&self) -> Vec<&MemoryUsagePeak> {
        let mut peaks: Vec<&MemoryUsagePeak> = self.memory_peaks.iter().collect();
        peaks.sort_by_key(|b| std::cmp::Reverse(b.memory_usage));
        peaks
    }

    /// Get top N hot call stacks by impact score
    pub fn get_top_hot_call_stacks(&self, n: usize) -> Vec<&CallStackHotspot> {
        let mut stacks = self.get_hot_call_stacks();
        stacks.truncate(n);
        stacks
    }

    /// Get top N memory peaks by memory usage
    pub fn get_top_memory_peaks(&self, n: usize) -> Vec<&MemoryUsagePeak> {
        let mut peaks = self.get_memory_peaks();
        peaks.truncate(n);
        peaks
    }

    /// Clear all collected data
    pub fn clear(&mut self) {
        self.hot_call_stacks.clear();
        self.frequency_data.clear();
        self.memory_peaks.clear();
    }

    /// Get analyzer configuration
    pub fn config(&self) -> &HotspotConfig {
        &self.config
    }

    /// Update analyzer configuration
    pub fn set_config(&mut self, config: HotspotConfig) {
        self.config = config;
    }

    /// Get statistics
    pub fn get_statistics(&self) -> HotspotStatistics {
        HotspotStatistics {
            total_call_stacks: self.hot_call_stacks.len(),
            hot_call_stacks: self.get_hot_call_stacks().len(),
            total_memory_peaks: self.memory_peaks.len(),
            total_frequency_data: self.frequency_data.len(),
            total_allocations_analyzed: self
                .hot_call_stacks
                .values()
                .map(|s| s.total_frequency)
                .sum(),
            total_memory_analyzed: self.hot_call_stacks.values().map(|s| s.total_size).sum(),
        }
    }
}

impl Default for HotspotAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Hotspot analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotStatistics {
    /// Total number of call stacks analyzed
    pub total_call_stacks: usize,
    /// Number of hot call stacks detected
    pub hot_call_stacks: usize,
    /// Total number of memory peaks detected
    pub total_memory_peaks: usize,
    /// Total frequency data collected
    pub total_frequency_data: usize,
    /// Total allocations analyzed
    pub total_allocations_analyzed: u64,
    /// Total memory analyzed (bytes)
    pub total_memory_analyzed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_call_stack_creation() {
        let stack = CallStackHotspot::new(12345);
        assert_eq!(stack.call_stack_hash, 12345);
        assert_eq!(stack.total_frequency, 0);
        assert_eq!(stack.total_size, 0);
    }

    #[test]
    fn test_hot_call_stack_add_allocation() {
        let mut stack = CallStackHotspot::new(12345);
        stack.add_allocation(1024, 1);
        stack.add_allocation(2048, 1);

        assert_eq!(stack.total_frequency, 2);
        assert_eq!(stack.total_size, 3072);
        assert_eq!(stack.impact_score, 6144);
        assert_eq!(stack.average_size, 1536.0);
        assert_eq!(stack.peak_memory, 2048);
        assert_eq!(stack.tasks.len(), 1);
    }

    #[test]
    fn test_hotspot_analyzer_creation() {
        let analyzer = HotspotAnalyzer::new();
        assert!(analyzer.hot_call_stacks.is_empty());
        assert!(analyzer.memory_peaks.is_empty());
    }

    #[test]
    fn test_analyze_allocation() {
        let mut analyzer = HotspotAnalyzer::new();
        analyzer.analyze_allocation(12345, 1024, 1, 1000);

        let stacks = analyzer.get_hot_call_stacks();
        assert!(!stacks.is_empty());
        assert_eq!(stacks[0].call_stack_hash, 12345);
        assert_eq!(stacks[0].total_frequency, 1);
    }

    #[test]
    fn test_frequency_pattern_detection() {
        let mut analyzer = HotspotAnalyzer::new();

        analyzer.analyze_frequency_pattern(12345, 50, 1000);
        let data = analyzer.get_frequency_data();
        assert_eq!(data[0].pattern, AllocationFrequencyPattern::Constant);
    }

    #[test]
    fn test_memory_peak_detection() {
        let mut analyzer = HotspotAnalyzer::new();
        analyzer.detect_memory_peak(1, 1024, 12345, 1000);
        analyzer.detect_memory_peak(1, 2048, 12345, 2000);
        analyzer.detect_memory_peak(1, 4096, 12345, 3000);

        let peaks = analyzer.get_memory_peaks();
        assert!(!peaks.is_empty());
        assert_eq!(peaks[0].memory_usage, 4096);
    }

    #[test]
    fn test_get_top_hot_call_stacks() {
        let mut analyzer = HotspotAnalyzer::new();
        analyzer.analyze_allocation(1, 1024, 1, 1000);
        analyzer.analyze_allocation(2, 2048, 1, 1000);
        analyzer.analyze_allocation(3, 4096, 1, 1000);

        let top = analyzer.get_top_hot_call_stacks(2);
        assert!(top.len() <= 2);
    }

    #[test]
    fn test_clear() {
        let mut analyzer = HotspotAnalyzer::new();
        analyzer.analyze_allocation(12345, 1024, 1, 1000);
        analyzer.clear();

        assert!(analyzer.hot_call_stacks.is_empty());
        assert!(analyzer.memory_peaks.is_empty());
    }

    #[test]
    fn test_get_statistics() {
        let mut analyzer = HotspotAnalyzer::new();
        analyzer.analyze_allocation(12345, 1024, 1, 1000);
        analyzer.analyze_allocation(12345, 2048, 1, 1000);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_call_stacks, 1);
        assert_eq!(stats.total_allocations_analyzed, 2);
        assert_eq!(stats.total_memory_analyzed, 3072);
    }

    #[test]
    fn test_frequency_pattern_description() {
        assert_eq!(
            AllocationFrequencyPattern::Constant.description(),
            "Constant allocation rate"
        );
        assert_eq!(
            AllocationFrequencyPattern::Increasing.description(),
            "Increasing allocation rate (potential memory leak)"
        );
    }

    #[test]
    fn test_custom_config() {
        let config = HotspotConfig {
            min_hot_frequency: 100,
            min_impact_score: 10000,
            max_hot_call_stacks: 50,
            ..Default::default()
        };
        let analyzer = HotspotAnalyzer::with_config(config);

        assert_eq!(analyzer.config().min_hot_frequency, 100);
        assert_eq!(analyzer.config().max_hot_call_stacks, 50);
    }
}
