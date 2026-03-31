//! Resource ranking and optimization recommendations
//!
//! This module provides comprehensive resource ranking capabilities,
//! including multi-dimensional resource comparison and optimization suggestions.

use serde::{Deserialize, Serialize};

/// Resource ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRanking {
    /// Task ID
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Task type
    pub task_type: String,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// I/O usage in MB
    pub io_usage_mb: f64,
    /// Network usage in MB
    pub network_usage_mb: f64,
    /// GPU usage percentage
    pub gpu_usage: f64,
    /// Overall efficiency score (0.0 to 1.0)
    pub overall_score: f64,
    /// Resource efficiency scores
    pub efficiency_scores: EfficiencyScores,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Efficiency scores for different resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyScores {
    /// CPU efficiency score
    pub cpu_efficiency: f64,
    /// Memory efficiency score
    pub memory_efficiency: f64,
    /// I/O efficiency score
    pub io_efficiency: f64,
    /// Network efficiency score
    pub network_efficiency: f64,
    /// GPU efficiency score
    pub gpu_efficiency: f64,
}

impl EfficiencyScores {
    /// Create new efficiency scores
    pub fn new(cpu: f64, memory: f64, io: f64, network: f64, gpu: f64) -> Self {
        Self {
            cpu_efficiency: cpu,
            memory_efficiency: memory,
            io_efficiency: io,
            network_efficiency: network,
            gpu_efficiency: gpu,
        }
    }

    /// Calculate overall efficiency score
    pub fn overall_score(&self) -> f64 {
        let scores = vec![
            self.cpu_efficiency,
            self.memory_efficiency,
            self.io_efficiency,
            self.network_efficiency,
            self.gpu_efficiency,
        ];
        let valid_scores: Vec<f64> = scores.into_iter().filter(|&s| s > 0.0).collect();

        if valid_scores.is_empty() {
            0.0
        } else {
            valid_scores.iter().sum::<f64>() / valid_scores.len() as f64
        }
    }
}

/// Resource ranking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    /// Weight for CPU usage in overall score
    pub cpu_weight: f64,
    /// Weight for memory usage in overall score
    pub memory_weight: f64,
    /// Weight for I/O usage in overall score
    pub io_weight: f64,
    /// Weight for network usage in overall score
    pub network_weight: f64,
    /// Weight for GPU usage in overall score
    pub gpu_weight: f64,
    /// Enable optimization recommendations
    pub enable_recommendations: bool,
    /// Minimum efficiency score threshold for recommendations
    pub min_efficiency_threshold: f64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            cpu_weight: 0.3,
            memory_weight: 0.3,
            io_weight: 0.2,
            network_weight: 0.1,
            gpu_weight: 0.1,
            enable_recommendations: true,
            min_efficiency_threshold: 0.6,
        }
    }
}

/// Resource ranking analyzer
pub struct ResourceRankingAnalyzer {
    config: RankingConfig,
    rankings: Vec<ResourceRanking>,
}

impl ResourceRankingAnalyzer {
    /// Create new resource ranking analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: RankingConfig::default(),
            rankings: Vec::new(),
        }
    }

    /// Create new resource ranking analyzer with custom configuration
    pub fn with_config(config: RankingConfig) -> Self {
        Self {
            config,
            rankings: Vec::new(),
        }
    }

    /// Analyze and rank resources for a task
    #[allow(clippy::too_many_arguments)]
    pub fn analyze_task(
        &mut self,
        task_id: u64,
        task_name: &str,
        task_type: &str,
        cpu_usage: f64,
        memory_usage_mb: f64,
        io_usage_mb: f64,
        network_usage_mb: f64,
        gpu_usage: f64,
    ) -> ResourceRanking {
        let efficiency_scores = EfficiencyScores::new(
            self.calculate_cpu_efficiency(cpu_usage),
            self.calculate_memory_efficiency(memory_usage_mb),
            self.calculate_io_efficiency(io_usage_mb),
            self.calculate_network_efficiency(network_usage_mb),
            self.calculate_gpu_efficiency(gpu_usage),
        );

        let overall_score = self.calculate_overall_score(
            cpu_usage,
            memory_usage_mb,
            io_usage_mb,
            network_usage_mb,
            gpu_usage,
        );

        let recommendations = if self.config.enable_recommendations {
            self.generate_recommendations(
                &efficiency_scores,
                overall_score,
                cpu_usage,
                memory_usage_mb,
                io_usage_mb,
                network_usage_mb,
                gpu_usage,
            )
        } else {
            Vec::new()
        };

        let ranking = ResourceRanking {
            task_id,
            task_name: task_name.to_string(),
            task_type: task_type.to_string(),
            cpu_usage,
            memory_usage_mb,
            io_usage_mb,
            network_usage_mb,
            gpu_usage,
            overall_score,
            efficiency_scores,
            recommendations,
        };

        self.rankings.push(ranking.clone());
        ranking
    }

    /// Calculate CPU efficiency score
    fn calculate_cpu_efficiency(&self, cpu_usage: f64) -> f64 {
        if cpu_usage <= 0.0 {
            return 0.0;
        }

        if cpu_usage <= 50.0 {
            1.0
        } else if cpu_usage <= 75.0 {
            0.8
        } else if cpu_usage <= 90.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate memory efficiency score
    fn calculate_memory_efficiency(&self, memory_mb: f64) -> f64 {
        if memory_mb <= 0.0 {
            return 0.0;
        }

        if memory_mb <= 100.0 {
            1.0
        } else if memory_mb <= 500.0 {
            0.8
        } else if memory_mb <= 1000.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate I/O efficiency score
    fn calculate_io_efficiency(&self, io_mb: f64) -> f64 {
        if io_mb <= 0.0 {
            return 0.0;
        }

        if io_mb <= 10.0 {
            1.0
        } else if io_mb <= 100.0 {
            0.8
        } else if io_mb <= 500.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate network efficiency score
    fn calculate_network_efficiency(&self, network_mb: f64) -> f64 {
        if network_mb <= 0.0 {
            return 0.0;
        }

        if network_mb <= 10.0 {
            1.0
        } else if network_mb <= 100.0 {
            0.8
        } else if network_mb <= 500.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate GPU efficiency score
    fn calculate_gpu_efficiency(&self, gpu_usage: f64) -> f64 {
        if gpu_usage <= 0.0 {
            return 0.0;
        }

        if gpu_usage <= 50.0 {
            1.0
        } else if gpu_usage <= 75.0 {
            0.8
        } else if gpu_usage <= 90.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Calculate overall score
    fn calculate_overall_score(
        &self,
        cpu_usage: f64,
        memory_mb: f64,
        io_mb: f64,
        network_mb: f64,
        gpu_usage: f64,
    ) -> f64 {
        let cpu_score = self.calculate_cpu_efficiency(cpu_usage);
        let memory_score = self.calculate_memory_efficiency(memory_mb);
        let io_score = self.calculate_io_efficiency(io_mb);
        let network_score = self.calculate_network_efficiency(network_mb);
        let gpu_score = self.calculate_gpu_efficiency(gpu_usage);

        let weighted_sum = cpu_score * self.config.cpu_weight
            + memory_score * self.config.memory_weight
            + io_score * self.config.io_weight
            + network_score * self.config.network_weight
            + gpu_score * self.config.gpu_weight;

        let total_weight = self.config.cpu_weight
            + self.config.memory_weight
            + self.config.io_weight
            + self.config.network_weight
            + self.config.gpu_weight;

        weighted_sum / total_weight
    }

    /// Generate optimization recommendations
    #[allow(clippy::too_many_arguments)]
    fn generate_recommendations(
        &self,
        efficiency: &EfficiencyScores,
        overall_score: f64,
        cpu_usage: f64,
        memory_mb: f64,
        io_mb: f64,
        network_mb: f64,
        gpu_usage: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if overall_score < self.config.min_efficiency_threshold {
            recommendations.push(
                "Overall efficiency is below threshold. Consider reviewing resource usage patterns.".to_string()
            );
        }

        if efficiency.cpu_efficiency < 0.6 {
            if cpu_usage > 90.0 {
                recommendations.push(
                    "CPU usage is critical (>90%). Consider parallelizing work or optimizing algorithms.".to_string()
                );
            } else if cpu_usage > 75.0 {
                recommendations.push(
                    "CPU usage is high (>75%). Profile hot paths and optimize critical sections."
                        .to_string(),
                );
            }
        }

        if efficiency.memory_efficiency < 0.6 {
            if memory_mb > 1000.0 {
                recommendations.push(
                    "Memory usage is high (>1GB). Implement memory pooling or reduce footprint."
                        .to_string(),
                );
            } else if memory_mb > 500.0 {
                recommendations.push(
                    "Memory usage is moderate (>500MB). Consider optimizing data structures."
                        .to_string(),
                );
            }
        }

        if efficiency.io_efficiency < 0.6 {
            if io_mb > 500.0 {
                recommendations.push(
                    "I/O usage is high (>500MB). Implement buffering or async I/O.".to_string(),
                );
            } else if io_mb > 100.0 {
                recommendations.push(
                    "I/O usage is moderate (>100MB). Consider batching operations.".to_string(),
                );
            }
        }

        if efficiency.network_efficiency < 0.6 {
            if network_mb > 500.0 {
                recommendations.push(
                    "Network usage is high (>500MB). Implement compression or connection pooling."
                        .to_string(),
                );
            } else if network_mb > 100.0 {
                recommendations.push(
                    "Network usage is moderate (>100MB). Consider caching or batching requests."
                        .to_string(),
                );
            }
        }

        if efficiency.gpu_efficiency < 0.6 {
            if gpu_usage > 90.0 {
                recommendations.push(
                    "GPU usage is critical (>90%). Optimize kernel execution or reduce workload."
                        .to_string(),
                );
            } else if gpu_usage > 75.0 {
                recommendations.push(
                    "GPU usage is high (>75%). Review compute kernel efficiency.".to_string(),
                );
            }
        }

        recommendations
    }

    /// Get all rankings sorted by overall score
    pub fn get_rankings(&self) -> Vec<&ResourceRanking> {
        let mut rankings: Vec<&ResourceRanking> = self.rankings.iter().collect();
        rankings.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());
        rankings
    }

    /// Get top N rankings
    pub fn get_top_rankings(&self, n: usize) -> Vec<&ResourceRanking> {
        let mut rankings = self.get_rankings();
        rankings.truncate(n);
        rankings
    }

    /// Get bottom N rankings
    pub fn get_bottom_rankings(&self, n: usize) -> Vec<&ResourceRanking> {
        let mut rankings: Vec<&ResourceRanking> = self.rankings.iter().collect();
        rankings.sort_by(|a, b| a.overall_score.partial_cmp(&b.overall_score).unwrap());
        rankings.truncate(n);
        rankings
    }

    /// Get rankings by task type
    pub fn get_rankings_by_type(&self, task_type: &str) -> Vec<&ResourceRanking> {
        self.rankings
            .iter()
            .filter(|r| r.task_type == task_type)
            .collect()
    }

    /// Get analyzer configuration
    pub fn config(&self) -> &RankingConfig {
        &self.config
    }

    /// Update analyzer configuration
    pub fn set_config(&mut self, config: RankingConfig) {
        self.config = config;
    }

    /// Clear all rankings
    pub fn clear(&mut self) {
        self.rankings.clear();
    }

    /// Get statistics
    pub fn get_statistics(&self) -> RankingStatistics {
        if self.rankings.is_empty() {
            return RankingStatistics::default();
        }

        let overall_scores: Vec<f64> = self.rankings.iter().map(|r| r.overall_score).collect();
        let avg_score = overall_scores.iter().sum::<f64>() / overall_scores.len() as f64;
        let max_score = overall_scores
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let min_score = overall_scores.iter().cloned().fold(f64::INFINITY, f64::min);

        let cpu_scores: Vec<f64> = self
            .rankings
            .iter()
            .map(|r| r.efficiency_scores.cpu_efficiency)
            .collect();
        let avg_cpu = cpu_scores.iter().sum::<f64>() / cpu_scores.len() as f64;

        let memory_scores: Vec<f64> = self
            .rankings
            .iter()
            .map(|r| r.efficiency_scores.memory_efficiency)
            .collect();
        let avg_memory = memory_scores.iter().sum::<f64>() / memory_scores.len() as f64;

        RankingStatistics {
            total_tasks: self.rankings.len(),
            average_overall_score: avg_score,
            max_overall_score: max_score,
            min_overall_score: min_score,
            average_cpu_efficiency: avg_cpu,
            average_memory_efficiency: avg_memory,
        }
    }
}

impl Default for ResourceRankingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource ranking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingStatistics {
    /// Total number of tasks ranked
    pub total_tasks: usize,
    /// Average overall score
    pub average_overall_score: f64,
    /// Maximum overall score
    pub max_overall_score: f64,
    /// Minimum overall score
    pub min_overall_score: f64,
    /// Average CPU efficiency
    pub average_cpu_efficiency: f64,
    /// Average memory efficiency
    pub average_memory_efficiency: f64,
}

impl Default for RankingStatistics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            average_overall_score: 0.0,
            max_overall_score: 0.0,
            min_overall_score: 0.0,
            average_cpu_efficiency: 0.0,
            average_memory_efficiency: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_efficiency_scores_creation() {
        let scores = EfficiencyScores::new(0.8, 0.7, 0.9, 0.6, 0.5);
        assert_eq!(scores.cpu_efficiency, 0.8);
        assert_eq!(scores.memory_efficiency, 0.7);
        assert_eq!(scores.io_efficiency, 0.9);
        assert_eq!(scores.network_efficiency, 0.6);
        assert_eq!(scores.gpu_efficiency, 0.5);
    }

    #[test]
    fn test_efficiency_scores_overall() {
        let scores = EfficiencyScores::new(0.8, 0.7, 0.9, 0.6, 0.5);
        let overall = scores.overall_score();
        assert!((overall - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_resource_ranking_analyzer_creation() {
        let analyzer = ResourceRankingAnalyzer::new();
        assert!(analyzer.rankings.is_empty());
    }

    #[test]
    fn test_analyze_task() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        let ranking = analyzer.analyze_task(
            1,
            "test_task",
            "CpuIntensive",
            60.0,
            200.0,
            50.0,
            30.0,
            40.0,
        );

        assert_eq!(ranking.task_id, 1);
        assert_eq!(ranking.task_name, "test_task");
        assert!(ranking.overall_score > 0.0);
    }

    #[test]
    fn test_get_rankings() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        analyzer.analyze_task(1, "task1", "CpuIntensive", 80.0, 200.0, 50.0, 30.0, 40.0);
        analyzer.analyze_task(2, "task2", "CpuIntensive", 40.0, 100.0, 30.0, 20.0, 30.0);

        let rankings = analyzer.get_rankings();
        assert_eq!(rankings.len(), 2);
        assert!(rankings[0].overall_score >= rankings[1].overall_score);
    }

    #[test]
    fn test_get_top_rankings() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        analyzer.analyze_task(1, "task1", "CpuIntensive", 80.0, 200.0, 50.0, 30.0, 40.0);
        analyzer.analyze_task(2, "task2", "CpuIntensive", 40.0, 100.0, 30.0, 20.0, 30.0);
        analyzer.analyze_task(3, "task3", "CpuIntensive", 60.0, 150.0, 40.0, 25.0, 35.0);

        let top = analyzer.get_top_rankings(2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_get_rankings_by_type() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        analyzer.analyze_task(1, "task1", "CpuIntensive", 60.0, 200.0, 50.0, 30.0, 40.0);
        analyzer.analyze_task(2, "task2", "IoIntensive", 40.0, 100.0, 80.0, 20.0, 30.0);

        let cpu_rankings = analyzer.get_rankings_by_type("CpuIntensive");
        assert_eq!(cpu_rankings.len(), 1);
    }

    #[test]
    fn test_custom_config() {
        let config = RankingConfig {
            cpu_weight: 0.5,
            memory_weight: 0.3,
            io_weight: 0.1,
            network_weight: 0.05,
            gpu_weight: 0.05,
            ..Default::default()
        };
        let analyzer = ResourceRankingAnalyzer::with_config(config);

        assert_eq!(analyzer.config().cpu_weight, 0.5);
        assert_eq!(analyzer.config().memory_weight, 0.3);
    }

    #[test]
    fn test_clear() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        analyzer.analyze_task(1, "task1", "CpuIntensive", 60.0, 200.0, 50.0, 30.0, 40.0);
        analyzer.clear();

        assert!(analyzer.rankings.is_empty());
    }

    #[test]
    fn test_get_statistics() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        analyzer.analyze_task(1, "task1", "CpuIntensive", 60.0, 200.0, 50.0, 30.0, 40.0);
        analyzer.analyze_task(2, "task2", "CpuIntensive", 40.0, 100.0, 30.0, 20.0, 30.0);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_tasks, 2);
        assert!(stats.average_overall_score > 0.0);
    }

    #[test]
    fn test_recommendations_generation() {
        let mut analyzer = ResourceRankingAnalyzer::new();
        let ranking = analyzer.analyze_task(
            1,
            "test_task",
            "CpuIntensive",
            95.0,
            1200.0,
            600.0,
            600.0,
            95.0,
        );

        assert!(!ranking.recommendations.is_empty());
    }
}
