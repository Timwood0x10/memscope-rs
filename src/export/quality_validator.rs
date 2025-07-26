//! 导出数据质量验证器
//!
//! 这个模块提供了全面的数据质量验证功能，确保导出的数据完整性、
//! 一致性和正确性，并在发现问题时提供详细的诊断信息。

use crate::core::types::{AllocationInfo, TrackingResult};
use crate::export::data_localizer::LocalizedExportData;
use crate::export::error_handling::{ExportError, ValidationType};
use crate::export::parallel_shard_processor::ProcessedShard;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Instant;

/// 数据质量验证器
#[derive(Debug)]
pub struct QualityValidator {
    /// 验证配置
    config: ValidationConfig,
    /// 验证统计
    stats: ValidationStats,
}

/// 验证配置
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// 是否启用 JSON 结构验证
    pub enable_json_validation: bool,
    /// 是否启用数据完整性验证
    pub enable_integrity_validation: bool,
    /// 是否启用分配计数验证
    pub enable_count_validation: bool,
    /// 是否启用文件大小验证
    pub enable_size_validation: bool,
    /// 是否启用编码验证
    pub enable_encoding_validation: bool,
    /// 最大允许的数据丢失率（百分比）
    pub max_data_loss_rate: f64,
    /// 最小预期文件大小（字节）
    pub min_expected_file_size: usize,
    /// 最大预期文件大小（字节）
    pub max_expected_file_size: usize,
    /// 是否启用详细日志
    pub verbose_logging: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_json_validation: true,
            enable_integrity_validation: true,
            enable_count_validation: true,
            enable_size_validation: true,
            enable_encoding_validation: true,
            max_data_loss_rate: 0.1, // 0.1% 最大数据丢失率
            min_expected_file_size: 1024, // 1KB 最小文件大小
            max_expected_file_size: 100 * 1024 * 1024, // 100MB 最大文件大小
            verbose_logging: false,
        }
    }
}

/// 验证统计
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// 总验证次数
    pub total_validations: usize,
    /// 成功验证次数
    pub successful_validations: usize,
    /// 失败验证次数
    pub failed_validations: usize,
    /// 各类型验证统计
    pub validation_type_stats: HashMap<ValidationType, ValidationTypeStats>,
    /// 总验证时间（毫秒）
    pub total_validation_time_ms: u64,
    /// 发现的问题数量
    pub issues_found: usize,
    /// 修复的问题数量
    pub issues_fixed: usize,
}

/// 单个验证类型的统计
#[derive(Debug, Clone, Default)]
pub struct ValidationTypeStats {
    /// 执行次数
    pub executions: usize,
    /// 成功次数
    pub successes: usize,
    /// 失败次数
    pub failures: usize,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 验证是否通过
    pub is_valid: bool,
    /// 验证类型
    pub validation_type: ValidationType,
    /// 验证消息
    pub message: String,
    /// 发现的问题
    pub issues: Vec<ValidationIssue>,
    /// 验证耗时
    pub validation_time_ms: u64,
    /// 验证的数据量
    pub data_size: usize,
}

/// 验证问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// 问题类型
    pub issue_type: IssueType,
    /// 问题描述
    pub description: String,
    /// 问题严重程度
    pub severity: IssueSeverity,
    /// 受影响的数据
    pub affected_data: String,
    /// 建议的修复方案
    pub suggested_fix: Option<String>,
    /// 是否可以自动修复
    pub auto_fixable: bool,
}

/// 问题类型
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    MissingData,
    CorruptedData,
    InconsistentData,
    InvalidFormat,
    SizeAnomaly,
    EncodingError,
    StructuralError,
    CountMismatch,
}

/// 问题严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl QualityValidator {
    /// 创建新的质量验证器
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            stats: ValidationStats::default(),
        }
    }

    /// 验证原始数据质量
    pub fn validate_source_data(&mut self, data: &LocalizedExportData) -> TrackingResult<ValidationResult> {
        let start_time = Instant::now();
        let mut issues = Vec::new();

        if self.config.verbose_logging {
            println!("🔍 开始验证原始数据质量...");
        }

        // 验证数据完整性
        if self.config.enable_integrity_validation {
            self.validate_data_integrity(data, &mut issues)?;
        }

        // 验证分配计数
        if self.config.enable_count_validation {
            self.validate_allocation_counts(data, &mut issues)?;
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        let is_valid = issues.iter().all(|issue| issue.severity != IssueSeverity::Critical);

        let result = ValidationResult {
            is_valid,
            validation_type: ValidationType::DataIntegrity,
            message: if is_valid {
                "原始数据质量验证通过".to_string()
            } else {
                format!("原始数据质量验证失败，发现 {} 个问题", issues.len())
            },
            issues,
            validation_time_ms: validation_time,
            data_size: data.allocations.len(),
        };

        self.update_stats(&result);

        if self.config.verbose_logging {
            self.print_validation_result(&result);
        }

        Ok(result)
    }

    /// 验证处理后的分片数据
    pub fn validate_processed_shards(&mut self, shards: &[ProcessedShard], original_count: usize) -> TrackingResult<ValidationResult> {
        let start_time = Instant::now();
        let mut issues = Vec::new();

        if self.config.verbose_logging {
            println!("🔍 开始验证处理后的分片数据...");
        }

        // 验证 JSON 结构
        if self.config.enable_json_validation {
            self.validate_json_structure(shards, &mut issues)?;
        }

        // 验证分配计数一致性
        if self.config.enable_count_validation {
            self.validate_shard_counts(shards, original_count, &mut issues)?;
        }

        // 验证数据大小
        if self.config.enable_size_validation {
            self.validate_data_sizes(shards, &mut issues)?;
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        let is_valid = issues.iter().all(|issue| issue.severity != IssueSeverity::Critical);

        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let result = ValidationResult {
            is_valid,
            validation_type: ValidationType::JsonStructure,
            message: if is_valid {
                "分片数据验证通过".to_string()
            } else {
                format!("分片数据验证失败，发现 {} 个问题", issues.len())
            },
            issues,
            validation_time_ms: validation_time,
            data_size: total_size,
        };

        self.update_stats(&result);

        if self.config.verbose_logging {
            self.print_validation_result(&result);
        }

        Ok(result)
    }

    /// 验证最终输出文件
    pub fn validate_output_file(&mut self, file_path: &str, expected_allocation_count: usize) -> TrackingResult<ValidationResult> {
        let start_time = Instant::now();
        let mut issues = Vec::new();

        if self.config.verbose_logging {
            println!("🔍 开始验证最终输出文件: {file_path}");
        }

        // 检查文件是否存在
        if !std::path::Path::new(file_path).exists() {
            issues.push(ValidationIssue {
                issue_type: IssueType::MissingData,
                description: "输出文件不存在".to_string(),
                severity: IssueSeverity::Critical,
                affected_data: file_path.to_string(),
                suggested_fix: Some("检查文件路径和写入权限".to_string()),
                auto_fixable: false,
            });
        } else {
            // 验证文件大小
            if self.config.enable_size_validation {
                self.validate_file_size(file_path, &mut issues)?;
            }

            // 验证文件内容
            if self.config.enable_json_validation {
                self.validate_file_content(file_path, expected_allocation_count, &mut issues)?;
            }

            // 验证编码
            if self.config.enable_encoding_validation {
                self.validate_file_encoding(file_path, &mut issues)?;
            }
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        let is_valid = issues.iter().all(|issue| issue.severity != IssueSeverity::Critical);

        let file_size = std::fs::metadata(file_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        let result = ValidationResult {
            is_valid,
            validation_type: ValidationType::FileSize,
            message: if is_valid {
                "输出文件验证通过".to_string()
            } else {
                format!("输出文件验证失败，发现 {} 个问题", issues.len())
            },
            issues,
            validation_time_ms: validation_time,
            data_size: file_size,
        };

        self.update_stats(&result);

        if self.config.verbose_logging {
            self.print_validation_result(&result);
        }

        Ok(result)
    }

    /// 验证数据完整性
    fn validate_data_integrity(&self, data: &LocalizedExportData, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        // 检查空数据
        if data.allocations.is_empty() {
            issues.push(ValidationIssue {
                issue_type: IssueType::MissingData,
                description: "分配数据为空".to_string(),
                severity: IssueSeverity::High,
                affected_data: "allocations".to_string(),
                suggested_fix: Some("检查内存跟踪器是否正常工作".to_string()),
                auto_fixable: false,
            });
        }

        // 检查数据一致性
        let mut ptr_set = HashSet::new();
        let mut duplicate_ptrs = Vec::new();

        for (index, allocation) in data.allocations.iter().enumerate() {
            // 检查重复指针
            if !ptr_set.insert(allocation.ptr) {
                duplicate_ptrs.push(allocation.ptr);
            }

            // 检查基本字段有效性
            if allocation.size == 0 {
                issues.push(ValidationIssue {
                    issue_type: IssueType::InvalidFormat,
                    description: format!("分配 {} 的大小为 0", index),
                    severity: IssueSeverity::Medium,
                    affected_data: format!("allocation[{}]", index),
                    suggested_fix: Some("检查分配跟踪逻辑".to_string()),
                    auto_fixable: false,
                });
            }

            // 检查时间戳有效性
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                if dealloc_time <= allocation.timestamp_alloc {
                    issues.push(ValidationIssue {
                        issue_type: IssueType::InconsistentData,
                        description: format!("分配 {} 的释放时间早于分配时间", index),
                        severity: IssueSeverity::High,
                        affected_data: format!("allocation[{}]", index),
                        suggested_fix: Some("检查时间戳生成逻辑".to_string()),
                        auto_fixable: false,
                    });
                }
            }
        }

        // 报告重复指针
        if !duplicate_ptrs.is_empty() {
            issues.push(ValidationIssue {
                issue_type: IssueType::InconsistentData,
                description: format!("发现 {} 个重复指针", duplicate_ptrs.len()),
                severity: IssueSeverity::High,
                affected_data: format!("pointers: {:?}", duplicate_ptrs),
                suggested_fix: Some("检查分配跟踪的去重逻辑".to_string()),
                auto_fixable: false,
            });
        }

        Ok(())
    }

    /// 验证分配计数
    fn validate_allocation_counts(&self, data: &LocalizedExportData, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        let allocation_count = data.allocations.len();
        let stats_count = data.stats.total_allocations;

        // 检查计数一致性
        if allocation_count != stats_count {
            let loss_rate = if stats_count > 0 {
                ((stats_count - allocation_count) as f64 / stats_count as f64) * 100.0
            } else {
                0.0
            };

            let severity = if loss_rate > self.config.max_data_loss_rate {
                IssueSeverity::Critical
            } else {
                IssueSeverity::Medium
            };

            issues.push(ValidationIssue {
                issue_type: IssueType::CountMismatch,
                description: format!("分配计数不一致: 实际 {allocation_count}, 统计 {stats_count}, 丢失率 {loss_rate:.2}%"),
                severity,
                affected_data: "allocation_count".to_string(),
                suggested_fix: Some("检查数据收集和统计逻辑".to_string()),
                auto_fixable: false,
            });
        }

        Ok(())
    }

    /// 验证 JSON 结构
    fn validate_json_structure(&self, shards: &[ProcessedShard], issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        for (index, shard) in shards.iter().enumerate() {
            // 尝试解析 JSON
            match serde_json::from_slice::<Vec<AllocationInfo>>(&shard.data) {
                Ok(allocations) => {
                    // 验证解析后的数据
                    if allocations.len() != shard.allocation_count {
                        issues.push(ValidationIssue {
                            issue_type: IssueType::CountMismatch,
                            description: format!("分片 {index} 的分配计数不匹配: 预期 {}, 实际 {}", 
                                               shard.allocation_count, allocations.len()),
                            severity: IssueSeverity::High,
                            affected_data: format!("shard[{}]", index),
                            suggested_fix: Some("检查分片处理逻辑".to_string()),
                            auto_fixable: false,
                        });
                    }
                }
                Err(e) => {
                    issues.push(ValidationIssue {
                        issue_type: IssueType::InvalidFormat,
                        description: format!("分片 {index} JSON 解析失败: {e}"),
                        severity: IssueSeverity::Critical,
                        affected_data: format!("shard[{}]", index),
                        suggested_fix: Some("检查 JSON 序列化逻辑".to_string()),
                        auto_fixable: false,
                    });
                }
            }
        }

        Ok(())
    }

    /// 验证分片计数
    fn validate_shard_counts(&self, shards: &[ProcessedShard], original_count: usize, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        let total_shard_count: usize = shards.iter().map(|s| s.allocation_count).sum();

        if total_shard_count != original_count {
            let loss_rate = if original_count > 0 {
                ((original_count - total_shard_count) as f64 / original_count as f64) * 100.0
            } else {
                0.0
            };

            let severity = if loss_rate > self.config.max_data_loss_rate {
                IssueSeverity::Critical
            } else {
                IssueSeverity::Medium
            };

            issues.push(ValidationIssue {
                issue_type: IssueType::CountMismatch,
                description: format!("分片总计数不匹配: 原始 {original_count}, 分片总计 {total_shard_count}, 丢失率 {loss_rate:.2}%"),
                severity,
                affected_data: "shard_counts".to_string(),
                suggested_fix: Some("检查分片处理和合并逻辑".to_string()),
                auto_fixable: false,
            });
        }

        Ok(())
    }

    /// 验证数据大小
    fn validate_data_sizes(&self, shards: &[ProcessedShard], issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        for (index, shard) in shards.iter().enumerate() {
            // 检查空分片
            if shard.data.is_empty() {
                issues.push(ValidationIssue {
                    issue_type: IssueType::MissingData,
                    description: format!("分片 {index} 数据为空"),
                    severity: IssueSeverity::High,
                    affected_data: format!("shard[{}]", index),
                    suggested_fix: Some("检查分片处理逻辑".to_string()),
                    auto_fixable: false,
                });
            }

            // 检查异常大小的分片
            let expected_min_size = shard.allocation_count * 50; // 每个分配至少 50 字节
            let expected_max_size = shard.allocation_count * 1000; // 每个分配最多 1000 字节

            if shard.data.len() < expected_min_size {
                issues.push(ValidationIssue {
                    issue_type: IssueType::SizeAnomaly,
                    description: format!("分片 {index} 大小异常小: {} 字节 (预期最少 {} 字节)", 
                                       shard.data.len(), expected_min_size),
                    severity: IssueSeverity::Medium,
                    affected_data: format!("shard[{}]", index),
                    suggested_fix: Some("检查序列化配置".to_string()),
                    auto_fixable: false,
                });
            }

            if shard.data.len() > expected_max_size {
                issues.push(ValidationIssue {
                    issue_type: IssueType::SizeAnomaly,
                    description: format!("分片 {index} 大小异常大: {} 字节 (预期最多 {} 字节)", 
                                       shard.data.len(), expected_max_size),
                    severity: IssueSeverity::Low,
                    affected_data: format!("shard[{}]", index),
                    suggested_fix: Some("考虑启用压缩".to_string()),
                    auto_fixable: false,
                });
            }
        }

        Ok(())
    }

    /// 验证文件大小
    fn validate_file_size(&self, file_path: &str, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| ExportError::DataQualityError {
                validation_type: ValidationType::FileSize,
                expected: "可读取的文件".to_string(),
                actual: format!("文件读取失败: {e}"),
                affected_records: 0,
            })?;

        let file_size = metadata.len() as usize;

        if file_size < self.config.min_expected_file_size {
            issues.push(ValidationIssue {
                issue_type: IssueType::SizeAnomaly,
                description: format!("文件大小过小: {} 字节 (最小预期 {} 字节)", 
                                   file_size, self.config.min_expected_file_size),
                severity: IssueSeverity::High,
                affected_data: file_path.to_string(),
                suggested_fix: Some("检查数据是否完整写入".to_string()),
                auto_fixable: false,
            });
        }

        if file_size > self.config.max_expected_file_size {
            issues.push(ValidationIssue {
                issue_type: IssueType::SizeAnomaly,
                description: format!("文件大小过大: {} 字节 (最大预期 {} 字节)", 
                                   file_size, self.config.max_expected_file_size),
                severity: IssueSeverity::Medium,
                affected_data: file_path.to_string(),
                suggested_fix: Some("考虑启用压缩或采样".to_string()),
                auto_fixable: false,
            });
        }

        Ok(())
    }

    /// 验证文件内容
    fn validate_file_content(&self, file_path: &str, expected_count: usize, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ExportError::DataQualityError {
                validation_type: ValidationType::JsonStructure,
                expected: "可读取的 JSON 文件".to_string(),
                actual: format!("文件读取失败: {e}"),
                affected_records: 0,
            })?;

        // 尝试解析 JSON
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => {
                // 检查 JSON 结构
                if let Some(allocations) = json.get("allocations") {
                    if let Some(array) = allocations.as_array() {
                        let actual_count = array.len();
                        if actual_count != expected_count {
                            let loss_rate = if expected_count > 0 {
                                ((expected_count - actual_count) as f64 / expected_count as f64) * 100.0
                            } else {
                                0.0
                            };

                            let severity = if loss_rate > self.config.max_data_loss_rate {
                                IssueSeverity::Critical
                            } else {
                                IssueSeverity::Medium
                            };

                            issues.push(ValidationIssue {
                                issue_type: IssueType::CountMismatch,
                                description: format!("文件中分配计数不匹配: 预期 {expected_count}, 实际 {actual_count}, 丢失率 {loss_rate:.2}%"),
                                severity,
                                affected_data: file_path.to_string(),
                                suggested_fix: Some("检查完整的导出流程".to_string()),
                                auto_fixable: false,
                            });
                        }
                    } else {
                        issues.push(ValidationIssue {
                            issue_type: IssueType::StructuralError,
                            description: "allocations 字段不是数组".to_string(),
                            severity: IssueSeverity::Critical,
                            affected_data: file_path.to_string(),
                            suggested_fix: Some("检查 JSON 结构生成逻辑".to_string()),
                            auto_fixable: false,
                        });
                    }
                } else {
                    issues.push(ValidationIssue {
                        issue_type: IssueType::StructuralError,
                        description: "缺少 allocations 字段".to_string(),
                        severity: IssueSeverity::Critical,
                        affected_data: file_path.to_string(),
                        suggested_fix: Some("检查 JSON 结构生成逻辑".to_string()),
                        auto_fixable: false,
                    });
                }
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    issue_type: IssueType::InvalidFormat,
                    description: format!("JSON 解析失败: {e}"),
                    severity: IssueSeverity::Critical,
                    affected_data: file_path.to_string(),
                    suggested_fix: Some("检查 JSON 格式和编码".to_string()),
                    auto_fixable: false,
                });
            }
        }

        Ok(())
    }

    /// 验证文件编码
    fn validate_file_encoding(&self, file_path: &str, issues: &mut Vec<ValidationIssue>) -> TrackingResult<()> {
        // 尝试以 UTF-8 读取文件
        match std::fs::read_to_string(file_path) {
            Ok(_) => {
                // UTF-8 读取成功，编码正确
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    issue_type: IssueType::EncodingError,
                    description: format!("文件编码验证失败: {e}"),
                    severity: IssueSeverity::High,
                    affected_data: file_path.to_string(),
                    suggested_fix: Some("确保文件以 UTF-8 编码保存".to_string()),
                    auto_fixable: false,
                });
            }
        }

        Ok(())
    }

    /// 更新统计信息
    fn update_stats(&mut self, result: &ValidationResult) {
        self.stats.total_validations += 1;
        
        if result.is_valid {
            self.stats.successful_validations += 1;
        } else {
            self.stats.failed_validations += 1;
        }

        self.stats.total_validation_time_ms += result.validation_time_ms;
        self.stats.issues_found += result.issues.len();

        // 更新验证类型统计
        let type_stats = self.stats.validation_type_stats
            .entry(result.validation_type.clone())
            .or_insert_with(ValidationTypeStats::default);

        type_stats.executions += 1;
        if result.is_valid {
            type_stats.successes += 1;
        } else {
            type_stats.failures += 1;
        }

        // 更新平均执行时间
        type_stats.avg_execution_time_ms = if type_stats.executions > 0 {
            (type_stats.avg_execution_time_ms * (type_stats.executions - 1) as f64 + result.validation_time_ms as f64) / type_stats.executions as f64
        } else {
            result.validation_time_ms as f64
        };
    }

    /// 打印验证结果
    fn print_validation_result(&self, result: &ValidationResult) {
        let status_icon = if result.is_valid { "✅" } else { "❌" };
        println!("{status_icon} 验证结果: {} ({}ms)", result.message, result.validation_time_ms);

        if !result.issues.is_empty() {
            println!("   发现的问题:");
            for (index, issue) in result.issues.iter().enumerate() {
                let severity_icon = match issue.severity {
                    IssueSeverity::Critical => "🔴",
                    IssueSeverity::High => "🟠",
                    IssueSeverity::Medium => "🟡",
                    IssueSeverity::Low => "🔵",
                    IssueSeverity::Info => "ℹ️",
                };
                println!("   {index}. {severity_icon} {}: {}", issue.issue_type, issue.description);
                if let Some(fix) = &issue.suggested_fix {
                    println!("      建议修复: {fix}");
                }
            }
        }
    }

    /// 获取验证统计
    pub fn get_stats(&self) -> &ValidationStats {
        &self.stats
    }

    /// 生成验证报告
    pub fn generate_validation_report(&self) -> ValidationReport {
        let success_rate = if self.stats.total_validations > 0 {
            (self.stats.successful_validations as f64 / self.stats.total_validations as f64) * 100.0
        } else {
            0.0
        };

        let avg_validation_time = if self.stats.total_validations > 0 {
            self.stats.total_validation_time_ms as f64 / self.stats.total_validations as f64
        } else {
            0.0
        };

        ValidationReport {
            total_validations: self.stats.total_validations,
            successful_validations: self.stats.successful_validations,
            failed_validations: self.stats.failed_validations,
            success_rate,
            avg_validation_time_ms: avg_validation_time,
            total_issues_found: self.stats.issues_found,
            total_issues_fixed: self.stats.issues_fixed,
            validation_type_breakdown: self.stats.validation_type_stats.clone(),
        }
    }
}

/// 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub total_validations: usize,
    pub successful_validations: usize,
    pub failed_validations: usize,
    pub success_rate: f64,
    pub avg_validation_time_ms: f64,
    pub total_issues_found: usize,
    pub total_issues_fixed: usize,
    pub validation_type_breakdown: HashMap<ValidationType, ValidationTypeStats>,
}

impl ValidationReport {
    /// 打印详细的验证报告
    pub fn print_detailed_report(&self) {
        println!("\n🔍 数据质量验证报告");
        println!("==================");
        
        println!("📊 总体统计:");
        println!("   总验证次数: {}", self.total_validations);
        println!("   成功验证: {} ({:.1}%)", self.successful_validations, self.success_rate);
        println!("   失败验证: {}", self.failed_validations);
        println!("   平均验证时间: {:.2}ms", self.avg_validation_time_ms);
        println!("   发现问题: {}", self.total_issues_found);
        println!("   修复问题: {}", self.total_issues_fixed);
        
        if !self.validation_type_breakdown.is_empty() {
            println!("\n🔍 验证类型统计:");
            for (validation_type, stats) in &self.validation_type_breakdown {
                let success_rate = if stats.executions > 0 {
                    (stats.successes as f64 / stats.executions as f64) * 100.0
                } else {
                    0.0
                };
                println!("   {validation_type:?}: {} 次执行, {:.1}% 成功率, {:.2}ms 平均时间", 
                        stats.executions, success_rate, stats.avg_execution_time_ms);
            }
        }
    }
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueType::MissingData => write!(f, "数据缺失"),
            IssueType::CorruptedData => write!(f, "数据损坏"),
            IssueType::InconsistentData => write!(f, "数据不一致"),
            IssueType::InvalidFormat => write!(f, "格式无效"),
            IssueType::SizeAnomaly => write!(f, "大小异常"),
            IssueType::EncodingError => write!(f, "编码错误"),
            IssueType::StructuralError => write!(f, "结构错误"),
            IssueType::CountMismatch => write!(f, "计数不匹配"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AllocationInfo, MemoryStats};
    use crate::analysis::unsafe_ffi_tracker::UnsafeFFIStats;
    use std::time::Instant;

    fn create_test_data(count: usize) -> LocalizedExportData {
        let mut allocations = Vec::new();
        for i in 0..count {
            allocations.push(AllocationInfo {
                ptr: 0x1000 + i,
                size: 64,
                type_name: Some("TestType".to_string()),
                var_name: Some(format!("var_{i}")),
                scope_name: Some("test_scope".to_string()),
                timestamp_alloc: 1000000 + i as u64,
                timestamp_dealloc: None,
                thread_id: "test_thread".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
            });
        }

        let mut stats = MemoryStats::new();
        stats.total_allocations = count;

        LocalizedExportData {
            allocations,
            enhanced_allocations: Vec::new(),
            stats,
            ffi_stats: UnsafeFFIStats::default(),
            scope_info: Vec::new(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_quality_validator_creation() {
        let config = ValidationConfig::default();
        let validator = QualityValidator::new(config);
        assert_eq!(validator.stats.total_validations, 0);
    }

    #[test]
    fn test_validate_source_data_success() {
        let mut validator = QualityValidator::new(ValidationConfig::default());
        let data = create_test_data(100);
        
        let result = validator.validate_source_data(&data);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.data_size, 100);
    }

    #[test]
    fn test_validate_source_data_empty() {
        let mut validator = QualityValidator::new(ValidationConfig::default());
        let data = create_test_data(0);
        
        let result = validator.validate_source_data(&data);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid); // 应该失败，因为数据为空
        assert!(!validation_result.issues.is_empty());
    }

    #[test]
    fn test_validate_processed_shards() {
        let mut validator = QualityValidator::new(ValidationConfig::default());
        
        let shards = vec![
            ProcessedShard {
                data: b"[{\"ptr\":4096,\"size\":64}]".to_vec(),
                allocation_count: 1,
                shard_index: 0,
                processing_time_ms: 10,
            }
        ];
        
        let result = validator.validate_processed_shards(&shards, 1);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
    }

    #[test]
    fn test_validation_stats() {
        let mut validator = QualityValidator::new(ValidationConfig::default());
        let data = create_test_data(50);
        
        // 执行几次验证
        let _ = validator.validate_source_data(&data);
        let _ = validator.validate_source_data(&data);
        
        let stats = validator.get_stats();
        assert_eq!(stats.total_validations, 2);
        assert_eq!(stats.successful_validations, 2);
        assert_eq!(stats.failed_validations, 0);
    }

    #[test]
    fn test_validation_report() {
        let mut validator = QualityValidator::new(ValidationConfig::default());
        let data = create_test_data(25);
        
        let _ = validator.validate_source_data(&data);
        
        let report = validator.generate_validation_report();
        assert_eq!(report.total_validations, 1);
        assert_eq!(report.success_rate, 100.0);
        assert!(report.avg_validation_time_ms > 0.0);
    }
}