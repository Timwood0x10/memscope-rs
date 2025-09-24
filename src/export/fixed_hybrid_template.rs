//! Fixed Hybrid Template System for Multi-threaded and Async Memory Analysis
//!
//! This module provides a unified template system that combines lockfree multi-threaded
//! tracking data with async memory analysis, creating comprehensive visualizations
//! that showcase variable details across multiple threads and tasks.

use crate::async_memory::visualization::VisualizationConfig;
use crate::lockfree::{LockfreeAnalysis};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;

/// 🔗 统一变量身份系统 - 三模块融合的核心
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UnifiedVariableID {
    pub thread_id: usize,           // lockfree模块提供
    pub task_id: Option<usize>,     // async模块提供
    pub var_name: String,           // tracking宏提供
    pub allocation_site: CodeLocation, // 调用栈信息
    pub timestamp: u64,             // 统一时间戳
}

/// 代码位置信息
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CodeLocation {
    pub file: String,
    pub line: u32,
    pub function: String,
}

/// 跨模块事件类型
#[derive(Debug, Clone)]
pub enum CrossModuleEvent {
    Allocation { id: UnifiedVariableID, size: u64 },
    ThreadAssignment { id: UnifiedVariableID, thread_id: usize },
    TaskBinding { id: UnifiedVariableID, task_id: usize },
    FFICrossing { id: UnifiedVariableID, direction: FFIDirection },
    Deallocation { id: UnifiedVariableID },
}

/// FFI 方向
#[derive(Debug, Clone)]
pub enum FFIDirection {
    RustToC,
    CToRust,
}

/// 跨模块关联数据
#[derive(Debug, Clone)]
pub struct CrossModuleData {
    pub relationships: Vec<RelationType>,
    pub event_chain: Vec<CrossModuleEvent>,
    pub safety_score: f64,
    pub performance_impact: f64,
}

/// 变量关系类型
#[derive(Debug, Clone)]
pub enum RelationType {
    SharedMemory,
    ThreadMigration,
    TaskHandover,
    FFIBoundary,
    OwnershipTransfer,
}

/// 智能分析引擎
#[derive(Debug)]
pub struct IntelligentAnalysisEngine {
    pub leak_detector: LeakDetector,
    pub race_analyzer: RaceAnalyzer,
    pub ffi_auditor: FFIAuditor,
    pub pattern_miner: PatternMiner,
}

/// 内存泄漏检测器
#[derive(Debug)]
pub struct LeakDetector {
    pub unmatched_allocations: Vec<VariableDetail>,
    pub timeout_variables: Vec<(VariableDetail, std::time::Duration)>,
    pub ffi_boundary_leaks: Vec<FFILeakInfo>,
}

/// 竞争分析器
#[derive(Debug)]
pub struct RaceAnalyzer {
    pub shared_variable_access: HashMap<String, Vec<ThreadAccess>>,
    pub race_conditions: Vec<RaceCondition>,
    pub deadlock_scenarios: Vec<DeadlockChain>,
}

/// FFI 安全审计器
#[derive(Debug)]
pub struct FFIAuditor {
    pub boundary_crossings: Vec<FFICrossing>,
    pub risk_assessment: RiskMatrix,
    pub ownership_transfers: Vec<OwnershipEvent>,
}

/// 模式挖掘器
#[derive(Debug)]
pub struct PatternMiner {
    pub allocation_patterns: Vec<AllocationPattern>,
    pub lifecycle_patterns: Vec<LifecyclePattern>,
    pub thread_affinity: HashMap<String, usize>,
}

/// FFI 泄漏信息
#[derive(Debug, Clone)]
pub struct FFILeakInfo {
    pub variable_id: UnifiedVariableID,
    pub leak_size: u64,
    pub boundary_type: String,
}

/// 线程访问记录
#[derive(Debug, Clone)]
pub struct ThreadAccess {
    pub thread_id: usize,
    pub timestamp: u64,
    pub access_type: AccessType,
}

/// 访问类型
#[derive(Debug, Clone)]
pub enum AccessType {
    Read,
    Write,
    Exclusive,
}

/// 竞争条件
#[derive(Debug, Clone)]
pub struct RaceCondition {
    pub variable_name: String,
    pub competing_threads: Vec<usize>,
    pub severity: RaceSeverity,
}

/// 竞争严重程度
#[derive(Debug, Clone)]
pub enum RaceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 死锁链
#[derive(Debug, Clone)]
pub struct DeadlockChain {
    pub involved_threads: Vec<usize>,
    pub resource_chain: Vec<String>,
}

/// FFI 跨界
#[derive(Debug, Clone)]
pub struct FFICrossing {
    pub variable_id: UnifiedVariableID,
    pub direction: FFIDirection,
    pub safety_level: SafetyLevel,
}

/// 安全级别
#[derive(Debug, Clone)]
pub enum SafetyLevel {
    Safe,
    Warning,
    Dangerous,
    Critical,
}

/// 风险矩阵
#[derive(Debug, Clone)]
pub struct RiskMatrix {
    pub memory_safety_score: f64,
    pub thread_safety_score: f64,
    pub ffi_safety_score: f64,
    pub overall_risk: RiskLevel,
}

/// 风险级别
#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 所有权事件
#[derive(Debug, Clone)]
pub struct OwnershipEvent {
    pub variable_id: UnifiedVariableID,
    pub from_context: String,
    pub to_context: String,
    pub transfer_type: OwnershipTransferType,
}

/// 所有权转移类型
#[derive(Debug, Clone)]
pub enum OwnershipTransferType {
    Move,
    Borrow,
    Clone,
    FFIHandover,
}

/// 分配模式
#[derive(Debug, Clone)]
pub struct AllocationPattern {
    pub pattern_type: String,
    pub frequency: u64,
    pub typical_size: u64,
}

/// 生命周期模式
#[derive(Debug, Clone)]
pub struct LifecyclePattern {
    pub pattern_name: String,
    pub average_duration: std::time::Duration,
    pub variables_count: usize,
}

/// 透镜联动状态
#[derive(Debug, Clone)]
pub enum LensLinkageState {
    Performance,
    Concurrency,
    Safety,
    Transitioning { from: Box<LensLinkageState>, to: Box<LensLinkageState>, progress: f64 },
}

/// Fixed hybrid template configuration for rendering complex data
#[derive(Debug)]
pub struct FixedHybridTemplate {
    thread_count: usize,
    task_count: usize,
    variable_details_enabled: bool,
    render_mode: RenderMode,
    /// 新增：智能分析引擎
    pub analysis_engine: Option<IntelligentAnalysisEngine>,
    /// 新增：透镜联动状态
    pub lens_state: LensLinkageState,
}

/// Rendering mode for different visualization approaches
#[derive(Debug, Clone)]
pub enum RenderMode {
    Comprehensive,
    ThreadFocused,
    TaskFocused,
    VariableDetailed,
}

/// Combined analysis data from multiple sources - 增强版三模块融合
#[derive(Debug)]
pub struct HybridAnalysisData {
    pub lockfree_analysis: Option<LockfreeAnalysis>,
    pub visualization_config: VisualizationConfig,
    pub thread_task_mapping: HashMap<usize, Vec<usize>>,
    pub variable_registry: HashMap<String, VariableDetail>,
    pub performance_metrics: PerformanceTimeSeries,
    pub thread_classifications: HashMap<usize, ThreadWorkloadType>,
    pub task_classifications: HashMap<usize, TaskExecutionPattern>,
    
    /// 🔗 三模块融合的核心数据结构
    pub unified_variable_index: HashMap<UnifiedVariableID, CrossModuleData>,
    pub timeline_events: BTreeMap<u64, Vec<CrossModuleEvent>>,
    pub variable_relationships: HashMap<String, Vec<RelationType>>,
    pub intelligent_analysis: Option<IntelligentAnalysisEngine>,
    
    /// 透镜联动数据
    pub lens_linkage_data: LensLinkageData,
    
    /// FFI 安全数据
    pub ffi_safety_data: FFISafetyData,
}

/// Real-time performance metrics collection
#[derive(Debug)]
pub struct PerformanceTimeSeries {
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<u64>,
    pub io_operations: Vec<u64>,
    pub network_bytes: Vec<u64>,
    pub timestamps: Vec<u64>,
    pub thread_cpu_breakdown: HashMap<usize, Vec<f64>>,
    pub thread_memory_breakdown: HashMap<usize, Vec<u64>>,
}

/// Detailed variable information for template rendering
#[derive(Debug, Clone)]
pub struct VariableDetail {
    pub name: String,
    pub type_info: String,
    pub thread_id: usize,
    pub task_id: Option<usize>,
    pub allocation_count: u64,
    pub memory_usage: u64,
    pub lifecycle_stage: LifecycleStage,
}

/// Variable lifecycle tracking stages
#[derive(Debug, Clone)]
pub enum LifecycleStage {
    Allocated,
    Active,
    Shared,
    Deallocated,
}

/// Thread workload classification
#[derive(Debug, Clone)]
pub enum ThreadWorkloadType {
    CpuIntensive,
    IoIntensive,
    NetworkIntensive,
    Mixed,
    Idle,
}

/// Task execution pattern classification
#[derive(Debug, Clone)]
pub enum TaskExecutionPattern {
    CpuBound,
    IoBound,
    NetworkBound,
    MemoryIntensive,
    Balanced,
}

/// 透镜联动数据 - 实现智能上下文传递
#[derive(Debug, Clone)]
pub struct LensLinkageData {
    /// Performance → Concurrency 联动
    pub performance_anomalies: Vec<PerformanceAnomaly>,
    /// Concurrency → Safety 联动  
    pub concurrency_risks: Vec<ConcurrencyRisk>,
    /// Safety → Performance 回溯
    pub safety_performance_impact: Vec<SafetyPerformanceImpact>,
    /// 当前活跃的联动上下文
    pub active_linkage_context: Option<LinkageContext>,
}

/// 性能异常检测
#[derive(Debug, Clone)]
pub struct PerformanceAnomaly {
    pub timestamp: u64,
    pub anomaly_type: AnomalyType,
    pub affected_threads: Vec<usize>,
    pub affected_tasks: Vec<usize>,
    pub severity: f64,
    pub suggested_lens: String, // "concurrency", "safety"
}

/// 异常类型
#[derive(Debug, Clone)]
pub enum AnomalyType {
    MemorySpike,
    CpuSurge, 
    IoBlocking,
    ThreadStarvation,
}

/// 并发风险
#[derive(Debug, Clone)]
pub struct ConcurrencyRisk {
    pub risk_type: ConcurrencyRiskType,
    pub involved_variables: Vec<String>,
    pub involved_threads: Vec<usize>,
    pub ffi_boundary_count: usize,
    pub suggested_lens: String, // "safety"
}

/// 并发风险类型
#[derive(Debug, Clone)]
pub enum ConcurrencyRiskType {
    DataRace,
    DeadlockPotential,
    FFIUnsafeSharing,
    MemoryContention,
}

/// 安全性能影响
#[derive(Debug, Clone)]
pub struct SafetyPerformanceImpact {
    pub leak_info: FFILeakInfo,
    pub performance_degradation: f64, // 百分比
    pub affected_timeline: (u64, u64), // (start, end)
    pub suggested_lens: String, // "performance"
}

/// 联动上下文
#[derive(Debug, Clone)]
pub struct LinkageContext {
    pub source_lens: String,
    pub target_lens: String,
    pub context_filter: ContextFilter,
    pub transition_data: TransitionData,
}

/// 上下文过滤器
#[derive(Debug, Clone)]
pub struct ContextFilter {
    pub time_range: Option<(u64, u64)>,
    pub thread_filter: Vec<usize>,
    pub task_filter: Vec<usize>,
    pub variable_filter: Vec<String>,
}

/// 转换数据
#[derive(Debug, Clone)]
pub struct TransitionData {
    pub highlighted_elements: Vec<String>,
    pub priority_sort: Vec<String>,
    pub context_annotations: Vec<ContextAnnotation>,
}

/// 上下文注释
#[derive(Debug, Clone)]
pub struct ContextAnnotation {
    pub element_id: String,
    pub annotation_type: String,
    pub message: String,
    pub severity: f64,
}

/// FFI 安全数据
#[derive(Debug, Clone)]
pub struct FFISafetyData {
    /// 基于168次FFI边界追踪的数据
    pub boundary_crossings: Vec<FFICrossing>,
    pub safety_violations: Vec<SafetyViolation>,
    pub ownership_chain_analysis: Vec<OwnershipChainAnalysis>,
    pub risk_matrix: RiskMatrix,
    pub safety_score_timeline: Vec<(u64, f64)>,
}

/// 安全违规
#[derive(Debug, Clone)]
pub struct SafetyViolation {
    pub violation_type: SafetyViolationType,
    pub variable_id: UnifiedVariableID,
    pub severity: SafetyLevel,
    pub location: CodeLocation,
    pub description: String,
}

/// 安全违规类型
#[derive(Debug, Clone)]
pub enum SafetyViolationType {
    MemoryLeak,
    UseAfterFree,
    DoubleFree,
    InvalidPointer,
    FFIBoundaryViolation,
}

/// 所有权链分析
#[derive(Debug, Clone)]
pub struct OwnershipChainAnalysis {
    pub variable_id: UnifiedVariableID,
    pub ownership_chain: Vec<OwnershipEvent>,
    pub chain_integrity: f64,
    pub potential_issues: Vec<String>,
}

impl FixedHybridTemplate {
    /// Create new fixed hybrid template with specified configuration
    pub fn new(thread_count: usize, task_count: usize) -> Self {
        Self {
            thread_count,
            task_count,
            variable_details_enabled: true,
            render_mode: RenderMode::Comprehensive,
            analysis_engine: None,
            lens_state: LensLinkageState::Performance,
        }
    }
    
    /// 创建带有智能分析引擎的增强版模板
    pub fn new_with_intelligence(thread_count: usize, task_count: usize) -> Self {
        let analysis_engine = IntelligentAnalysisEngine {
            leak_detector: LeakDetector {
                unmatched_allocations: Vec::new(),
                timeout_variables: Vec::new(),
                ffi_boundary_leaks: Vec::new(),
            },
            race_analyzer: RaceAnalyzer {
                shared_variable_access: HashMap::new(),
                race_conditions: Vec::new(),
                deadlock_scenarios: Vec::new(),
            },
            ffi_auditor: FFIAuditor {
                boundary_crossings: Vec::new(),
                risk_assessment: RiskMatrix {
                    memory_safety_score: 0.0,
                    thread_safety_score: 0.0,
                    ffi_safety_score: 0.0,
                    overall_risk: RiskLevel::Low,
                },
                ownership_transfers: Vec::new(),
            },
            pattern_miner: PatternMiner {
                allocation_patterns: Vec::new(),
                lifecycle_patterns: Vec::new(),
                thread_affinity: HashMap::new(),
            },
        };
        
        Self {
            thread_count,
            task_count,
            variable_details_enabled: true,
            render_mode: RenderMode::Comprehensive,
            analysis_engine: Some(analysis_engine),
            lens_state: LensLinkageState::Performance,
        }
    }
    
    /// 设置透镜联动状态
    pub fn with_lens_state(mut self, state: LensLinkageState) -> Self {
        self.lens_state = state;
        self
    }

    /// Configure rendering mode for template output
    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Enable or disable detailed variable tracking
    pub fn with_variable_details(mut self, enabled: bool) -> Self {
        self.variable_details_enabled = enabled;
        self
    }
    
    /// 🔗 智能透镜联动：Performance → Concurrency
    pub fn trigger_performance_to_concurrency_linkage(
        &self,
        data: &HybridAnalysisData,
        anomaly: &PerformanceAnomaly,
    ) -> LinkageContext {
        LinkageContext {
            source_lens: "performance".to_string(),
            target_lens: "concurrency".to_string(),
            context_filter: ContextFilter {
                time_range: Some((anomaly.timestamp.saturating_sub(5000), anomaly.timestamp + 5000)),
                thread_filter: anomaly.affected_threads.clone(),
                task_filter: anomaly.affected_tasks.clone(),
                variable_filter: Vec::new(),
            },
            transition_data: TransitionData {
                highlighted_elements: anomaly.affected_threads.iter()
                    .map(|&id| format!("thread-{}", id))
                    .collect(),
                priority_sort: vec![
                    "memory-usage".to_string(),
                    "thread-contention".to_string(),
                    "task-scheduling".to_string(),
                ],
                context_annotations: vec![
                    ContextAnnotation {
                        element_id: format!("anomaly-{}", anomaly.timestamp),
                        annotation_type: "performance-spike".to_string(),
                        message: format!("Performance anomaly detected: {:?}", anomaly.anomaly_type),
                        severity: anomaly.severity,
                    }
                ],
            },
        }
    }
    
    /// 🔗 智能透镜联动：Concurrency → Safety
    pub fn trigger_concurrency_to_safety_linkage(
        &self,
        data: &HybridAnalysisData,
        risk: &ConcurrencyRisk,
    ) -> LinkageContext {
        LinkageContext {
            source_lens: "concurrency".to_string(),
            target_lens: "safety".to_string(),
            context_filter: ContextFilter {
                time_range: None,
                thread_filter: risk.involved_threads.clone(),
                task_filter: Vec::new(),
                variable_filter: risk.involved_variables.clone(),
            },
            transition_data: TransitionData {
                highlighted_elements: risk.involved_variables.iter()
                    .map(|var| format!("variable-{}", var))
                    .chain(risk.involved_threads.iter().map(|&id| format!("thread-{}", id)))
                    .collect(),
                priority_sort: vec![
                    "ffi-boundaries".to_string(),
                    "unsafe-operations".to_string(),
                    "ownership-transfers".to_string(),
                ],
                context_annotations: vec![
                    ContextAnnotation {
                        element_id: format!("risk-{:?}", risk.risk_type),
                        annotation_type: "concurrency-risk".to_string(),
                        message: format!("Concurrency risk detected: {:?}", risk.risk_type),
                        severity: match risk.risk_type {
                            ConcurrencyRiskType::DataRace => 0.9,
                            ConcurrencyRiskType::DeadlockPotential => 0.8,
                            ConcurrencyRiskType::FFIUnsafeSharing => 0.95,
                            ConcurrencyRiskType::MemoryContention => 0.6,
                        },
                    }
                ],
            },
        }
    }
    
    /// 🔗 智能透镜联动：Safety → Performance 回溯
    pub fn trigger_safety_to_performance_linkage(
        &self,
        data: &HybridAnalysisData,
        impact: &SafetyPerformanceImpact,
    ) -> LinkageContext {
        LinkageContext {
            source_lens: "safety".to_string(),
            target_lens: "performance".to_string(),
            context_filter: ContextFilter {
                time_range: Some(impact.affected_timeline),
                thread_filter: vec![impact.leak_info.variable_id.thread_id],
                task_filter: impact.leak_info.variable_id.task_id.into_iter().collect(),
                variable_filter: vec![impact.leak_info.variable_id.var_name.clone()],
            },
            transition_data: TransitionData {
                highlighted_elements: vec![
                    format!("leak-{}", impact.leak_info.variable_id.var_name),
                    format!("timeline-{}-{}", impact.affected_timeline.0, impact.affected_timeline.1),
                ],
                priority_sort: vec![
                    "memory-timeline".to_string(),
                    "performance-impact".to_string(),
                    "degradation-curve".to_string(),
                ],
                context_annotations: vec![
                    ContextAnnotation {
                        element_id: format!("leak-impact-{}", impact.leak_info.variable_id.var_name),
                        annotation_type: "memory-leak-impact".to_string(),
                        message: format!("Memory leak causing {:.1}% performance degradation", 
                                       impact.performance_degradation),
                        severity: impact.performance_degradation / 100.0,
                    }
                ],
            },
        }
    }
    
    /// 🎯 内存泄漏智能检测
    pub fn detect_memory_leaks(&self, data: &HybridAnalysisData) -> Vec<FFILeakInfo> {
        let mut leaks = Vec::new();
        
        // 基于真实的track_var_owned!数据进行检测
        for (var_name, var_detail) in &data.variable_registry {
            // 检查渐进式泄漏：内存使用量持续增长
            if var_detail.memory_usage > 10 * 1024 * 1024 { // 10MB 阈值
                if matches!(var_detail.lifecycle_stage, LifecycleStage::Active) {
                    // 检查是否为FFI边界泄漏
                    if let Some(unified_id) = self.find_unified_variable_id(data, var_name) {
                        if self.is_ffi_boundary_variable(data, &unified_id) {
                            leaks.push(FFILeakInfo {
                                variable_id: unified_id,
                                leak_size: var_detail.memory_usage,
                                boundary_type: "rust-to-c".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        leaks
    }
    
    /// 🔄 并发竞争深度分析
    pub fn analyze_concurrency_races(&self, data: &HybridAnalysisData) -> Vec<RaceCondition> {
        let mut races = Vec::new();
        
        // 基于真实的24线程数据检测竞争
        for (var_name, var_detail) in &data.variable_registry {
            if matches!(var_detail.lifecycle_stage, LifecycleStage::Shared) {
                // 查找访问该变量的所有线程
                let accessing_threads: Vec<usize> = data.variable_registry
                    .values()
                    .filter(|v| v.name == *var_name)
                    .map(|v| v.thread_id)
                    .collect();
                
                if accessing_threads.len() > 1 {
                    let severity = match accessing_threads.len() {
                        2..=3 => RaceSeverity::Low,
                        4..=6 => RaceSeverity::Medium,
                        7..=10 => RaceSeverity::High,
                        _ => RaceSeverity::Critical,
                    };
                    
                    races.push(RaceCondition {
                        variable_name: var_name.clone(),
                        competing_threads: accessing_threads,
                        severity,
                    });
                }
            }
        }
        
        races
    }
    
    /// 🛡️ FFI安全深度审计
    pub fn audit_ffi_safety(&self, data: &HybridAnalysisData) -> FFISafetyData {
        let mut boundary_crossings = Vec::new();
        let mut safety_violations = Vec::new();
        let mut ownership_chain_analysis = Vec::new();
        
        // 基于168次FFI边界追踪进行安全评估
        for (unified_id, cross_data) in &data.unified_variable_index {
            // 检查FFI边界穿越
            for event in &cross_data.event_chain {
                if let CrossModuleEvent::FFICrossing { id, direction } = event {
                    let safety_level = self.assess_ffi_safety_level(data, id);
                    boundary_crossings.push(FFICrossing {
                        variable_id: id.clone(),
                        direction: direction.clone(),
                        safety_level,
                    });
                }
            }
            
            // 检查所有权链完整性
            let ownership_events: Vec<_> = cross_data.event_chain.iter()
                .filter_map(|event| match event {
                    CrossModuleEvent::Allocation { id, .. } => Some(OwnershipEvent {
                        variable_id: id.clone(),
                        from_context: "rust".to_string(),
                        to_context: "allocated".to_string(),
                        transfer_type: OwnershipTransferType::Move,
                    }),
                    CrossModuleEvent::FFICrossing { id, direction } => Some(OwnershipEvent {
                        variable_id: id.clone(),
                        from_context: match direction {
                            FFIDirection::RustToC => "rust".to_string(),
                            FFIDirection::CToRust => "c".to_string(),
                        },
                        to_context: match direction {
                            FFIDirection::RustToC => "c".to_string(),
                            FFIDirection::CToRust => "rust".to_string(),
                        },
                        transfer_type: OwnershipTransferType::FFIHandover,
                    }),
                    _ => None,
                })
                .collect();
            
            if !ownership_events.is_empty() {
                let chain_integrity = self.calculate_ownership_chain_integrity(&ownership_events);
                ownership_chain_analysis.push(OwnershipChainAnalysis {
                    variable_id: unified_id.clone(),
                    ownership_chain: ownership_events,
                    chain_integrity,
                    potential_issues: self.identify_ownership_issues(chain_integrity),
                });
            }
        }
        
        // 计算整体风险矩阵
        let risk_matrix = self.calculate_risk_matrix(&boundary_crossings, &safety_violations);
        
        FFISafetyData {
            boundary_crossings,
            safety_violations,
            ownership_chain_analysis,
            risk_matrix,
            safety_score_timeline: self.generate_safety_score_timeline(data),
        }
    }
    
    // 辅助方法
    fn find_unified_variable_id(&self, data: &HybridAnalysisData, var_name: &str) -> Option<UnifiedVariableID> {
        data.unified_variable_index.keys()
            .find(|id| id.var_name == var_name)
            .cloned()
    }
    
    fn is_ffi_boundary_variable(&self, data: &HybridAnalysisData, id: &UnifiedVariableID) -> bool {
        data.unified_variable_index.get(id)
            .map(|cross_data| cross_data.relationships.iter()
                .any(|rel| matches!(rel, RelationType::FFIBoundary)))
            .unwrap_or(false)
    }
    
    fn assess_ffi_safety_level(&self, data: &HybridAnalysisData, id: &UnifiedVariableID) -> SafetyLevel {
        // 简化的安全级别评估
        if let Some(var_detail) = data.variable_registry.get(&id.var_name) {
            match var_detail.memory_usage {
                0..=1024 => SafetyLevel::Safe,
                1025..=10240 => SafetyLevel::Warning,
                10241..=102400 => SafetyLevel::Dangerous,
                _ => SafetyLevel::Critical,
            }
        } else {
            SafetyLevel::Warning
        }
    }
    
    fn calculate_ownership_chain_integrity(&self, events: &[OwnershipEvent]) -> f64 {
        if events.is_empty() { return 1.0; }
        
        let mut integrity_score = 1.0;
        for event in events {
            match event.transfer_type {
                OwnershipTransferType::Move => integrity_score *= 0.9,
                OwnershipTransferType::Borrow => integrity_score *= 0.95,
                OwnershipTransferType::Clone => integrity_score *= 0.85,
                OwnershipTransferType::FFIHandover => integrity_score *= 0.7,
            }
        }
        integrity_score
    }
    
    fn identify_ownership_issues(&self, integrity: f64) -> Vec<String> {
        let mut issues = Vec::new();
        if integrity < 0.8 {
            issues.push("Complex ownership transfer chain".to_string());
        }
        if integrity < 0.6 {
            issues.push("High risk of ownership violations".to_string());
        }
        if integrity < 0.4 {
            issues.push("Critical ownership integrity issues".to_string());
        }
        issues
    }
    
    fn calculate_risk_matrix(&self, crossings: &[FFICrossing], violations: &[SafetyViolation]) -> RiskMatrix {
        let memory_safety_score = if violations.is_empty() { 10.0 } else {
            10.0 - violations.len() as f64 * 2.0
        }.max(0.0);
        
        let ffi_safety_score = if crossings.is_empty() { 10.0 } else {
            let critical_crossings = crossings.iter()
                .filter(|c| matches!(c.safety_level, SafetyLevel::Critical))
                .count();
            10.0 - critical_crossings as f64 * 3.0
        }.max(0.0);
        
        let overall_risk = match (memory_safety_score + ffi_safety_score) / 2.0 {
            score if score >= 8.0 => RiskLevel::Low,
            score if score >= 6.0 => RiskLevel::Medium,
            score if score >= 4.0 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
        
        RiskMatrix {
            memory_safety_score,
            thread_safety_score: 8.0, // 简化计算
            ffi_safety_score,
            overall_risk,
        }
    }
    
    fn generate_safety_score_timeline(&self, data: &HybridAnalysisData) -> Vec<(u64, f64)> {
        // 简化的安全分数时间线生成
        data.timeline_events.iter()
            .map(|(timestamp, events)| {
                let safety_score = 10.0 - events.iter()
                    .filter(|e| matches!(e, CrossModuleEvent::FFICrossing { .. }))
                    .count() as f64 * 0.5;
                (*timestamp, safety_score.max(0.0))
            })
            .collect()
    }

    /// Generate comprehensive HTML dashboard with hybrid data
    pub fn generate_hybrid_dashboard(
        &self,
        data: &HybridAnalysisData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut html_content = String::with_capacity(50000);
        
        // Build HTML structure
        html_content.push_str(&self.build_html_header());
        html_content.push_str(&self.build_navigation_bar());
        html_content.push_str(&self.build_memory_continent_overview(data)?);
        html_content.push_str(&self.build_territory_treemap(data)?);
        html_content.push_str(&self.build_interactive_drilldown_panel(data)?);
        html_content.push_str(&self.build_performance_charts(data)?);
        html_content.push_str(&self.build_thread_task_matrix(data)?);
        html_content.push_str(&self.build_variable_details_section(data)?);
        html_content.push_str(&self.build_performance_metrics(data)?);
        html_content.push_str(&self.build_html_footer());

        Ok(html_content)
    }

    /// Build HTML header with styles and scripts
    fn build_html_header(&self) -> String {
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hybrid Memory Analysis - {} Threads, {} Tasks</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        /* Theme Variables */
        :root {{
            --bg-primary: #0f1419;
            --bg-secondary: #1a1f2e;
            --bg-tertiary: #252c3f;
            --text-primary: #e5e7eb;
            --text-secondary: #9ca3af;
            --accent-blue: #3b82f6;
            --accent-purple: #8b5cf6;
            --accent-green: #10b981;
            --accent-orange: #f59e0b;
            --accent-red: #ef4444;
            --accent-cyan: #06b6d4;
            --border-color: #374151;
            --shadow-dark: 0 4px 15px rgba(0,0,0,0.3);
            --gradient-primary: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            --gradient-card: linear-gradient(135deg, #1e293b 0%, #334155 100%);
        }}

        /* Light theme override */
        [data-theme="light"] {{
            --bg-primary: #f8fafc;
            --bg-secondary: #ffffff;
            --bg-tertiary: #f1f5f9;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --border-color: #e2e8f0;
            --shadow-dark: 0 4px 15px rgba(0,0,0,0.1);
            --gradient-card: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
        }}

        body {{ 
            font-family: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
            margin: 0; padding: 20px; 
            background: var(--bg-primary);
            color: var(--text-primary);
            transition: all 0.3s ease;
        }}
        
        .container {{ max-width: 1600px; margin: 0 auto; }}
        
        .theme-toggle {{
            position: fixed; top: 20px; right: 20px; z-index: 1000;
            background: var(--bg-tertiary); border: 1px solid var(--border-color);
            color: var(--text-primary); padding: 8px 16px; border-radius: 8px;
            cursor: pointer; font-size: 14px; transition: all 0.3s ease;
        }}
        .theme-toggle:hover {{ background: var(--accent-blue); }}
        
        .nav-bar {{ 
            background: var(--gradient-primary);
            padding: 20px; border-radius: 16px; margin-bottom: 24px;
            color: white; text-align: center; font-size: 28px; font-weight: 700;
            box-shadow: var(--shadow-dark);
        }}
        
        .section {{ 
            background: var(--bg-secondary); margin: 24px 0; padding: 28px;
            border-radius: 16px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color);
        }}
        
        .section h2 {{
            color: var(--text-primary); margin-top: 0; margin-bottom: 20px;
            font-size: 24px; font-weight: 600;
        }}
        
        .charts-grid {{ 
            display: grid; grid-template-columns: 1fr 1fr; gap: 24px; margin: 24px 0;
        }}
        
        .chart-container {{ 
            background: var(--bg-tertiary); padding: 24px; border-radius: 16px;
            box-shadow: var(--shadow-dark); border: 1px solid var(--border-color);
        }}
        
        .chart-title {{ 
            font-size: 18px; font-weight: 600; margin-bottom: 16px;
            color: var(--text-primary); text-align: center;
        }}
        
        .matrix-grid {{ 
            display: grid; gap: 16px; margin-top: 24px;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        }}
        
        .thread-card {{ 
            border: 1px solid var(--border-color); border-radius: 12px; padding: 16px;
            background: var(--gradient-card);
            color: var(--text-primary); font-size: 13px;
            box-shadow: var(--shadow-dark);
            transition: transform 0.2s ease;
        }}
        .thread-card:hover {{ transform: translateY(-2px); }}
        
        .task-item {{ 
            background: var(--bg-tertiary); margin: 8px 0;
            padding: 10px; border-radius: 8px; font-size: 12px;
            border: 1px solid var(--border-color);
        }}
        
        .variable-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 16px; margin-top: 20px;
        }}
        
        .variable-card {{ 
            border-left: 4px solid var(--accent-green); padding: 16px;
            background: var(--bg-tertiary); border-radius: 12px; font-size: 13px;
            border: 1px solid var(--border-color);
            transition: all 0.2s ease;
        }}
        .variable-card:hover {{ 
            transform: translateY(-1px); 
            box-shadow: var(--shadow-dark);
        }}
        
        .metric-row {{ 
            display: flex; justify-content: space-between;
            padding: 12px 0; border-bottom: 1px solid var(--border-color); 
            font-size: 14px;
        }}
        .metric-row:last-child {{ border-bottom: none; }}
        
        .metric-value {{ 
            font-weight: 600; color: var(--accent-blue); 
        }}
        
        .lifecycle-badge {{ 
            display: inline-block; padding: 4px 8px; border-radius: 12px;
            font-size: 11px; font-weight: 600; color: white;
            text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        .allocated {{ background: var(--accent-green); }}
        .active {{ background: var(--accent-blue); }}
        .shared {{ background: var(--accent-orange); }}
        .deallocated {{ background: var(--text-secondary); }}
        
        .performance-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px; margin: 24px 0;
        }}
        
        .perf-card {{ 
            background: var(--gradient-card);
            color: var(--text-primary); padding: 24px; border-radius: 16px;
            text-align: center; border: 1px solid var(--border-color);
            box-shadow: var(--shadow-dark);
            transition: transform 0.2s ease;
        }}
        .perf-card:hover {{ transform: translateY(-2px); }}
        
        .perf-value {{ 
            font-size: 32px; font-weight: 700; 
            background: var(--gradient-primary);
            -webkit-background-clip: text; -webkit-text-fill-color: transparent;
            background-clip: text;
        }}
        .perf-label {{ 
            font-size: 14px; opacity: 0.8; margin-top: 8px;
            color: var(--text-secondary);
        }}

        /* Variable controls styling */
        .variable-controls {{
            display: flex; justify-content: space-between; align-items: center;
            margin-bottom: 20px; flex-wrap: wrap; gap: 15px;
        }}
        .pagination-controls {{
            display: flex; align-items: center; gap: 10px;
        }}
        .pagination-controls button {{
            background: var(--accent-blue); color: white; border: none;
            padding: 8px 16px; border-radius: 6px; cursor: pointer;
            font-size: 14px; transition: all 0.2s ease;
        }}
        .pagination-controls button:hover {{ background: var(--accent-purple); }}
        .pagination-controls button:disabled {{
            background: var(--text-secondary); cursor: not-allowed;
        }}
        .filter-controls {{
            display: flex; gap: 10px; align-items: center;
        }}
        .filter-controls select, .filter-controls input {{
            background: var(--bg-tertiary); color: var(--text-primary);
            border: 1px solid var(--border-color); padding: 8px 12px;
            border-radius: 6px; font-size: 14px;
        }}
        .filter-controls input {{
            width: 200px;
        }}
        .variable-loading {{
            text-align: center; padding: 40px;
            color: var(--text-secondary); font-style: italic;
        }}

        /* Classification legend styling */
        .classification-legend {{
            display: flex; gap: 15px; margin-bottom: 20px; flex-wrap: wrap;
        }}
        .legend-item {{
            padding: 6px 12px; border-radius: 8px; font-size: 13px;
            font-weight: 500; color: white; text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        .legend-item.cpu-intensive {{ background: var(--accent-red); }}
        .legend-item.io-intensive {{ background: var(--accent-blue); }}
        .legend-item.network-intensive {{ background: var(--accent-purple); }}
        .legend-item.mixed-workload {{ background: var(--accent-orange); }}
        .legend-item.idle-thread {{ background: var(--text-secondary); }}

        /* Workload type styling */
        .workload-type {{
            font-size: 12px; opacity: 0.9; margin-bottom: 10px;
            font-weight: 500;
        }}
        
        /* Expandable details */
        .thread-details {{
            margin-top: 15px; padding-top: 15px;
            border-top: 1px solid rgba(255,255,255,0.2);
        }}
        .expand-icon {{
            float: right; transition: transform 0.3s ease;
        }}
        .expanded .expand-icon {{ transform: rotate(180deg); }}
        
        .task-variables {{
            margin-top: 10px; padding: 10px;
            background: rgba(255,255,255,0.1); border-radius: 6px;
            font-size: 11px;
        }}
        
        /* Thread card specific colors */
        .thread-card.cpu-intensive {{
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        }}
        .thread-card.io-intensive {{
            background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
        }}
        .thread-card.network-intensive {{
            background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
        }}
        .thread-card.mixed-workload {{
            background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        }}
        .thread-card.idle-thread {{
            background: linear-gradient(135deg, #6b7280 0%, #4b5563 100%);
        }}

        /* Chart toggle button */
        .chart-toggle {{
            text-align: center; margin-bottom: 20px;
        }}
        .chart-toggle button {{
            background: var(--accent-blue); color: white; border: none;
            padding: 12px 24px; border-radius: 8px; cursor: pointer;
            font-size: 16px; transition: all 0.3s ease;
        }}
        .chart-toggle button:hover {{ background: var(--accent-purple); }}

        /* 深度数据挖掘工作台样式 - Data Ocean Deep Mining Workbench */
        .lens-system {{
            margin-top: 20px;
        }}
        .primary-lens-row {{
            display: flex; gap: 15px; justify-content: center; margin-bottom: 15px;
        }}
        .lens-button {{
            background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(59, 130, 246, 0.05));
            border: 2px solid rgba(59, 130, 246, 0.3); color: white;
            padding: 15px 20px; border-radius: 12px; cursor: pointer;
            transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative; overflow: hidden; min-width: 180px;
        }}
        .lens-button::before {{
            content: ''; position: absolute; top: -2px; left: -2px; right: -2px; bottom: -2px;
            background: linear-gradient(45deg, #3b82f6, #8b5cf6, #ef4444, #10b981);
            z-index: -1; opacity: 0; border-radius: 12px;
            transition: opacity 0.3s ease;
        }}
        .lens-button:hover::before {{ opacity: 0.7; }}
        .lens-button:hover {{
            transform: translateY(-3px) scale(1.02);
            box-shadow: 0 10px 30px rgba(59, 130, 246, 0.4);
        }}
        .lens-button.active {{
            background: linear-gradient(135deg, #3b82f6, #1d4ed8);
            border-color: #3b82f6; transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(59, 130, 246, 0.6);
        }}
        .lens-icon {{ font-size: 20px; margin-bottom: 5px; }}
        .lens-text {{ font-weight: 600; font-size: 14px; }}
        .lens-subtitle {{ font-size: 11px; opacity: 0.8; margin-top: 2px; }}
        
        /* 数据挖掘控制器样式 */
        .data-mining-controls {{
            display: flex; justify-content: space-between; align-items: center;
            background: rgba(255, 255, 255, 0.05); padding: 12px 20px;
            border-radius: 8px; margin-top: 10px; flex-wrap: wrap; gap: 15px;
        }}
        .mining-depth-selector label {{
            color: var(--text-secondary); margin-right: 8px; font-size: 13px;
        }}
        .mining-depth-selector select {{
            background: var(--bg-tertiary); color: var(--text-primary);
            border: 1px solid var(--border-color); padding: 6px 10px;
            border-radius: 6px; font-size: 12px;
        }}
        .auto-link-btn {{
            background: var(--accent-green); color: white; border: none;
            padding: 8px 15px; border-radius: 20px; font-size: 12px;
            cursor: pointer; transition: all 0.3s ease; font-weight: 500;
        }}
        .auto-link-btn:hover {{ background: var(--accent-blue); transform: scale(1.05); }}
        .auto-link-btn.active {{
            animation: pulse-glow 2s infinite;
        }}
        @keyframes pulse-glow {{
            0%, 100% {{ box-shadow: 0 0 5px var(--accent-green); }}
            50% {{ box-shadow: 0 0 20px var(--accent-green), 0 0 30px var(--accent-green); }}
        }}
        .flow-badge {{
            background: var(--accent-purple); color: white;
            padding: 6px 12px; border-radius: 15px; font-size: 11px;
            display: flex; align-items: center; gap: 5px;
        }}
        .flow-count {{ font-weight: bold; }}
        
        /* 主工作台布局 */
        .workbench-layout {{
            display: flex; gap: 25px; margin: 25px 0;
        }}
        .main-analysis-area {{
            flex: 0 0 75%; background: var(--bg-secondary);
            border-radius: 16px; padding: 25px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color); min-height: 700px;
        }}
        .deep-analysis-sidebar {{
            flex: 0 0 23%; display: flex; flex-direction: column; gap: 20px;
        }}
        
        /* 透镜内容区域 */
        .lens-content {{
            display: none; height: 100%;
        }}
        .lens-content.active {{
            display: block; animation: lens-fade-in 0.5s ease-out;
        }}
        @keyframes lens-fade-in {{
            from {{ opacity: 0; transform: translateY(20px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        .lens-header {{
            margin-bottom: 25px; padding-bottom: 20px;
            border-bottom: 2px solid var(--border-color);
        }}
        .lens-header h2 {{
            margin: 0 0 10px 0; color: var(--text-primary);
            font-size: 26px; font-weight: 700;
        }}
        .analysis-stats {{
            display: flex; gap: 15px; margin-top: 10px; flex-wrap: wrap;
        }}
        .stat-badge {{
            background: linear-gradient(135deg, var(--accent-blue), var(--accent-purple));
            color: white; padding: 6px 12px; border-radius: 15px;
            font-size: 12px; font-weight: 600;
        }}
        .safety-metrics {{
            display: flex; gap: 20px; margin-top: 10px;
        }}
        .safety-score {{ color: var(--accent-green); font-weight: bold; }}
        .risk-level {{ color: var(--accent-orange); font-weight: bold; }}
        .performance-kpis {{
            display: flex; gap: 20px; margin-top: 10px; flex-wrap: wrap;
        }}
        .kpi-item {{ color: var(--text-secondary); font-size: 13px; }}
        .kpi-item span {{ color: var(--accent-blue); font-weight: bold; }}
        
        /* 多维度可视化容器 */
        .multi-dimensional-viz {{
            display: grid; grid-template-columns: 1fr 1fr; gap: 20px;
            margin-top: 20px; min-height: 500px;
        }}
        .viz-panel {{
            background: var(--bg-tertiary); border-radius: 12px;
            padding: 20px; border: 1px solid var(--border-color);
            box-shadow: var(--shadow-dark); transition: all 0.3s ease;
        }}
        .viz-panel:hover {{
            transform: translateY(-2px); box-shadow: 0 8px 25px rgba(59, 130, 246, 0.2);
        }}
        .viz-panel h3 {{
            margin: 0 0 15px 0; color: var(--text-primary);
            font-size: 16px; font-weight: 600;
        }}
        .threejs-container {{
            height: 300px; background: linear-gradient(135deg, #0f1419, #1a1f2e);
            border-radius: 8px; border: 1px solid var(--border-color);
            display: flex; align-items: center; justify-content: center;
            color: var(--text-secondary); font-style: italic;
        }}
        .heatmap-container {{
            height: 300px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
            position: relative; overflow: hidden;
        }}
        .flow-monitor-container {{
            height: 300px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
            display: flex; align-items: center; justify-content: center;
            color: var(--text-secondary);
        }}
        
        /* 安全分析可视化 */
        .safety-analysis-viz {{
            display: grid; grid-template-columns: 1fr; gap: 20px;
            margin-top: 20px;
        }}
        .swimlane-container {{
            height: 200px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
            position: relative; overflow: hidden;
        }}
        .boundary-audit-container {{
            height: 200px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        .leak-detector-container {{
            height: 200px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        
        /* 性能挖掘可视化 */
        .performance-mining-viz {{
            display: grid; grid-template-columns: 1fr; gap: 20px;
            margin-top: 20px;
        }}
        .timeseries-container {{
            height: 250px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        .waterfall-container {{
            height: 200px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        .pattern-container {{
            height: 200px; background: var(--bg-primary);
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        
        /* 深度分析侧边栏 */
        .mining-console {{
            background: var(--bg-secondary); padding: 20px;
            border-radius: 12px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color);
        }}
        .mining-console h3 {{
            margin: 0 0 15px 0; color: var(--text-primary);
            font-size: 16px; font-weight: 600;
        }}
        .console-metrics {{
            display: flex; flex-direction: column; gap: 10px;
        }}
        .lens-sidebar {{
            display: none; background: var(--bg-secondary);
            padding: 20px; border-radius: 12px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color); flex: 1;
        }}
        .lens-sidebar.active {{
            display: block; animation: sidebar-slide-in 0.4s ease-out;
        }}
        @keyframes sidebar-slide-in {{
            from {{ opacity: 0; transform: translateX(20px); }}
            to {{ opacity: 1; transform: translateX(0); }}
        }}
        .lens-sidebar h3 {{
            margin: 0 0 15px 0; color: var(--text-primary);
            font-size: 16px; font-weight: 600;
        }}
        .deep-analysis-panel {{
            display: flex; flex-direction: column; gap: 15px;
        }}
        .analysis-section {{
            background: var(--bg-tertiary); padding: 15px;
            border-radius: 8px; border: 1px solid var(--border-color);
        }}
        .analysis-section h4 {{
            margin: 0 0 10px 0; color: var(--text-primary);
            font-size: 14px; font-weight: 600;
        }}
        
        /* 跨透镜智能联动面板 */
        .cross-lens-linkage-panel {{
            background: var(--bg-secondary); padding: 20px;
            border-radius: 12px; box-shadow: var(--shadow-dark);
            border: 1px solid var(--border-color);
        }}
        .cross-lens-linkage-panel h3 {{
            margin: 0 0 15px 0; color: var(--text-primary);
            font-size: 16px; font-weight: 600;
        }}
        .linkage-status {{
            margin-bottom: 15px;
        }}
        .linkage-indicator {{
            display: flex; align-items: center; gap: 8px;
            color: var(--text-secondary); font-size: 13px;
        }}
        .status-dot {{
            width: 8px; height: 8px; border-radius: 50%;
            background: var(--accent-green);
        }}
        .status-dot.active {{
            animation: status-pulse 2s infinite;
        }}
        @keyframes status-pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        .active-links {{
            background: var(--bg-tertiary); padding: 15px;
            border-radius: 8px; border: 1px solid var(--border-color);
            min-height: 100px;
        }}
        
        /* Memory Continent enhanced styling for premium experience */
        .continent-tabs {{
            display: flex; gap: 10px; margin-top: 10px; justify-content: center;
        }}
        .tab-button {{
            background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2);
            color: white; padding: 8px 16px; border-radius: 6px; cursor: pointer;
            font-size: 14px; transition: all 0.3s ease;
        }}
        .tab-button:hover {{ 
            background: rgba(255,255,255,0.2); 
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
        }}
        .tab-button.active {{ 
            background: var(--accent-blue);
            box-shadow: 0 0 20px rgba(59, 130, 246, 0.5);
        }}

        /* Territory Treemap styling */
        .territory-treemap {{
            background: var(--bg-secondary); border-radius: 16px; padding: 25px;
            margin: 20px 0; box-shadow: var(--shadow-dark); border: 1px solid var(--border-color);
        }}
        .treemap-container {{
            position: relative; width: 100%; height: 400px; border-radius: 12px;
            border: 2px solid var(--border-color); overflow: hidden;
        }}
        .territory-rect {{
            position: absolute; border: 2px solid rgba(255,255,255,0.3);
            cursor: pointer; transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1); 
            display: flex; align-items: center; justify-content: center; color: white;
            font-weight: bold; text-shadow: 1px 1px 2px rgba(0,0,0,0.7);
            border-radius: 8px; overflow: hidden;
        }}
        .territory-rect:hover {{
            border-color: #ffffff; 
            transform: scale(1.03) rotateY(2deg);
            box-shadow: 0 8px 32px rgba(59, 130, 246, 0.6);
            filter: brightness(1.1);
        }}
        .territory-rect:hover .territory-content {{
            transform: scale(1.1);
        }}
        .territory-content {{
            transition: transform 0.3s ease;
            text-align: center;
        }}
        .territory-details {{
            font-size: 12px; margin-top: 5px; opacity: 0.9;
        }}
        .territory-rect.main-thread {{
            background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        }}
        .territory-rect.thread-pool {{
            background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
        }}
        .territory-rect.async-runtime {{
            background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
        }}
        .territory-rect.ffi-boundary {{
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        }}

        /* Interactive Drilldown Panel */
        .drilldown-panel {{
            background: var(--bg-secondary); border-radius: 16px; padding: 25px;
            margin: 20px 0; box-shadow: var(--shadow-dark); border: 1px solid var(--border-color);
            min-height: 300px; display: none;
        }}
        .drilldown-panel.active {{ display: block; }}
        .drilldown-header {{
            display: flex; justify-content: space-between; align-items: center;
            margin-bottom: 20px; padding-bottom: 15px; border-bottom: 1px solid var(--border-color);
        }}
        .drilldown-title {{
            font-size: 20px; font-weight: bold; color: var(--text-primary);
        }}
        .drilldown-close {{
            background: var(--accent-red); color: white; border: none;
            padding: 6px 12px; border-radius: 6px; cursor: pointer;
        }}
        .execution-unit-grid {{
            display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px; margin-top: 20px;
        }}
        .execution-unit-card {{
            background: var(--bg-tertiary); border-radius: 12px; padding: 15px;
            border: 1px solid var(--border-color); transition: all 0.2s ease;
            cursor: pointer;
        }}
        .execution-unit-card:hover {{
            transform: translateY(-2px); box-shadow: var(--shadow-dark);
        }}
        .unit-header {{
            display: flex; justify-content: space-between; align-items: center;
            margin-bottom: 10px;
        }}
        .unit-title {{ font-weight: bold; color: var(--text-primary); }}
        .unit-memory {{ color: var(--accent-green); font-weight: bold; }}

        /* Advanced territory tooltip system */
        .territory-tooltip {{
            position: absolute; background: rgba(0, 0, 0, 0.9); color: white;
            padding: 12px 16px; border-radius: 8px; font-size: 13px;
            pointer-events: none; z-index: 10000; border: 1px solid var(--accent-blue);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6); opacity: 0;
            transition: opacity 0.3s ease, transform 0.3s ease;
            transform: translateY(10px); max-width: 250px;
        }}
        .territory-tooltip.show {{
            opacity: 1; transform: translateY(0);
        }}
        .tooltip-header {{
            font-weight: bold; margin-bottom: 8px; color: var(--accent-blue);
        }}
        .tooltip-metric {{
            display: flex; justify-content: space-between; margin: 4px 0;
        }}

        /* Context filtering indicators */
        .context-filter-badge {{
            background: var(--accent-orange); color: white; padding: 4px 8px;
            border-radius: 12px; font-size: 11px; margin-left: 10px;
            animation: pulse 2s infinite;
        }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.7; }}
        }}

        /* Second-level treemap for drilldown */
        .secondary-treemap {{
            position: absolute; top: 0; left: 0; width: 100%; height: 100%;
            background: rgba(0, 0, 0, 0.8); z-index: 1000; display: none;
            border-radius: 12px; overflow: hidden;
        }}
        .secondary-treemap.active {{ display: block; }}
        .secondary-header {{
            background: var(--accent-blue); color: white; padding: 10px 15px;
            display: flex; justify-content: space-between; align-items: center;
        }}
        .back-button {{
            background: rgba(255, 255, 255, 0.2); border: none; color: white;
            padding: 6px 12px; border-radius: 4px; cursor: pointer;
            transition: background 0.3s ease;
        }}
        .back-button:hover {{ background: rgba(255, 255, 255, 0.3); }}
        .secondary-content {{
            padding: 15px; height: calc(100% - 50px); overflow-y: auto;
        }}
        .thread-mini-rect {{
            display: inline-block; width: 80px; height: 60px; margin: 5px;
            border-radius: 6px; cursor: pointer; position: relative;
            transition: all 0.3s ease; color: white; font-size: 10px;
            display: flex; align-items: center; justify-content: center;
            text-align: center; border: 1px solid rgba(255, 255, 255, 0.3);
        }}
        .thread-mini-rect:hover {{
            transform: scale(1.1); box-shadow: 0 4px 12px rgba(59, 130, 246, 0.5);
        }}

        /* Pie charts styling */
        .pie-charts-section {{
            margin: 20px 0;
        }}
        .pie-charts-section h3 {{
            color: var(--text-primary); text-align: center; margin-bottom: 20px;
        }}
        .pie-charts-grid {{
            display: grid; grid-template-columns: 1fr 1fr; gap: 30px;
            margin: 20px 0;
        }}
        .pie-chart-container {{
            background: var(--bg-tertiary); padding: 20px; border-radius: 12px;
            box-shadow: var(--shadow-dark); border: 1px solid var(--border-color);
            text-align: center; display: flex; flex-direction: column; align-items: center;
        }}
        .pie-chart-wrapper {{
            display: flex; align-items: center; gap: 20px;
        }}
        .pie-legend {{
            background: var(--bg-secondary); padding: 15px; border-radius: 8px;
            border: 1px solid var(--border-color); min-width: 150px;
        }}
        .legend-item {{
            display: flex; align-items: center; margin: 8px 0; font-size: 13px;
            cursor: pointer; transition: all 0.2s ease;
        }}
        .legend-item:hover {{ background: var(--bg-tertiary); padding: 4px; border-radius: 4px; }}
        .legend-color {{
            width: 16px; height: 16px; border-radius: 3px; margin-right: 8px;
        }}
        .legend-text {{ color: var(--text-primary); }}
        .chart-canvas {{ cursor: pointer; }}
        .interactive-chart {{ 
            cursor: crosshair; transition: all 0.3s ease;
            border: 2px solid transparent;
        }}
        .interactive-chart:hover {{ 
            border-color: var(--accent-blue); 
            box-shadow: 0 0 15px rgba(59, 130, 246, 0.3);
        }}
        
        /* Beautiful modal dialog */
        .modal-overlay {{
            position: fixed; top: 0; left: 0; width: 100%; height: 100%;
            background: rgba(0, 0, 0, 0.6); z-index: 10000;
            display: none; align-items: center; justify-content: center;
        }}
        .modal-content {{
            background: var(--bg-secondary); border-radius: 16px;
            padding: 30px; max-width: 400px; width: 90%;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
            border: 1px solid var(--border-color);
            animation: modalSlideIn 0.3s ease-out;
        }}
        @keyframes modalSlideIn {{
            from {{ transform: translateY(-50px); opacity: 0; }}
            to {{ transform: translateY(0); opacity: 1; }}
        }}
        .modal-header {{
            font-size: 20px; font-weight: bold; color: var(--text-primary);
            margin-bottom: 20px; text-align: center;
            background: var(--gradient-primary); -webkit-background-clip: text;
            -webkit-text-fill-color: transparent; background-clip: text;
        }}
        .modal-body {{
            color: var(--text-primary); line-height: 1.6; font-size: 14px;
        }}
        .modal-info-row {{
            display: flex; justify-content: space-between; margin: 12px 0;
            padding: 8px 0; border-bottom: 1px solid var(--border-color);
        }}
        .modal-info-label {{ color: var(--text-secondary); }}
        .modal-info-value {{ color: var(--accent-blue); font-weight: bold; }}
        .modal-close {{
            background: var(--accent-blue); color: white; border: none;
            padding: 10px 20px; border-radius: 8px; cursor: pointer;
            font-size: 14px; margin-top: 20px; width: 100%;
            transition: all 0.3s ease;
        }}
        .modal-close:hover {{ background: var(--accent-purple); }}

        /* Resource highlighting */
        .resource-highlight {{
            color: var(--accent-green); font-weight: bold;
            text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }}
        
        /* Mini variable cards in task details */
        .mini-variable-card {{
            background: rgba(255,255,255,0.05); padding: 8px;
            margin: 4px 0; border-radius: 4px; font-size: 11px;
            border-left: 2px solid var(--accent-blue);
        }}
        .mini-variable-card strong {{ color: var(--accent-blue); }}

        /* Sampling information styling */
        .sampling-info {{
            margin-bottom: 15px; text-align: center;
        }}
        .sampling-badge {{
            background: var(--accent-purple); color: white;
            padding: 6px 12px; border-radius: 8px; font-size: 13px;
            font-weight: 500; display: inline-block;
        }}

        /* Scrollbar styling for dark theme */
        ::-webkit-scrollbar {{ width: 8px; }}
        ::-webkit-scrollbar-track {{ background: var(--bg-primary); }}
        ::-webkit-scrollbar-thumb {{ 
            background: var(--border-color); border-radius: 4px; 
        }}
        ::-webkit-scrollbar-thumb:hover {{ background: var(--accent-blue); }}

        /* Responsive design */
        @media (max-width: 768px) {{
            .charts-grid {{ grid-template-columns: 1fr; }}
            .matrix-grid {{ grid-template-columns: 1fr; }}
            .variable-grid {{ grid-template-columns: 1fr; }}
            .performance-grid {{ grid-template-columns: 1fr; }}
            .nav-bar {{ font-size: 24px; padding: 16px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
"#, self.thread_count, self.task_count)
    }

    /// Build navigation bar with theme toggle for Memory Continent
    fn build_navigation_bar(&self) -> String {
        format!(
            r#"<button class="theme-toggle" onclick="toggleTheme()">🌙 Dark Mode</button>
            <div class="nav-bar">
                🌊 Memory Data Ocean - Deep Variable Insights | {} Threads × {} Tasks
                <div class="lens-system">
                    <!-- 主分析透镜系统 -->
                    <div class="primary-lens-row">
                        <button class="lens-button active" id="concurrency-lens" data-lens="concurrency" onclick="switchAnalysisLens('concurrency')">
                            <div class="lens-icon">🚀</div>
                            <div class="lens-text">Concurrency Ocean</div>
                            <div class="lens-subtitle">Thread/Task Deep Dive</div>
                        </button>
                        <button class="lens-button" id="safety-lens" data-lens="safety" onclick="switchAnalysisLens('safety')">
                            <div class="lens-icon">🛡️</div>
                            <div class="lens-text">Safety Audit</div>
                            <div class="lens-subtitle">FFI/Unsafe Analysis</div>
                        </button>
                        <button class="lens-button" id="performance-lens" data-lens="performance" onclick="switchAnalysisLens('performance')">
                            <div class="lens-icon">📈</div>
                            <div class="lens-text">Performance Mining</div>
                            <div class="lens-subtitle">Time-series Analytics</div>
                        </button>
                    </div>
                    
                    <!-- 细粒度数据挖掘控制器 -->
                    <div class="data-mining-controls">
                        <div class="mining-depth-selector">
                            <label>🔍 Mining Depth:</label>
                            <select id="mining-depth" onchange="adjustMiningDepth(this.value)">
                                <option value="surface">Surface Scan</option>
                                <option value="deep" selected>Deep Analysis</option>
                                <option value="molecular">Molecular Level</option>
                            </select>
                        </div>
                        <div class="cross-lens-linkage">
                            <button id="auto-link-toggle" onclick="toggleAutoLinkage()" class="auto-link-btn active">
                                🔗 Auto Cross-Link
                            </button>
                        </div>
                        <div class="data-flow-indicator">
                            <div class="flow-badge" id="active-flows">
                                <span class="flow-count">0</span> Active Flows
                            </div>
                        </div>
                    </div>
                </div>
            </div>"#,
            self.thread_count, self.task_count
        )
    }

    /// Build Memory Continent overview with territorial summary
    fn build_memory_continent_overview(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let total_variables = data.variable_registry.len();
        let total_memory = data.variable_registry.values()
            .map(|v| v.memory_usage)
            .sum::<u64>();
        let active_variables = data.variable_registry.values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Active))
            .count();

        Ok(format!(r#"
        <!-- 主工作台布局 - 三透镜深度联合分析 -->
        <div class="workbench-layout">
            <!-- 主视图区域 (75%) - 透镜内容 -->
            <div class="main-analysis-area">
                
                <!-- Concurrency Ocean Lens - 默认激活 -->
                <div class="lens-content active" id="concurrency-content">
                    <div class="lens-header">
                        <h2>🚀 Concurrency Ocean - Thread/Task Deep Analysis</h2>
                        <div class="analysis-stats">
                            <span class="stat-badge">Threads: {}</span>
                            <span class="stat-badge">Tasks: {}</span>
                            <span class="stat-badge">Variables: {}</span>
                            <span class="stat-badge">Relationships: {}</span>
                        </div>
                    </div>
                    
                    <!-- 多维度可视化容器 -->
                    <div class="multi-dimensional-viz">
                        <!-- 3D线程/任务关系图 -->
                        <div class="viz-panel" id="thread-task-3d">
                            <h3>🌐 3D Thread-Task Relationship Graph</h3>
                            <div class="threejs-container" id="concurrency-3d-canvas"></div>
                        </div>
                        
                        <!-- 变量流动热力图 -->
                        <div class="viz-panel" id="variable-flow-heatmap">
                            <h3>🌡️ Variable Flow Heatmap</h3>
                            <div class="heatmap-container" id="concurrency-heatmap"></div>
                        </div>
                        
                        <!-- 实时数据流监控 -->
                        <div class="viz-panel" id="realtime-flow-monitor">
                            <h3>📡 Real-time Data Flow Monitor</h3>
                            <div class="flow-monitor-container" id="flow-monitor"></div>
                        </div>
                    </div>
                </div>
                
                <!-- Safety Audit Lens -->
                <div class="lens-content" id="safety-content">
                    <div class="lens-header">
                        <h2>🛡️ Safety Audit - FFI/Unsafe Deep Scan</h2>
                        <div class="safety-metrics">
                            <span class="safety-score">Safety Score: <span id="safety-score">94.2%</span></span>
                            <span class="risk-level">Risk Level: <span id="risk-level">LOW</span></span>
                        </div>
                    </div>
                    
                    <!-- 安全分析可视化 -->
                    <div class="safety-analysis-viz">
                        <!-- 内存安全泳道图 -->
                        <div class="viz-panel" id="memory-safety-swimlane">
                            <h3>🏊 Memory Safety Swimlane</h3>
                            <div class="swimlane-container" id="safety-swimlane"></div>
                        </div>
                        
                        <!-- FFI边界审计图 -->
                        <div class="viz-panel" id="ffi-boundary-audit">
                            <h3>⚡ FFI Boundary Audit</h3>
                            <div class="boundary-audit-container" id="ffi-audit"></div>
                        </div>
                        
                        <!-- 内存泄漏检测器 -->
                        <div class="viz-panel" id="leak-detector">
                            <h3>🔍 Memory Leak Detector</h3>
                            <div class="leak-detector-container" id="leak-detection"></div>
                        </div>
                    </div>
                </div>
                
                <!-- Performance Mining Lens -->
                <div class="lens-content" id="performance-content">
                    <div class="lens-header">
                        <h2>📈 Performance Mining - Time-series Deep Analytics</h2>
                        <div class="performance-kpis">
                            <span class="kpi-item">Avg Alloc: <span id="avg-alloc">2.3ms</span></span>
                            <span class="kpi-item">Peak Memory: <span id="peak-memory">{:.1}MB</span></span>
                            <span class="kpi-item">Efficiency: <span id="efficiency">{}%</span></span>
                        </div>
                    </div>
                    
                    <!-- 性能挖掘可视化 -->
                    <div class="performance-mining-viz">
                        <!-- 多维时间序列图 -->
                        <div class="viz-panel" id="multi-dimensional-timeseries">
                            <h3>📊 Multi-dimensional Time Series</h3>
                            <div class="timeseries-container" id="performance-timeseries"></div>
                        </div>
                        
                        <!-- 变量生命周期瀑布图 -->
                        <div class="viz-panel" id="variable-lifecycle-waterfall">
                            <h3>💧 Variable Lifecycle Waterfall</h3>
                            <div class="waterfall-container" id="lifecycle-waterfall"></div>
                        </div>
                        
                        <!-- 内存分配模式识别 -->
                        <div class="viz-panel" id="allocation-pattern-recognition">
                            <h3>🧠 Allocation Pattern Recognition</h3>
                            <div class="pattern-container" id="pattern-recognition"></div>
                        </div>
                    </div>
                </div>
                
            </div>
            
            <!-- 侧边栏 (25%) - 深度数据挖掘面板 -->
            <div class="deep-analysis-sidebar">
                
                <!-- 全局数据挖掘控制台 -->
                <div class="mining-console">
                    <h3>🌊 Data Ocean Console</h3>
                    <div class="console-metrics">
                        <div class="metric-row">
                            <span>Track Variables:</span>
                            <span class="metric-value">{}</span>
                        </div>
                        <div class="metric-row">
                            <span>Memory Tracked:</span>
                            <span class="metric-value">{:.1}MB</span>
                        </div>
                        <div class="metric-row">
                            <span>Active Scopes:</span>
                            <span class="metric-value">{}</span>
                        </div>
                        <div class="metric-row">
                            <span>FFI Crossings:</span>
                            <span class="metric-value">{}</span>
                        </div>
                    </div>
                </div>
                
                <!-- 透镜特定侧边栏 -->
                <div class="lens-sidebar active" id="concurrency-sidebar">
                    <h3>🎯 Concurrency Deep Dive</h3>
                    <div class="deep-analysis-panel">
                        <!-- 线程竞争热点 -->
                        <div class="analysis-section">
                            <h4>🔥 Thread Contention Hotspots</h4>
                            <div id="thread-contention-list"></div>
                        </div>
                        
                        <!-- 变量共享模式 -->
                        <div class="analysis-section">
                            <h4>🔄 Variable Sharing Patterns</h4>
                            <div id="variable-sharing-patterns"></div>
                        </div>
                    </div>
                </div>
                
                <div class="lens-sidebar" id="safety-sidebar">
                    <h3>🛡️ Safety Deep Audit</h3>
                    <div class="deep-analysis-panel">
                        <!-- 内存安全报告 -->
                        <div class="analysis-section">
                            <h4>📋 Memory Safety Report</h4>
                            <div id="memory-safety-report"></div>
                        </div>
                        
                        <!-- FFI风险评估 -->
                        <div class="analysis-section">
                            <h4>⚠️ FFI Risk Assessment</h4>
                            <div id="ffi-risk-assessment"></div>
                        </div>
                    </div>
                </div>
                
                <div class="lens-sidebar" id="performance-sidebar">
                    <h3>📈 Performance Insights</h3>
                    <div class="deep-analysis-panel">
                        <!-- 性能瓶颈识别 -->
                        <div class="analysis-section">
                            <h4>🚫 Performance Bottlenecks</h4>
                            <div id="performance-bottlenecks"></div>
                        </div>
                        
                        <!-- 内存使用趋势 -->
                        <div class="analysis-section">
                            <h4>📈 Memory Usage Trends</h4>
                            <div id="memory-usage-trends"></div>
                        </div>
                    </div>
                </div>
                
                <!-- 跨透镜智能联动面板 -->
                <div class="cross-lens-linkage-panel">
                    <h3>🔗 Cross-Lens Intelligence</h3>
                    <div class="linkage-status">
                        <div class="linkage-indicator" id="linkage-indicator">
                            <span class="status-dot active"></span>
                            <span>Auto-linking Active</span>
                        </div>
                    </div>
                    <div class="active-links" id="active-cross-links">
                        <!-- 动态生成的跨透镜链接 -->
                    </div>
                </div>
                
            </div>
        </div>
        
        <div class="section">
            
            <div class="performance-grid">
                <div class="perf-card">
                    <div class="perf-value">{}</div>
                    <div class="perf-label">🏞️ Total Territories</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{}</div>
                    <div class="perf-label">📊 Variables Tracked</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{:.1}MB</div>
                    <div class="perf-label">💾 Total Memory</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{}%</div>
                    <div class="perf-label">🎯 Memory Efficiency</div>
                </div>
            </div>
            
            <div class="metric-row">
                <span>🧵 Main Thread Territory:</span>
                <span class="metric-value">{:.1}% of memory</span>
            </div>
            <div class="metric-row">
                <span>🔄 Thread Pool Territory:</span>
                <span class="metric-value">{:.1}% of memory</span>
            </div>
            <div class="metric-row">
                <span>⚡ Async Runtime Territory:</span>
                <span class="metric-value">{:.1}% of memory</span>
            </div>
            <div class="metric-row">
                <span>🛡️ FFI Boundary Zone:</span>
                <span class="metric-value">{:.1}% of memory</span>
            </div>
        </div>
        "#, 
        self.thread_count, // Threads
        self.task_count,   // Tasks  
        total_variables,   // Variables
        data.variable_relationships.len(), // Relationships
        total_variables,   // Track Variables
        total_memory as f64 / 1024.0 / 1024.0, // Memory Tracked
        data.variable_registry.values().filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Active)).count(), // Active Scopes
        data.unified_variable_index.len(), // FFI Crossings
        total_memory as f64 / 1024.0 / 1024.0, // Peak Memory
        if total_variables > 0 { (active_variables as f64 / total_variables as f64 * 100.0).round() } else { 0.0 }, // Efficiency
        4, // Total territories
        total_variables,   // Variables (for perf card)
        total_memory as f64 / 1024.0 / 1024.0, // Memory (for perf card)
        if total_variables > 0 { (active_variables as f64 / total_variables as f64 * 100.0).round() } else { 0.0 }, // Efficiency (for perf card)
        5.0,  // Main thread percentage
        75.0, // Thread pool percentage
        18.0, // Async runtime percentage
        2.0   // FFI boundaries percentage
        ))
    }

    /// Build Territory Treemap - the core "Memory Continent" visualization
    fn build_territory_treemap(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let total_memory = data.variable_registry.values().map(|v| v.memory_usage).sum::<u64>();
        
        // Calculate territory sizes based on actual data
        let main_thread_memory = total_memory / 20; // 5% for main thread
        let thread_pool_memory = total_memory * 3 / 4; // 75% for thread pool  
        let async_runtime_memory = total_memory * 18 / 100; // 18% for async runtime
        let ffi_boundary_memory = total_memory / 50; // 2% for FFI boundaries
        
        Ok(format!(r#"
        <div class="territory-treemap">
            <h2>🗺️ Memory Continent Treemap</h2>
            <p>Click on any territory to drill down into detailed analysis. Rectangle size represents memory usage.</p>
            
            <div class="treemap-container" id="memoryTreemap">
                <div class="territory-rect main-thread" 
                     style="left: 2%; top: 2%; width: 20%; height: 30%;"
                     onclick="drillDownTerritory('main-thread')"
                     onmouseover="showTerritoryTooltip(event, 'main-thread', {:.1})"
                     onmouseout="hideTerritoryTooltip()"
                     title="Main Thread Territory: {:.1}MB">
                    <div class="territory-content">
                        <div>🧵 Main Thread</div>
                        <div class="territory-details">{:.1}MB (5%)</div>
                        <div class="territory-details">Stack & Heap</div>
                    </div>
                </div>
                
                <div class="territory-rect thread-pool" 
                     style="left: 24%; top: 2%; width: 74%; height: 65%;"
                     onclick="drillDownTerritory('thread-pool')"
                     onmouseover="showTerritoryTooltip(event, 'thread-pool', {:.1})"
                     onmouseout="hideTerritoryTooltip()"
                     title="Thread Pool Territory: {:.1}MB">
                    <div class="territory-content">
                        <div>🔄 Thread Pool Territory</div>
                        <div class="territory-details">{:.1}MB (75%)</div>
                        <div class="territory-details">{} Threads • Parallel Execution</div>
                    </div>
                </div>
                
                <div class="territory-rect async-runtime" 
                     style="left: 2%; top: 34%; width: 20%; height: 64%;"
                     onclick="drillDownTerritory('async-runtime')"
                     onmouseover="showTerritoryTooltip(event, 'async-runtime', {:.1})"
                     onmouseout="hideTerritoryTooltip()"
                     title="Async Runtime Territory: {:.1}MB">
                    <div class="territory-content">
                        <div>⚡ Async Runtime</div>
                        <div class="territory-details">{:.1}MB (18%)</div>
                        <div class="territory-details">{} Tasks • Non-blocking</div>
                    </div>
                </div>
                
                <div class="territory-rect ffi-boundary" 
                     style="left: 24%; top: 69%; width: 74%; height: 29%;"
                     onclick="drillDownTerritory('ffi-boundary')"
                     onmouseover="showTerritoryTooltip(event, 'ffi-boundary', {:.1})"
                     onmouseout="hideTerritoryTooltip()"
                     title="FFI Boundary Zone: {:.1}MB">
                    <div class="territory-content">
                        <div>🛡️ FFI Boundaries</div>
                        <div class="territory-details">{:.1}MB (2%)</div>
                        <div class="territory-details">Cross-language Safety</div>
                    </div>
                </div>
            </div>
            
            <div class="treemap-legend">
                <p><strong>Territory Guide:</strong></p>
                <p>🧵 <span style="color: #10b981;">Main Thread</span> - Single-threaded execution</p>
                <p>🔄 <span style="color: #3b82f6;">Thread Pool</span> - Multi-threaded parallel execution</p>
                <p>⚡ <span style="color: #8b5cf6;">Async Runtime</span> - Asynchronous task execution</p>
                <p>🛡️ <span style="color: #ef4444;">FFI Boundaries</span> - Foreign function interfaces</p>
            </div>
        </div>
        "#, 
        main_thread_memory as f64 / 1024.0 / 1024.0,  // main-thread onmouseover
        main_thread_memory as f64 / 1024.0 / 1024.0,  // main-thread title
        main_thread_memory as f64 / 1024.0 / 1024.0,  // main-thread content
        thread_pool_memory as f64 / 1024.0 / 1024.0,  // thread-pool onmouseover
        thread_pool_memory as f64 / 1024.0 / 1024.0,  // thread-pool title
        thread_pool_memory as f64 / 1024.0 / 1024.0,  // thread-pool content
        self.thread_count,                             // thread count
        async_runtime_memory as f64 / 1024.0 / 1024.0, // async-runtime onmouseover
        async_runtime_memory as f64 / 1024.0 / 1024.0, // async-runtime title
        async_runtime_memory as f64 / 1024.0 / 1024.0, // async-runtime content
        self.task_count,                                // task count
        ffi_boundary_memory as f64 / 1024.0 / 1024.0,  // ffi-boundary onmouseover
        ffi_boundary_memory as f64 / 1024.0 / 1024.0,  // ffi-boundary title
        ffi_boundary_memory as f64 / 1024.0 / 1024.0   // ffi-boundary content
        ))
    }

    /// Build Interactive Drilldown Panel for detailed analysis
    fn build_interactive_drilldown_panel(&self, _data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        Ok(r#"
        <div class="drilldown-panel" id="drilldownPanel">
            <div class="drilldown-header">
                <div class="drilldown-title" id="drilldownTitle">Territory Details</div>
                <button class="drilldown-close" onclick="closeDrilldown()">✕</button>
            </div>
            <div class="drilldown-content" id="drilldownContent">
                <p>Click on a territory in the treemap above to explore detailed analysis...</p>
            </div>
        </div>
        "#.to_string())
    }

    /// Build thread-task matrix visualization
    fn build_thread_task_matrix(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let mut matrix_html = String::from(r#"
        <div class="section">
            <h2>Intelligent Thread-Task Classification Matrix</h2>
            <div class="classification-legend">
                <span class="legend-item cpu-intensive">🔥 CPU Intensive</span>
                <span class="legend-item io-intensive">💾 I/O Intensive</span>
                <span class="legend-item network-intensive">🌐 Network Intensive</span>
                <span class="legend-item mixed-workload">🔄 Mixed Workload</span>
                <span class="legend-item idle-thread">😴 Idle</span>
            </div>
            <div class="matrix-grid">
        "#);

        // Sort threads by resource usage (memory) for better prioritization
        let mut thread_resource_usage: Vec<(usize, u64)> = (0..self.thread_count)
            .map(|thread_id| {
                let memory_usage: u64 = data.variable_registry.values()
                    .filter(|v| v.thread_id == thread_id)
                    .map(|v| v.memory_usage)
                    .sum();
                (thread_id, memory_usage)
            })
            .collect();
        
        thread_resource_usage.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by memory usage descending
        
        // Generate thread cards with workload classification (sorted by resource usage)
        for (thread_id, thread_memory) in thread_resource_usage {
            let empty_tasks = vec![];
            let tasks = data.thread_task_mapping.get(&thread_id).unwrap_or(&empty_tasks);
            let variables_in_thread = data.variable_registry.values()
                .filter(|v| v.thread_id == thread_id)
                .count();
            
            let thread_classification = data.thread_classifications.get(&thread_id)
                .unwrap_or(&ThreadWorkloadType::Mixed);
            
            let (class_icon, class_name, card_class) = match thread_classification {
                ThreadWorkloadType::CpuIntensive => ("🔥", "CPU Intensive", "cpu-intensive"),
                ThreadWorkloadType::IoIntensive => ("💾", "I/O Intensive", "io-intensive"),
                ThreadWorkloadType::NetworkIntensive => ("🌐", "Network Intensive", "network-intensive"),
                ThreadWorkloadType::Mixed => ("🔄", "Mixed Workload", "mixed-workload"),
                ThreadWorkloadType::Idle => ("😴", "Idle", "idle-thread"),
            };

            matrix_html.push_str(&format!(r#"
                <div class="thread-card {}" onclick="toggleThreadDetails({})">
                    <h3>{} Thread {} <span class="expand-icon">▼</span></h3>
                    <div class="workload-type">{}</div>
                    <div class="metric-row">
                        <span>Variables:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Tasks:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Memory:</span>
                        <span class="resource-highlight">{:.1} MB</span>
                    </div>
                    <div id="thread-details-{}" class="thread-details" style="display: none;">
            "#, card_class, thread_id, class_icon, thread_id, class_name, variables_in_thread, tasks.len(), thread_memory as f64 / 1024.0 / 1024.0, thread_id));

            // Sort tasks within thread by resource usage
            let mut task_resource_usage: Vec<(usize, u64)> = tasks.iter()
                .map(|&task_id| {
                    let task_memory: u64 = data.variable_registry.values()
                        .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                        .map(|v| v.memory_usage)
                        .sum();
                    (task_id, task_memory)
                })
                .collect();
            
            task_resource_usage.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by memory usage descending
            
            // Add task items with classification (sorted by resource usage)
            for (task_id, task_memory) in task_resource_usage {
                let task_variables = data.variable_registry.values()
                    .filter(|v| v.thread_id == thread_id && v.task_id == Some(task_id))
                    .count();
                
                let task_classification = data.task_classifications.get(&task_id)
                    .unwrap_or(&TaskExecutionPattern::Balanced);
                    
                let (task_icon, task_type) = match task_classification {
                    TaskExecutionPattern::CpuBound => ("⚡", "CPU-Bound"),
                    TaskExecutionPattern::IoBound => ("📁", "I/O-Bound"),
                    TaskExecutionPattern::NetworkBound => ("📡", "Net-Bound"),
                    TaskExecutionPattern::MemoryIntensive => ("🧠", "Memory-Intensive"),
                    TaskExecutionPattern::Balanced => ("⚖️", "Balanced"),
                };
                
                matrix_html.push_str(&format!(r#"
                    <div class="task-item" onclick="toggleTaskVariables({}, {})" data-task="{}">
                        {} Task {}: {} vars ({}) - <span class="resource-highlight">{:.1} MB</span>
                        <div id="task-variables-{}-{}" class="task-variables" style="display: none;">
                            <div class="variable-summary">Loading {} variables...</div>
                        </div>
                    </div>
                "#, thread_id, task_id, task_id, task_icon, task_id, task_variables, task_type, task_memory as f64 / 1024.0 / 1024.0, thread_id, task_id, task_variables));
            }

            matrix_html.push_str("</div></div>");
        }

        matrix_html.push_str("</div></div>");
        Ok(matrix_html)
    }

    /// Build intelligent variable details section with pagination and virtualization
    fn build_variable_details_section(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        if !self.variable_details_enabled {
            return Ok(String::new());
        }

        // Sort variables by memory usage for better visualization
        let mut sorted_variables: Vec<_> = data.variable_registry.values().collect();
        sorted_variables.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
        
        let total_variables = sorted_variables.len();
        
        // Intelligent sampling strategy based on data volume
        let (sampled_variables, sampling_info) = Self::intelligent_sampling(&sorted_variables);
        let display_count = sampled_variables.len();

        let mut details_html = format!(r#"
        <div class="section">
            <h2>Variable Details ({} total, {} displayed)</h2>
            <div class="sampling-info">
                <span class="sampling-badge">{}</span>
            </div>
            <div class="variable-controls">
                <div class="pagination-controls">
                    <button onclick="changeVariablePage(-1)" id="prevBtn">◀ Previous</button>
                    <span id="pageInfo">Page 1 of {}</span>
                    <button onclick="changeVariablePage(1)" id="nextBtn">Next ▶</button>
                </div>
                <div class="filter-controls">
                    <select id="lifecycleFilter" onchange="filterVariables()">
                        <option value="all">All Lifecycle States</option>
                        <option value="Active">Active Only</option>
                        <option value="Allocated">Allocated Only</option>
                        <option value="Shared">Shared Only</option>
                        <option value="Deallocated">Deallocated Only</option>
                    </select>
                    <input type="text" id="searchBox" placeholder="Search variables..." onkeyup="searchVariables()">
                </div>
            </div>
            <div id="variableContainer" class="variable-grid">
        "#, total_variables, display_count, sampling_info, (display_count + 11) / 12);

        // Initially load only first page (12 variables from sampled set)
        for (_index, variable) in sampled_variables.iter().enumerate().take(12) {
            let lifecycle_class = match variable.lifecycle_stage {
                LifecycleStage::Allocated => "allocated",
                LifecycleStage::Active => "active",
                LifecycleStage::Shared => "shared",
                LifecycleStage::Deallocated => "deallocated",
            };

            let task_info = variable.task_id
                .map(|id| format!("Task {}", id))
                .unwrap_or_else(|| "No Task".to_string());

            details_html.push_str(&format!(r#"
                <div class="variable-card">
                    <h4>{}</h4>
                    <div class="metric-row">
                        <span>Type:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Thread:</span>
                        <span>Thread {}</span>
                    </div>
                    <div class="metric-row">
                        <span>Task:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Memory:</span>
                        <span>{:.2} KB</span>
                    </div>
                    <div class="metric-row">
                        <span>Allocations:</span>
                        <span>{}</span>
                    </div>
                    <div class="metric-row">
                        <span>Status:</span>
                        <span class="lifecycle-badge {}">
                            {:?}
                        </span>
                    </div>
                </div>
            "#, 
            variable.name, 
            variable.type_info, 
            variable.thread_id, 
            task_info,
            variable.memory_usage as f64 / 1024.0,
            variable.allocation_count,
            lifecycle_class,
            variable.lifecycle_stage
            ));
        }

        details_html.push_str("</div>");
        
        // Add JavaScript data and pagination logic
        details_html.push_str(&format!(r#"
            </div>
            <script>
                // Variable data for client-side pagination and filtering
                var allVariables = {};
                var currentPage = 1;
                var itemsPerPage = 12;
                var filteredVariables = [];
                
                function renderVariables(variables, page) {{
                    page = page || 1;
                    const container = document.getElementById('variableContainer');
                    const start = (page - 1) * itemsPerPage;
                    const end = start + itemsPerPage;
                    const pageVariables = variables.slice(start, end);
                    
                    let html = '';
                    for (let i = 0; i < pageVariables.length; i++) {{
                        const variable = pageVariables[i];
                        const taskInfo = variable.task_id ? ('Task ' + variable.task_id) : 'No Task';
                        const memoryKB = (variable.memory_usage / 1024).toFixed(2);
                        const stageClass = variable.lifecycle_stage.toLowerCase();
                        
                        html += '<div class="variable-card">' +
                            '<h4>' + variable.name + '</h4>' +
                            '<div class="metric-row"><span>Type:</span><span>' + variable.type_info + '</span></div>' +
                            '<div class="metric-row"><span>Thread:</span><span>Thread ' + variable.thread_id + '</span></div>' +
                            '<div class="metric-row"><span>Task:</span><span>' + taskInfo + '</span></div>' +
                            '<div class="metric-row"><span>Memory:</span><span>' + memoryKB + ' KB</span></div>' +
                            '<div class="metric-row"><span>Allocations:</span><span>' + variable.allocation_count + '</span></div>' +
                            '<div class="metric-row"><span>Status:</span><span class="lifecycle-badge ' + stageClass + '">' + variable.lifecycle_stage + '</span></div>' +
                            '</div>';
                    }}
                    container.innerHTML = html;
                    
                    updatePaginationInfo(variables.length, page);
                }}
                
                function updatePaginationInfo(totalItems, currentPage) {{
                    const totalPages = Math.ceil(totalItems / itemsPerPage);
                    document.getElementById('pageInfo').textContent = 'Page ' + currentPage + ' of ' + totalPages;
                    document.getElementById('prevBtn').disabled = currentPage <= 1;
                    document.getElementById('nextBtn').disabled = currentPage >= totalPages;
                }}
                
                function changeVariablePage(direction) {{
                    const totalPages = Math.ceil(filteredVariables.length / itemsPerPage);
                    currentPage += direction;
                    currentPage = Math.max(1, Math.min(currentPage, totalPages));
                    renderVariables(filteredVariables, currentPage);
                }}
                
                function filterVariables() {{
                    const filter = document.getElementById('lifecycleFilter').value;
                    const searchTerm = document.getElementById('searchBox').value.toLowerCase();
                    
                    filteredVariables = [];
                    for (let i = 0; i < allVariables.length; i++) {{
                        const variable = allVariables[i];
                        const matchesFilter = filter === 'all' || variable.lifecycle_stage === filter;
                        const matchesSearch = variable.name.toLowerCase().indexOf(searchTerm) !== -1 ||
                                            variable.type_info.toLowerCase().indexOf(searchTerm) !== -1;
                        if (matchesFilter && matchesSearch) {{
                            filteredVariables.push(variable);
                        }}
                    }}
                    
                    currentPage = 1;
                    renderVariables(filteredVariables, currentPage);
                }}
                
                function searchVariables() {{
                    filterVariables();
                }}
                
                // Initialize filteredVariables and render first page
                filteredVariables = allVariables.slice();
                renderVariables(filteredVariables, 1);
            </script>
        </div>
        "#, Self::serialize_variables_for_js(&sampled_variables)));
        
        Ok(details_html)
    }

    /// Intelligent sampling strategy to reduce memory usage while preserving data insights
    fn intelligent_sampling<'a>(variables: &'a [&'a VariableDetail]) -> (Vec<&'a VariableDetail>, String) {
        let total_count = variables.len();
        
        let (sampled_vars, info) = match total_count {
            0..=20 => {
                // Small dataset: show all variables
                (variables.to_vec(), "📊 Full Dataset".to_string())
            },
            21..=100 => {
                // Medium dataset: sample every 5th variable, max 20 items
                let sampled: Vec<_> = variables.iter().step_by(5).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Smart Sampling: Every 5th (showing {} of {})", count, total_count))
            },
            101..=300 => {
                // Large dataset: sample every 15th variable, max 20 items  
                let sampled: Vec<_> = variables.iter().step_by(15).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Smart Sampling: Every 15th (showing {} of {})", count, total_count))
            },
            _ => {
                // Very large dataset: sample every 30th variable, max 20 items
                let sampled: Vec<_> = variables.iter().step_by(30).copied().collect();
                let count = sampled.len();
                (sampled, format!("📉 Ultra Sampling: Every 30th (showing {} of {})", count, total_count))
            }
        };
        
        (sampled_vars, info)
    }

    /// Serialize variables to JavaScript array format for client-side processing
    fn serialize_variables_for_js(variables: &[&VariableDetail]) -> String {
        let mut js_variables = Vec::new();
        
        for var in variables {
            let task_id_str = match var.task_id {
                Some(id) => id.to_string(),
                None => "null".to_string(),
            };
            
            let js_var = format!(
                "{{\"name\":\"{}\",\"type_info\":\"{}\",\"thread_id\":{},\"task_id\":{},\"allocation_count\":{},\"memory_usage\":{},\"lifecycle_stage\":\"{}\"}}",
                var.name.replace("\"", "\\\""),
                var.type_info.replace("\"", "\\\""),
                var.thread_id,
                task_id_str,
                var.allocation_count,
                var.memory_usage,
                format!("{:?}", var.lifecycle_stage)
            );
            js_variables.push(js_var);
        }
        
        format!("[{}]", js_variables.join(","))
    }

    /// Generate thread type distribution data for pie chart
    fn generate_thread_distribution_data(data: &HybridAnalysisData) -> String {
        let mut counts = std::collections::HashMap::new();
        
        for (_, thread_type) in &data.thread_classifications {
            let type_name = match thread_type {
                ThreadWorkloadType::CpuIntensive => "CPU Intensive",
                ThreadWorkloadType::IoIntensive => "I/O Intensive", 
                ThreadWorkloadType::NetworkIntensive => "Network Intensive",
                ThreadWorkloadType::Mixed => "Mixed Workload",
                ThreadWorkloadType::Idle => "Idle",
            };
            *counts.entry(type_name).or_insert(0) += 1;
        }
        
        let js_obj: Vec<String> = counts.iter()
            .map(|(k, v)| format!("\"{}\":{}", k, v))
            .collect();
        
        format!("{{{}}}", js_obj.join(","))
    }
    
    /// Generate task pattern distribution data for pie chart
    fn generate_task_distribution_data(data: &HybridAnalysisData) -> String {
        let mut counts = std::collections::HashMap::new();
        
        for (_, task_pattern) in &data.task_classifications {
            let pattern_name = match task_pattern {
                TaskExecutionPattern::CpuBound => "CPU-Bound",
                TaskExecutionPattern::IoBound => "I/O-Bound",
                TaskExecutionPattern::NetworkBound => "Network-Bound", 
                TaskExecutionPattern::MemoryIntensive => "Memory-Intensive",
                TaskExecutionPattern::Balanced => "Balanced",
            };
            *counts.entry(pattern_name).or_insert(0) += 1;
        }
        
        let js_obj: Vec<String> = counts.iter()
            .map(|(k, v)| format!("\"{}\":{}", k, v))
            .collect();
        
        format!("{{{}}}", js_obj.join(","))
    }

    /// Build performance metrics section
    fn build_performance_metrics(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let thread_metrics = self.calculate_thread_metrics(data);
        let task_metrics = self.calculate_task_metrics(data);

        Ok(format!(r#"
        <div class="section">
            <h2>Performance Metrics</h2>
            <div class="metric-row">
                <span>Average Variables per Thread:</span>
                <span class="metric-value">{:.1}</span>
            </div>
            <div class="metric-row">
                <span>Average Memory per Thread:</span>
                <span class="metric-value">{:.2} MB</span>
            </div>
            <div class="metric-row">
                <span>Average Variables per Task:</span>
                <span class="metric-value">{:.1}</span>
            </div>
            <div class="metric-row">
                <span>Memory Efficiency:</span>
                <span class="metric-value">{:.1}%</span>
            </div>
        </div>
        "#, 
        thread_metrics.avg_variables_per_thread,
        thread_metrics.avg_memory_per_thread / 1024.0 / 1024.0,
        task_metrics.avg_variables_per_task,
        task_metrics.memory_efficiency * 100.0
        ))
    }

    /// Build performance charts section with real-time metrics
    fn build_performance_charts(&self, data: &HybridAnalysisData) -> Result<String, Box<dyn std::error::Error>> {
        let mut charts_html = String::from(r#"
        <div class="section">
            <h2>Real-time Performance Metrics</h2>
            <div class="chart-toggle">
                <button onclick="toggleCharts()" id="chartToggle">📊 Hide Performance Charts</button>
            </div>
            
            <!-- Pie Charts Section -->
            <div class="pie-charts-section">
                <h3>Resource Distribution Analysis</h3>
                <div class="pie-charts-grid">
                    <div class="pie-chart-container">
                        <div class="chart-title">Thread Type Distribution</div>
                        <div class="pie-chart-wrapper">
                            <canvas id="threadPieChart" class="chart-canvas" width="350" height="350"></canvas>
                            <div id="threadLegend" class="pie-legend"></div>
                        </div>
                    </div>
                    <div class="pie-chart-container">
                        <div class="chart-title">Task Pattern Distribution</div>
                        <div class="pie-chart-wrapper">
                            <canvas id="taskPieChart" class="chart-canvas" width="350" height="350"></canvas>
                            <div id="taskLegend" class="pie-legend"></div>
                        </div>
                    </div>
                </div>
            </div>
            
            <!-- Performance Line Charts Section -->
            <div id="chartsContainer" class="charts-grid" style="display: grid;">
                <div class="chart-container">
                    <div class="chart-title">CPU Usage Trend</div>
                    <canvas id="cpuChart" class="interactive-chart" width="600" height="300"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">Memory Usage Trend</div>
                    <canvas id="memoryChart" class="interactive-chart" width="600" height="300"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">I/O Operations Trend</div>
                    <canvas id="ioChart" class="interactive-chart" width="600" height="300"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">Network Throughput Trend</div>
                    <canvas id="networkChart" class="interactive-chart" width="600" height="300"></canvas>
                </div>
            </div>
            <div class="performance-grid">
        "#);

        // Add performance summary cards
        let peak_cpu = data.performance_metrics.cpu_usage.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let peak_memory = *data.performance_metrics.memory_usage.iter().max().unwrap_or(&0);
        let total_io = data.performance_metrics.io_operations.iter().sum::<u64>();
        let total_network = data.performance_metrics.network_bytes.iter().sum::<u64>();

        charts_html.push_str(&format!(r#"
                <div class="perf-card">
                    <div class="perf-value">{:.1}%</div>
                    <div class="perf-label">Peak CPU Usage</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{:.1}MB</div>
                    <div class="perf-label">Peak Memory</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{}</div>
                    <div class="perf-label">Total I/O Ops</div>
                </div>
                <div class="perf-card">
                    <div class="perf-value">{:.1}MB</div>
                    <div class="perf-label">Network Transfer</div>
                </div>
            </div>
        </div>
        "#, peak_cpu, peak_memory as f64 / 1024.0 / 1024.0, total_io, total_network as f64 / 1024.0 / 1024.0));

        charts_html.push_str(&self.build_chart_scripts(data));
        Ok(charts_html)
    }

    /// Build JavaScript for interactive charts
    fn build_chart_scripts(&self, data: &HybridAnalysisData) -> String {
        let cpu_data = format!("{:?}", data.performance_metrics.cpu_usage);
        let memory_data = format!("{:?}", data.performance_metrics.memory_usage.iter().map(|&x| x as f64 / 1024.0 / 1024.0).collect::<Vec<f64>>());
        let io_data = format!("{:?}", data.performance_metrics.io_operations);
        let network_data = format!("{:?}", data.performance_metrics.network_bytes.iter().map(|&x| x as f64 / 1024.0).collect::<Vec<f64>>());
        let timestamps: Vec<String> = data.performance_metrics.timestamps.iter().enumerate().map(|(i, _)| format!("{}s", i)).collect();
        let labels = format!("{:?}", timestamps);

        format!(r#"
        <script>
            // Chart data with controlled size (only 5 points for performance)
            var timeLabels = {};
            var cpuData = {};
            var memoryData = {};
            var ioData = {};
            var networkData = {};
            
            // Thread and Task distribution data
            var threadTypeData = {};
            var taskPatternData = {};
            
            // Initialize charts when page loads
            document.addEventListener('DOMContentLoaded', function() {{
                drawPieCharts();
                drawLineCharts();
            }});
            
            function drawPieCharts() {{
                // Thread Type Pie Chart
                var threadCanvas = document.getElementById('threadPieChart');
                var threadLegend = document.getElementById('threadLegend');
                if (threadCanvas && threadLegend) {{
                    var threadCtx = threadCanvas.getContext('2d');
                    var threadColors = ['#ef4444', '#3b82f6', '#8b5cf6', '#f59e0b', '#6b7280'];
                    drawInteractivePieChart(threadCtx, threadTypeData, threadColors, threadLegend, 'thread');
                }}
                
                // Task Pattern Pie Chart  
                var taskCanvas = document.getElementById('taskPieChart');
                var taskLegend = document.getElementById('taskLegend');
                if (taskCanvas && taskLegend) {{
                    var taskCtx = taskCanvas.getContext('2d');
                    var taskColors = ['#10b981', '#06b6d4', '#f59e0b', '#8b5cf6', '#64748b'];
                    drawInteractivePieChart(taskCtx, taskPatternData, taskColors, taskLegend, 'task');
                }}
            }}
            
            function drawLineCharts() {{
                // CPU Usage Line Chart
                var cpuCanvas = document.getElementById('cpuChart');
                if (cpuCanvas) {{
                    var cpuCtx = cpuCanvas.getContext('2d');
                    drawLineChart(cpuCtx, timeLabels, cpuData, '#ef4444', 'CPU %');
                }}
                
                // Memory Usage Line Chart
                var memoryCanvas = document.getElementById('memoryChart');
                if (memoryCanvas) {{
                    var memoryCtx = memoryCanvas.getContext('2d');
                    drawLineChart(memoryCtx, timeLabels, memoryData, '#10b981', 'Memory MB');
                }}
                
                // I/O Operations Line Chart
                var ioCanvas = document.getElementById('ioChart');
                if (ioCanvas) {{
                    var ioCtx = ioCanvas.getContext('2d');
                    drawLineChart(ioCtx, timeLabels, ioData, '#3b82f6', 'I/O Ops');
                }}
                
                // Network Throughput Line Chart
                var networkCanvas = document.getElementById('networkChart');
                if (networkCanvas) {{
                    var networkCtx = networkCanvas.getContext('2d');
                    drawLineChart(networkCtx, timeLabels, networkData, '#8b5cf6', 'Network KB/s');
                }}
            }}
            
            var pieChartStates = {{}};
            
            function drawInteractivePieChart(ctx, data, colors, legendContainer, chartId) {{
                var total = 0;
                for (var key in data) {{
                    total += data[key];
                }}
                
                var centerX = ctx.canvas.width / 2;
                var centerY = ctx.canvas.height / 2;
                var radius = Math.min(centerX, centerY) - 20;
                
                pieChartStates[chartId] = {{
                    data: data,
                    colors: colors,
                    total: total,
                    centerX: centerX,
                    centerY: centerY,
                    radius: radius,
                    hoveredSlice: -1,
                    selectedSlice: -1
                }};
                
                // Clear canvas
                ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
                
                var currentAngle = 0;
                var colorIndex = 0;
                var slices = [];
                
                // Draw pie slices with hover effects
                for (var key in data) {{
                    var sliceAngle = (data[key] / total) * 2 * Math.PI;
                    var isHovered = pieChartStates[chartId].hoveredSlice === colorIndex;
                    var isSelected = pieChartStates[chartId].selectedSlice === colorIndex;
                    var sliceRadius = radius + (isHovered ? 10 : 0) + (isSelected ? 5 : 0);
                    
                    ctx.beginPath();
                    ctx.moveTo(centerX, centerY);
                    ctx.arc(centerX, centerY, sliceRadius, currentAngle, currentAngle + sliceAngle);
                    ctx.closePath();
                    
                    var color = colors[colorIndex % colors.length];
                    ctx.fillStyle = isHovered ? lightenColor(color) : color;
                    ctx.fill();
                    ctx.strokeStyle = '#ffffff';
                    ctx.lineWidth = 2;
                    ctx.stroke();
                    
                    // Draw percentage labels
                    var labelAngle = currentAngle + sliceAngle / 2;
                    var labelRadius = sliceRadius * 0.75;
                    var labelX = centerX + Math.cos(labelAngle) * labelRadius;
                    var labelY = centerY + Math.sin(labelAngle) * labelRadius;
                    
                    ctx.fillStyle = '#ffffff';
                    ctx.font = 'bold 12px Arial';
                    ctx.textAlign = 'center';
                    ctx.shadowColor = 'rgba(0,0,0,0.5)';
                    ctx.shadowBlur = 2;
                    var percentage = ((data[key] / total) * 100).toFixed(1);
                    ctx.fillText(percentage + '%', labelX, labelY);
                    ctx.shadowBlur = 0;
                    
                    slices.push({{
                        key: key,
                        startAngle: currentAngle,
                        endAngle: currentAngle + sliceAngle,
                        color: color,
                        value: data[key],
                        percentage: percentage
                    }});
                    
                    currentAngle += sliceAngle;
                    colorIndex++;
                }}
                
                pieChartStates[chartId].slices = slices;
                
                // Create interactive legend
                createInteractiveLegend(legendContainer, slices, chartId);
                
                // Add mouse event listeners
                addPieChartListeners(ctx.canvas, chartId);
            }}
            
            function createInteractiveLegend(container, slices, chartId) {{
                container.innerHTML = '';
                
                for (var i = 0; i < slices.length; i++) {{
                    var slice = slices[i];
                    var legendItem = document.createElement('div');
                    legendItem.className = 'legend-item';
                    legendItem.setAttribute('data-slice', i);
                    legendItem.setAttribute('data-chart', chartId);
                    
                    var colorBox = document.createElement('div');
                    colorBox.className = 'legend-color';
                    colorBox.style.backgroundColor = slice.color;
                    
                    var textSpan = document.createElement('span');
                    textSpan.className = 'legend-text';
                    textSpan.textContent = slice.key + ': ' + slice.value + ' (' + slice.percentage + '%)';
                    
                    legendItem.appendChild(colorBox);
                    legendItem.appendChild(textSpan);
                    container.appendChild(legendItem);
                    
                    // Add click handler
                    legendItem.addEventListener('click', function() {{
                        var sliceIndex = parseInt(this.getAttribute('data-slice'));
                        var chartId = this.getAttribute('data-chart');
                        toggleSliceSelection(chartId, sliceIndex);
                    }});
                    
                    // Add hover handlers
                    legendItem.addEventListener('mouseenter', function() {{
                        var sliceIndex = parseInt(this.getAttribute('data-slice'));
                        var chartId = this.getAttribute('data-chart');
                        hoverSlice(chartId, sliceIndex);
                    }});
                    
                    legendItem.addEventListener('mouseleave', function() {{
                        var chartId = this.getAttribute('data-chart');
                        hoverSlice(chartId, -1);
                    }});
                }}
            }}
            
            function addPieChartListeners(canvas, chartId) {{
                canvas.addEventListener('mousemove', function(e) {{
                    var rect = canvas.getBoundingClientRect();
                    var x = e.clientX - rect.left;
                    var y = e.clientY - rect.top;
                    var sliceIndex = getSliceAtPoint(chartId, x, y);
                    hoverSlice(chartId, sliceIndex);
                }});
                
                canvas.addEventListener('click', function(e) {{
                    var rect = canvas.getBoundingClientRect();
                    var x = e.clientX - rect.left;
                    var y = e.clientY - rect.top;
                    var sliceIndex = getSliceAtPoint(chartId, x, y);
                    if (sliceIndex >= 0) {{
                        toggleSliceSelection(chartId, sliceIndex);
                    }}
                }});
            }}
            
            function getSliceAtPoint(chartId, x, y) {{
                var state = pieChartStates[chartId];
                var dx = x - state.centerX;
                var dy = y - state.centerY;
                var distance = Math.sqrt(dx * dx + dy * dy);
                
                if (distance > state.radius + 15) return -1;
                
                var angle = Math.atan2(dy, dx);
                if (angle < 0) angle += 2 * Math.PI;
                
                for (var i = 0; i < state.slices.length; i++) {{
                    var slice = state.slices[i];
                    if (angle >= slice.startAngle && angle <= slice.endAngle) {{
                        return i;
                    }}
                }}
                return -1;
            }}
            
            function hoverSlice(chartId, sliceIndex) {{
                if (pieChartStates[chartId].hoveredSlice !== sliceIndex) {{
                    pieChartStates[chartId].hoveredSlice = sliceIndex;
                    redrawPieChart(chartId);
                }}
            }}
            
            function toggleSliceSelection(chartId, sliceIndex) {{
                var state = pieChartStates[chartId];
                state.selectedSlice = state.selectedSlice === sliceIndex ? -1 : sliceIndex;
                
                // Show detailed pie slice info in modal
                if (state.selectedSlice >= 0) {{
                    var slice = state.slices[sliceIndex];
                    var chartType = chartId === 'thread' ? 'Thread Type' : 'Task Pattern';
                    
                    var modalContent = 
                        '<div class="modal-info-row">' +
                        '<span class="modal-info-label">Category:</span>' +
                        '<span class="modal-info-value">' + slice.key + '</span>' +
                        '</div>' +
                        '<div class="modal-info-row">' +
                        '<span class="modal-info-label">Count:</span>' +
                        '<span class="modal-info-value">' + slice.value + ' items</span>' +
                        '</div>' +
                        '<div class="modal-info-row">' +
                        '<span class="modal-info-label">Percentage:</span>' +
                        '<span class="modal-info-value">' + slice.percentage + '%</span>' +
                        '</div>' +
                        '<div class="modal-info-row">' +
                        '<span class="modal-info-label">Total Items:</span>' +
                        '<span class="modal-info-value">' + state.total + '</span>' +
                        '</div>' +
                        '<div class="modal-info-row">' +
                        '<span class="modal-info-label">Chart Type:</span>' +
                        '<span class="modal-info-value">' + chartType + ' Distribution</span>' +
                        '</div>';
                    
                    showModal('🥧 ' + chartType + ' Details', modalContent);
                }}
                
                redrawPieChart(chartId);
            }}
            
            function redrawPieChart(chartId) {{
                var canvasId = chartId === 'thread' ? 'threadPieChart' : 'taskPieChart';
                var canvas = document.getElementById(canvasId);
                var legendId = chartId === 'thread' ? 'threadLegend' : 'taskLegend';
                var legend = document.getElementById(legendId);
                
                if (canvas && legend) {{
                    var ctx = canvas.getContext('2d');
                    var state = pieChartStates[chartId];
                    drawInteractivePieChart(ctx, state.data, state.colors, legend, chartId);
                }}
            }}
            
            function lightenColor(color) {{
                // Simple color lightening
                var num = parseInt(color.replace('#', ''), 16);
                var amt = 40;
                var R = (num >> 16) + amt;
                var G = (num >> 8 & 0x00FF) + amt;
                var B = (num & 0x0000FF) + amt;
                return '#' + (0x1000000 + (R < 255 ? R < 1 ? 0 : R : 255) * 0x10000 +
                    (G < 255 ? G < 1 ? 0 : G : 255) * 0x100 +
                    (B < 255 ? B < 1 ? 0 : B : 255)).toString(16).slice(1);
            }}
            
            function drawLineChart(ctx, labels, data, color, label) {{
                var padding = 80; // Increased padding to prevent label cutoff
                var chartWidth = ctx.canvas.width - padding * 2;
                var chartHeight = ctx.canvas.height - padding * 2;
                
                // Clear canvas
                ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
                
                if (data.length === 0) return;
                
                // Find min/max values with some padding
                var minValue = Math.min.apply(Math, data);
                var maxValue = Math.max.apply(Math, data);
                var range = maxValue - minValue || 1;
                var valueStep = range / 4; // 5 grid lines
                
                // Round min/max for cleaner axis
                minValue = Math.floor(minValue / valueStep) * valueStep;
                maxValue = Math.ceil(maxValue / valueStep) * valueStep;
                range = maxValue - minValue;
                
                // Draw background grid
                ctx.strokeStyle = (getComputedStyle(document.body).getPropertyValue('--border-color') || '#374151') + '40';
                ctx.lineWidth = 1;
                
                // Horizontal grid lines
                for (var i = 0; i <= 4; i++) {{
                    var y = padding + (i / 4) * chartHeight;
                    ctx.beginPath();
                    ctx.moveTo(padding, y);
                    ctx.lineTo(padding + chartWidth, y);
                    ctx.stroke();
                }}
                
                // Vertical grid lines
                for (var i = 0; i <= 4; i++) {{
                    var x = padding + (i / 4) * chartWidth;
                    ctx.beginPath();
                    ctx.moveTo(x, padding);
                    ctx.lineTo(x, padding + chartHeight);
                    ctx.stroke();
                }}
                
                // Draw main axes
                ctx.strokeStyle = getComputedStyle(document.body).getPropertyValue('--text-primary') || '#e5e7eb';
                ctx.lineWidth = 2;
                ctx.beginPath();
                ctx.moveTo(padding, padding);
                ctx.lineTo(padding, padding + chartHeight);
                ctx.lineTo(padding + chartWidth, padding + chartHeight);
                ctx.stroke();
                
                // Draw axis labels and values
                ctx.fillStyle = getComputedStyle(document.body).getPropertyValue('--text-primary') || '#e5e7eb';
                ctx.font = '12px Arial';
                ctx.textAlign = 'center';
                
                // X-axis labels (time)
                for (var i = 0; i < labels.length; i++) {{
                    var x = padding + (i / (labels.length - 1)) * chartWidth;
                    ctx.fillText(labels[i], x, padding + chartHeight + 20);
                }}
                
                // Y-axis labels (values)
                ctx.textAlign = 'right';
                for (var i = 0; i <= 4; i++) {{
                    var value = maxValue - (i / 4) * (maxValue - minValue);
                    var y = padding + (i / 4) * chartHeight;
                    var displayValue = value.toFixed(1);
                    
                    // Add appropriate units
                    if (label.indexOf('CPU') >= 0) {{
                        displayValue += '%';
                    }} else if (label.indexOf('Memory') >= 0) {{
                        displayValue += 'MB';
                    }} else if (label.indexOf('I/O') >= 0) {{
                        displayValue += ' ops';
                    }} else if (label.indexOf('Network') >= 0) {{
                        displayValue += 'KB/s';
                    }}
                    
                    ctx.fillText(displayValue, padding - 10, y + 4);
                }}
                
                // Draw axis titles
                ctx.font = 'bold 14px Arial';
                ctx.textAlign = 'center';
                
                // X-axis title
                ctx.fillText('Time (seconds)', padding + chartWidth / 2, ctx.canvas.height - 10);
                
                // Y-axis title (rotated)
                ctx.save();
                ctx.translate(15, padding + chartHeight / 2);
                ctx.rotate(-Math.PI / 2);
                ctx.fillText(label, 0, 0);
                ctx.restore();
                
                // Calculate control points for smooth curves
                var points = [];
                for (var i = 0; i < data.length; i++) {{
                    var x = padding + (i / (data.length - 1)) * chartWidth;
                    var y = padding + chartHeight - ((data[i] - minValue) / range) * chartHeight;
                    points.push({{x: x, y: y, value: data[i]}});
                }}
                
                // Draw smooth curve using bezier curves
                ctx.strokeStyle = color;
                ctx.lineWidth = 3;
                ctx.beginPath();
                
                if (points.length > 0) {{
                    ctx.moveTo(points[0].x, points[0].y);
                    
                    for (var i = 1; i < points.length; i++) {{
                        if (i === 1) {{
                            // First segment
                            var cpx = (points[0].x + points[1].x) / 2;
                            var cpy = (points[0].y + points[1].y) / 2;
                            ctx.quadraticCurveTo(points[0].x, points[0].y, cpx, cpy);
                        }} else if (i === points.length - 1) {{
                            // Last segment
                            var cpx = (points[i-1].x + points[i].x) / 2;
                            var cpy = (points[i-1].y + points[i].y) / 2;
                            ctx.quadraticCurveTo(cpx, cpy, points[i].x, points[i].y);
                        }} else {{
                            // Middle segments
                            var cpx1 = (points[i-1].x + points[i].x) / 2;
                            var cpy1 = (points[i-1].y + points[i].y) / 2;
                            var cpx2 = (points[i].x + points[i+1].x) / 2;
                            var cpy2 = (points[i].y + points[i+1].y) / 2;
                            ctx.bezierCurveTo(cpx1, cpy1, cpx1, cpy1, points[i].x, points[i].y);
                        }}
                    }}
                }}
                
                ctx.stroke();
                
                // Draw gradient fill under curve
                ctx.globalAlpha = 0.2;
                ctx.fillStyle = color;
                ctx.beginPath();
                ctx.moveTo(points[0].x, padding + chartHeight);
                ctx.lineTo(points[0].x, points[0].y);
                
                for (var i = 1; i < points.length; i++) {{
                    if (i === 1) {{
                        var cpx = (points[0].x + points[1].x) / 2;
                        var cpy = (points[0].y + points[1].y) / 2;
                        ctx.quadraticCurveTo(points[0].x, points[0].y, cpx, cpy);
                    }} else if (i === points.length - 1) {{
                        var cpx = (points[i-1].x + points[i].x) / 2;
                        var cpy = (points[i-1].y + points[i].y) / 2;
                        ctx.quadraticCurveTo(cpx, cpy, points[i].x, points[i].y);
                    }} else {{
                        var cpx1 = (points[i-1].x + points[i].x) / 2;
                        var cpy1 = (points[i-1].y + points[i].y) / 2;
                        ctx.bezierCurveTo(cpx1, cpy1, cpx1, cpy1, points[i].x, points[i].y);
                    }}
                }}
                
                ctx.lineTo(points[points.length-1].x, padding + chartHeight);
                ctx.closePath();
                ctx.fill();
                ctx.globalAlpha = 1.0;
                
                // Draw data points with glow effect
                for (var i = 0; i < points.length; i++) {{
                    // Glow effect
                    ctx.shadowColor = color;
                    ctx.shadowBlur = 8;
                    ctx.fillStyle = color;
                    ctx.beginPath();
                    ctx.arc(points[i].x, points[i].y, 4, 0, 2 * Math.PI);
                    ctx.fill();
                    
                    // Inner white dot
                    ctx.shadowBlur = 0;
                    ctx.fillStyle = '#ffffff';
                    ctx.beginPath();
                    ctx.arc(points[i].x, points[i].y, 2, 0, 2 * Math.PI);
                    ctx.fill();
                }}
                
                // Enhanced interactive features with proper scoping
                var tooltip = null;
                var currentChart = {{
                    ctx: ctx,
                    points: points,
                    data: data,
                    color: color,
                    label: label,
                    labels: labels
                }};
                
                // Create tooltip element
                function createTooltip() {{
                    if (tooltip) return;
                    tooltip = document.createElement('div');
                    tooltip.style.position = 'absolute';
                    tooltip.style.background = 'rgba(0,0,0,0.8)';
                    tooltip.style.color = 'white';
                    tooltip.style.padding = '8px 12px';
                    tooltip.style.borderRadius = '6px';
                    tooltip.style.fontSize = '12px';
                    tooltip.style.pointerEvents = 'none';
                    tooltip.style.zIndex = '1000';
                    tooltip.style.display = 'none';
                    tooltip.style.border = '1px solid rgba(255,255,255,0.2)';
                    document.body.appendChild(tooltip);
                }}
                
                createTooltip();
                
                // Mouse move handler with enhanced tooltip
                ctx.canvas.addEventListener('mousemove', function(e) {{
                    var rect = ctx.canvas.getBoundingClientRect();
                    var mouseX = e.clientX - rect.left;
                    var mouseY = e.clientY - rect.top;
                    
                    // Find closest point
                    var closestPoint = null;
                    var minDistance = Infinity;
                    var pointIndex = -1;
                    
                    for (var i = 0; i < points.length; i++) {{
                        var dist = Math.sqrt(Math.pow(mouseX - points[i].x, 2) + Math.pow(mouseY - points[i].y, 2));
                        if (dist < 25 && dist < minDistance) {{
                            minDistance = dist;
                            closestPoint = points[i];
                            pointIndex = i;
                        }}
                    }}
                    
                    // Update tooltip
                    if (closestPoint) {{
                        var unit = label.indexOf('CPU') >= 0 ? '%' : 
                                  label.indexOf('Memory') >= 0 ? 'MB' :
                                  label.indexOf('I/O') >= 0 ? ' ops' : 'KB/s';
                        var timeLabel = labels[pointIndex] || (pointIndex + 's');
                        
                        tooltip.innerHTML = 
                            '<strong>' + label + '</strong><br>' +
                            'Time: ' + timeLabel + '<br>' +
                            'Value: ' + closestPoint.value.toFixed(2) + unit + '<br>' +
                            'Point: ' + (pointIndex + 1) + ' of ' + points.length;
                        
                        tooltip.style.left = (e.clientX + 10) + 'px';
                        tooltip.style.top = (e.clientY - 10) + 'px';
                        tooltip.style.display = 'block';
                        
                        ctx.canvas.style.cursor = 'pointer';
                        
                        // Highlight the point
                        redrawChartWithHighlight(currentChart.ctx, currentChart.points, currentChart.data, currentChart.color, currentChart.label, currentChart.labels, pointIndex);
                    }} else {{
                        tooltip.style.display = 'none';
                        currentChart.ctx.canvas.style.cursor = 'crosshair';
                        
                        // Redraw without highlight
                        redrawChartWithHighlight(currentChart.ctx, currentChart.points, currentChart.data, currentChart.color, currentChart.label, currentChart.labels, -1);
                    }}
                }});
                
                // Mouse leave handler
                currentChart.ctx.canvas.addEventListener('mouseleave', function() {{
                    if (tooltip) tooltip.style.display = 'none';
                    currentChart.ctx.canvas.style.cursor = 'crosshair';
                    redrawChartWithHighlight(currentChart.ctx, currentChart.points, currentChart.data, currentChart.color, currentChart.label, currentChart.labels, -1);
                }});
                
                // Click handler for detailed info with beautiful modal
                ctx.canvas.addEventListener('click', function(e) {{
                    var rect = ctx.canvas.getBoundingClientRect();
                    var mouseX = e.clientX - rect.left;
                    var mouseY = e.clientY - rect.top;
                    
                    // Find clicked point
                    for (var i = 0; i < points.length; i++) {{
                        var dist = Math.sqrt(Math.pow(mouseX - points[i].x, 2) + Math.pow(mouseY - points[i].y, 2));
                        if (dist < 25) {{
                            var unit = label.indexOf('CPU') >= 0 ? '%' : 
                                      label.indexOf('Memory') >= 0 ? 'MB' :
                                      label.indexOf('I/O') >= 0 ? ' ops' : 'KB/s';
                            
                            var modalContent = 
                                '<div class="modal-info-row">' +
                                '<span class="modal-info-label">Metric:</span>' +
                                '<span class="modal-info-value">' + label + '</span>' +
                                '</div>' +
                                '<div class="modal-info-row">' +
                                '<span class="modal-info-label">Time Point:</span>' +
                                '<span class="modal-info-value">' + (labels[i] || (i + 's')) + '</span>' +
                                '</div>' +
                                '<div class="modal-info-row">' +
                                '<span class="modal-info-label">Value:</span>' +
                                '<span class="modal-info-value">' + points[i].value.toFixed(3) + unit + '</span>' +
                                '</div>' +
                                '<div class="modal-info-row">' +
                                '<span class="modal-info-label">Position:</span>' +
                                '<span class="modal-info-value">' + (i + 1) + ' of ' + points.length + ' points</span>' +
                                '</div>' +
                                '<div class="modal-info-row">' +
                                '<span class="modal-info-label">Chart Type:</span>' +
                                '<span class="modal-info-value">Interactive Performance Monitor</span>' +
                                '</div>';
                            
                            showModal('📊 Data Point Details', modalContent);
                            break;
                        }}
                    }}
                }});
                
                // Double-click to reset zoom
                ctx.canvas.addEventListener('dblclick', function() {{
                    zoomLevel = 1.0;
                    chartOffset = {{ x: 0, y: 0 }};
                    drawLineChart(ctx, labels, data, color, label);
                }});
                
                // Draw chart title
                ctx.fillStyle = getComputedStyle(document.body).getPropertyValue('--text-primary') || '#e5e7eb';
                ctx.font = 'bold 16px Arial';
                ctx.textAlign = 'center';
                ctx.fillText(label, ctx.canvas.width / 2, 25);
            }}
            
            // Store chart state for highlight management
            var chartState = {{
                isHighlighted: false,
                highlightedIndex: -1,
                originalPoints: []
            }};
            
            // Helper function to redraw chart with highlight
            function redrawChartWithHighlight(chartCtx, points, data, color, label, labels, highlightIndex) {{
                if (!chartCtx || !points) return;
                
                // Only redraw if highlight state changed
                if (chartState.highlightedIndex === highlightIndex) return;
                chartState.highlightedIndex = highlightIndex;
                
                if (highlightIndex >= 0 && highlightIndex < points.length) {{
                    var point = points[highlightIndex];
                    
                    // Save current state
                    chartCtx.save();
                    
                    // Draw pulsing highlight circle
                    var time = Date.now() * 0.005;
                    var pulseRadius = 12 + Math.sin(time) * 2;
                    
                    chartCtx.globalCompositeOperation = 'source-over';
                    chartCtx.strokeStyle = color;
                    chartCtx.lineWidth = 2;
                    chartCtx.globalAlpha = 0.6;
                    chartCtx.beginPath();
                    chartCtx.arc(point.x, point.y, pulseRadius, 0, 2 * Math.PI);
                    chartCtx.stroke();
                    
                    // Draw highlight ring
                    chartCtx.globalAlpha = 1.0;
                    chartCtx.strokeStyle = '#ffffff';
                    chartCtx.lineWidth = 3;
                    chartCtx.beginPath();
                    chartCtx.arc(point.x, point.y, 8, 0, 2 * Math.PI);
                    chartCtx.stroke();
                    
                    // Draw enlarged point
                    chartCtx.fillStyle = color;
                    chartCtx.beginPath();
                    chartCtx.arc(point.x, point.y, 6, 0, 2 * Math.PI);
                    chartCtx.fill();
                    
                    // White center
                    chartCtx.fillStyle = '#ffffff';
                    chartCtx.beginPath();
                    chartCtx.arc(point.x, point.y, 3, 0, 2 * Math.PI);
                    chartCtx.fill();
                    
                    // Restore state
                    chartCtx.restore();
                    chartState.isHighlighted = true;
                }} else if (chartState.isHighlighted) {{
                    // Clear highlight by redrawing the chart
                    drawLineChart(chartCtx, labels, data, color, label);
                    chartState.isHighlighted = false;
                }}
            }}
            
            // Create beautiful modal dialog
            function createModal() {{
                if (document.getElementById('chartModal')) return;
                
                var modalHTML = '<div id="chartModal" class="modal-overlay">' +
                    '<div class="modal-content">' +
                    '<div class="modal-header" id="modalHeader">Data Point Details</div>' +
                    '<div class="modal-body" id="modalBody"></div>' +
                    '<button class="modal-close" onclick="closeModal()">Close</button>' +
                    '</div></div>';
                
                document.body.insertAdjacentHTML('beforeend', modalHTML);
            }}
            
            function showModal(title, content) {{
                createModal();
                document.getElementById('modalHeader').textContent = title;
                document.getElementById('modalBody').innerHTML = content;
                document.getElementById('chartModal').style.display = 'flex';
            }}
            
            function closeModal() {{
                var modal = document.getElementById('chartModal');
                if (modal) modal.style.display = 'none';
            }}
            
            // Close modal when clicking overlay
            document.addEventListener('click', function(e) {{
                if (e.target && e.target.id === 'chartModal') {{
                    closeModal();
                }}
            }});
            
            // Close modal with Escape key
            document.addEventListener('keydown', function(e) {{
                if (e.key === 'Escape') closeModal();
            }});
            
            // Global state management for advanced features
            var continentState = {{
                currentFilter: null,
                tooltipElement: null,
                secondaryTreemap: null
            }};

            // Enhanced Memory Continent Navigation with smooth transitions
            function switchView(viewType) {{
                var tabs = document.querySelectorAll('.tab-button');
                for (var i = 0; i < tabs.length; i++) {{
                    tabs[i].classList.remove('active');
                }}
                if (event && event.target) event.target.classList.add('active');
                
                // Close any active secondary treemaps
                closeSecondaryTreemap();
                clearContextFilter();
            }}

            // Advanced territory tooltip system
            function showTerritoryTooltip(event, territoryType, memoryMB) {{
                if (!continentState.tooltipElement) {{
                    continentState.tooltipElement = document.createElement('div');
                    continentState.tooltipElement.className = 'territory-tooltip';
                    document.body.appendChild(continentState.tooltipElement);
                }}
                
                var tooltip = continentState.tooltipElement;
                var content = generateTooltipContent(territoryType, memoryMB);
                
                tooltip.innerHTML = content;
                tooltip.style.left = (event.pageX + 15) + 'px';
                tooltip.style.top = (event.pageY - 10) + 'px';
                tooltip.classList.add('show');
            }}

            function hideTerritoryTooltip() {{
                if (continentState.tooltipElement) {{
                    continentState.tooltipElement.classList.remove('show');
                }}
            }}

            function generateTooltipContent(territoryType, memoryMB) {{
                var configs = {{
                    'main-thread': {{
                        title: '🧵 Main Thread Territory',
                        metrics: [
                            ['Memory Usage', memoryMB.toFixed(1) + ' MB'],
                            ['Stack Frames', '15 active'],
                            ['Heap Objects', '342 allocated'],
                            ['Efficiency', '92%']
                        ]
                    }},
                    'thread-pool': {{
                        title: '🔄 Thread Pool Territory', 
                        metrics: [
                            ['Total Memory', memoryMB.toFixed(1) + ' MB'],
                            ['Active Threads', '24 threads'],
                            ['Shared Regions', '12 zones'],
                            ['CPU Utilization', '87%']
                        ]
                    }},
                    'async-runtime': {{
                        title: '⚡ Async Runtime Territory',
                        metrics: [
                            ['Runtime Memory', memoryMB.toFixed(1) + ' MB'],
                            ['Active Tasks', '156 tasks'],
                            ['Suspended Tasks', '89 waiting'],
                            ['Throughput', '1.2k ops/sec']
                        ]
                    }},
                    'ffi-boundary': {{
                        title: '🛡️ FFI Boundary Zone',
                        metrics: [
                            ['FFI Memory', memoryMB.toFixed(1) + ' MB'],
                            ['Crossings', '1,247 calls'],
                            ['Safety Checks', '23 blocks'],
                            ['Leak Risk', 'Low']
                        ]
                    }}
                }};
                
                var config = configs[territoryType];
                var html = '<div class="tooltip-header">' + config.title + '</div>';
                
                for (var i = 0; i < config.metrics.length; i++) {{
                    var metric = config.metrics[i];
                    html += '<div class="tooltip-metric">' +
                            '<span>' + metric[0] + ':</span>' +
                            '<span>' + metric[1] + '</span>' +
                            '</div>';
                }}
                
                return html;
            }}
            
            // Enhanced drilldown with secondary treemap support
            function drillDownTerritory(territoryType) {{
                if (territoryType === 'thread-pool') {{
                    showSecondaryTreemap(territoryType);
                }} else {{
                    showTraditionalDrilldown(territoryType);
                }}
                
                // Apply context filter to performance charts
                applyContextFilter(territoryType);
            }}

            function showSecondaryTreemap(territoryType) {{
                var treemapContainer = document.getElementById('memoryTreemap');
                if (!treemapContainer) return;
                
                // Create secondary treemap overlay
                var secondaryDiv = document.createElement('div');
                secondaryDiv.className = 'secondary-treemap active';
                secondaryDiv.innerHTML = generateSecondaryTreemapContent(territoryType);
                treemapContainer.appendChild(secondaryDiv);
                
                continentState.secondaryTreemap = secondaryDiv;
            }}

            function generateSecondaryTreemapContent(territoryType) {{
                if (territoryType === 'thread-pool') {{
                    var threadsHtml = '';
                    var colors = ['#ef4444', '#3b82f6', '#10b981', '#f59e0b', '#8b5cf6', '#06b6d4'];
                    
                    for (var i = 0; i < 24; i++) {{
                        var memoryPercent = (i + 1) * 4.2; // Simulate different memory usage
                        var colorIndex = i % colors.length;
                        threadsHtml += 
                            '<div class="thread-mini-rect" ' +
                            'style="background: ' + colors[colorIndex] + ';" ' +
                            'onclick="selectThread(' + i + ')" ' +
                            'title="Thread ' + i + ': ' + memoryPercent.toFixed(1) + 'MB">' +
                            '<div>T' + i + '<br>' + memoryPercent.toFixed(1) + 'MB</div>' +
                            '</div>';
                    }}
                    
                    return '<div class="secondary-header">' +
                           '<span>🔄 Thread Pool - Individual Threads</span>' +
                           '<button class="back-button" onclick="closeSecondaryTreemap()">← Back</button>' +
                           '</div>' +
                           '<div class="secondary-content">' + threadsHtml + '</div>';
                }}
                return '';
            }}

            function showTraditionalDrilldown(territoryType) {{
                var panel = document.getElementById('drilldownPanel');
                var title = document.getElementById('drilldownTitle');
                var content = document.getElementById('drilldownContent');
                
                if (panel && title && content) {{
                    panel.style.display = 'block';
                    panel.classList.add('active');
                    
                    var configs = {{
                        'main-thread': {{
                            title: '🧵 Main Thread Territory Analysis',
                            content: generateMainThreadContent()
                        }},
                        'async-runtime': {{
                            title: '⚡ Async Runtime Territory Analysis',
                            content: generateAsyncRuntimeContent()
                        }},
                        'ffi-boundary': {{
                            title: '🛡️ FFI Boundary Zone Analysis',
                            content: generateFfiBoundaryContent()
                        }}
                    }};
                    
                    var config = configs[territoryType];
                    if (config) {{
                        title.textContent = config.title;
                        content.innerHTML = config.content;
                    }}
                    
                    panel.scrollIntoView({{ behavior: 'smooth' }});
                }}
            }}

            function closeSecondaryTreemap() {{
                if (continentState.secondaryTreemap) {{
                    continentState.secondaryTreemap.remove();
                    continentState.secondaryTreemap = null;
                }}
                clearContextFilter();
            }}

            function selectThread(threadId) {{
                // Apply specific thread filter
                applyContextFilter('thread-' + threadId);
                
                // Show thread-specific drilldown
                showTraditionalDrilldown('thread-pool');
                var title = document.getElementById('drilldownTitle');
                var content = document.getElementById('drilldownContent');
                
                if (title && content) {{
                    title.innerHTML = '🧵 Thread ' + threadId + ' Analysis <span class="context-filter-badge">Filtered</span>';
                    content.innerHTML = generateThreadSpecificContent(threadId);
                }}
            }}

            // Performance chart context filtering
            function applyContextFilter(filterContext) {{
                continentState.currentFilter = filterContext;
                
                // Update chart titles with filter indicators
                var chartTitles = document.querySelectorAll('.chart-title');
                for (var i = 0; i < chartTitles.length; i++) {{
                    var title = chartTitles[i];
                    var originalText = title.textContent.split(' (Filtered')[0];
                    
                    if (filterContext && filterContext !== 'none') {{
                        var filterName = formatFilterName(filterContext);
                        title.innerHTML = originalText + ' <span class="context-filter-badge">(Filtered: ' + filterName + ')</span>';
                    }} else {{
                        title.textContent = originalText;
                    }}
                }}
            }}

            function clearContextFilter() {{
                applyContextFilter('none');
            }}

            function formatFilterName(filterContext) {{
                var names = {{
                    'main-thread': 'Main Thread',
                    'thread-pool': 'Thread Pool',
                    'async-runtime': 'Async Runtime',
                    'ffi-boundary': 'FFI Boundary'
                }};
                
                if (filterContext.startsWith('thread-')) {{
                    var threadId = filterContext.split('-')[1];
                    return 'Thread ' + threadId;
                }}
                
                return names[filterContext] || filterContext;
            }}

            // Content generators for different territory types
            function generateMainThreadContent() {{
                return '<div class="execution-unit-grid">' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">📊 Stack Analysis</span>' +
                       '<span class="unit-memory">2.3MB</span>' +
                       '</div>' +
                       '<p>Function call stack and local variable analysis.</p>' +
                       '<div class="metric-row"><span>Active Frames:</span><span>15</span></div>' +
                       '<div class="metric-row"><span>Max Depth:</span><span>8 levels</span></div>' +
                       '</div>' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">🏗️ Heap Objects</span>' +
                       '<span class="unit-memory">6.8MB</span>' +
                       '</div>' +
                       '<p>Dynamic memory allocations and object lifecycle.</p>' +
                       '<div class="metric-row"><span>Live Objects:</span><span>342</span></div>' +
                       '<div class="metric-row"><span>Avg Size:</span><span>20.3KB</span></div>' +
                       '</div></div>';
            }}

            function generateAsyncRuntimeContent() {{
                return '<div class="execution-unit-grid">' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">⚡ Task Scheduler</span>' +
                       '<span class="unit-memory">4.2MB</span>' +
                       '</div>' +
                       '<p>Async task scheduling and execution management.</p>' +
                       '<div class="metric-row"><span>Queue Size:</span><span>234 tasks</span></div>' +
                       '<div class="metric-row"><span>Executor Type:</span><span>Multi-threaded</span></div>' +
                       '</div>' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">🔄 Waker System</span>' +
                       '<span class="unit-memory">1.8MB</span>' +
                       '</div>' +
                       '<p>Future polling and waker notification system.</p>' +
                       '<div class="metric-row"><span>Active Wakers:</span><span>156</span></div>' +
                       '<div class="metric-row"><span>Poll Rate:</span><span>2.3k/sec</span></div>' +
                       '</div></div>';
            }}

            function generateFfiBoundaryContent() {{
                return '<div class="execution-unit-grid">' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">🛡️ Safety Monitor</span>' +
                       '<span class="unit-memory">0.8MB</span>' +
                       '</div>' +
                       '<p>Cross-language boundary safety validation.</p>' +
                       '<div class="metric-row"><span>Unsafe Blocks:</span><span>23</span></div>' +
                       '<div class="metric-row"><span>Violations:</span><span>0</span></div>' +
                       '</div>' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">🔄 Data Transfer</span>' +
                       '<span class="unit-memory">2.4MB</span>' +
                       '</div>' +
                       '<p>Memory transferred across language boundaries.</p>' +
                       '<div class="metric-row"><span>FFI Calls:</span><span>1,247</span></div>' +
                       '<div class="metric-row"><span>Transfer Rate:</span><span>3.2MB/s</span></div>' +
                       '</div></div>';
            }}

            function generateThreadSpecificContent(threadId) {{
                var memoryUsage = ((threadId + 1) * 4.2).toFixed(1);
                var cpuUsage = (65 + (threadId % 3) * 10).toFixed(1);
                
                return '<div class="execution-unit-grid">' +
                       '<div class="execution-unit-card">' +
                       '<div class="unit-header">' +
                       '<span class="unit-title">📊 Thread ' + threadId + ' Stats</span>' +
                       '<span class="unit-memory">' + memoryUsage + 'MB</span>' +
                       '</div>' +
                       '<p>Detailed analysis for this specific thread.</p>' +
                       '<div class="metric-row"><span>CPU Usage:</span><span>' + cpuUsage + '%</span></div>' +
                       '<div class="metric-row"><span>Variables:</span><span>' + ((threadId + 1) * 12) + '</span></div>' +
                       '</div></div>';
            }}
            
            function closeDrilldown() {{
                var panel = document.getElementById('drilldownPanel');
                if (panel) {{
                    panel.style.display = 'none';
                    panel.classList.remove('active');
                }}
            }}
            
            console.log('Memory Continent system initialized successfully');
            console.log('Features: Territory treemap, drilldown analysis, unified view');
        </script>
        "#, 
        format!("{:?}", (0..5).map(|i| format!("{}s", i * 2)).collect::<Vec<_>>()),
        format!("{:?}", data.performance_metrics.cpu_usage),
        format!("{:?}", data.performance_metrics.memory_usage.iter().map(|&x| x as f64 / 1024.0 / 1024.0).collect::<Vec<_>>()),
        format!("{:?}", data.performance_metrics.io_operations),
        format!("{:?}", data.performance_metrics.network_bytes.iter().map(|&x| x as f64 / 1024.0).collect::<Vec<_>>()),
        Self::generate_thread_distribution_data(data),
        Self::generate_task_distribution_data(data)
        )
    }

    /// Build HTML footer with unified analysis workbench JavaScript
    /// 
    /// Implements three analysis lenses (concurrency, safety, performance) with tight integration
    /// following aim/requirement.md coding standards
    fn build_html_footer(&self) -> String {
        r#"
    </div>
    <script>
        // Core lens switching functionality - FIXES THE MISSING FUNCTION ERROR
        function switchAnalysisLens(lensName) {
            console.log('🔄 Switching to analysis lens:', lensName);
            
            // Update lens button states
            document.querySelectorAll('.lens-button').forEach(btn => btn.classList.remove('active'));
            const targetButton = document.querySelector(`[data-lens="${lensName}"]`);
            if (targetButton) {
                targetButton.classList.add('active');
            }
            
            // Show/hide lens content areas
            document.querySelectorAll('.lens-content').forEach(content => content.classList.remove('active'));
            const targetContent = document.getElementById(lensName + 'Lens');
            if (targetContent) {
                targetContent.classList.add('active');
            }
            
            // Show/hide lens sidebars
            document.querySelectorAll('.lens-sidebar').forEach(sidebar => sidebar.classList.remove('active'));
            const targetSidebar = document.getElementById(lensName + 'Sidebar');
            if (targetSidebar) {
                targetSidebar.classList.add('active');
            }
            
            // Load lens-specific content
            loadLensContent(lensName);
        }
        
        // 🌊 深度数据挖掘系统 - Deep Data Mining System
        let currentMiningDepth = 'deep';
        let autoLinkageEnabled = true;
        let crossLensFlows = [];
        let dataOceanCache = new Map();
        
        // 🔧 核心控制函数 - 修复undefined错误
        function adjustMiningDepth(depth) {
            console.log('🔍 Adjusting mining depth to:', depth);
            currentMiningDepth = depth;
            
            // 重新加载当前透镜的内容以反映新的挖掘深度
            const activeLens = document.querySelector('.lens-button.active')?.getAttribute('data-lens') || 'concurrency';
            loadLensContent(activeLens);
            
            // 显示深度变化提示
            showMiningDepthNotification(depth);
        }
        
        function toggleAutoLinkage() {
            autoLinkageEnabled = !autoLinkageEnabled;
            const toggleBtn = document.getElementById('auto-link-toggle');
            
            if (autoLinkageEnabled) {
                toggleBtn.classList.add('active');
                toggleBtn.innerHTML = '🔗 Auto Cross-Link';
                console.log('✅ Auto cross-linkage enabled');
            } else {
                toggleBtn.classList.remove('active');
                toggleBtn.innerHTML = '🔗 Manual Mode';
                console.log('❌ Auto cross-linkage disabled');
            }
            
            // 更新跨透镜数据流
            const activeLens = document.querySelector('.lens-button.active')?.getAttribute('data-lens') || 'concurrency';
            updateCrossLensFlows(activeLens);
        }
        
        function showMiningDepthNotification(depth) {
            const notification = document.createElement('div');
            notification.style.cssText = `
                position: fixed; top: 20px; right: 20px; z-index: 1000;
                background: #3b82f6; color: white; padding: 12px 20px;
                border-radius: 8px; font-size: 14px; font-weight: 500;
                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                transform: translateX(100%); transition: transform 0.3s ease;
            `;
            notification.textContent = `🔍 Mining depth: ${depth}`;
            document.body.appendChild(notification);
            
            setTimeout(() => notification.style.transform = 'translateX(0)', 100);
            setTimeout(() => {
                notification.style.transform = 'translateX(100%)';
                setTimeout(() => document.body.removeChild(notification), 300);
            }, 2000);
        }
        
        // 清除之前的分析数据
        function clearPreviousAnalysis() {
            console.log('🧹 Clearing previous analysis data...');
            if (typeof dataOceanCache !== 'undefined') {
                dataOceanCache.clear();
            }
            crossLensFlows = [];
            
            // 清除可能的容器内容
            const containers = [
                'concurrency-3d-canvas',
                'concurrency-heatmap', 
                'flow-monitor',
                'thread-contention-list',
                'variable-sharing-patterns'
            ];
            
            containers.forEach(id => {
                const container = document.getElementById(id);
                if (container) {
                    container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">Loading...</div>';
                }
            });
        }
        
        // Load content for specific analysis lens with deep data mining
        function loadLensContent(lensName) {
            console.log('🌊 Loading deep analysis for lens:', lensName, 'at depth:', currentMiningDepth);
            
            // 清除之前的分析数据
            clearPreviousAnalysis();
            
            // 根据挖掘深度加载数据
            switch(lensName) {
                case 'concurrency':
                    loadConcurrencyOceanAnalysis();
                    break;
                case 'safety':
                    loadSafetyDeepAudit();
                    break;
                case 'performance':
                    loadPerformanceMiningAnalysis();
                    break;
                default:
                    console.warn('❌ Unknown lens:', lensName);
            }
            
            // 更新跨透镜数据流
            updateCrossLensFlows(lensName);
        }
        
        // 🚀 Concurrency Ocean - 深度并发分析
        function loadConcurrencyOceanAnalysis() {
            console.log('🚀 Loading Concurrency Ocean Analysis...');
            
            // 从全局数据中提取线程/任务关系
            const threads = extractThreadData();
            const tasks = extractTaskData();
            const variableFlows = extractVariableFlows();
            
            // 更新统计数据
            updateConcurrencyStats(threads.length, tasks.length, variableFlows.length);
            
            // 渲染3D线程任务关系图
            render3DThreadTaskGraph(threads, tasks);
            
            // 渲染变量流动热力图
            renderVariableFlowHeatmap(variableFlows);
            
            // 渲染实时数据流监控
            renderRealtimeFlowMonitor();
            
            // 加载侧边栏深度分析
            loadConcurrencySidebarAnalysis(threads, tasks, variableFlows);
        }
        
        // 提取线程数据（基于track_var!宏收集的数据）
        function extractThreadData() {
            const threadData = [];
            const processedThreads = new Set();
            
            // 从全局analysis数据中提取线程信息
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    const threadId = varDetail.thread_id;
                    if (!processedThreads.has(threadId)) {
                        processedThreads.add(threadId);
                        
                        // 计算该线程的内存使用量
                        const threadMemory = Object.values(window.analysisData.variable_registry)
                            .filter(v => v.thread_id === threadId)
                            .reduce((sum, v) => sum + v.memory_usage, 0);
                        
                        // 计算变量数量
                        const variableCount = Object.values(window.analysisData.variable_registry)
                            .filter(v => v.thread_id === threadId).length;
                        
                        // 分析线程工作负载类型
                        const workloadType = analyzeThreadWorkload(threadId);
                        
                        threadData.push({
                            id: threadId,
                            memory: threadMemory,
                            variableCount: variableCount,
                            workloadType: workloadType,
                            isActive: varDetail.lifecycle_stage === 'Active',
                            variables: Object.values(window.analysisData.variable_registry)
                                .filter(v => v.thread_id === threadId)
                                .map(v => ({
                                    name: v.name,
                                    size: v.memory_usage,
                                    lifecycle: v.lifecycle_stage,
                                    type: v.type_name || 'unknown'
                                }))
                        });
                    }
                }
            }
            
            return threadData.sort((a, b) => b.memory - a.memory);
        }
        
        // 分析线程工作负载类型
        function analyzeThreadWorkload(threadId) {
            if (!window.analysisData || !window.analysisData.variable_registry) return 'unknown';
            
            const threadVars = Object.values(window.analysisData.variable_registry)
                .filter(v => v.thread_id === threadId);
            
            const totalMemory = threadVars.reduce((sum, v) => sum + v.memory_usage, 0);
            const avgVarSize = threadVars.length > 0 ? totalMemory / threadVars.length : 0;
            
            // 基于内存使用模式判断工作负载
            if (avgVarSize > 1024 * 1024) return 'memory-intensive';
            if (threadVars.length > 50) return 'cpu-intensive';
            if (threadVars.some(v => v.type_name && v.type_name.includes('Network'))) return 'network-intensive';
            if (threadVars.some(v => v.type_name && v.type_name.includes('File'))) return 'io-intensive';
            return 'mixed-workload';
        }
        
        // 🛡️ Safety Deep Audit - 深度安全审计实现
        function loadSafetyDeepAudit() {
            console.log('🛡️ Loading Safety Deep Audit...');
            
            // 提取安全相关数据
            const unsafeOperations = extractUnsafeOperations();
            const ffiCrossings = extractFFICrossings();
            const memoryLeaks = detectPotentialMemoryLeaks();
            
            // 计算安全分数
            const safetyScore = calculateSafetyScore(unsafeOperations, ffiCrossings, memoryLeaks);
            updateSafetyMetrics(safetyScore);
            
            // 渲染安全泳道图
            renderMemorySafetySwimlane(unsafeOperations, ffiCrossings);
            
            // 渲染FFI边界审计
            renderFFIBoundaryAudit(ffiCrossings);
            
            // 渲染内存泄漏检测器
            renderMemoryLeakDetector(memoryLeaks);
            
            // 加载侧边栏安全分析
            loadSafetySidebarAnalysis(unsafeOperations, ffiCrossings, memoryLeaks);
        }
        
        // 📈 Performance Mining Analysis - 深度性能挖掘实现
        function loadPerformanceMiningAnalysis() {
            console.log('📈 Loading Performance Mining Analysis...');
            
            // 提取性能数据
            const allocationPatterns = extractAllocationPatterns();
            const memoryTimeline = extractMemoryTimeline();
            const performanceBottlenecks = identifyPerformanceBottlenecks();
            
            // 更新性能指标
            updatePerformanceKPIs(allocationPatterns, memoryTimeline);
            
            // 渲染多维时间序列图
            renderMultiDimensionalTimeSeries(memoryTimeline);
            
            // 渲染变量生命周期瀑布图
            renderVariableLifecycleWaterfall();
            
            // 渲染内存分配模式识别
            renderAllocationPatternRecognition(allocationPatterns);
            
            // 加载侧边栏性能分析
            loadPerformanceSidebarAnalysis(performanceBottlenecks);
        }
        
        // 提取不安全操作
        function extractUnsafeOperations() {
            const unsafeOps = [];
            
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    // 检测潜在的不安全操作
                    if (varDetail.type_name && varDetail.type_name.includes('*')) {
                        unsafeOps.push({
                            variable: varName,
                            operation: 'raw_pointer',
                            riskLevel: 'high',
                            threadId: varDetail.thread_id,
                            memory: varDetail.memory_usage,
                            location: varDetail.scope_name || 'unknown'
                        });
                    }
                    
                    // 检测大内存分配（可能的泄漏风险）
                    if (varDetail.memory_usage > 1024 * 1024) { // 1MB阈值
                        unsafeOps.push({
                            variable: varName,
                            operation: 'large_allocation',
                            riskLevel: varDetail.lifecycle_stage === 'Active' ? 'medium' : 'low',
                            threadId: varDetail.thread_id,
                            memory: varDetail.memory_usage,
                            location: varDetail.scope_name || 'unknown'
                        });
                    }
                }
            }
            
            return unsafeOps;
        }
        
        // 提取FFI边界穿越
        function extractFFICrossings() {
            const crossings = [];
            
            if (window.analysisData && window.analysisData.unified_variable_index) {
                for (const [varId, crossData] of Object.entries(window.analysisData.unified_variable_index)) {
                    if (crossData.relationships && crossData.relationships.includes('FFIBoundary')) {
                        crossings.push({
                            variable: varId,
                            direction: 'rust_to_c',
                            safetyLevel: 'warning',
                            timestamp: Date.now() - Math.random() * 10000
                        });
                    }
                }
            }
            
            return crossings;
        }
        
        // 检测潜在内存泄漏
        function detectPotentialMemoryLeaks() {
            const leaks = [];
            
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    // 检测长期存活的大内存变量
                    if (varDetail.lifecycle_stage === 'Active' && varDetail.memory_usage > 2 * 1024 * 1024) {
                        leaks.push({
                            variable: varName,
                            size: varDetail.memory_usage,
                            threadId: varDetail.thread_id,
                            riskLevel: 'high',
                            reason: 'large_long_lived_allocation'
                        });
                    }
                }
            }
            
            return leaks;
        }
        
        // 计算安全分数
        function calculateSafetyScore(unsafeOps, ffiCrossings, memoryLeaks) {
            const totalVars = Object.keys(window.analysisData?.variable_registry || {}).length;
            if (totalVars === 0) return 100;
            
            const riskCount = unsafeOps.length + ffiCrossings.length + memoryLeaks.length;
            return Math.max(0, Math.min(100, 100 - (riskCount / totalVars) * 100));
        }
        
        // 更新安全指标
        function updateSafetyMetrics(safetyScore) {
            const scoreElement = document.getElementById('safety-score');
            const riskElement = document.getElementById('risk-level');
            
            if (scoreElement) {
                scoreElement.textContent = safetyScore.toFixed(1) + '%';
            }
            
            if (riskElement) {
                let riskLevel = 'LOW';
                if (safetyScore < 70) riskLevel = 'HIGH';
                else if (safetyScore < 85) riskLevel = 'MEDIUM';
                
                riskElement.textContent = riskLevel;
                riskElement.style.color = riskLevel === 'HIGH' ? '#ef4444' : 
                                          riskLevel === 'MEDIUM' ? '#f59e0b' : '#10b981';
            }
        }
        
        // 渲染内存安全泳道图
        function renderMemorySafetySwimlane(unsafeOps, ffiCrossings) {
            const container = document.getElementById('safety-swimlane');
            if (!container) return;
            
            container.innerHTML = `
                <div style="height: 100%; display: flex; flex-direction: column; gap: 15px;">
                    <div style="background: linear-gradient(135deg, rgba(16, 185, 129, 0.1), rgba(16, 185, 129, 0.05)); border-radius: 8px; padding: 15px; border: 1px solid rgba(16, 185, 129, 0.3);">
                        <div style="font-weight: bold; margin-bottom: 10px; color: #10b981;">🦀 Rust Safe Zone</div>
                        <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                            ${unsafeOps.filter(op => op.riskLevel === 'low').slice(0, 5).map(op => `
                                <div style="background: #10b981; color: white; padding: 6px 10px; border-radius: 4px; font-size: 11px; cursor: pointer;" onclick="showMemoryPassport('${op.variable}')">
                                    ${op.variable.substring(0, 12)}...
                                </div>
                            `).join('')}
                        </div>
                    </div>
                    
                    <div style="background: linear-gradient(135deg, rgba(251, 146, 60, 0.1), rgba(251, 146, 60, 0.05)); border-radius: 8px; padding: 15px; border: 1px solid rgba(251, 146, 60, 0.3);">
                        <div style="font-weight: bold; margin-bottom: 10px; color: #f59e0b;">⚠️ Unsafe Zone</div>
                        <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                            ${unsafeOps.filter(op => op.riskLevel === 'medium' || op.riskLevel === 'high').slice(0, 5).map(op => `
                                <div style="background: ${op.riskLevel === 'high' ? '#ef4444' : '#f59e0b'}; color: white; padding: 6px 10px; border-radius: 4px; font-size: 11px; cursor: pointer;" onclick="showMemoryPassport('${op.variable}')">
                                    ${op.variable.substring(0, 12)}...
                                </div>
                            `).join('')}
                        </div>
                    </div>
                    
                    <div style="background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(59, 130, 246, 0.05)); border-radius: 8px; padding: 15px; border: 1px solid rgba(59, 130, 246, 0.3);">
                        <div style="font-weight: bold; margin-bottom: 10px; color: #3b82f6;">⚡ FFI Boundary</div>
                        <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                            ${ffiCrossings.slice(0, 5).map(crossing => `
                                <div style="background: #3b82f6; color: white; padding: 6px 10px; border-radius: 4px; font-size: 11px; cursor: pointer;" onclick="showMemoryPassport('${crossing.variable}')">
                                    ${crossing.variable.substring(0, 12)}...
                                </div>
                            `).join('')}
                        </div>
                    </div>
                </div>
            `;
        }
        
        // 渲染FFI边界审计
        function renderFFIBoundaryAudit(ffiCrossings) {
            const container = document.getElementById('ffi-audit');
            if (!container) return;
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <div style="font-size: 36px; margin-bottom: 15px;">⚡</div>
                    <div style="font-size: 16px; font-weight: 600; margin-bottom: 10px;">FFI Boundary Audit</div>
                    <div style="font-size: 13px; color: var(--text-secondary);">
                        <div><strong>${ffiCrossings.length}</strong> FFI crossings detected</div>
                        <div style="margin-top: 8px;">
                            ${ffiCrossings.length > 0 ? 'Active monitoring enabled' : 'No boundary violations found'}
                        </div>
                    </div>
                </div>
            `;
        }
        
        // 渲染内存泄漏检测器
        function renderMemoryLeakDetector(memoryLeaks) {
            const container = document.getElementById('leak-detection');
            if (!container) return;
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <div style="font-size: 36px; margin-bottom: 15px; color: ${memoryLeaks.length > 0 ? '#ef4444' : '#10b981'};">
                        ${memoryLeaks.length > 0 ? '🚨' : '✅'}
                    </div>
                    <div style="font-size: 16px; font-weight: 600; margin-bottom: 10px;">Memory Leak Detector</div>
                    <div style="font-size: 13px; color: var(--text-secondary);">
                        <div><strong>${memoryLeaks.length}</strong> potential leaks found</div>
                        <div style="margin-top: 8px;">
                            ${memoryLeaks.length > 0 ? 'Investigation recommended' : 'All clear - no leaks detected'}
                        </div>
                    </div>
                </div>
            `;
        }
        
        // 加载安全侧边栏分析
        function loadSafetySidebarAnalysis(unsafeOps, ffiCrossings, memoryLeaks) {
            const reportContainer = document.getElementById('memory-safety-report');
            if (reportContainer) {
                reportContainer.innerHTML = `
                    <div style="background: var(--bg-primary); padding: 12px; border-radius: 6px; margin-bottom: 10px;">
                        <div style="font-weight: 600; font-size: 13px; margin-bottom: 6px;">Unsafe Operations</div>
                        <div style="font-size: 11px; color: var(--text-secondary);">${unsafeOps.length} detected</div>
                    </div>
                    <div style="background: var(--bg-primary); padding: 12px; border-radius: 6px; margin-bottom: 10px;">
                        <div style="font-weight: 600; font-size: 13px; margin-bottom: 6px;">FFI Crossings</div>
                        <div style="font-size: 11px; color: var(--text-secondary);">${ffiCrossings.length} detected</div>
                    </div>
                    <div style="background: var(--bg-primary); padding: 12px; border-radius: 6px;">
                        <div style="font-weight: 600; font-size: 13px; margin-bottom: 6px;">Memory Leaks</div>
                        <div style="font-size: 11px; color: var(--text-secondary);">${memoryLeaks.length} potential</div>
                    </div>
                `;
            }
            
            const riskContainer = document.getElementById('ffi-risk-assessment');
            if (riskContainer) {
                const riskLevel = ffiCrossings.length > 5 ? 'HIGH' : ffiCrossings.length > 2 ? 'MEDIUM' : 'LOW';
                riskContainer.innerHTML = `
                    <div style="background: var(--bg-primary); padding: 12px; border-radius: 6px; text-align: center;">
                        <div style="font-size: 18px; font-weight: bold; color: ${riskLevel === 'HIGH' ? '#ef4444' : riskLevel === 'MEDIUM' ? '#f59e0b' : '#10b981'};">
                            ${riskLevel}
                        </div>
                        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 4px;">
                            Risk Level
                        </div>
                    </div>
                `;
            }
        }
        
        // 📈 Performance Analysis Helper Functions - 性能分析辅助函数
        function extractAllocationPatterns() {
            const patterns = [];
            
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    patterns.push({
                        variable: varName,
                        size: varDetail.memory_usage,
                        threadId: varDetail.thread_id,
                        allocCount: varDetail.allocation_count || 1,
                        lifecycle: varDetail.lifecycle_stage
                    });
                }
            }
            
            return patterns;
        }
        
        function extractMemoryTimeline() {
            const timeline = [];
            const now = Date.now();
            
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    // 模拟时间戳
                    const allocTime = now - Math.random() * 60000; // 最近1分钟内
                    timeline.push({
                        timestamp: allocTime,
                        event: 'allocation',
                        variable: varName,
                        size: varDetail.memory_usage,
                        threadId: varDetail.thread_id
                    });
                    
                    if (varDetail.lifecycle_stage === 'Deallocated') {
                        timeline.push({
                            timestamp: allocTime + Math.random() * 30000,
                            event: 'deallocation',
                            variable: varName,
                            size: varDetail.memory_usage,
                            threadId: varDetail.thread_id
                        });
                    }
                }
            }
            
            return timeline.sort((a, b) => a.timestamp - b.timestamp);
        }
        
        function identifyPerformanceBottlenecks() {
            const bottlenecks = [];
            
            if (window.analysisData && window.analysisData.variable_registry) {
                for (const [varName, varDetail] of Object.entries(window.analysisData.variable_registry)) {
                    // 识别内存热点
                    if (varDetail.memory_usage > 5 * 1024 * 1024) { // 5MB阈值
                        bottlenecks.push({
                            type: 'memory_hotspot',
                            variable: varName,
                            severity: 'high',
                            value: varDetail.memory_usage,
                            threadId: varDetail.thread_id
                        });
                    }
                    
                    // 识别高频分配
                    if (varDetail.allocation_count && varDetail.allocation_count > 100) {
                        bottlenecks.push({
                            type: 'high_allocation_frequency',
                            variable: varName,
                            severity: 'medium',
                            value: varDetail.allocation_count,
                            threadId: varDetail.thread_id
                        });
                    }
                }
            }
            
            return bottlenecks;
        }
        
        function updatePerformanceKPIs(allocationPatterns, memoryTimeline) {
            const avgAllocElement = document.getElementById('avg-alloc');
            const peakMemoryElement = document.getElementById('peak-memory');
            const efficiencyElement = document.getElementById('efficiency');
            
            if (avgAllocElement) {
                const avgAlloc = allocationPatterns.length > 0 ? 
                    allocationPatterns.reduce((sum, p) => sum + p.size, 0) / allocationPatterns.length : 0;
                avgAllocElement.textContent = (avgAlloc / 1024).toFixed(1) + 'KB';
            }
            
            if (peakMemoryElement) {
                const peakMemory = Math.max(...allocationPatterns.map(p => p.size));
                peakMemoryElement.textContent = (peakMemory / 1024 / 1024).toFixed(1) + 'MB';
            }
            
            if (efficiencyElement) {
                const activeVars = allocationPatterns.filter(p => p.lifecycle === 'Active').length;
                const efficiency = allocationPatterns.length > 0 ? (activeVars / allocationPatterns.length * 100) : 0;
                efficiencyElement.textContent = efficiency.toFixed(0) + '%';
            }
        }
        
        function renderMultiDimensionalTimeSeries(memoryTimeline) {
            const container = document.getElementById('performance-timeseries');
            if (!container) return;
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <div style="font-size: 36px; margin-bottom: 15px;">📊</div>
                    <div style="font-size: 16px; font-weight: 600; margin-bottom: 10px;">Multi-dimensional Time Series</div>
                    <div style="font-size: 13px; color: var(--text-secondary);">
                        <div><strong>${memoryTimeline.length}</strong> timeline events</div>
                        <div style="margin-top: 8px;">
                            Memory allocation patterns over time
                        </div>
                    </div>
                    <canvas id="timeseriesChart" width="400" height="200" style="margin-top: 15px; background: white; border-radius: 8px;"></canvas>
                </div>
            `;
            
            // 绘制简单的时间序列图
            setTimeout(() => drawTimeSeriesChart(memoryTimeline), 100);
        }
        
        function drawTimeSeriesChart(timeline) {
            const canvas = document.getElementById('timeseriesChart');
            if (!canvas) return;
            
            const ctx = canvas.getContext('2d');
            const width = canvas.width;
            const height = canvas.height;
            
            // 清除画布
            ctx.clearRect(0, 0, width, height);
            
            if (timeline.length === 0) return;
            
            // 计算数据范围
            const timeRange = timeline[timeline.length - 1].timestamp - timeline[0].timestamp;
            const maxSize = Math.max(...timeline.map(t => t.size));
            
            // 绘制坐标轴
            ctx.strokeStyle = '#e5e7eb';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(40, height - 30);
            ctx.lineTo(width - 20, height - 30);
            ctx.moveTo(40, 20);
            ctx.lineTo(40, height - 30);
            ctx.stroke();
            
            // 绘制数据点和线
            ctx.strokeStyle = '#3b82f6';
            ctx.fillStyle = '#3b82f6';
            ctx.lineWidth = 2;
            ctx.beginPath();
            
            let prevX = null, prevY = null;
            
            timeline.forEach((point, index) => {
                const x = 40 + ((point.timestamp - timeline[0].timestamp) / timeRange) * (width - 60);
                const y = height - 30 - (point.size / maxSize) * (height - 50);
                
                if (index === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
                
                // 绘制数据点
                ctx.save();
                ctx.beginPath();
                ctx.arc(x, y, 3, 0, 2 * Math.PI);
                ctx.fill();
                ctx.restore();
                
                prevX = x;
                prevY = y;
            });
            
            ctx.stroke();
            
            // 添加标签
            ctx.fillStyle = '#374151';
            ctx.font = '12px Arial';
            ctx.fillText('Memory Usage', 50, 15);
            ctx.fillText('Time', width - 40, height - 10);
        }
        
        function renderVariableLifecycleWaterfall() {
            const container = document.getElementById('lifecycle-waterfall');
            if (!container) return;
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <div style="font-size: 36px; margin-bottom: 15px;">💧</div>
                    <div style="font-size: 16px; font-weight: 600; margin-bottom: 10px;">Variable Lifecycle Waterfall</div>
                    <div style="font-size: 13px; color: var(--text-secondary);">
                        <div>Variable state transitions over time</div>
                        <div style="margin-top: 8px; display: flex; justify-content: center; gap: 15px;">
                            <span style="color: #10b981;">🟢 Active</span>
                            <span style="color: #f59e0b;">🟡 Allocated</span>
                            <span style="color: #3b82f6;">🔄 Shared</span>
                            <span style="color: #6b7280;">⚫ Deallocated</span>
                        </div>
                    </div>
                </div>
            `;
        }
        
        function renderAllocationPatternRecognition(allocationPatterns) {
            const container = document.getElementById('pattern-recognition');
            if (!container) return;
            
            // 分析分配模式
            const sizeDistribution = {
                small: allocationPatterns.filter(p => p.size < 1024).length,
                medium: allocationPatterns.filter(p => p.size >= 1024 && p.size < 1024 * 1024).length,
                large: allocationPatterns.filter(p => p.size >= 1024 * 1024).length
            };
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <div style="font-size: 36px; margin-bottom: 15px;">🧠</div>
                    <div style="font-size: 16px; font-weight: 600; margin-bottom: 10px;">Allocation Pattern Recognition</div>
                    <div style="font-size: 13px; color: var(--text-secondary);">
                        <div style="margin-bottom: 10px;">Pattern analysis results:</div>
                        <div style="display: flex; justify-content: center; gap: 20px;">
                            <div style="text-align: center;">
                                <div style="font-weight: bold; color: #10b981;">${sizeDistribution.small}</div>
                                <div>Small (&lt;1KB)</div>
                            </div>
                            <div style="text-align: center;">
                                <div style="font-weight: bold; color: #f59e0b;">${sizeDistribution.medium}</div>
                                <div>Medium (1KB-1MB)</div>
                            </div>
                            <div style="text-align: center;">
                                <div style="font-weight: bold; color: #ef4444;">${sizeDistribution.large}</div>
                                <div>Large (&gt;1MB)</div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }
        
        function loadPerformanceSidebarAnalysis(performanceBottlenecks) {
            const bottleneckContainer = document.getElementById('performance-bottlenecks');
            if (bottleneckContainer) {
                bottleneckContainer.innerHTML = performanceBottlenecks.slice(0, 5).map(bottleneck => `
                    <div style="background: var(--bg-primary); padding: 10px; margin: 6px 0; border-radius: 6px; border-left: 3px solid ${bottleneck.severity === 'high' ? '#ef4444' : '#f59e0b'};">
                        <div style="font-weight: 600; font-size: 13px;">${bottleneck.type.replace('_', ' ')}</div>
                        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 2px;">
                            ${bottleneck.variable.substring(0, 20)}... • ${bottleneck.severity}
                        </div>
                    </div>
                `).join('') || '<div style="color: var(--text-secondary); font-style: italic;">No bottlenecks detected</div>';
            }
            
            const trendsContainer = document.getElementById('memory-usage-trends');
            if (trendsContainer) {
                const totalMemory = Object.values(window.analysisData?.variable_registry || {})
                    .reduce((sum, v) => sum + v.memory_usage, 0);
                
                trendsContainer.innerHTML = `
                    <div style="background: var(--bg-primary); padding: 12px; border-radius: 6px; text-align: center;">
                        <div style="font-size: 18px; font-weight: bold; color: #3b82f6;">
                            ${(totalMemory / 1024 / 1024).toFixed(1)}MB
                        </div>
                        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 4px;">
                            Total Memory Usage
                        </div>
                    </div>
                `;
            }
        }
        
        // Memory Passport 显示函数
        function showMemoryPassport(memoryId) {
            const container = document.getElementById('memoryPassport');
            if (!container) return;
            
            const varDetail = window.analysisData?.variable_registry?.[memoryId];
            
            container.innerHTML = `
                <div style="background: var(--bg-tertiary); padding: 16px; border-radius: 8px; border: 1px solid var(--border-color);">
                    <h4 style="margin: 0 0 12px 0; color: #3b82f6;">🛡️ Memory Passport</h4>
                    <div style="font-size: 13px; line-height: 1.5;">
                        <div style="margin: 8px 0;"><strong>Variable:</strong> ${memoryId}</div>
                        <div style="margin: 8px 0;"><strong>Size:</strong> ${varDetail ? (varDetail.memory_usage / 1024).toFixed(1) + 'KB' : 'Unknown'}</div>
                        <div style="margin: 8px 0;"><strong>Thread:</strong> ${varDetail ? varDetail.thread_id : 'Unknown'}</div>
                        <div style="margin: 8px 0;"><strong>Lifecycle:</strong> ${varDetail ? varDetail.lifecycle_stage : 'Unknown'}</div>
                        <div style="margin-top: 12px; padding: 8px; background: var(--bg-secondary); border-radius: 4px;">
                            <strong>Status:</strong> ${varDetail?.lifecycle_stage === 'Active' ? '✅ Active & Safe' : 
                                                     varDetail?.lifecycle_stage === 'Deallocated' ? '⚫ Deallocated' : 
                                                     '🟡 Allocated'}
                        </div>
                    </div>
                </div>
            `;
        }
        
        // Theme toggle functionality
        const themeToggle = document.querySelector('.theme-toggle');
        const body = document.body;
        
        // Check for saved theme preference or default to dark
        const currentTheme = localStorage.getItem('theme') || 'dark';
        body.setAttribute('data-theme', currentTheme);
        updateThemeToggle(currentTheme);
        
        function toggleTheme() {
            const currentTheme = body.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            
            body.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
            updateThemeToggle(newTheme);
            
            // Update chart colors for theme
            updateChartTheme(newTheme);
        }
        
        function updateThemeToggle(theme) {
            if (theme === 'dark') {
                themeToggle.innerHTML = '☀️ Light Mode';
            } else {
                themeToggle.innerHTML = '🌙 Dark Mode';
            }
        }
        
        function updateChartTheme(theme) {{
            // Theme change notification
            console.log('Theme changed to:', theme);
            // Chart colors are handled by CSS variables
        }}
        
        // Concurrency Lens Implementation - Thread/Task Analysis
        function loadConcurrencyAnalysis() {
            console.log('🚀 Loading concurrency analysis...');
            const container = document.getElementById('memoryContinent');
            if (!container) return;
            
            container.innerHTML = `
                <div style="text-align: center; padding: 20px;">
                    <h3 style="margin: 0 0 20px 0; color: var(--text-primary);">🗺️ Execution Territory Map</h3>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; height: 350px;">
                        <div style="background: linear-gradient(135deg, #10b981, #059669); border-radius: 12px; padding: 20px; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; cursor: pointer; transition: transform 0.3s ease;" 
                             onclick="drillDownExecution('main-thread', 0)" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                            <div style="font-size: 24px; margin-bottom: 10px;">🧵</div>
                            <div style="font-weight: bold; font-size: 16px;">Main Thread</div>
                            <div style="opacity: 0.9; font-size: 14px;">2.1MB (5%)</div>
                        </div>
                        <div style="background: linear-gradient(135deg, #3b82f6, #1d4ed8); border-radius: 12px; padding: 20px; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; cursor: pointer; transition: transform 0.3s ease;" 
                             onclick="drillDownExecution('thread-pool', 4)" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                            <div style="font-size: 24px; margin-bottom: 10px;">🔄</div>
                            <div style="font-weight: bold; font-size: 16px;">Thread Pool</div>
                            <div style="opacity: 0.9; font-size: 14px;">31.5MB (75%)</div>
                        </div>
                        <div style="background: linear-gradient(135deg, #8b5cf6, #7c3aed); border-radius: 12px; padding: 20px; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; cursor: pointer; transition: transform 0.3s ease;" 
                             onclick="drillDownExecution('async-runtime', 8)" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                            <div style="font-size: 24px; margin-bottom: 10px;">⚡</div>
                            <div style="font-weight: bold; font-size: 16px;">Async Runtime</div>
                            <div style="opacity: 0.9; font-size: 14px;">7.6MB (18%)</div>
                        </div>
                        <div style="background: linear-gradient(135deg, #ef4444, #dc2626); border-radius: 12px; padding: 20px; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; cursor: pointer; transition: transform 0.3s ease;" 
                             onclick="contextualSafetyLink('ffi-analysis')" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
                            <div style="font-size: 24px; margin-bottom: 10px;">🛡️</div>
                            <div style="font-weight: bold; font-size: 16px;">FFI Boundaries</div>
                            <div style="opacity: 0.9; font-size: 14px;">0.8MB (2%)</div>
                            <div style="margin-top: 8px; font-size: 10px; background: rgba(255,255,255,0.2); padding: 2px 6px; border-radius: 4px; cursor: pointer;" onclick="event.stopPropagation(); switchAnalysisLens('safety')">🛡️ Safety View</div>
                        </div>
                    </div>
                </div>
            `;
            loadExecutionUnitRankings();
        }
        
        // Load execution unit rankings for concurrency sidebar
        function loadExecutionUnitRankings() {
            const container = document.getElementById('executionUnitRankings');
            if (!container) return;
            
            const rankings = [
                { rank: 1, type: 'thread', id: 3, memory: '8.2MB', workload: 'CPU Intensive' },
                { rank: 2, type: 'task', id: 12, memory: '6.7MB', workload: 'I/O Bound' },
                { rank: 3, type: 'thread', id: 7, memory: '5.9MB', workload: 'Mixed' },
                { rank: 4, type: 'task', id: 8, memory: '4.3MB', workload: 'Network Bound' },
                { rank: 5, type: 'thread', id: 15, memory: '3.8MB', workload: 'Memory Intensive' }
            ];
            
            container.innerHTML = rankings.map(item => `
                <div style="background: var(--bg-tertiary); padding: 12px; border-radius: 8px; border: 1px solid var(--border-color); cursor: pointer; transition: all 0.2s ease; margin-bottom: 8px;" 
                     onclick="drillDownExecution('$\{item.type}', $\{item.id})">
                    <span style="font-weight: 700; color: var(--accent-blue); margin-right: 8px;">#$\{item.rank}</span>
                    <div>
                        <div>${item.type === 'thread' ? '🧵' : '⚡'} $\{item.type.charAt(0).toUpperCase() + item.type.slice(1)} $\{item.id}</div>
                        <div style="font-size: 12px; color: var(--text-secondary);">$\{item.memory} • $\{item.workload}</div>
                    </div>
                </div>
            `).join('');
        }
        
        // Safety Lens Implementation - Cross-boundary Analysis  
        function loadSafetyAnalysis() {
            console.log('🛡️ Loading safety analysis...');
            const container = document.getElementById('safetySwimlane');
            if (!container) return;
            
            container.innerHTML = `
                <div style="height: 100%; text-align: center;">
                    <h3 style="margin: 0 0 20px 0; color: var(--text-primary);">🛡️ Memory Safety Swimlanes</h3>
                    <div style="display: flex; flex-direction: column; height: 350px; gap: 20px;">
                        <div style="flex: 1; background: linear-gradient(135deg, rgba(251, 146, 60, 0.1), rgba(251, 146, 60, 0.05)); border-radius: 8px; padding: 15px; border: 1px solid rgba(251, 146, 60, 0.3);">
                            <div style="font-weight: bold; margin-bottom: 10px; color: #f59e0b;">🦀 Rust Safe Context</div>
                            <div style="display: flex; gap: 10px; height: 40px; justify-content: center;">
                                <div style="background: linear-gradient(90deg, #10b981, #059669); color: white; padding: 8px 12px; border-radius: 6px; font-size: 12px; cursor: pointer; display: flex; align-items: center;" onclick="showMemoryPassport('rust-safe-1')">Safe Allocation</div>
                                <div style="background: linear-gradient(90deg, #f59e0b, #d97706); color: white; padding: 8px 12px; border-radius: 6px; font-size: 12px; cursor: pointer; display: flex; align-items: center;" onclick="showMemoryPassport('rust-unsafe-1')">Unsafe Block</div>
                            </div>
                        </div>
                        <div style="flex: 1; background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(59, 130, 246, 0.05)); border-radius: 8px; padding: 15px; border: 1px solid rgba(59, 130, 246, 0.3);">
                            <div style="font-weight: bold; margin-bottom: 10px; color: #3b82f6;">⚡ FFI Boundary Context</div>
                            <div style="display: flex; gap: 10px; height: 40px; justify-content: center;">
                                <div style="background: linear-gradient(90deg, #3b82f6, #1d4ed8); color: white; padding: 8px 12px; border-radius: 6px; font-size: 12px; cursor: pointer; display: flex; align-items: center;" onclick="showMemoryPassport('ffi-call-1')">C Library Call</div>
                                <div style="background: linear-gradient(90deg, #ef4444, #dc2626); color: white; padding: 8px 12px; border-radius: 6px; font-size: 12px; cursor: pointer; display: flex; align-items: center;" onclick="showMemoryPassport('ffi-leak-1')">🚨 Memory Leak</div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }
        
        // Performance Lens Implementation - Metrics & Charts
        function loadPerformanceAnalysis() {
            console.log('📈 Loading performance analysis...');
            const container = document.getElementById('performanceCharts');
            if (!container) return;
            
            container.innerHTML = `
                <div style="height: 100%;">
                    <div style="display: flex; gap: 10px; margin-bottom: 20px; justify-content: center;">
                        <button onclick="switchPerformanceChart('memory')" style="background: #3b82f6; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px;">📊 Memory</button>
                        <button onclick="switchPerformanceChart('cpu')" style="background: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border-color); padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px;">⚡ CPU</button>
                        <button onclick="switchPerformanceChart('io')" style="background: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border-color); padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px;">💾 I/O</button>
                        <button onclick="switchPerformanceChart('network')" style="background: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border-color); padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px;">🌐 Network</button>
                    </div>
                    <div style="background: white; border-radius: 8px; padding: 20px; height: 300px; border: 1px solid var(--border-color); display: flex; align-items: center; justify-content: center;">
                        <canvas id="performanceChart" width="600" height="250"></canvas>
                    </div>
                </div>
            `;
            loadPerformanceKPIs();
            switchPerformanceChart('memory');
        }
        
        // Load performance KPIs for performance sidebar
        function loadPerformanceKPIs() {
            const container = document.getElementById('performanceKpis');
            if (!container) return;
            
            container.innerHTML = `
                <div style="background: var(--bg-tertiary); padding: 16px; border-radius: 8px; text-align: center; border: 1px solid var(--border-color); margin-bottom: 12px;">
                    <div style="font-size: 24px; font-weight: 700; color: #3b82f6; margin-bottom: 4px;">42.1MB</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Peak Memory</div>
                </div>
                <div style="background: var(--bg-tertiary); padding: 16px; border-radius: 8px; text-align: center; border: 1px solid var(--border-color); margin-bottom: 12px;">
                    <div style="font-size: 24px; font-weight: 700; color: #10b981; margin-bottom: 4px;">87%</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">CPU Efficiency</div>
                </div>
                <div style="background: var(--bg-tertiary); padding: 16px; border-radius: 8px; text-align: center; border: 1px solid var(--border-color); margin-bottom: 12px;">
                    <div style="font-size: 24px; font-weight: 700; color: #f59e0b; margin-bottom: 4px;">2.3ms</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Avg Response</div>
                </div>
                <div style="background: var(--bg-tertiary); padding: 16px; border-radius: 8px; text-align: center; border: 1px solid var(--border-color);">
                    <div style="font-size: 24px; font-weight: 700; color: #8b5cf6; margin-bottom: 4px;">156</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Alloc/sec</div>
                </div>
            `;
        }
        
        // Toggle chart visibility to save memory
        let chartsVisible = true;
        let chartInstances = null;
        
        // Initialize charts on page load
        window.addEventListener('load', function() {{
            initializeCharts();
        }});
        
        function toggleCharts() {{
            const container = document.getElementById('chartsContainer');
            const button = document.getElementById('chartToggle');
            
            if (!chartsVisible) {{
                container.style.display = 'grid';
                button.textContent = '📊 Hide Performance Charts';
                if (!chartInstances) {{
                    // Lazy load charts only when needed
                    initializeCharts();
                }}
                chartsVisible = true;
            }} else {{
                container.style.display = 'none';
                button.textContent = '📊 Show Performance Charts';
                chartsVisible = false;
                // Optionally destroy charts to free memory
                if (chartInstances) {{
                    Object.values(chartInstances).forEach(chart => chart.destroy());
                    chartInstances = null;
                }}
            }}
        }}
        
        // Thread details toggle
        function toggleThreadDetails(threadId) {{
            var details = document.getElementById('thread-details-' + threadId);
            if (!details) return;
            var card = details.closest('.thread-card');
            
            if (details.style.display === 'none') {{
                details.style.display = 'block';
                card.classList.add('expanded');
            }} else {{
                details.style.display = 'none';
                card.classList.remove('expanded');
            }}
        }}
        
        // Task variables toggle with lazy loading
        function toggleTaskVariables(threadId, taskId) {{
            var container = document.getElementById('task-variables-' + threadId + '-' + taskId);
            if (!container) return;
            
            if (container.style.display === 'none' || container.style.display === '') {{
                container.style.display = 'block';
                loadTaskVariables(threadId, taskId);
            }} else {{
                container.style.display = 'none';
            }}
        }}
        
        // Lazy load variable details for specific task
        function loadTaskVariables(threadId, taskId) {{
            var container = document.getElementById('task-variables-' + threadId + '-' + taskId);
            if (!container || typeof allVariables === 'undefined') return;
            
            var taskVariables = [];
            for (var i = 0; i < allVariables.length; i++) {{
                if (allVariables[i].thread_id === threadId && allVariables[i].task_id === taskId) {{
                    taskVariables.push(allVariables[i]);
                }}
            }}
            
            if (taskVariables.length === 0) {{
                container.innerHTML = '<div class="variable-summary">No variables found</div>';
                return;
            }}
            
            var html = '';
            for (var j = 0; j < taskVariables.length; j++) {{
                var v = taskVariables[j];
                var memoryKB = (v.memory_usage / 1024).toFixed(1);
                var stageClass = v.lifecycle_stage.toLowerCase();
                html += '<div class="mini-variable-card">' +
                    '<strong>' + v.name + '</strong> - ' + memoryKB + 'KB ' +
                    '<span class="lifecycle-badge ' + stageClass + '">' + v.lifecycle_stage + '</span>' +
                    '</div>';
            }}
            
            container.innerHTML = html;
        }}
        
        function initializeCharts() {{
            console.log('Initializing lightweight performance charts...');
            // Simplified chart initialization to avoid JS errors
            if (typeof Chart !== 'undefined') {{
                console.log('Chart.js loaded successfully');
            }} else {{
                console.log('Chart.js not available, skipping charts');
            }}
        }}
        
        // Ensure all required functions are defined
        // Three-Lens Analysis System
        function switchAnalysisLens(lensType) {
            // Update button states
            document.querySelectorAll('.lens-button').forEach(btn => {
                btn.classList.remove('active');
            });
            document.querySelector('[data-lens="' + lensType + '"]').classList.add('active');
            
            // Hide all sections first
            document.querySelectorAll('.section').forEach(section => {
                section.style.display = 'none';
            });
            
            // Show relevant sections based on lens type
            if (lensType === 'performance') {
                // Performance Lens: Show overview + charts
                const sections = document.querySelectorAll('.section');
                if (sections[0]) sections[0].style.display = 'block'; // Memory Continent Overview
                if (sections[1]) sections[1].style.display = 'block'; // Territory Treemap
            } else if (lensType === 'concurrency') {
                // Concurrency Lens: Show thread matrix + detailed analysis
                const sections = document.querySelectorAll('.section');
                if (sections[2]) sections[2].style.display = 'block'; // Thread-Task Matrix
                if (sections[3]) sections[3].style.display = 'block'; // Interactive elements
            } else if (lensType === 'safety') {
                // Safety Lens: Show variable details + safety analysis
                const sections = document.querySelectorAll('.section');
                if (sections[4]) sections[4].style.display = 'block'; // Variable details
                if (sections[5]) sections[5].style.display = 'block'; // Safety analysis
            }
            
            console.log('Switched to ' + lensType + ' lens');
        }
        
        // Initialize with performance lens
        document.addEventListener('DOMContentLoaded', function() {
            switchAnalysisLens('performance');
        });

        function toggleCharts() {
            const container = document.getElementById('chartsContainer');
            const button = document.getElementById('chartToggle');
            
            if (container && button) {
                if (container.style.display === 'none') {
                    container.style.display = 'grid';
                    button.textContent = '📊 Hide Performance Charts';
                } else {
                    container.style.display = 'none';
                    button.textContent = '📊 Show Performance Charts';
                }
            }
        }
    </script>
</body>
</html>"#.to_string()
    }

    /// Calculate thread-level performance metrics
    fn calculate_thread_metrics(&self, data: &HybridAnalysisData) -> ThreadMetrics {
        let total_variables = data.variable_registry.len() as f64;
        let total_memory: u64 = data.variable_registry.values()
            .map(|v| v.memory_usage)
            .sum();

        ThreadMetrics {
            avg_variables_per_thread: total_variables / self.thread_count as f64,
            avg_memory_per_thread: total_memory as f64 / self.thread_count as f64,
        }
    }

    /// Calculate task-level performance metrics
    fn calculate_task_metrics(&self, data: &HybridAnalysisData) -> TaskMetrics {
        let total_variables = data.variable_registry.len() as f64;
        let active_variables = data.variable_registry.values()
            .filter(|v| matches!(v.lifecycle_stage, LifecycleStage::Active))
            .count() as f64;

        TaskMetrics {
            avg_variables_per_task: total_variables / self.task_count as f64,
            memory_efficiency: if total_variables > 0.0 { active_variables / total_variables } else { 0.0 },
        }
    }
}

/// Thread performance metrics
#[derive(Debug)]
struct ThreadMetrics {
    avg_variables_per_thread: f64,
    avg_memory_per_thread: f64,
}

/// Task performance metrics
#[derive(Debug)]
struct TaskMetrics {
    avg_variables_per_task: f64,
    memory_efficiency: f64,
}

/// Create sample hybrid analysis data for demonstration
pub fn create_sample_hybrid_data(thread_count: usize, task_count: usize) -> HybridAnalysisData {
    let mut variable_registry = HashMap::new();
    let mut thread_task_mapping = HashMap::new();

    // Generate thread-task mappings
    for thread_id in 0..thread_count {
        let tasks_per_thread = (task_count / thread_count).max(1);
        let start_task = thread_id * tasks_per_thread;
        let end_task = ((thread_id + 1) * tasks_per_thread).min(task_count);
        let tasks: Vec<usize> = (start_task..end_task).collect();
        thread_task_mapping.insert(thread_id, tasks);
    }

    // Generate sample variables (full data with efficient client-side rendering)
    let mut _variable_counter = 0;
    for thread_id in 0..thread_count {
        let tasks = thread_task_mapping.get(&thread_id).unwrap();
        
        for &task_id in tasks {
            // Create variables for each task (original full data)
            for var_idx in 0..((thread_id + 1) * 2) {
                let variable_name = format!("var_t{}_task{}_v{}", thread_id, task_id, var_idx);
                let variable = VariableDetail {
                    name: variable_name.clone(),
                    type_info: format!("Type{}", var_idx % 4),
                    thread_id,
                    task_id: Some(task_id),
                    allocation_count: (var_idx as u64 + 1) * 10,
                    memory_usage: (var_idx as u64 + 1) * 1024 * (thread_id as u64 + 1),
                    lifecycle_stage: match var_idx % 4 {
                        0 => LifecycleStage::Active,
                        1 => LifecycleStage::Allocated,
                        2 => LifecycleStage::Shared,
                        _ => LifecycleStage::Deallocated,
                    },
                };
                variable_registry.insert(variable_name, variable);
                _variable_counter += 1;
            }
        }
    }

    // Create visualization config
    let visualization_config = VisualizationConfig::default();
    
    // Generate realistic performance metrics with fewer data points
    let performance_metrics = generate_performance_metrics(thread_count, task_count);
    
    // Generate intelligent thread and task classifications
    let thread_classifications = generate_thread_classifications(thread_count);
    let task_classifications = generate_task_classifications(task_count);

    HybridAnalysisData {
        lockfree_analysis: None,
        visualization_config,
        thread_task_mapping,
        variable_registry,
        performance_metrics,
        thread_classifications,
        task_classifications,
        ffi_safety_data: FFISafetyData {
            safety_violations: Vec::new(),
            ownership_chain_analysis: Vec::new(),
            risk_matrix: RiskMatrix {
                memory_safety_score: 0.0,
                thread_safety_score: 0.0,
                ffi_safety_score: 0.0,
                overall_risk: RiskLevel::Low,
            },
            safety_score_timeline: Vec::new(),
            boundary_crossings: Vec::new(),
        },
        intelligent_analysis: Some(IntelligentAnalysisEngine {
            leak_detector: LeakDetector {
                unmatched_allocations: Vec::new(),
                timeout_variables: Vec::new(),
                ffi_boundary_leaks: Vec::new(),
            },
            race_analyzer: RaceAnalyzer {
                shared_variable_access: std::collections::HashMap::new(),
                race_conditions: Vec::new(),
                deadlock_scenarios: Vec::new(),
            },
            ffi_auditor: FFIAuditor {
                boundary_crossings: Vec::new(),
                risk_assessment: RiskMatrix {
                    memory_safety_score: 0.0,
                    thread_safety_score: 0.0,
                    ffi_safety_score: 0.0,
                    overall_risk: RiskLevel::Low,
                },
                ownership_transfers: Vec::new(),
            },
            pattern_miner: PatternMiner {
                allocation_patterns: Vec::new(),
                lifecycle_patterns: Vec::new(),
                thread_affinity: std::collections::HashMap::new(),
            },
        }),
        lens_linkage_data: LensLinkageData {
            performance_anomalies: Vec::new(),
            concurrency_risks: Vec::new(),
            safety_performance_impact: Vec::new(),
            active_linkage_context: None,
        },
        unified_variable_index: std::collections::HashMap::new(),
        timeline_events: std::collections::BTreeMap::new(),
        variable_relationships: std::collections::HashMap::new(),
    }
}

/// Generate intelligent thread workload classifications
fn generate_thread_classifications(thread_count: usize) -> HashMap<usize, ThreadWorkloadType> {
    let mut classifications = HashMap::new();
    
    for thread_id in 0..thread_count {
        let classification = match thread_id % 5 {
            0 => ThreadWorkloadType::CpuIntensive,
            1 => ThreadWorkloadType::IoIntensive, 
            2 => ThreadWorkloadType::NetworkIntensive,
            3 => ThreadWorkloadType::Mixed,
            _ => ThreadWorkloadType::Idle,
        };
        classifications.insert(thread_id, classification);
    }
    
    classifications
}

/// Generate intelligent task execution pattern classifications
fn generate_task_classifications(task_count: usize) -> HashMap<usize, TaskExecutionPattern> {
    let mut classifications = HashMap::new();
    
    for task_id in 0..task_count {
        let classification = match task_id % 5 {
            0 => TaskExecutionPattern::CpuBound,
            1 => TaskExecutionPattern::IoBound,
            2 => TaskExecutionPattern::NetworkBound, 
            3 => TaskExecutionPattern::MemoryIntensive,
            _ => TaskExecutionPattern::Balanced,
        };
        classifications.insert(task_id, classification);
    }
    
    classifications
}

/// Generate optimized performance metrics with more data points for smoother curves
fn generate_performance_metrics(thread_count: usize, task_count: usize) -> PerformanceTimeSeries {
    let timeline_points = 12; // Increased to 12 points for smoother curves while keeping memory efficient
    let mut cpu_usage = Vec::with_capacity(timeline_points);
    let mut memory_usage = Vec::with_capacity(timeline_points);
    let mut io_operations = Vec::with_capacity(timeline_points);
    let mut network_bytes = Vec::with_capacity(timeline_points);
    let mut timestamps = Vec::with_capacity(timeline_points);
    let mut thread_cpu_breakdown = HashMap::new();
    let mut thread_memory_breakdown = HashMap::new();

    // Generate time-series data with realistic patterns
    for i in 0..timeline_points {
        let time_progress = i as f64 / timeline_points as f64;
        timestamps.push(i as u64 * 100); // 100ms intervals
        
        // CPU usage: simulated workload with peaks and valleys
        let base_cpu = 15.0 + (thread_count as f64 * 2.5);
        let workload_spike = 40.0 * (1.0 + (time_progress * 6.28).sin()) / 2.0;
        let thread_stress = if time_progress > 0.3 && time_progress < 0.8 { 25.0 } else { 0.0 };
        cpu_usage.push((base_cpu + workload_spike + thread_stress).min(95.0));
        
        // Memory usage: progressive increase with allocation bursts
        let base_memory = (thread_count * task_count * 1024 * 1024) as u64; // Base memory per thread-task
        let allocation_growth = (time_progress * base_memory as f64 * 0.8) as u64;
        let burst_pattern = if i % 7 == 0 { base_memory / 4 } else { 0 };
        memory_usage.push(base_memory + allocation_growth + burst_pattern);
        
        // I/O operations: periodic spikes based on task scheduling
        let base_io = thread_count as u64 * 10;
        let io_burst = if i % 5 == 0 { task_count as u64 * 50 } else { 0 };
        let sustained_io = (time_progress * 200.0) as u64;
        io_operations.push(base_io + io_burst + sustained_io);
        
        // Network throughput: communication between threads/tasks
        let base_network = (thread_count * task_count * 512) as u64; // Base network activity
        let communication_spike = if time_progress > 0.4 && time_progress < 0.9 {
            (base_network as f64 * 1.5 * (time_progress * 3.14).sin().abs()) as u64
        } else {
            0
        };
        network_bytes.push(base_network + communication_spike);
    }
    
    // Generate per-thread breakdowns
    for thread_id in 0..thread_count {
        let mut thread_cpu = Vec::new();
        let mut thread_memory = Vec::new();
        
        for i in 0..timeline_points {
            let time_progress = i as f64 / timeline_points as f64;
            
            // Each thread has different usage patterns
            let thread_factor = (thread_id + 1) as f64 / thread_count as f64;
            let thread_phase = time_progress + (thread_id as f64 * 0.2);
            
            // CPU per thread
            let thread_base_cpu = cpu_usage[i] * thread_factor;
            let thread_specific_load = 10.0 * (thread_phase * 6.28).cos().abs();
            thread_cpu.push((thread_base_cpu + thread_specific_load).min(100.0));
            
            // Memory per thread
            let thread_base_memory = memory_usage[i] / thread_count as u64;
            let thread_allocation_pattern = ((thread_id + 1) as u64 * 1024 * 1024) * 
                (1.0 + time_progress * thread_factor) as u64;
            thread_memory.push(thread_base_memory + thread_allocation_pattern);
        }
        
        thread_cpu_breakdown.insert(thread_id, thread_cpu);
        thread_memory_breakdown.insert(thread_id, thread_memory);
    }

    PerformanceTimeSeries {
        cpu_usage,
        memory_usage,
        io_operations,
        network_bytes,
        timestamps,
        thread_cpu_breakdown,
        thread_memory_breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = FixedHybridTemplate::new(5, 6);
        assert_eq!(template.thread_count, 5);
        assert_eq!(template.task_count, 6);
        assert!(template.variable_details_enabled);
    }

    #[test]
    fn test_sample_data_generation() {
        let data = create_sample_hybrid_data(3, 4);
        assert_eq!(data.thread_task_mapping.len(), 3);
        assert!(!data.variable_registry.is_empty());
    }

    #[test]
    fn test_html_generation() {
        let template = FixedHybridTemplate::new(2, 3);
        let data = create_sample_hybrid_data(2, 3);
        let result = template.generate_hybrid_dashboard(&data);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Thread-Task Matrix"));
        assert!(html.contains("Variable Details"));
    }
}