//! System optimizer module
//!
//! This module provides system resource detection, configuration optimization recommendations, and performance analysis tools.

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::FastExportConfigBuilder;
// use crate::export::performance_testing::{OptimizationTarget, PerformanceTestResult}; // Removed - using local definitions
use crate::export::config_optimizer::OptimizationTarget;

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub duration_ms: u64,
    pub memory_usage_mb: f64,
    pub success: bool,
}

use serde::{Deserialize, Serialize};

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Available memory (MB)
    pub available_memory_mb: usize,
    /// System load
    pub system_load: f64,
    /// Available disk space (MB)
    pub disk_space_mb: usize,
    /// System type
    pub system_type: SystemType,
}

/// System type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemType {
    /// High performance server
    HighPerformanceServer,
    /// Development workstation
    DevelopmentWorkstation,
    /// Desktop
    Desktop,
    /// Laptop
    Laptop,
    /// Embedded system
    Embedded,
    /// Unknown system
    Unknown,
}

/// Configuration recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRecommendation {
    /// Recommended shard size
    pub recommended_shard_size: usize,
    /// Recommended thread count
    pub recommended_thread_count: usize,
    /// Recommended buffer size
    pub recommended_buffer_size: usize,
    /// Optimization target
    pub optimization_target: OptimizationTarget,
    /// Expected performance gain
    pub expected_performance_gain: f64,
    /// Expected memory usage
    pub expected_memory_usage_mb: f64,
    /// Reasoning
    pub reasoning: Vec<String>,
    /// Configuration confidence (0.0-1.0)
    pub confidence: f64,
}

/// Performance diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDiagnosis {
    /// Diagnosis time (Unix timestamp)
    pub diagnosis_time: u64,
    /// System resource status
    pub system_status: SystemResourceStatus,
    /// Performance bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    /// Overall health score (0-100)
    pub health_score: u8,
}

/// System resource status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceStatus {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// System load status
    pub load_status: LoadStatus,
}

/// Load status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadStatus {
    /// Low load
    Low,
    /// Medium load
    Medium,
    /// High load
    High,
    /// Overloaded
    Overloaded,
}

/// Performance bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck type
    pub bottleneck_type: BottleneckType,
    /// Severity level (1-10)
    pub severity: u8,
    /// Description
    pub description: String,
    /// Impact
    pub impact: String,
    /// Suggested solutions
    pub suggested_solutions: Vec<String>,
}

/// Bottleneck type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU bottleneck
    Cpu,
    /// Memory bottleneck
    Memory,
    /// I/O bottleneck
    Io,
    /// Network bottleneck
    Network,
    /// Configuration bottleneck
    Configuration,
}

/// Optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// suggestion type
    pub suggestion_type: SuggestionType,
    /// priority (1-10)
    pub priority: u8,
    /// title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// expected impact
    pub expected_impact: String,
    /// implementation difficulty (1-10)
    pub implementation_difficulty: u8,
}

/// Suggestion type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// configuration tuning
    ConfigurationTuning,
    /// hardware upgrade
    HardwareUpgrade,
    /// software optimization
    SoftwareOptimization,
    /// environment tuning
    EnvironmentTuning,
}

/// System optimizer
pub struct SystemOptimizer {
    /// System resource information
    system_resources: SystemResources,
    /// performance history
    performance_history: Vec<PerformanceTestResult>,
    /// configuration validation rules
    validation_rules: ConfigurationValidationRules,
}

/// Configuration validation rules
#[derive(Debug, Clone)]
pub struct ConfigurationValidationRules {
    /// minimum shard size
    pub min_shard_size: usize,
    /// maximum shard size
    pub max_shard_size: usize,
    /// minimum thread count
    pub min_thread_count: usize,
    /// maximum thread count
    pub max_thread_count: usize,
    /// minimum buffer size
    pub min_buffer_size: usize,
    /// maximum buffer size
    pub max_buffer_size: usize,
    /// maximum memory limit (MB)
    pub max_memory_limit_mb: usize,
}

impl Default for ConfigurationValidationRules {
    fn default() -> Self {
        Self {
            min_shard_size: 100,
            max_shard_size: 10000,
            min_thread_count: 1,
            max_thread_count: 32,
            min_buffer_size: 16 * 1024,        // 16KB
            max_buffer_size: 16 * 1024 * 1024, // 16MB
            max_memory_limit_mb: 512,
        }
    }
}

impl SystemOptimizer {
    /// Create new system optimizer
    pub fn new() -> TrackingResult<Self> {
        let system_resources = Self::detect_system_resources()?;

        Ok(Self {
            system_resources,
            performance_history: Vec::new(),
            validation_rules: ConfigurationValidationRules::default(),
        })
    }

    /// Detect system resources
    pub fn detect_system_resources() -> TrackingResult<SystemResources> {
        let cpu_cores = num_cpus::get();
        let available_memory_mb = Self::get_available_memory_mb();
        let system_load = Self::get_system_load();
        let disk_space_mb = Self::get_disk_space_mb();
        let system_type = Self::classify_system_type(cpu_cores, available_memory_mb);

        Ok(SystemResources {
            cpu_cores,
            available_memory_mb,
            system_load,
            disk_space_mb,
            system_type,
        })
    }

    /// Get available memory (MB)
    fn get_available_memory_mb() -> usize {
        // 简化实现 - 在实际应用中可以使用 sysinfo 等库获取准确信息
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb / 1024; // 转换为 MB
                            }
                        }
                    }
                }
            }
        }

        // 回退到估算值
        4096 // 假设 4GB 可用内存
    }

    /// Get system load
    fn get_system_load() -> f64 {
        // 简化实现 - 在实际应用中可以读取 /proc/loadavg
        #[cfg(target_os = "linux")]
        {
            if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
                if let Some(load_str) = loadavg.split_whitespace().next() {
                    if let Ok(load) = load_str.parse::<f64>() {
                        return load;
                    }
                }
            }
        }

        0.5 // 默认低负载
    }

    /// Get available disk space (MB)
    fn get_disk_space_mb() -> usize {
        // 简化实现 - 在实际应用中可以使用 statvfs 系统调用
        10240 // 假设 10GB 可用空间
    }

    /// Classify system type
    fn classify_system_type(cpu_cores: usize, memory_mb: usize) -> SystemType {
        match (cpu_cores, memory_mb) {
            (cores, mem) if cores >= 16 && mem >= 32768 => SystemType::HighPerformanceServer,
            (cores, mem) if cores >= 8 && mem >= 16384 => SystemType::DevelopmentWorkstation,
            (cores, mem) if cores >= 4 && mem >= 8192 => SystemType::Desktop,
            (cores, mem) if cores >= 2 && mem >= 4096 => SystemType::Laptop,
            (cores, mem) if cores >= 1 && mem >= 1024 => SystemType::Embedded,
            _ => SystemType::Unknown,
        }
    }

    /// Generate configuration recommendations
    pub fn generate_configuration_recommendation(
        &self,
        target: OptimizationTarget,
        dataset_size: Option<usize>,
    ) -> ConfigurationRecommendation {
        let dataset_size = dataset_size.unwrap_or(10000);

        let (shard_size, thread_count, buffer_size, reasoning) = match target {
            OptimizationTarget::Speed => self.optimize_for_speed(dataset_size),
            OptimizationTarget::Memory => self.optimize_for_memory(dataset_size),
            OptimizationTarget::Balanced => self.optimize_for_balance(dataset_size),
        };

        let expected_performance_gain =
            self.estimate_performance_gain(&target, shard_size, thread_count);
        let expected_memory_usage =
            self.estimate_memory_usage(shard_size, thread_count, buffer_size);
        let confidence = self.calculate_confidence(&target);

        ConfigurationRecommendation {
            recommended_shard_size: shard_size,
            recommended_thread_count: thread_count,
            recommended_buffer_size: buffer_size,
            optimization_target: target,
            expected_performance_gain,
            expected_memory_usage_mb: expected_memory_usage,
            reasoning,
            confidence,
        }
    }

    /// Speed optimization
    fn optimize_for_speed(&self, dataset_size: usize) -> (usize, usize, usize, Vec<String>) {
        let mut reasoning = Vec::new();

        // based on system resources and dataset size
        let base_shard_size = match self.system_resources.system_type {
            SystemType::HighPerformanceServer => 5000,
            SystemType::DevelopmentWorkstation => 3000,
            SystemType::Desktop => 2000,
            SystemType::Laptop => 1500,
            SystemType::Embedded => 500,
            SystemType::Unknown => 1000,
        };

        // based on dataset size
        let shard_size = if dataset_size > 50000 {
            base_shard_size * 2
        } else if dataset_size > 20000 {
            (base_shard_size as f64 * 1.5) as usize
        } else {
            base_shard_size
        }
        .min(self.validation_rules.max_shard_size);

        reasoning.push(format!(
            "basic {:?} system type, recommended shard size: {}",
            self.system_resources.system_type, shard_size
        ));

        // based on system load
        let thread_count = match self.system_resources.system_load {
            load if load < 0.5 => self.system_resources.cpu_cores,
            load if load < 1.0 => (self.system_resources.cpu_cores * 3 / 4).max(1),
            load if load < 2.0 => (self.system_resources.cpu_cores / 2).max(1),
            _ => (self.system_resources.cpu_cores / 4).max(1),
        }
        .min(self.validation_rules.max_thread_count);

        reasoning.push(format!(
            "basic system load {:.2}, recommended thread count: {}",
            self.system_resources.system_load, thread_count
        ));

        // buffer size - large buffer size for better I/O performance
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 8192 => 2 * 1024 * 1024, // 2MB
            mem if mem >= 4096 => 1024 * 1024,     // 1MB
            mem if mem >= 2048 => 512 * 1024,      // 512KB
            _ => 256 * 1024,                       // 256KB
        }
        .min(self.validation_rules.max_buffer_size);

        reasoning.push(format!(
            "basic available memory {} MB, recommended buffer size: {} KB",
            self.system_resources.available_memory_mb,
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// Memory optimization
    fn optimize_for_memory(&self, _dataset_size: usize) -> (usize, usize, usize, Vec<String>) {
        let mut reasoning = Vec::new();

        // small shard size to save memory
        let shard_size = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 1000,
            mem if mem >= 2048 => 750,
            mem if mem >= 1024 => 500,
            _ => 250,
        }
        .max(self.validation_rules.min_shard_size);

        reasoning.push(format!("basic available memory {} MB, recommended shard size: {shard_size}", self.system_resources.available_memory_mb));

        // less thread count to reduce concurrent memory usage
        let thread_count = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 4,
            mem if mem >= 2048 => 2,
            _ => 1,
        }
        .min(self.system_resources.cpu_cores / 2)
        .max(1);

        reasoning.push(format!("basic available memory {} MB, recommended thread count: {thread_count}", self.system_resources.available_memory_mb));

        // small buffer size to save memory
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 2048 => 256 * 1024, // 256KB
            mem if mem >= 1024 => 128 * 1024, // 128KB
            _ => 64 * 1024,                   // 64KB
        }
        .max(self.validation_rules.min_buffer_size);

        reasoning.push(format!(
            "basic available memory {} MB, recommended buffer size: {} KB",
            self.system_resources.available_memory_mb,
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// balance optimization
    fn optimize_for_balance(&self, dataset_size: usize) -> (usize, usize, usize, Vec<String>) {
        let mut reasoning = Vec::new();

        // balanced shard size
        let shard_size = match (self.system_resources.cpu_cores, dataset_size) {
            (cores, size) if cores >= 8 && size > 20000 => 2000,
            (cores, size) if cores >= 4 && size > 10000 => 1500,
            (cores, _) if cores >= 2 => 1000,
            _ => 750,
        }
        .min(self.validation_rules.max_shard_size);

        reasoning.push(format!("basic cpu cores {}, dataset size {dataset_size}, recommended shard size: {shard_size}", self.system_resources.cpu_cores));

        // balanced thread count
        let thread_count = (self.system_resources.cpu_cores / 2)
            .max(2)
            .min(6)
            .min(self.validation_rules.max_thread_count);

        reasoning.push(format!(
            "basic cpu cores {}, dataset size {dataset_size}, recommended thread count: {thread_count}",
            self.system_resources.cpu_cores,
        ));

        // balanced buffer size
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 512 * 1024, // 512KB
            mem if mem >= 2048 => 256 * 1024, // 256KB
            _ => 128 * 1024,                  // 128KB
        };

        reasoning.push(format!(
            "basic available memory {} MB, recommended buffer size: {} KB",
            self.system_resources.available_memory_mb,
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// estimate performance gain
    fn estimate_performance_gain(
        &self,
        target: &OptimizationTarget,
        shard_size: usize,
        thread_count: usize,
    ) -> f64 {
        let base_gain = match target {
            OptimizationTarget::Speed => 3.0,
            OptimizationTarget::Memory => 1.5,
            OptimizationTarget::Balanced => 2.0,
        };

        // based on thread count
        let thread_multiplier = (thread_count as f64).sqrt();

        // based on shard size
        let shard_multiplier = if shard_size > 2000 {
            1.2
        } else if shard_size < 500 {
            0.8
        } else {
            1.0
        };

        base_gain * thread_multiplier * shard_multiplier
    }

    /// estimate memory usage
    fn estimate_memory_usage(
        &self,
        shard_size: usize,
        thread_count: usize,
        buffer_size: usize,
    ) -> f64 {
        // base memory usage
        let base_memory = 20.0; // 20MB base overhead

        // shard memory usage (each allocation is about 500 bytes)
        let shard_memory = (shard_size as f64 * 500.0 * thread_count as f64) / (1024.0 * 1024.0);

        // buffer memory usage
        let buffer_memory = (buffer_size as f64 * thread_count as f64) / (1024.0 * 1024.0);

        base_memory + shard_memory + buffer_memory
    }

    /// calculate confidence
    fn calculate_confidence(&self, target: &OptimizationTarget) -> f64 {
        let mut confidence: f64 = 0.7; // base confidence

        // based on system type adjustment
        confidence += match self.system_resources.system_type {
            SystemType::HighPerformanceServer => 0.2,
            SystemType::DevelopmentWorkstation => 0.15,
            SystemType::Desktop => 0.1,
            SystemType::Laptop => 0.05,
            SystemType::Embedded => -0.1,
            SystemType::Unknown => -0.2,
        };

        // based on history data adjustment
        if !self.performance_history.is_empty() {
            confidence += 0.1;
        }

        // based on optimization target adjustment
        confidence += match target {
            OptimizationTarget::Balanced => 0.1,
            OptimizationTarget::Speed => 0.05,
            OptimizationTarget::Memory => 0.05,
        };

        confidence.clamp(0.0, 1.0)
    }

    /// validate configuration
    pub fn validate_configuration(
        &self,
        _config: &FastExportConfigBuilder,
    ) -> ConfigurationValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // validate shard size
        let shard_size = 1000; // default value, should be from config
        if shard_size < self.validation_rules.min_shard_size {
            errors.push(format!(
                "shard size {} is less than minimum value {}",
                shard_size, self.validation_rules.min_shard_size
            ));
        } else if shard_size > self.validation_rules.max_shard_size {
            errors.push(format!(
                "shard size {} is greater than maximum value {}",
                shard_size, self.validation_rules.max_shard_size
            ));
        }

        // validate thread count
        let thread_count = num_cpus::get(); // default value, should be from config
        if thread_count > self.system_resources.cpu_cores * 2 {
            warnings.push(format!(
                "thread count {thread_count} exceeds twice the number of CPU cores ({}), may cause context switch overhead",
                self.system_resources.cpu_cores
            ));
        }

        // validate memory usage
        let estimated_memory = self.estimate_memory_usage(shard_size, thread_count, 256 * 1024);
        if estimated_memory > self.system_resources.available_memory_mb as f64 * 0.8 {
            errors.push(format!(
                "estimated memory usage {:.1} MB exceeds 80% of available memory ({:.1} MB)",
                estimated_memory,
                self.system_resources.available_memory_mb as f64 * 0.8
            ));
        }

        // generate optimization suggestions
        if shard_size < 500 && self.system_resources.cpu_cores >= 4 {
            suggestions.push("consider increasing shard size for better parallelism".to_string());
        }

        if thread_count == 1 && self.system_resources.cpu_cores > 2 {
            suggestions.push("consider enabling multi-threading for better performance".to_string());
        }

        ConfigurationValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
            estimated_performance_impact: self
                .estimate_configuration_impact(shard_size, thread_count),
        }
    }

    /// estimate configuration impact
    fn estimate_configuration_impact(
        &self,
        shard_size: usize,
        thread_count: usize,
    ) -> ConfigurationImpact {
        let performance_score = match (shard_size, thread_count) {
            (s, t) if s >= 2000 && t >= 4 => 9,
            (s, t) if s >= 1000 && t >= 2 => 7,
            (s, t) if s >= 500 && t >= 1 => 5,
            _ => 3,
        };

        let memory_efficiency = if shard_size <= 1000 && thread_count <= 4 {
            8
        } else {
            6
        };
        let stability_score = if thread_count <= self.system_resources.cpu_cores {
            9
        } else {
            6
        };

        ConfigurationImpact {
            performance_score,
            memory_efficiency,
            stability_score,
            overall_score: (performance_score + memory_efficiency + stability_score) / 3,
        }
    }

    /// diagnose performance issues
    pub fn diagnose_performance(&self) -> PerformanceDiagnosis {
        let system_status = self.get_system_resource_status();
        let bottlenecks = self.identify_bottlenecks(&system_status);
        let optimization_suggestions = self.generate_optimization_suggestions(&bottlenecks);
        let health_score = self.calculate_health_score(&system_status, &bottlenecks);

        PerformanceDiagnosis {
            diagnosis_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            system_status,
            bottlenecks,
            optimization_suggestions,
            health_score,
        }
    }

    /// get system resource status
    fn get_system_resource_status(&self) -> SystemResourceStatus {
        let cpu_usage = self.get_cpu_usage();
        let memory_usage =
            (self.system_resources.available_memory_mb as f64 / 8192.0 * 100.0).min(100.0);
        let disk_usage = 50.0; // simplified implementation

        let load_status = match self.system_resources.system_load {
            load if load < 1.0 => LoadStatus::Low,
            load if load < 2.0 => LoadStatus::Medium,
            load if load < 4.0 => LoadStatus::High,
            _ => LoadStatus::Overloaded,
        };

        SystemResourceStatus {
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            disk_usage_percent: disk_usage,
            load_status,
        }
    }

    /// get CPU usage percentage
    fn get_cpu_usage(&self) -> f64 {
        // simplified implementation - based on system load estimation
        (self.system_resources.system_load / self.system_resources.cpu_cores as f64 * 100.0)
            .min(100.0)
    }

    /// identify performance bottlenecks
    fn identify_bottlenecks(&self, status: &SystemResourceStatus) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // CPU bottleneck detection
        if status.cpu_usage_percent > 80.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Cpu,
                severity: if status.cpu_usage_percent > 95.0 {
                    9
                } else {
                    7
                },
                description: format!("CPU usage is high: {:.1}%", status.cpu_usage_percent),
                impact: "Export performance significantly degraded, response time increased".to_string(),
                suggested_solutions: vec![
                    "reduce parallel thread count".to_string(),
                    "increase shard size to reduce thread switch overhead".to_string(),
                    "consider exporting when system load is low".to_string(),
                ],
            });
        }

        // memory bottleneck detection
        if status.memory_usage_percent > 85.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Memory,
                severity: if status.memory_usage_percent > 95.0 {
                    10
                } else {
                    8
                },
                description: format!("Memory usage is high: {:.1}%", status.memory_usage_percent),
                impact: "possible memory underutilization, system may slow down or crash".to_string(),
                suggested_solutions: vec![
                    "reduce shard size".to_string(),
                    "reduce parallel thread count".to_string(),
                    "reduce buffer size".to_string(),
                    "enable streaming processing mode".to_string(),
                ],
            });
        }

        // configuration bottleneck detection
        if self.system_resources.cpu_cores >= 8 && status.cpu_usage_percent < 30.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Configuration,
                severity: 5,
                description: "CPU resource underutilized".to_string(),
                impact: "Export performance suboptimal".to_string(),
                suggested_solutions: vec![
                    "add more threads".to_string(),
                    "reduce shard size to increase parallelism".to_string(),
                    "enable speed optimization mode".to_string(),
                ],
            });
        }

        bottlenecks
    }

    /// generate optimization suggestions
    fn generate_optimization_suggestions(
        &self,
        bottlenecks: &[PerformanceBottleneck],
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // based on bottleneck generate suggestions
        for bottleneck in bottlenecks {
            match bottleneck.bottleneck_type {
                BottleneckType::Cpu => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "Optimize CPU usage".to_string(),
                        description: "Adjust parallel configuration to optimize CPU usage".to_string(),
                        expected_impact: "Increase export speed by 20-40%".to_string(),
                        implementation_difficulty: 3,
                    });
                }
                BottleneckType::Memory => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "Optimize memory usage".to_string(),
                        description: "Adjust shard size and buffer size to reduce memory usage".to_string(),
                        expected_impact: "Reduce memory usage by 30-50%".to_string(),
                        implementation_difficulty: 2,
                    });
                }
                BottleneckType::Configuration => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "Optimize configuration parameters".to_string(),
                        description: "Adjust configuration to fully utilize system resources".to_string(),
                        expected_impact: "Increase overall performance by 15-30%".to_string(),
                        implementation_difficulty: 1,
                    });
                }
                _ => {}
            }
        }

        // general optimization suggestions
        if self.system_resources.system_type == SystemType::HighPerformanceServer {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::ConfigurationTuning,
                priority: 6,
                title: "Enable high performance mode".to_string(),
                description: "Enable maximum performance configuration on high performance servers".to_string(),
                expected_impact: "Increase export speed by 50-80%".to_string(),
                implementation_difficulty: 2,
            });
        }

        suggestions
    }

    /// calculate health score
    fn calculate_health_score(
        &self,
        status: &SystemResourceStatus,
        bottlenecks: &[PerformanceBottleneck],
    ) -> u8 {
        let mut score = 100u8;

        // based on resource usage rate to score
        if status.cpu_usage_percent > 80.0 {
            score = score.saturating_sub(20);
        } else if status.cpu_usage_percent > 60.0 {
            score = score.saturating_sub(10);
        }

        if status.memory_usage_percent > 85.0 {
            score = score.saturating_sub(25);
        } else if status.memory_usage_percent > 70.0 {
            score = score.saturating_sub(15);
        }

        // based on bottleneck to score
        for bottleneck in bottlenecks {
            score = score.saturating_sub(bottleneck.severity * 2);
        }

        score.max(10) // minimum score is 10
    }

    /// add performance history data
    pub fn add_performance_data(&mut self, result: PerformanceTestResult) {
        self.performance_history.push(result);

        // keep history data within a reasonable range
        if self.performance_history.len() > 100 {
            self.performance_history.remove(0);
        }
    }

    /// get system resources
    pub fn get_system_resources(&self) -> &SystemResources {
        &self.system_resources
    }

    /// update system resources
    pub fn refresh_system_resources(&mut self) -> TrackingResult<()> {
        self.system_resources = Self::detect_system_resources()?;
        Ok(())
    }
}

impl Default for SystemOptimizer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            system_resources: SystemResources {
                cpu_cores: num_cpus::get(),
                available_memory_mb: 4096,
                system_load: 0.5,
                disk_space_mb: 10240,
                system_type: SystemType::Unknown,
            },
            performance_history: Vec::new(),
            validation_rules: ConfigurationValidationRules::default(),
        })
    }
}

/// configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationValidationResult {
    /// configuration is valid
    pub is_valid: bool,
    /// error list
    pub errors: Vec<String>,
    /// warning list
    pub warnings: Vec<String>,
    /// suggestion list
    pub suggestions: Vec<String>,
    /// estimated performance impact
    pub estimated_performance_impact: ConfigurationImpact,
}

/// configuration impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationImpact {
    /// performance score (1-10)
    pub performance_score: u8,
    /// memory efficiency (1-10)
    pub memory_efficiency: u8,
    /// stability score (1-10)
    pub stability_score: u8,
    /// overall score (1-10)
    pub overall_score: u8,
}
