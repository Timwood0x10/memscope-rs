//! Efficiency scoring for task-level memory profiling
//!
//! This module provides comprehensive efficiency scoring algorithms
//! for different task types based on CPU, memory, IO, and network usage.

use serde::{Deserialize, Serialize};

/// Efficiency scoring configuration
#[derive(Debug, Clone)]
pub struct EfficiencyConfig {
    /// Enable CPU efficiency calculation
    pub enable_cpu_efficiency: bool,
    /// Enable memory efficiency calculation
    pub enable_memory_efficiency: bool,
    /// Enable IO efficiency calculation
    pub enable_io_efficiency: bool,
    /// Enable network efficiency calculation
    pub enable_network_efficiency: bool,
    /// Custom weights for task types
    pub custom_weights: Option<EfficiencyWeights>,
}

impl Default for EfficiencyConfig {
    fn default() -> Self {
        Self {
            enable_cpu_efficiency: true,
            enable_memory_efficiency: true,
            enable_io_efficiency: true,
            enable_network_efficiency: true,
            custom_weights: None,
        }
    }
}

impl EfficiencyConfig {
    /// Create configuration for minimal scoring
    pub fn minimal() -> Self {
        Self {
            enable_cpu_efficiency: true,
            enable_memory_efficiency: true,
            enable_io_efficiency: false,
            enable_network_efficiency: false,
            custom_weights: None,
        }
    }

    /// Create configuration for comprehensive scoring
    pub fn comprehensive() -> Self {
        Self {
            enable_cpu_efficiency: true,
            enable_memory_efficiency: true,
            enable_io_efficiency: true,
            enable_network_efficiency: true,
            custom_weights: None,
        }
    }
}

/// Custom weights for different task types
#[derive(Debug, Clone)]
pub struct EfficiencyWeights {
    /// CPU weight
    pub cpu_weight: f64,
    /// Memory weight
    pub memory_weight: f64,
    /// IO weight
    pub io_weight: f64,
    /// Network weight
    pub network_weight: f64,
}

impl Default for EfficiencyWeights {
    fn default() -> Self {
        Self {
            cpu_weight: 0.25,
            memory_weight: 0.25,
            io_weight: 0.25,
            network_weight: 0.25,
        }
    }
}

/// Component efficiency scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    /// CPU efficiency score (0.0 to 1.0)
    pub cpu_efficiency: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// IO efficiency score (0.0 to 1.0)
    pub io_efficiency: f64,
    /// Network efficiency score (0.0 to 1.0)
    pub network_efficiency: f64,
}

impl Default for ComponentScores {
    fn default() -> Self {
        Self {
            cpu_efficiency: 0.0,
            memory_efficiency: 0.0,
            io_efficiency: 0.0,
            network_efficiency: 0.0,
        }
    }
}

impl ComponentScores {
    /// Calculate overall efficiency score
    pub fn overall(&self) -> f64 {
        let sum = self.cpu_efficiency
            + self.memory_efficiency
            + self.io_efficiency
            + self.network_efficiency;

        let count = if self.cpu_efficiency > 0.0 { 1 } else { 0 }
            + if self.memory_efficiency > 0.0 { 1 } else { 0 }
            + if self.io_efficiency > 0.0 { 1 } else { 0 }
            + if self.network_efficiency > 0.0 { 1 } else { 0 };

        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
}

/// Efficiency scorer for task-level memory profiling
pub struct EfficiencyScorer {
    config: EfficiencyConfig,
}

impl EfficiencyScorer {
    /// Create new efficiency scorer with default configuration
    pub fn new() -> Self {
        Self {
            config: EfficiencyConfig::default(),
        }
    }

    /// Create new efficiency scorer with custom configuration
    pub fn with_config(config: EfficiencyConfig) -> Self {
        Self { config }
    }

    /// Calculate efficiency score for a task profile
    pub fn calculate_efficiency(
        &self,
        profile: &crate::capture::backends::task_profile::TaskMemoryProfile,
        cpu_usage_percent: f64,
        io_bytes_processed: u64,
        network_bytes_transferred: u64,
    ) -> ComponentScores {
        let cpu_efficiency = if self.config.enable_cpu_efficiency {
            self.calculate_cpu_efficiency(cpu_usage_percent)
        } else {
            0.0
        };

        let memory_efficiency = if self.config.enable_memory_efficiency {
            self.calculate_memory_efficiency(profile)
        } else {
            0.0
        };

        let io_efficiency = if self.config.enable_io_efficiency {
            self.calculate_io_efficiency(io_bytes_processed)
        } else {
            0.0
        };

        let network_efficiency = if self.config.enable_network_efficiency {
            self.calculate_network_efficiency(network_bytes_transferred)
        } else {
            0.0
        };

        ComponentScores {
            cpu_efficiency,
            memory_efficiency,
            io_efficiency,
            network_efficiency,
        }
    }

    /// Calculate CPU efficiency score
    fn calculate_cpu_efficiency(&self, usage_percent: f64) -> f64 {
        let usage = (usage_percent / 100.0).clamp(0.0, 1.0);

        if usage <= 0.0 {
            return 0.0;
        }

        let efficiency = if usage <= 0.5 {
            usage * 2.0
        } else if usage <= 0.8 {
            0.5 + usage * 0.625
        } else {
            1.0
        };

        efficiency.clamp(0.0, 1.0)
    }

    /// Calculate memory efficiency score
    fn calculate_memory_efficiency(
        &self,
        task_profile: &crate::capture::backends::task_profile::TaskMemoryProfile,
    ) -> f64 {
        if task_profile.total_bytes == 0 {
            return 1.0;
        }

        let utilization = task_profile.current_memory as f64 / task_profile.total_bytes as f64;
        let efficiency = 1.0 - utilization;

        efficiency.clamp(0.0, 1.0)
    }

    /// Calculate IO efficiency score
    fn calculate_io_efficiency(&self, bytes_processed: u64) -> f64 {
        if bytes_processed == 0 {
            return 0.0;
        }

        let optimal_size = 65536.0;
        let efficiency = (bytes_processed as f64 / optimal_size).min(1.0);

        efficiency.clamp(0.0, 1.0)
    }

    /// Calculate network efficiency score
    fn calculate_network_efficiency(&self, bytes_transferred: u64) -> f64 {
        if bytes_transferred == 0 {
            return 0.0;
        }

        let bytes_per_second = bytes_transferred as f64 / 1_048_576.0;
        let efficiency = (bytes_per_second / 1_048_576.0).min(1.0);

        efficiency.clamp(0.0, 1.0)
    }

    /// Calculate weighted efficiency score based on task type
    pub fn calculate_weighted_efficiency(
        &self,
        task_type: &crate::capture::backends::task_profile::TaskType,
        component_scores: &ComponentScores,
    ) -> f64 {
        let weights = self.config.custom_weights.as_ref().map_or_else(
            || Self::default_weights_for_task_type(task_type),
            |w| w.clone(),
        );

        let weighted_score = component_scores.cpu_efficiency * weights.cpu_weight
            + component_scores.memory_efficiency * weights.memory_weight
            + component_scores.io_efficiency * weights.io_weight
            + component_scores.network_efficiency * weights.network_weight;

        weighted_score.clamp(0.0, 1.0)
    }

    /// Get default weights for a task type
    fn default_weights_for_task_type(
        task_type: &crate::capture::backends::task_profile::TaskType,
    ) -> EfficiencyWeights {
        match task_type {
            crate::capture::backends::task_profile::TaskType::CpuIntensive => EfficiencyWeights {
                cpu_weight: 0.6,
                memory_weight: 0.2,
                io_weight: 0.1,
                network_weight: 0.1,
            },
            crate::capture::backends::task_profile::TaskType::IoIntensive => EfficiencyWeights {
                cpu_weight: 0.2,
                memory_weight: 0.1,
                io_weight: 0.6,
                network_weight: 0.1,
            },
            crate::capture::backends::task_profile::TaskType::NetworkIntensive => {
                EfficiencyWeights {
                    cpu_weight: 0.2,
                    memory_weight: 0.1,
                    io_weight: 0.1,
                    network_weight: 0.6,
                }
            }
            crate::capture::backends::task_profile::TaskType::MemoryIntensive => {
                EfficiencyWeights {
                    cpu_weight: 0.2,
                    memory_weight: 0.6,
                    io_weight: 0.1,
                    network_weight: 0.1,
                }
            }
            crate::capture::backends::task_profile::TaskType::GpuCompute => EfficiencyWeights {
                cpu_weight: 0.5,
                memory_weight: 0.2,
                io_weight: 0.1,
                network_weight: 0.2,
            },
            _ => EfficiencyWeights::default(),
        }
    }
}

impl Default for EfficiencyScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_efficiency_config_default() {
        let config = EfficiencyConfig::default();
        assert!(config.enable_cpu_efficiency);
        assert!(config.enable_memory_efficiency);
        assert!(config.enable_io_efficiency);
        assert!(config.enable_network_efficiency);
    }

    #[test]
    fn test_efficiency_config_minimal() {
        let config = EfficiencyConfig::minimal();
        assert!(config.enable_cpu_efficiency);
        assert!(config.enable_memory_efficiency);
        assert!(!config.enable_io_efficiency);
        assert!(!config.enable_network_efficiency);
    }

    #[test]
    fn test_task_type_weights_default() {
        let weights = EfficiencyWeights::default();
        assert_eq!(weights.cpu_weight, 0.25);
        assert_eq!(weights.memory_weight, 0.25);
        assert_eq!(weights.io_weight, 0.25);
        assert_eq!(weights.network_weight, 0.25);
    }

    #[test]
    fn test_component_scores_default() {
        let scores = ComponentScores::default();
        assert_eq!(scores.cpu_efficiency, 0.0);
        assert_eq!(scores.memory_efficiency, 0.0);
        assert_eq!(scores.io_efficiency, 0.0);
        assert_eq!(scores.network_efficiency, 0.0);
    }

    #[test]
    fn test_component_scores_overall() {
        let mut scores = ComponentScores::default();
        scores.cpu_efficiency = 0.8;
        scores.memory_efficiency = 0.6;
        scores.io_efficiency = 0.4;
        scores.network_efficiency = 0.2;

        let overall = scores.overall();
        assert_eq!(overall, 0.5);
    }

    #[test]
    fn test_cpu_efficiency_calculation() {
        let scorer = EfficiencyScorer::new();

        let efficiency = scorer.calculate_cpu_efficiency(0.0);
        assert_eq!(efficiency, 0.0);

        let efficiency = scorer.calculate_cpu_efficiency(50.0);
        assert_eq!(efficiency, 1.0);

        let efficiency = scorer.calculate_cpu_efficiency(100.0);
        assert_eq!(efficiency, 1.0);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let scorer = EfficiencyScorer::new();

        let mut profile = crate::capture::backends::task_profile::TaskMemoryProfile::new(
            1,
            "test".to_string(),
            crate::capture::backends::task_profile::TaskType::default(),
        );

        let efficiency = scorer.calculate_memory_efficiency(&profile);
        assert_eq!(efficiency, 1.0);

        profile.total_bytes = 1000;
        profile.current_memory = 500;
        let efficiency = scorer.calculate_memory_efficiency(&profile);
        assert_eq!(efficiency, 0.5);
    }

    #[test]
    fn test_io_efficiency_calculation() {
        let scorer = EfficiencyScorer::new();

        let efficiency = scorer.calculate_io_efficiency(0);
        assert_eq!(efficiency, 0.0);

        let efficiency = scorer.calculate_io_efficiency(65536);
        assert_eq!(efficiency, 1.0);
    }

    #[test]
    fn test_network_efficiency_calculation() {
        let scorer = EfficiencyScorer::new();

        let efficiency = scorer.calculate_network_efficiency(0);
        assert_eq!(efficiency, 0.0);

        let efficiency = scorer.calculate_network_efficiency(1_048_576);
        assert_eq!(efficiency, 1.0);
    }

    #[test]
    fn test_weighted_efficiency_cpu_intensive() {
        let scorer = EfficiencyScorer::new();

        let task_type = crate::capture::backends::task_profile::TaskType::CpuIntensive;
        let component_scores = ComponentScores {
            cpu_efficiency: 0.8,
            memory_efficiency: 0.6,
            io_efficiency: 0.4,
            network_efficiency: 0.2,
        };

        let weighted = scorer.calculate_weighted_efficiency(&task_type, &component_scores);
        assert_eq!(weighted, 0.68);
    }

    #[test]
    fn test_weighted_efficiency_io_intensive() {
        let scorer = EfficiencyScorer::new();

        let task_type = crate::capture::backends::task_profile::TaskType::IoIntensive;
        let component_scores = ComponentScores {
            cpu_efficiency: 0.6,
            memory_efficiency: 0.4,
            io_efficiency: 0.8,
            network_efficiency: 0.2,
        };

        let weighted = scorer.calculate_weighted_efficiency(&task_type, &component_scores);
        assert_eq!(weighted, 0.68);
    }

    #[test]
    fn test_weighted_efficiency_memory_intensive() {
        let scorer = EfficiencyScorer::new();

        let task_type = crate::capture::backends::task_profile::TaskType::MemoryIntensive;
        let component_scores = ComponentScores {
            cpu_efficiency: 0.6,
            memory_efficiency: 0.8,
            io_efficiency: 0.4,
            network_efficiency: 0.2,
        };

        let weighted = scorer.calculate_weighted_efficiency(&task_type, &component_scores);
        assert_eq!(weighted, 0.68);
    }

    #[test]
    fn test_weighted_efficiency_network_intensive() {
        let scorer = EfficiencyScorer::new();

        let task_type = crate::capture::backends::task_profile::TaskType::NetworkIntensive;
        let component_scores = ComponentScores {
            cpu_efficiency: 0.6,
            memory_efficiency: 0.4,
            io_efficiency: 0.2,
            network_efficiency: 0.8,
        };

        let weighted = scorer.calculate_weighted_efficiency(&task_type, &component_scores);
        assert_eq!(weighted, 0.68);
    }
}
