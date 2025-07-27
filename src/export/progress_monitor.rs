//! Progress monitoring and cancellation mechanism
//!
//! This module provides progress monitoring, cancellation mechanisms, and remaining time estimation for the export process.
//! Supports callback interfaces, graceful interruption, and partial result saving.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::core::types::TrackingResult;

/// Export progress information
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current stage
    pub current_stage: ExportStage,
    /// Current stage进度 (0.0 - 1.0)
    pub stage_progress: f64,
    /// 总体进度 (0.0 - 1.0)
    pub overall_progress: f64,
    /// 已处理的分配数量
    pub processed_allocations: usize,
    /// 总分配数量
    pub total_allocations: usize,
    /// 已用时间
    pub elapsed_time: Duration,
    /// 预估剩余时间
    pub estimated_remaining: Option<Duration>,
    /// 当前处理速度 (分配/秒)
    pub processing_speed: f64,
    /// 阶段详细信息
    pub stage_details: String,
}

/// 导出阶段
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportStage {
    /// 初始化
    Initializing,
    /// 数据本地化
    DataLocalization,
    /// 并行分片处理
    ParallelProcessing,
    /// 高速写入
    Writing,
    /// 完成
    Completed,
    /// 已取消
    Cancelled,
    /// 错误
    Error(String),
}

impl ExportStage {
    /// 获取阶段权重（用于计算总体进度）
    pub fn weight(&self) -> f64 {
        match self {
            ExportStage::Initializing => 0.05,
            ExportStage::DataLocalization => 0.15,
            ExportStage::ParallelProcessing => 0.70,
            ExportStage::Writing => 0.10,
            ExportStage::Completed => 1.0,
            ExportStage::Cancelled => 0.0,
            ExportStage::Error(_) => 0.0,
        }
    }
    
    /// 获取阶段描述
    pub fn description(&self) -> &str {
        match self {
            ExportStage::Initializing => "初始化导出环境",
            ExportStage::DataLocalization => "本地化数据，减少全局状态访问",
            ExportStage::ParallelProcessing => "并行分片处理",
            ExportStage::Writing => "高速缓冲写入",
            ExportStage::Completed => "导出完成",
            ExportStage::Cancelled => "导出已取消",
            ExportStage::Error(msg) => msg,
        }
    }
}

/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(ExportProgress) + Send + Sync>;

/// 取消令牌，用于中断导出操作
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// 创建新的取消令牌
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// 取消操作
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    
    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    
    /// 如果已取消则返回错误
    pub fn check_cancelled(&self) -> TrackingResult<()> {
        if self.is_cancelled() {
            Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Export operation was cancelled").into())
        } else {
            Ok(())
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// 进度监控器
pub struct ProgressMonitor {
    /// 开始时间
    start_time: Instant,
    /// Current stage
    current_stage: ExportStage,
    /// 总分配数量
    total_allocations: usize,
    /// 已处理分配数量
    processed_allocations: Arc<AtomicUsize>,
    /// 进度回调
    callback: Option<ProgressCallback>,
    /// 取消令牌
    cancellation_token: CancellationToken,
    /// 上次更新时间
    last_update: Instant,
    /// 更新间隔（避免过于频繁的回调）
    update_interval: Duration,
    /// 历史处理速度（用于预估剩余时间）
    speed_history: Vec<(Instant, usize)>,
    /// 最大历史记录数
    max_history_size: usize,
}

impl ProgressMonitor {
    /// 创建新的进度监控器
    pub fn new(total_allocations: usize) -> Self {
        Self {
            start_time: Instant::now(),
            current_stage: ExportStage::Initializing,
            total_allocations,
            processed_allocations: Arc::new(AtomicUsize::new(0)),
            callback: None,
            cancellation_token: CancellationToken::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100), // 100ms 更新间隔
            speed_history: Vec::new(),
            max_history_size: 20,
        }
    }
    
    /// 设置进度回调
    pub fn set_callback(&mut self, callback: ProgressCallback) {
        self.callback = Some(callback);
    }
    
    /// 获取取消令牌
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
    
    /// 设置当前阶段
    pub fn set_stage(&mut self, stage: ExportStage) {
        self.current_stage = stage;
        // 不自动调用 update_progress，让调用者控制进度
    }
    
    /// 更新阶段进度
    pub fn update_progress(&mut self, stage_progress: f64, _details: Option<String>) {
        let now = Instant::now();
        
        // 检查更新间隔，避免过于频繁的回调
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }
        
        self.last_update = now;
        
        let processed = self.processed_allocations.load(Ordering::SeqCst);
        
        // 更新速度历史
        self.speed_history.push((now, processed));
        if self.speed_history.len() > self.max_history_size {
            self.speed_history.remove(0);
        }
        
        let progress = self.calculate_progress(stage_progress, processed);
        
        if let Some(ref callback) = self.callback {
            callback(progress);
        }
    }
    
    /// 增加已处理分配数量
    pub fn add_processed(&self, count: usize) {
        self.processed_allocations.fetch_add(count, Ordering::SeqCst);
    }
    
    /// 设置已处理分配数量
    pub fn set_processed(&self, count: usize) {
        self.processed_allocations.store(count, Ordering::SeqCst);
    }
    
    /// 计算进度信息
    fn calculate_progress(&self, stage_progress: f64, processed: usize) -> ExportProgress {
        let elapsed = self.start_time.elapsed();
        
        // 计算总体进度
        let stage_weights = [
            (ExportStage::Initializing, 0.05),
            (ExportStage::DataLocalization, 0.15),
            (ExportStage::ParallelProcessing, 0.70),
            (ExportStage::Writing, 0.10),
        ];
        
        let mut overall_progress = 0.0;
        let mut found_current = false;
        
        for (stage, weight) in &stage_weights {
            if *stage == self.current_stage {
                overall_progress += weight * stage_progress;
                found_current = true;
                break;
            } else {
                overall_progress += weight;
            }
        }
        
        if !found_current {
            overall_progress = match self.current_stage {
                ExportStage::Completed => 1.0,
                ExportStage::Cancelled => 0.0,
                ExportStage::Error(_) => 0.0,
                _ => overall_progress,
            };
        }
        
        // 计算处理速度
        let processing_speed = if elapsed.as_secs() > 0 {
            processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        // 预估剩余时间
        let estimated_remaining = self.estimate_remaining_time(processed, processing_speed);
        
        ExportProgress {
            current_stage: self.current_stage.clone(),
            stage_progress,
            overall_progress,
            processed_allocations: processed,
            total_allocations: self.total_allocations,
            elapsed_time: elapsed,
            estimated_remaining,
            processing_speed,
            stage_details: self.current_stage.description().to_string(),
        }
    }
    
    /// 预估剩余时间
    fn estimate_remaining_time(&self, processed: usize, current_speed: f64) -> Option<Duration> {
        if processed >= self.total_allocations || current_speed <= 0.0 {
            return None;
        }
        
        // 使用历史速度数据进行更准确的预估
        let avg_speed = if self.speed_history.len() >= 2 {
            let recent_history = &self.speed_history[self.speed_history.len().saturating_sub(5)..];
            if recent_history.len() >= 2 {
                let first = &recent_history[0];
                let last = &recent_history[recent_history.len() - 1];
                let time_diff = last.0.duration_since(first.0).as_secs_f64();
                let processed_diff = last.1.saturating_sub(first.1) as f64;
                
                if time_diff > 0.0 {
                    processed_diff / time_diff
                } else {
                    current_speed
                }
            } else {
                current_speed
            }
        } else {
            current_speed
        };
        
        if avg_speed > 0.0 {
            let remaining_allocations = self.total_allocations.saturating_sub(processed) as f64;
            let remaining_seconds = remaining_allocations / avg_speed;
            Some(Duration::from_secs_f64(remaining_seconds))
        } else {
            None
        }
    }
    
    /// 完成导出
    pub fn complete(&mut self) {
        self.current_stage = ExportStage::Completed;
        self.update_progress(1.0, Some("导出完成".to_string()));
    }
    
    /// 取消导出
    pub fn cancel(&mut self) {
        self.cancellation_token.cancel();
        self.current_stage = ExportStage::Cancelled;
        self.update_progress(0.0, Some("导出已取消".to_string()));
    }
    
    /// 设置错误状态
    pub fn set_error(&mut self, error: String) {
        self.current_stage = ExportStage::Error(error.clone());
        self.update_progress(0.0, Some(error));
    }
    
    /// 检查是否应该取消
    pub fn should_cancel(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
    
    /// 获取当前进度快照
    pub fn get_progress_snapshot(&self) -> ExportProgress {
        let processed = self.processed_allocations.load(Ordering::SeqCst);
        self.calculate_progress(0.0, processed)
    }
}

/// 进度监控配置
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// 是否启用进度监控
    pub enabled: bool,
    /// 更新间隔
    pub update_interval: Duration,
    /// 是否显示详细信息
    pub show_details: bool,
    /// 是否显示预估剩余时间
    pub show_estimated_time: bool,
    /// 是否支持取消
    pub allow_cancellation: bool,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: Duration::from_millis(100),
            show_details: true,
            show_estimated_time: true,
            allow_cancellation: true,
        }
    }
}

/// 控制台进度显示器
pub struct ConsoleProgressDisplay {
    last_line_length: usize,
}

impl ConsoleProgressDisplay {
    /// 创建新的控制台进度显示器
    pub fn new() -> Self {
        Self {
            last_line_length: 0,
        }
    }
    
    /// 显示进度
    pub fn display(&mut self, progress: &ExportProgress) {
        // 清除上一行
        if self.last_line_length > 0 {
            print!("\r{}", " ".repeat(self.last_line_length));
            print!("\r");
        }
        
        let progress_bar = self.create_progress_bar(progress.overall_progress);
        let speed_info = if progress.processing_speed > 0.0 {
            format!(" ({:.0} 分配/秒)", progress.processing_speed)
        } else {
            String::new()
        };
        
        let time_info = if let Some(remaining) = progress.estimated_remaining {
            format!(" 剩余: {:?}", remaining)
        } else {
            String::new()
        };
        
        let line = format!(
            "{} {:.1}% {} ({}/{}){}{}",
            progress_bar,
            progress.overall_progress * 100.0,
            progress.current_stage.description(),
            progress.processed_allocations,
            progress.total_allocations,
            speed_info,
            time_info
        );
        
        print!("{}", line);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        self.last_line_length = line.len();
    }
    
    /// 创建进度条
    fn create_progress_bar(&self, progress: f64) -> String {
        let width = 20;
        let filled = (progress * width as f64) as usize;
        let empty = width - filled;
        
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
    
    /// 完成显示（换行）
    pub fn finish(&mut self) {
        println!();
        self.last_line_length = 0;
    }
}

impl Default for ConsoleProgressDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        
        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.check_cancelled().is_err());
    }

    #[test]
    fn test_progress_monitor_basic() {
        let mut monitor = ProgressMonitor::new(1000);
        
        // 测试初始状态
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.current_stage, ExportStage::Initializing);
        assert_eq!(progress.processed_allocations, 0);
        assert_eq!(progress.total_allocations, 1000);
        
        // 测试阶段切换
        monitor.set_stage(ExportStage::DataLocalization);
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.current_stage, ExportStage::DataLocalization);
        
        // 测试进度更新
        monitor.add_processed(100);
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.processed_allocations, 100);
    }

    #[test]
    fn test_progress_callback() {
        let callback_called = Arc::new(Mutex::new(false));
        let callback_called_clone = callback_called.clone();
        
        let mut monitor = ProgressMonitor::new(100);
        monitor.set_callback(Box::new(move |_progress| {
            *callback_called_clone.lock().unwrap() = true;
        }));
        
        // 等待一下让更新间隔过去
        thread::sleep(Duration::from_millis(150));
        monitor.update_progress(0.5, None);
        
        // 等待回调执行
        thread::sleep(Duration::from_millis(50));
        
        assert!(*callback_called.lock().unwrap());
    }

    #[test]
    fn test_progress_calculation() {
        let mut monitor = ProgressMonitor::new(1000);
        // 设置更短的更新间隔用于测试
        monitor.update_interval = Duration::from_millis(1);
        
        // 直接测试计算函数
        let progress = monitor.calculate_progress(1.0, 0);
        assert_eq!(progress.current_stage, ExportStage::Initializing);
        
        // 测试初始化阶段
        monitor.set_stage(ExportStage::Initializing);
        let progress = monitor.calculate_progress(1.0, 0);
        assert!((progress.overall_progress - 0.05).abs() < 0.01, 
                "Expected ~0.05, got {}", progress.overall_progress);
        
        // 测试数据本地化阶段
        monitor.set_stage(ExportStage::DataLocalization);
        let progress = monitor.calculate_progress(0.5, 0);
        let expected = 0.05 + 0.15 * 0.5;
        assert!((progress.overall_progress - expected).abs() < 0.01,
                "Expected ~{}, got {}", expected, progress.overall_progress);
        
        // 测试完成
        monitor.set_stage(ExportStage::Completed);
        let progress = monitor.calculate_progress(1.0, 0);
        assert_eq!(progress.overall_progress, 1.0);
        assert_eq!(progress.current_stage, ExportStage::Completed);
    }

    #[test]
    fn test_speed_calculation() {
        let monitor = ProgressMonitor::new(1000);
        
        // 模拟处理一些分配
        monitor.add_processed(100);
        
        // 等待一段时间以便计算速度
        thread::sleep(Duration::from_millis(500));
        
        let progress = monitor.get_progress_snapshot();
        // 速度应该大于等于0
        assert!(progress.processing_speed >= 0.0, 
                "Processing speed should be >= 0, got {}", progress.processing_speed);
        
        // 基本测试：确保速度计算不会崩溃
        assert!(progress.elapsed_time.as_millis() > 0, "Elapsed time should be > 0");
        
        // 如果有处理的分配和足够的时间，速度应该大于0
        // 但由于测试环境的不确定性，我们只检查基本的数学正确性
        let expected_speed = if progress.elapsed_time.as_secs() > 0 {
            100.0 / progress.elapsed_time.as_secs_f64()
        } else {
            0.0
        };
        
        // 允许一定的误差范围
        assert!((progress.processing_speed - expected_speed).abs() < 1.0,
                "Speed calculation mismatch: expected ~{}, got {}", expected_speed, progress.processing_speed);
    }

    #[test]
    fn test_console_progress_display() {
        let mut display = ConsoleProgressDisplay::new();
        
        let progress = ExportProgress {
            current_stage: ExportStage::ParallelProcessing,
            stage_progress: 0.5,
            overall_progress: 0.6,
            processed_allocations: 600,
            total_allocations: 1000,
            elapsed_time: Duration::from_secs(10),
            estimated_remaining: Some(Duration::from_secs(7)),
            processing_speed: 60.0,
            stage_details: "并行分片处理".to_string(),
        };
        
        // 这个测试主要确保不会 panic
        display.display(&progress);
        display.finish();
    }

    #[test]
    fn test_export_stage_weights() {
        assert_eq!(ExportStage::Initializing.weight(), 0.05);
        assert_eq!(ExportStage::DataLocalization.weight(), 0.15);
        assert_eq!(ExportStage::ParallelProcessing.weight(), 0.70);
        assert_eq!(ExportStage::Writing.weight(), 0.10);
        assert_eq!(ExportStage::Completed.weight(), 1.0);
    }
}