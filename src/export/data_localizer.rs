//! 数据本地化器 - 减少全局状态访问开销
//!
//! 这个模块实现了数据本地化功能，一次性获取所有导出需要的数据，
//! 避免在导出过程中重复访问全局状态，从而显著提高导出性能。

use crate::analysis::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, EnhancedAllocationInfo, UnsafeFFIStats};
use crate::core::scope_tracker::get_global_scope_tracker;
use crate::core::types::ScopeInfo;
use crate::core::tracker::get_global_tracker;
use crate::core::types::MemoryStats;
use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use std::time::{Duration, Instant};

/// 数据本地化器 - 一次性获取所有导出需要的数据
pub struct DataLocalizer {
    /// 缓存的基础分配数据
    cached_allocations: Option<Vec<AllocationInfo>>,
    /// 缓存的 FFI 增强数据
    cached_ffi_data: Option<Vec<EnhancedAllocationInfo>>,
    /// 缓存的内存统计数据
    cached_stats: Option<MemoryStats>,
    /// 缓存的 FFI 统计数据
    cached_ffi_stats: Option<UnsafeFFIStats>,
    /// 缓存的作用域信息
    cached_scope_info: Option<Vec<ScopeInfo>>,
    /// 上次更新时间
    last_update: Instant,
    /// 缓存生存时间
    cache_ttl: Duration,
}

/// 本地化的导出数据，包含所有需要的信息
#[derive(Debug, Clone)]
pub struct LocalizedExportData {
    /// 基础内存分配信息
    pub allocations: Vec<AllocationInfo>,
    /// FFI 增强分配信息
    pub enhanced_allocations: Vec<EnhancedAllocationInfo>,
    /// 内存统计信息
    pub stats: MemoryStats,
    /// FFI 统计信息
    pub ffi_stats: UnsafeFFIStats,
    /// 作用域信息
    pub scope_info: Vec<ScopeInfo>,
    /// 数据获取时间戳
    pub timestamp: Instant,
}

/// 数据获取性能统计
#[derive(Debug, Clone)]
pub struct DataGatheringStats {
    /// 总耗时
    pub total_time_ms: u64,
    /// 基础数据获取耗时
    pub basic_data_time_ms: u64,
    /// FFI 数据获取耗时
    pub ffi_data_time_ms: u64,
    /// 作用域数据获取耗时
    pub scope_data_time_ms: u64,
    /// 获取的分配数量
    pub allocation_count: usize,
    /// 获取的 FFI 分配数量
    pub ffi_allocation_count: usize,
    /// 获取的作用域数量
    pub scope_count: usize,
}

impl DataLocalizer {
    /// 创建新的数据本地化器
    pub fn new() -> Self {
        Self {
            cached_allocations: None,
            cached_ffi_data: None,
            cached_stats: None,
            cached_ffi_stats: None,
            cached_scope_info: None,
            last_update: Instant::now(),
            cache_ttl: Duration::from_millis(100), // 100ms 缓存，避免过于频繁的数据获取
        }
    }

    /// 创建带自定义缓存时间的数据本地化器
    pub fn with_cache_ttl(cache_ttl: Duration) -> Self {
        Self {
            cached_allocations: None,
            cached_ffi_data: None,
            cached_stats: None,
            cached_ffi_stats: None,
            cached_scope_info: None,
            last_update: Instant::now(),
            cache_ttl,
        }
    }

    /// 一次性获取所有导出需要的数据
    /// 
    /// 这是核心方法，它会批量获取所有需要的数据，避免后续重复访问全局状态。
    /// 这样可以显著减少锁竞争和缓存未命中的问题。
    pub fn gather_all_export_data(&mut self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        let total_start = Instant::now();
        
        println!("🔄 开始数据本地化，减少全局状态访问...");

        // 检查缓存是否仍然有效
        if self.is_cache_valid() {
            println!("✅ 使用缓存数据，跳过重复获取");
            return self.get_cached_data();
        }

        // 第一步：获取基础内存跟踪数据
        let basic_start = Instant::now();
        let tracker = get_global_tracker();
        let allocations = tracker.get_active_allocations()
            .map_err(|e| TrackingError::ExportError(format!("获取基础分配数据失败: {}", e)))?;
        let stats = tracker.get_stats()
            .map_err(|e| TrackingError::ExportError(format!("获取内存统计失败: {}", e)))?;
        let basic_time = basic_start.elapsed();

        // 第二步：获取 FFI 相关数据
        let ffi_start = Instant::now();
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let enhanced_allocations = ffi_tracker.get_enhanced_allocations()
            .unwrap_or_else(|e| {
                eprintln!("⚠️ 获取 FFI 增强数据失败: {}, 使用空数据", e);
                Vec::new()
            });
        let ffi_stats = ffi_tracker.get_stats();
        let ffi_time = ffi_start.elapsed();

        // 第三步：获取作用域数据
        let scope_start = Instant::now();
        let scope_tracker = get_global_scope_tracker();
        let scope_info = scope_tracker.get_all_scopes();
        let scope_time = scope_start.elapsed();

        let total_time = total_start.elapsed();

        // 更新缓存
        self.cached_allocations = Some(allocations.clone());
        self.cached_ffi_data = Some(enhanced_allocations.clone());
        self.cached_stats = Some(stats.clone());
        self.cached_ffi_stats = Some(ffi_stats.clone());
        self.cached_scope_info = Some(scope_info.clone());
        self.last_update = Instant::now();

        let localized_data = LocalizedExportData {
            allocations: allocations.clone(),
            enhanced_allocations: enhanced_allocations.clone(),
            stats,
            ffi_stats,
            scope_info: scope_info.clone(),
            timestamp: total_start,
        };

        let gathering_stats = DataGatheringStats {
            total_time_ms: total_time.as_millis() as u64,
            basic_data_time_ms: basic_time.as_millis() as u64,
            ffi_data_time_ms: ffi_time.as_millis() as u64,
            scope_data_time_ms: scope_time.as_millis() as u64,
            allocation_count: allocations.len(),
            ffi_allocation_count: enhanced_allocations.len(),
            scope_count: scope_info.len(),
        };

        // 打印性能统计
        println!("✅ 数据本地化完成:");
        println!("   总耗时: {:?}", total_time);
        println!("   基础数据: {:?} ({} 个分配)", basic_time, gathering_stats.allocation_count);
        println!("   FFI 数据: {:?} ({} 个增强分配)", ffi_time, gathering_stats.ffi_allocation_count);
        println!("   作用域数据: {:?} ({} 个作用域)", scope_time, gathering_stats.scope_count);
        println!("   数据本地化减少了后续 {} 次全局状态访问", 
                self.estimate_avoided_global_accesses(&gathering_stats));

        Ok((localized_data, gathering_stats))
    }

    /// 强制刷新缓存，重新获取数据
    pub fn refresh_cache(&mut self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        self.invalidate_cache();
        self.gather_all_export_data()
    }

    /// 检查缓存是否仍然有效
    fn is_cache_valid(&self) -> bool {
        self.cached_allocations.is_some() 
            && self.cached_ffi_data.is_some()
            && self.cached_stats.is_some()
            && self.cached_ffi_stats.is_some()
            && self.cached_scope_info.is_some()
            && self.last_update.elapsed() < self.cache_ttl
    }

    /// 获取缓存的数据
    fn get_cached_data(&self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        let localized_data = LocalizedExportData {
            allocations: self.cached_allocations.as_ref().unwrap().clone(),
            enhanced_allocations: self.cached_ffi_data.as_ref().unwrap().clone(),
            stats: self.cached_stats.as_ref().unwrap().clone(),
            ffi_stats: self.cached_ffi_stats.as_ref().unwrap().clone(),
            scope_info: self.cached_scope_info.as_ref().unwrap().clone(),
            timestamp: self.last_update,
        };

        let gathering_stats = DataGatheringStats {
            total_time_ms: 0, // 缓存命中，无需时间
            basic_data_time_ms: 0,
            ffi_data_time_ms: 0,
            scope_data_time_ms: 0,
            allocation_count: localized_data.allocations.len(),
            ffi_allocation_count: localized_data.enhanced_allocations.len(),
            scope_count: localized_data.scope_info.len(),
        };

        Ok((localized_data, gathering_stats))
    }

    /// 使缓存失效
    pub fn invalidate_cache(&mut self) {
        self.cached_allocations = None;
        self.cached_ffi_data = None;
        self.cached_stats = None;
        self.cached_ffi_stats = None;
        self.cached_scope_info = None;
    }

    /// 估算避免的全局状态访问次数
    fn estimate_avoided_global_accesses(&self, stats: &DataGatheringStats) -> usize {
        // 在传统的导出过程中，每个分配可能需要多次访问全局状态
        // 这里估算我们通过数据本地化避免了多少次访问
        let basic_accesses = stats.allocation_count * 2; // 每个分配需要访问 tracker 2 次
        let ffi_accesses = stats.ffi_allocation_count * 3; // FFI 分配需要更多访问
        let scope_accesses = stats.scope_count * 1; // 作用域访问
        
        basic_accesses + ffi_accesses + scope_accesses
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            is_cached: self.is_cache_valid(),
            cache_age_ms: self.last_update.elapsed().as_millis() as u64,
            cache_ttl_ms: self.cache_ttl.as_millis() as u64,
            cached_allocation_count: self.cached_allocations.as_ref().map(|v| v.len()).unwrap_or(0),
            cached_ffi_count: self.cached_ffi_data.as_ref().map(|v| v.len()).unwrap_or(0),
            cached_scope_count: self.cached_scope_info.as_ref().map(|v| v.len()).unwrap_or(0),
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 是否有有效缓存
    pub is_cached: bool,
    /// 缓存年龄（毫秒）
    pub cache_age_ms: u64,
    /// 缓存生存时间（毫秒）
    pub cache_ttl_ms: u64,
    /// 缓存的分配数量
    pub cached_allocation_count: usize,
    /// 缓存的 FFI 分配数量
    pub cached_ffi_count: usize,
    /// 缓存的作用域数量
    pub cached_scope_count: usize,
}

impl Default for DataLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizedExportData {
    /// 获取数据的年龄
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// 检查数据是否仍然新鲜
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        self.age() < max_age
    }

    /// 获取总的分配数量（基础 + FFI）
    pub fn total_allocation_count(&self) -> usize {
        self.allocations.len() + self.enhanced_allocations.len()
    }

    /// 获取数据摘要
    pub fn get_summary(&self) -> String {
        format!(
            "LocalizedExportData {{ allocations: {}, ffi_allocations: {}, scopes: {}, age: {:?} }}",
            self.allocations.len(),
            self.enhanced_allocations.len(),
            self.scope_info.len(),
            self.age()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_localizer_creation() {
        let localizer = DataLocalizer::new();
        assert!(!localizer.is_cache_valid());
        
        let cache_stats = localizer.get_cache_stats();
        assert!(!cache_stats.is_cached);
        assert_eq!(cache_stats.cached_allocation_count, 0);
    }

    #[test]
    fn test_cache_ttl() {
        let short_ttl = Duration::from_millis(1);
        let mut localizer = DataLocalizer::with_cache_ttl(short_ttl);
        
        // 模拟缓存数据
        localizer.cached_allocations = Some(vec![]);
        localizer.cached_ffi_data = Some(vec![]);
        localizer.cached_stats = Some(MemoryStats::default());
        localizer.cached_ffi_stats = Some(UnsafeFFIStats::default());
        localizer.cached_scope_info = Some(vec![]);
        localizer.last_update = Instant::now();
        
        assert!(localizer.is_cache_valid());
        
        // 等待缓存过期
        std::thread::sleep(Duration::from_millis(2));
        assert!(!localizer.is_cache_valid());
    }

    #[test]
    fn test_localized_export_data() {
        let data = LocalizedExportData {
            allocations: vec![],
            enhanced_allocations: vec![],
            stats: MemoryStats::default(),
            ffi_stats: UnsafeFFIStats::default(),
            scope_info: vec![],
            timestamp: Instant::now(),
        };

        assert_eq!(data.total_allocation_count(), 0);
        assert!(data.is_fresh(Duration::from_secs(1)));
        
        let summary = data.get_summary();
        assert!(summary.contains("allocations: 0"));
    }
}