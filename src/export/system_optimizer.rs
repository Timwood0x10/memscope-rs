//! System optimizer module
//!
//! This module provides system resource detection, configuration optimization recommendations, and performance analysis tools.

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::FastExportConfigBuilder;
use crate::export::performance_testing::{OptimizationTarget, PerformanceTestResult};

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
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 优先级 (1-10)
    pub priority: u8,
    /// 标题
    pub title: String,
    /// Detailed description
    pub description: String,
    /// 预期效果
    pub expected_impact: String,
    /// 实施难度 (1-10)
    pub implementation_difficulty: u8,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// 配置调整
    ConfigurationTuning,
    /// 硬件升级
    HardwareUpgrade,
    /// 软件优化
    SoftwareOptimization,
    /// 环境调整
    EnvironmentTuning,
}

/// 系统优化器
pub struct SystemOptimizer {
    /// System resource information
    system_resources: SystemResources,
    /// 历史性能数据
    performance_history: Vec<PerformanceTestResult>,
    /// 配置验证规则
    validation_rules: ConfigurationValidationRules,
}

/// 配置验证规则
#[derive(Debug, Clone)]
pub struct ConfigurationValidationRules {
    /// 最小分片大小
    pub min_shard_size: usize,
    /// 最大分片大小
    pub max_shard_size: usize,
    /// 最小线程数
    pub min_thread_count: usize,
    /// 最大线程数
    pub max_thread_count: usize,
    /// 最小缓冲区大小
    pub min_buffer_size: usize,
    /// 最大缓冲区大小
    pub max_buffer_size: usize,
    /// 最大内存使用限制 (MB)
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

        // 基于系统资源和数据集大小优化
        let base_shard_size = match self.system_resources.system_type {
            SystemType::HighPerformanceServer => 5000,
            SystemType::DevelopmentWorkstation => 3000,
            SystemType::Desktop => 2000,
            SystemType::Laptop => 1500,
            SystemType::Embedded => 500,
            SystemType::Unknown => 1000,
        };

        // 根据数据集大小调整分片大小
        let shard_size = if dataset_size > 50000 {
            base_shard_size * 2
        } else if dataset_size > 20000 {
            (base_shard_size as f64 * 1.5) as usize
        } else {
            base_shard_size
        }
        .min(self.validation_rules.max_shard_size);

        reasoning.push(format!(
            "基于 {:?} 系统类型，推荐分片大小: {}",
            self.system_resources.system_type, shard_size
        ));

        // 线程数优化 - 充分利用 CPU 核心
        let thread_count = match self.system_resources.system_load {
            load if load < 0.5 => self.system_resources.cpu_cores,
            load if load < 1.0 => (self.system_resources.cpu_cores * 3 / 4).max(1),
            load if load < 2.0 => (self.system_resources.cpu_cores / 2).max(1),
            _ => (self.system_resources.cpu_cores / 4).max(1),
        }
        .min(self.validation_rules.max_thread_count);

        reasoning.push(format!(
            "基于系统负载 {:.2}，推荐线程数: {}",
            self.system_resources.system_load, thread_count
        ));

        // 缓冲区大小 - 大缓冲区提高 I/O 性能
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 8192 => 2 * 1024 * 1024, // 2MB
            mem if mem >= 4096 => 1024 * 1024,     // 1MB
            mem if mem >= 2048 => 512 * 1024,      // 512KB
            _ => 256 * 1024,                       // 256KB
        }
        .min(self.validation_rules.max_buffer_size);

        reasoning.push(format!(
            "基于可用内存 {} MB，推荐缓冲区大小: {} KB",
            self.system_resources.available_memory_mb,
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// Memory optimization
    fn optimize_for_memory(&self, _dataset_size: usize) -> (usize, usize, usize, Vec<String>) {
        let mut reasoning = Vec::new();

        // 小分片减少内存占用
        let shard_size = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 1000,
            mem if mem >= 2048 => 750,
            mem if mem >= 1024 => 500,
            _ => 250,
        }
        .max(self.validation_rules.min_shard_size);

        reasoning.push(format!("为节省内存，推荐较小分片大小: {}", shard_size));

        // 少线程减少并发内存使用
        let thread_count = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 4,
            mem if mem >= 2048 => 2,
            _ => 1,
        }
        .min(self.system_resources.cpu_cores / 2)
        .max(1);

        reasoning.push(format!("为控制内存使用，推荐线程数: {}", thread_count));

        // 小缓冲区节省内存
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 2048 => 256 * 1024, // 256KB
            mem if mem >= 1024 => 128 * 1024, // 128KB
            _ => 64 * 1024,                   // 64KB
        }
        .max(self.validation_rules.min_buffer_size);

        reasoning.push(format!(
            "为节省内存，推荐较小缓冲区: {} KB",
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// 平衡优化
    fn optimize_for_balance(&self, dataset_size: usize) -> (usize, usize, usize, Vec<String>) {
        let mut reasoning = Vec::new();

        // 平衡分片大小
        let shard_size = match (self.system_resources.cpu_cores, dataset_size) {
            (cores, size) if cores >= 8 && size > 20000 => 2000,
            (cores, size) if cores >= 4 && size > 10000 => 1500,
            (cores, _) if cores >= 2 => 1000,
            _ => 750,
        }
        .min(self.validation_rules.max_shard_size);

        reasoning.push(format!("平衡性能和内存，推荐分片大小: {}", shard_size));

        // 适中的线程数
        let thread_count = (self.system_resources.cpu_cores / 2)
            .max(2)
            .min(6)
            .min(self.validation_rules.max_thread_count);

        reasoning.push(format!(
            "平衡并行度和资源使用，推荐线程数: {}",
            thread_count
        ));

        // 中等缓冲区大小
        let buffer_size = match self.system_resources.available_memory_mb {
            mem if mem >= 4096 => 512 * 1024, // 512KB
            mem if mem >= 2048 => 256 * 1024, // 256KB
            _ => 128 * 1024,                  // 128KB
        };

        reasoning.push(format!(
            "平衡 I/O 性能和内存使用，推荐缓冲区: {} KB",
            buffer_size / 1024
        ));

        (shard_size, thread_count, buffer_size, reasoning)
    }

    /// 估算性能提升
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

        // 基于线程数的额外提升
        let thread_multiplier = (thread_count as f64).sqrt();

        // 基于分片大小的调整
        let shard_multiplier = if shard_size > 2000 {
            1.2
        } else if shard_size < 500 {
            0.8
        } else {
            1.0
        };

        base_gain * thread_multiplier * shard_multiplier
    }

    /// 估算内存使用
    fn estimate_memory_usage(
        &self,
        shard_size: usize,
        thread_count: usize,
        buffer_size: usize,
    ) -> f64 {
        // 基础内存使用
        let base_memory = 20.0; // 20MB 基础开销

        // 分片内存使用 (每个分配大约 500 字节)
        let shard_memory = (shard_size as f64 * 500.0 * thread_count as f64) / (1024.0 * 1024.0);

        // 缓冲区内存使用
        let buffer_memory = (buffer_size as f64 * thread_count as f64) / (1024.0 * 1024.0);

        base_memory + shard_memory + buffer_memory
    }

    /// 计算配置置信度
    fn calculate_confidence(&self, target: &OptimizationTarget) -> f64 {
        let mut confidence: f64 = 0.7; // 基础置信度

        // 基于系统类型调整置信度
        confidence += match self.system_resources.system_type {
            SystemType::HighPerformanceServer => 0.2,
            SystemType::DevelopmentWorkstation => 0.15,
            SystemType::Desktop => 0.1,
            SystemType::Laptop => 0.05,
            SystemType::Embedded => -0.1,
            SystemType::Unknown => -0.2,
        };

        // 基于历史数据调整置信度
        if !self.performance_history.is_empty() {
            confidence += 0.1;
        }

        // 基于优化目标调整置信度
        confidence += match target {
            OptimizationTarget::Balanced => 0.1,
            OptimizationTarget::Speed => 0.05,
            OptimizationTarget::Memory => 0.05,
        };

        confidence.min(1.0).max(0.0)
    }

    /// 验证配置
    pub fn validate_configuration(
        &self,
        _config: &FastExportConfigBuilder,
    ) -> ConfigurationValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // 这里需要从 FastExportConfigBuilder 中提取配置值
        // 由于 FastExportConfigBuilder 可能没有公开的 getter 方法，
        // 我们需要创建一个配置并检查其值

        // 验证分片大小
        let shard_size = 1000; // 默认值，实际应该从配置中获取
        if shard_size < self.validation_rules.min_shard_size {
            errors.push(format!(
                "分片大小 {} 小于最小值 {}",
                shard_size, self.validation_rules.min_shard_size
            ));
        } else if shard_size > self.validation_rules.max_shard_size {
            errors.push(format!(
                "分片大小 {} 超过最大值 {}",
                shard_size, self.validation_rules.max_shard_size
            ));
        }

        // 验证线程数
        let thread_count = num_cpus::get(); // 默认值
        if thread_count > self.system_resources.cpu_cores * 2 {
            warnings.push(format!(
                "线程数 {} 超过 CPU 核心数的两倍 ({}), 可能导致上下文切换开销",
                thread_count, self.system_resources.cpu_cores
            ));
        }

        // 验证内存使用
        let estimated_memory = self.estimate_memory_usage(shard_size, thread_count, 256 * 1024);
        if estimated_memory > self.system_resources.available_memory_mb as f64 * 0.8 {
            errors.push(format!(
                "预估内存使用 {:.1} MB 超过可用内存的 80% ({:.1} MB)",
                estimated_memory,
                self.system_resources.available_memory_mb as f64 * 0.8
            ));
        }

        // 生成优化建议
        if shard_size < 500 && self.system_resources.cpu_cores >= 4 {
            suggestions.push("考虑增加分片大小以提高并行效率".to_string());
        }

        if thread_count == 1 && self.system_resources.cpu_cores > 2 {
            suggestions.push("考虑启用多线程处理以提高性能".to_string());
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

    /// 估算配置影响
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

    /// 诊断性能问题
    pub fn diagnose_performance(&self) -> PerformanceDiagnosis {
        let system_status = self.get_system_resource_status();
        let bottlenecks = self.identify_bottlenecks(&system_status);
        let optimization_suggestions = self.generate_optimization_suggestions(&bottlenecks);
        let health_score = self.calculate_health_score(&system_status, &bottlenecks);

        PerformanceDiagnosis {
            diagnosis_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            system_status,
            bottlenecks,
            optimization_suggestions,
            health_score,
        }
    }

    /// 获取系统资源状态
    fn get_system_resource_status(&self) -> SystemResourceStatus {
        let cpu_usage = self.get_cpu_usage();
        let memory_usage =
            (self.system_resources.available_memory_mb as f64 / 8192.0 * 100.0).min(100.0);
        let disk_usage = 50.0; // 简化实现

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

    /// 获取 CPU 使用率
    fn get_cpu_usage(&self) -> f64 {
        // 简化实现 - 基于系统负载估算
        (self.system_resources.system_load / self.system_resources.cpu_cores as f64 * 100.0)
            .min(100.0)
    }

    /// 识别性能瓶颈
    fn identify_bottlenecks(&self, status: &SystemResourceStatus) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // CPU 瓶颈检测
        if status.cpu_usage_percent > 80.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Cpu,
                severity: if status.cpu_usage_percent > 95.0 {
                    9
                } else {
                    7
                },
                description: format!("CPU 使用率过高: {:.1}%", status.cpu_usage_percent),
                impact: "导出性能显著下降，响应时间增加".to_string(),
                suggested_solutions: vec![
                    "减少并行线程数".to_string(),
                    "增加分片大小以减少线程切换开销".to_string(),
                    "考虑在系统负载较低时进行导出".to_string(),
                ],
            });
        }

        // 内存瓶颈检测
        if status.memory_usage_percent > 85.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Memory,
                severity: if status.memory_usage_percent > 95.0 {
                    10
                } else {
                    8
                },
                description: format!("内存使用率过高: {:.1}%", status.memory_usage_percent),
                impact: "可能导致内存不足，系统变慢或崩溃".to_string(),
                suggested_solutions: vec![
                    "减少分片大小".to_string(),
                    "减少并行线程数".to_string(),
                    "减少缓冲区大小".to_string(),
                    "启用流式处理模式".to_string(),
                ],
            });
        }

        // 配置瓶颈检测
        if self.system_resources.cpu_cores >= 8 && status.cpu_usage_percent < 30.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::Configuration,
                severity: 5,
                description: "CPU 资源未充分利用".to_string(),
                impact: "导出性能未达到最优".to_string(),
                suggested_solutions: vec![
                    "增加并行线程数".to_string(),
                    "减少分片大小以增加并行度".to_string(),
                    "启用速度优化模式".to_string(),
                ],
            });
        }

        bottlenecks
    }

    /// 生成优化建议
    fn generate_optimization_suggestions(
        &self,
        bottlenecks: &[PerformanceBottleneck],
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // 基于瓶颈生成建议
        for bottleneck in bottlenecks {
            match bottleneck.bottleneck_type {
                BottleneckType::Cpu => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "优化 CPU 使用".to_string(),
                        description: "调整并行配置以优化 CPU 使用率".to_string(),
                        expected_impact: "提高导出速度 20-40%".to_string(),
                        implementation_difficulty: 3,
                    });
                }
                BottleneckType::Memory => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "优化内存使用".to_string(),
                        description: "调整分片和缓冲区配置以减少内存使用".to_string(),
                        expected_impact: "减少内存使用 30-50%".to_string(),
                        implementation_difficulty: 2,
                    });
                }
                BottleneckType::Configuration => {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: SuggestionType::ConfigurationTuning,
                        priority: bottleneck.severity,
                        title: "优化配置参数".to_string(),
                        description: "调整配置以充分利用系统资源".to_string(),
                        expected_impact: "提高整体性能 15-30%".to_string(),
                        implementation_difficulty: 1,
                    });
                }
                _ => {}
            }
        }

        // 通用优化建议
        if self.system_resources.system_type == SystemType::HighPerformanceServer {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::ConfigurationTuning,
                priority: 6,
                title: "启用高性能模式".to_string(),
                description: "在高性能服务器上启用最大性能配置".to_string(),
                expected_impact: "提高导出速度 50-80%".to_string(),
                implementation_difficulty: 2,
            });
        }

        suggestions
    }

    /// 计算健康评分
    fn calculate_health_score(
        &self,
        status: &SystemResourceStatus,
        bottlenecks: &[PerformanceBottleneck],
    ) -> u8 {
        let mut score = 100u8;

        // 基于资源使用率扣分
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

        // 基于瓶颈扣分
        for bottleneck in bottlenecks {
            score = score.saturating_sub(bottleneck.severity * 2);
        }

        score.max(10) // 最低分数 10
    }

    /// 添加性能历史数据
    pub fn add_performance_data(&mut self, result: PerformanceTestResult) {
        self.performance_history.push(result);

        // 保持历史数据在合理范围内
        if self.performance_history.len() > 100 {
            self.performance_history.remove(0);
        }
    }

    /// 获取系统资源信息
    pub fn get_system_resources(&self) -> &SystemResources {
        &self.system_resources
    }

    /// 更新系统资源信息
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

/// 配置验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationValidationResult {
    /// 配置是否有效
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<String>,
    /// 警告列表
    pub warnings: Vec<String>,
    /// 建议列表
    pub suggestions: Vec<String>,
    /// 预估性能影响
    pub estimated_performance_impact: ConfigurationImpact,
}

/// 配置影响
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationImpact {
    /// 性能评分 (1-10)
    pub performance_score: u8,
    /// 内存效率 (1-10)
    pub memory_efficiency: u8,
    /// 稳定性评分 (1-10)
    pub stability_score: u8,
    /// 总体评分 (1-10)
    pub overall_score: u8,
}
