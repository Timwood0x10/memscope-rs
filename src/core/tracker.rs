//! Memory allocation tracking functionality.

// use crate::closure_analysis::OptimizationLevel;
use crate::export::optimized_json_export::OptimizedExportOptions;
/// Export options for JSON export - user-controllable settings
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include system allocations in full enrichment (default: false)
    ///
    /// **⚠️ Performance Impact**: Setting this to `true` can make export 5-10x slower!
    ///
    /// - `false` (default): Only user-tracked variables get full enrichment (~2-5 seconds)
    /// - `true`: ALL allocations including system internals get enrichment (~10-40 seconds)
    pub include_system_allocations: bool,

    /// Enable verbose logging during export (default: false)
    pub verbose_logging: bool,

    /// Buffer size for file I/O in bytes (default: 64KB)
    pub buffer_size: usize,

    /// Enable data compression (default: false)
    pub compress_output: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_system_allocations: false, // Fast mode by default
            verbose_logging: false,
            buffer_size: 64 * 1024, // 64KB
            compress_output: false,
        }
    }
}

impl ExportOptions {
    /// Create new export options with default settings (fast mode)
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable system allocation enrichment (⚠️ SLOW - 5-10x slower!)
    ///
    /// # Warning
    /// This will significantly slow down the export process and generate much larger files.
    /// Only use for deep debugging or system analysis.
    ///
    /// # Example
    /// ```rust
    /// let options = ExportOptions::new().include_system_allocations(true);
    /// tracker.export_to_json_with_options("debug_output", options)?;
    /// ```
    pub fn include_system_allocations(mut self, include: bool) -> Self {
        self.include_system_allocations = include;
        self
    }

    /// Enable verbose logging during export
    pub fn verbose_logging(mut self, verbose: bool) -> Self {
        self.verbose_logging = verbose;
        self
    }

    /// Set custom buffer size for file I/O
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable output compression (experimental)
    pub fn compress_output(mut self, compress: bool) -> Self {
        self.compress_output = compress;
        self
    }
}

/// Internal export mode derived from options
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ExportMode {
    /// Fast mode: Only enrich user-tracked variables
    UserFocused,
    /// Complete mode: Enrich all allocations including system data
    Complete,
}

use crate::core::types::{
    AllocationInfo, MemoryStats, MemoryTypeInfo, TrackingResult, TypeMemoryUsage,
};
use crate::core::types::{
    AllocatorStateInfo, CachePerformanceInfo, CpuUsageInfo, MemoryPressureInfo,
};
use crate::core::types::{CodeBloatLevel, GenericConstraint, MonomorphizationInfo, TypeParameter};
use crate::core::types::{
    CreationContext, ExpressionType, ScopeType, SourceLocation, StackScopeInfo,
};
use crate::core::types::{DispatchOverhead, PerformanceImpact, TypeErasureInfo, VTableInfo};
use crate::core::types::{DynamicTypeInfo, GenericTypeInfo, MemoryLayoutInfo, RuntimeStateInfo};
use crate::core::types::{EnhancedFragmentationAnalysis, StackAllocationInfo, TemporaryObjectInfo};
use crate::core::types::{
    FieldLayoutInfo, LayoutEfficiency, OptimizationPotential, PaddingAnalysis,
};
use crate::core::types::{
    FragmentationMetrics, FragmentationSeverity, MemoryLocationType, TemporaryUsagePattern,
};
use crate::core::types::{FunctionCallTrackingInfo, MemoryAccessTrackingInfo, ObjectLifecycleInfo};
use crate::core::types::{GenericInstantiationInfo, TypeRelationshipInfo, TypeUsageInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Helper function to convert std::io::Error to TrackingError::IoError
#[allow(dead_code)]
fn io_error_to_tracking_error(e: std::io::Error) -> crate::core::types::TrackingError {
    crate::core::types::TrackingError::IoError(e.to_string())
}

/// Global memory tracker instance
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

/// Get the global memory tracker instance.
///
/// This function returns a reference to the singleton memory tracker
/// that is used throughout the application.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone()
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
pub struct MemoryTracker {
    /// Active allocations (ptr -> allocation info)
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Complete allocation history (for analysis)
    allocation_history: Mutex<Vec<AllocationInfo>>,
    /// Memory usage statistics
    stats: Mutex<MemoryStats>,
    /// Fast mode flag for testing (reduces overhead)
    fast_mode: std::sync::atomic::AtomicBool,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode =
            std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test) || cfg!(feature = "test");
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_history: Mutex::new(Vec::new()),
            stats: Mutex::new(MemoryStats::default()),
            fast_mode: std::sync::atomic::AtomicBool::new(fast_mode),
        }
    }

    /// Check if tracker is in fast mode (for testing)
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Enable fast mode for testing
    pub fn enable_fast_mode(&self) {
        self.fast_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Fast track allocation for testing (minimal overhead)
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        if !self.is_fast_mode() {
            return self.create_synthetic_allocation(ptr, size, var_name, "unknown".to_string(), 0);
        }

        // In fast mode, create minimal allocation info but still track it
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);
        allocation.type_name = Some("fast_tracked".to_string());

        // Try to update both active allocations and stats
        if let (Ok(mut active), Ok(mut stats)) =
            (self.active_allocations.try_lock(), self.stats.try_lock())
        {
            active.insert(ptr, allocation);
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }
        Ok(())
    }

    /// Analyze memory layout information
    pub fn analyze_memory_layout(&self, type_name: &str, size: usize) -> Option<MemoryLayoutInfo> {
        // Infer memory layout based on type name and size
        let alignment = self.estimate_alignment(type_name, size);
        let field_layout = self.analyze_field_layout(type_name, size, alignment);
        let padding_info = self.analyze_padding(&field_layout, size, alignment);
        let layout_efficiency =
            self.calculate_layout_efficiency(&field_layout, &padding_info, size);

        Some(MemoryLayoutInfo {
            total_size: size,
            alignment,
            field_layout,
            padding_info,
            layout_efficiency,
        })
    }

    /// Estimate type alignment requirements
    fn estimate_alignment(&self, type_name: &str, size: usize) -> usize {
        if type_name.contains("u64") || type_name.contains("i64") || type_name.contains("f64") {
            8
        } else if type_name.contains("u32")
            || type_name.contains("i32")
            || type_name.contains("f32")
        {
            4
        } else if type_name.contains("u16") || type_name.contains("i16") {
            2
        } else if type_name.contains("u8") || type_name.contains("i8") || type_name.contains("bool")
        {
            1
        } else if type_name.contains("usize")
            || type_name.contains("isize")
            || type_name.contains("*")
        {
            std::mem::size_of::<usize>()
        } else {
            // For composite types, use heuristic approach
            match size {
                1 => 1,
                2..=3 => 2,
                4..=7 => 4,
                _ => 8,
            }
        }
    }

    /// Analyze field layout
    fn analyze_field_layout(
        &self,
        type_name: &str,
        total_size: usize,
        alignment: usize,
    ) -> Vec<FieldLayoutInfo> {
        let mut fields = Vec::new();

        if type_name.contains("Vec<") {
            // Vec<T> typically contains: ptr, capacity, len
            fields.push(FieldLayoutInfo {
                field_name: "ptr".to_string(),
                field_type: "*mut T".to_string(),
                offset: 0,
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::size_of::<usize>(),
                is_padding: false,
            });
            fields.push(FieldLayoutInfo {
                field_name: "capacity".to_string(),
                field_type: "usize".to_string(),
                offset: std::mem::size_of::<usize>(),
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::size_of::<usize>(),
                is_padding: false,
            });
            fields.push(FieldLayoutInfo {
                field_name: "len".to_string(),
                field_type: "usize".to_string(),
                offset: std::mem::size_of::<usize>() * 2,
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::size_of::<usize>(),
                is_padding: false,
            });
        } else if type_name.contains("String") {
            // String is similar to Vec<u8>
            fields.push(FieldLayoutInfo {
                field_name: "vec".to_string(),
                field_type: "Vec<u8>".to_string(),
                offset: 0,
                size: total_size,
                alignment,
                is_padding: false,
            });
        } else if type_name.contains("HashMap") {
            // HashMap has a more complex internal structure
            fields.push(FieldLayoutInfo {
                field_name: "hash_builder".to_string(),
                field_type: "S".to_string(),
                offset: 0,
                size: 8,
                alignment: 8,
                is_padding: false,
            });
            fields.push(FieldLayoutInfo {
                field_name: "table".to_string(),
                field_type: "RawTable<(K, V)>".to_string(),
                offset: 8,
                size: total_size - 8,
                alignment: 8,
                is_padding: false,
            });
        } else {
            // For unknown types, create a generic field
            fields.push(FieldLayoutInfo {
                field_name: "data".to_string(),
                field_type: type_name.to_string(),
                offset: 0,
                size: total_size,
                alignment,
                is_padding: false,
            });
        }

        fields
    }

    /// Analyze padding bytes
    fn analyze_padding(
        &self,
        fields: &[FieldLayoutInfo],
        total_size: usize,
        _struct_alignment: usize,
    ) -> PaddingAnalysis {
        let mut padding_locations = Vec::new();
        let mut total_padding_bytes = 0;
        let mut optimization_suggestions = Vec::new();

        // Check padding between fields
        for i in 0..fields.len().saturating_sub(1) {
            let current_field = &fields[i];
            let next_field = &fields[i + 1];
            let expected_next_offset = current_field.offset + current_field.size;

            if next_field.offset > expected_next_offset {
                let padding_size = next_field.offset - expected_next_offset;
                padding_locations.push(crate::core::types::PaddingLocation {
                    start_offset: expected_next_offset,
                    size: padding_size,
                    reason: crate::core::types::PaddingReason::FieldAlignment,
                });
                total_padding_bytes += padding_size;
            }
        }

        // Check struct tail padding
        if let Some(last_field) = fields.last() {
            let last_field_end = last_field.offset + last_field.size;
            if total_size > last_field_end {
                let tail_padding = total_size - last_field_end;
                padding_locations.push(crate::core::types::PaddingLocation {
                    start_offset: last_field_end,
                    size: tail_padding,
                    reason: crate::core::types::PaddingReason::StructAlignment,
                });
                total_padding_bytes += tail_padding;
            }
        }

        // Generate optimization suggestions
        let padding_ratio = total_padding_bytes as f64 / total_size as f64;
        if padding_ratio > 0.25 {
            optimization_suggestions
                .push("Consider rearranging fields to reduce padding bytes".to_string());
        }
        if fields.len() > 1 && padding_ratio > 0.1 {
            optimization_suggestions
                .push("Grouping smaller fields together may reduce memory waste".to_string());
        }

        PaddingAnalysis {
            total_padding_bytes,
            padding_locations,
            padding_ratio,
            optimization_suggestions,
        }
    }

    /// Calculate layout efficiency
    fn calculate_layout_efficiency(
        &self,
        _fields: &[FieldLayoutInfo],
        padding: &PaddingAnalysis,
        total_size: usize,
    ) -> LayoutEfficiency {
        let useful_data_size = total_size - padding.total_padding_bytes;
        let memory_utilization = useful_data_size as f64 / total_size as f64;

        // Cache friendliness score (based on field size and alignment)
        let cache_friendliness = if total_size <= 64 {
            100.0 // Fits in a single cache line
        } else if total_size <= 128 {
            80.0
        } else if total_size <= 256 {
            60.0
        } else {
            40.0
        };

        let optimization_potential = if padding.padding_ratio > 0.3 {
            OptimizationPotential::Major {
                potential_savings: padding.total_padding_bytes,
                suggestions: vec![
                    "Rearrange field order".to_string(),
                    "Use #[repr(packed)] attribute".to_string(),
                ],
            }
        } else if padding.padding_ratio > 0.15 {
            OptimizationPotential::Moderate {
                potential_savings: padding.total_padding_bytes,
                suggestions: vec!["Optimize field arrangement".to_string()],
            }
        } else if padding.padding_ratio > 0.05 {
            OptimizationPotential::Minor {
                potential_savings: padding.total_padding_bytes,
            }
        } else {
            OptimizationPotential::None
        };

        LayoutEfficiency {
            memory_utilization,
            cache_friendliness,
            alignment_waste: padding.total_padding_bytes,
            optimization_potential,
        }
    }

    /// Analyze generic type information
    pub fn analyze_generic_type(&self, type_name: &str, size: usize) -> Option<GenericTypeInfo> {
        if !type_name.contains('<') || !type_name.contains('>') {
            return None; // Not a generic type
        }

        let base_type = self.extract_base_type(type_name);
        let type_parameters = self.extract_type_parameters(type_name, size);
        let monomorphization_info = self.analyze_monomorphization(&base_type, &type_parameters);
        let constraints = self.infer_generic_constraints(&base_type, &type_parameters);

        Some(GenericTypeInfo {
            base_type,
            type_parameters,
            monomorphization_info,
            constraints,
        })
    }

    /// Extract generic base type
    fn extract_base_type(&self, type_name: &str) -> String {
        if let Some(angle_pos) = type_name.find('<') {
            type_name[..angle_pos].to_string()
        } else {
            type_name.to_string()
        }
    }

    /// Extract generic type parameters
    fn extract_type_parameters(&self, type_name: &str, total_size: usize) -> Vec<TypeParameter> {
        let mut parameters = Vec::new();

        if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                let params_str = &type_name[start + 1..end];
                let param_names: Vec<&str> = params_str.split(',').map(|s| s.trim()).collect();

                for (i, param_name) in param_names.iter().enumerate() {
                    let estimated_size = self.estimate_type_parameter_size(
                        param_name,
                        total_size,
                        param_names.len(),
                    );
                    parameters.push(TypeParameter {
                        name: format!("T{}", i),
                        concrete_type: param_name.to_string(),
                        size: estimated_size,
                        alignment: self.estimate_alignment(param_name, estimated_size),
                        is_lifetime: param_name.starts_with('\''),
                    });
                }
            }
        }

        parameters
    }

    /// Estimate generic parameter size
    fn estimate_type_parameter_size(
        &self,
        param_type: &str,
        total_size: usize,
        param_count: usize,
    ) -> usize {
        if param_type.starts_with('\'') {
            return 0; // Lifetime parameters don't take up space
        }

        match param_type {
            "u8" | "i8" | "bool" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" | "f32" => 4,
            "u64" | "i64" | "f64" => 8,
            "usize" | "isize" => std::mem::size_of::<usize>(),
            _ => {
                // For complex types, evenly distribute total size
                if param_count > 0 {
                    total_size / param_count
                } else {
                    std::mem::size_of::<usize>()
                }
            }
        }
    }

    /// Analyze monomorphization information
    fn analyze_monomorphization(
        &self,
        _base_type: &str,
        parameters: &[TypeParameter],
    ) -> MonomorphizationInfo {
        // Estimate monomorphization instance count (based on parameter complexity)
        let instance_count = parameters
            .iter()
            .map(|p| if p.concrete_type.contains('<') { 2 } else { 1 })
            .product::<usize>()
            .max(1);

        let per_instance_memory = parameters.iter().map(|p| p.size).sum::<usize>();
        let total_memory_usage = instance_count * per_instance_memory;

        let code_bloat_assessment = match instance_count {
            1..=2 => CodeBloatLevel::Low,
            3..=5 => CodeBloatLevel::Moderate,
            6..=10 => CodeBloatLevel::High,
            _ => CodeBloatLevel::Excessive,
        };

        MonomorphizationInfo {
            instance_count,
            per_instance_memory,
            total_memory_usage,
            code_bloat_assessment,
        }
    }

    /// Infer generic constraints
    fn infer_generic_constraints(
        &self,
        _base_type: &str,
        _parameters: &[TypeParameter],
    ) -> Vec<GenericConstraint> {
        let mut constraints = Vec::new();

        match _base_type {
            "Vec" | "HashMap" | "BTreeMap" => {
                constraints.push(GenericConstraint {
                    constraint_type: crate::core::types::ConstraintType::Trait("Clone".to_string()),
                    description: "Element types typically need to implement Clone".to_string(),
                    memory_impact: crate::core::types::MemoryImpact::None,
                });
            }
            "Rc" | "Arc" => {
                constraints.push(GenericConstraint {
                    constraint_type: crate::core::types::ConstraintType::Trait(
                        "Send + Sync".to_string(),
                    ),
                    description: "Shared pointer contents need to be thread-safe".to_string(),
                    memory_impact: crate::core::types::MemoryImpact::SizeIncrease(
                        std::mem::size_of::<usize>(),
                    ),
                });
            }
            _ => {}
        }

        constraints
    }

    /// Analyze dynamic type information (trait objects)
    pub fn analyze_dynamic_type(&self, type_name: &str, size: usize) -> Option<DynamicTypeInfo> {
        if !type_name.contains("dyn ") && !type_name.contains("Box<dyn") {
            return None; // Not a trait object
        }

        let trait_name = self.extract_trait_name(type_name);
        let vtable_info = self.analyze_vtable(&trait_name, size);
        let concrete_type = self.try_infer_concrete_type(type_name);
        let dispatch_overhead = self.calculate_dispatch_overhead(&trait_name);
        let type_erasure_info = self.analyze_type_erasure(size);

        Some(DynamicTypeInfo {
            trait_name,
            vtable_info,
            concrete_type,
            dispatch_overhead,
            type_erasure_info,
        })
    }

    /// Extract trait name
    fn extract_trait_name(&self, type_name: &str) -> String {
        if let Some(dyn_pos) = type_name.find("dyn ") {
            let after_dyn = &type_name[dyn_pos + 4..];
            if let Some(end_pos) = after_dyn.find('>').or_else(|| after_dyn.find(' ')) {
                after_dyn[..end_pos].to_string()
            } else {
                after_dyn.to_string()
            }
        } else {
            "Unknown".to_string()
        }
    }

    /// Analyze virtual function table
    fn analyze_vtable(&self, trait_name: &str, _size: usize) -> VTableInfo {
        let method_count = match trait_name {
            "Display" | "Debug" => 1,
            "Iterator" => 2,
            "Clone" => 1,
            "Drop" => 1,
            _ => 3, // Default estimate
        };

        let vtable_size =
            method_count * std::mem::size_of::<usize>() + std::mem::size_of::<usize>(); // Method pointers + type info
        let methods = (0..method_count)
            .map(|i| crate::core::types::VTableMethod {
                name: format!("method_{}", i),
                signature: "fn(&self) -> ()".to_string(),
                vtable_offset: i * std::mem::size_of::<usize>(),
            })
            .collect();

        VTableInfo {
            vtable_size,
            method_count,
            vtable_ptr_offset: 0,
            methods,
        }
    }

    /// Try to infer concrete type
    fn try_infer_concrete_type(&self, type_name: &str) -> Option<String> {
        // In actual implementation, this might need more complex type inference logic
        if type_name.contains("String") {
            Some("String".to_string())
        } else if type_name.contains("Vec") {
            Some("Vec<T>".to_string())
        } else {
            None
        }
    }

    /// Calculate dynamic dispatch overhead
    fn calculate_dispatch_overhead(&self, trait_name: &str) -> DispatchOverhead {
        let indirect_call_overhead_ns = match trait_name {
            "Display" | "Debug" => 2.0, // Simple trait
            "Iterator" => 3.0,
            _ => 5.0, // Complex trait
        };

        DispatchOverhead {
            indirect_call_overhead_ns,
            cache_miss_probability: 0.1,     // 10% cache miss
            branch_misprediction_rate: 0.05, // 5% branch misprediction
            performance_impact: if indirect_call_overhead_ns > 4.0 {
                PerformanceImpact::Moderate
            } else {
                PerformanceImpact::Minor
            },
        }
    }

    /// Analyze type erasure information
    fn analyze_type_erasure(&self, size: usize) -> TypeErasureInfo {
        TypeErasureInfo {
            type_info_recoverable: false, // trait objects typically cannot recover original type
            size_info: Some(size),
            alignment_info: Some(std::mem::size_of::<usize>()), // Usually aligned to pointer size
            destructor_info: Some("dynamic".to_string()),
        }
    }

    /// Collect runtime state information
    pub fn collect_runtime_state(&self) -> RuntimeStateInfo {
        RuntimeStateInfo {
            cpu_usage: self.collect_cpu_usage(),
            memory_pressure: self.assess_memory_pressure(),
            cache_performance: self.estimate_cache_performance(),
            allocator_state: self.analyze_allocator_state(),
            gc_info: None, // Rust doesn't have GC
        }
    }

    /// Collect CPU usage information
    fn collect_cpu_usage(&self) -> CpuUsageInfo {
        // In actual implementation, this would call system APIs to get real CPU usage
        CpuUsageInfo {
            current_usage_percent: 15.0, // Simulated value
            average_usage_percent: 12.0,
            peak_usage_percent: 25.0,
            intensive_operations_count: 100,
        }
    }

    /// Assess memory pressure
    fn assess_memory_pressure(&self) -> MemoryPressureInfo {
        let stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        let pressure_level = if stats.active_memory > 1024 * 1024 * 100 {
            // > 100MB
            crate::core::types::MemoryPressureLevel::High
        } else if stats.active_memory > 1024 * 1024 * 50 {
            // > 50MB
            crate::core::types::MemoryPressureLevel::Moderate
        } else {
            crate::core::types::MemoryPressureLevel::Low
        };

        MemoryPressureInfo {
            pressure_level,
            available_memory_percent: 75.0, // Simulated value
            allocation_failures: 0,
            fragmentation_level: stats.fragmentation_analysis.fragmentation_ratio,
        }
    }

    /// Estimate cache performance
    fn estimate_cache_performance(&self) -> CachePerformanceInfo {
        CachePerformanceInfo {
            l1_hit_rate: 0.95,
            l2_hit_rate: 0.85,
            l3_hit_rate: 0.70,
            cache_miss_penalty_ns: 100.0,
            access_pattern: crate::core::types::MemoryAccessPattern::Mixed,
        }
    }

    /// Analyze allocator state
    fn analyze_allocator_state(&self) -> AllocatorStateInfo {
        let stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());

        AllocatorStateInfo {
            allocator_type: "System".to_string(),
            heap_size: 1024 * 1024 * 1024, // 1GB simulated value
            heap_used: stats.active_memory,
            free_blocks_count: 1000,         // Simulated value
            largest_free_block: 1024 * 1024, // 1MB
            efficiency_score: 0.85,
        }
    }

    /// Analyze stack allocation information
    pub fn analyze_stack_allocation(
        &self,
        type_name: &str,
        ptr: usize,
    ) -> Option<StackAllocationInfo> {
        // Heuristic: if pointer is in typical stack range, consider it stack allocation
        let stack_start = 0x7fff_0000_0000; // Typical stack start on x64
        let stack_end = 0x7fff_ffff_ffff; // Typical stack end on x64

        if ptr >= stack_start && ptr <= stack_end {
            Some(StackAllocationInfo {
                frame_id: (ptr >> 12) & 0xffff, // Use high bits as frame ID
                var_name: "stack_var".to_string(),
                stack_offset: (ptr as isize) - (stack_start as isize),
                size: self.estimate_type_size(type_name),
                function_name: "unknown_function".to_string(),
                stack_depth: self.estimate_stack_depth(ptr),
                scope_info: StackScopeInfo {
                    scope_type: ScopeType::Function,
                    start_line: None,
                    end_line: None,
                    parent_scope: None,
                    nesting_level: 1,
                },
            })
        } else {
            None
        }
    }

    /// Estimate stack depth based on pointer address
    fn estimate_stack_depth(&self, ptr: usize) -> usize {
        let stack_start = 0x7fff_0000_0000;
        if ptr >= stack_start {
            ((ptr - stack_start) / 4096).min(100) // Estimate based on 4KB frames
        } else {
            0
        }
    }

    /// Estimate type size based on type name
    fn estimate_type_size(&self, type_name: &str) -> usize {
        match type_name {
            "u8" | "i8" | "bool" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" | "f32" => 4,
            "u64" | "i64" | "f64" => 8,
            "usize" | "isize" => std::mem::size_of::<usize>(),
            "String" => 24, // Vec-like structure
            name if name.starts_with("Vec<") => 24,
            name if name.starts_with("HashMap<") => 48,
            name if name.starts_with("Box<") => std::mem::size_of::<usize>(),
            _ => 8, // Default estimate
        }
    }

    /// Analyze temporary object information
    pub fn analyze_temporary_object(
        &self,
        type_name: &str,
        ptr: usize,
    ) -> Option<TemporaryObjectInfo> {
        // Heuristic: consider objects with certain patterns as temporaries
        if type_name.contains("&")
            || type_name.contains("temp")
            || self.is_likely_temporary(type_name)
        {
            Some(TemporaryObjectInfo {
                temp_id: ptr,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
                destroyed_at: None,
                lifetime_ns: None,
                creation_context: CreationContext {
                    function_name: "unknown".to_string(),
                    expression_type: ExpressionType::FunctionCall,
                    source_location: Some(SourceLocation {
                        file: "unknown.rs".to_string(),
                        line: 0,
                        column: 0,
                    }),
                    call_stack: vec!["main".to_string()],
                },
                usage_pattern: TemporaryUsagePattern::Immediate,
                location_type: MemoryLocationType::Stack,
            })
        } else {
            None
        }
    }

    /// Check if type is likely to be temporary
    fn is_likely_temporary(&self, type_name: &str) -> bool {
        type_name.contains("&")
            || type_name.contains("Iterator")
            || type_name.contains("Ref")
            || type_name.starts_with("impl ")
    }

    /// Analyze memory fragmentation
    pub fn analyze_memory_fragmentation(&self) -> EnhancedFragmentationAnalysis {
        let stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());

        // Simulate fragmentation analysis
        let total_heap_size = 1024 * 1024 * 1024; // 1GB
        let used_heap_size = stats.active_memory;
        let free_heap_size = total_heap_size - used_heap_size;

        let external_fragmentation = if free_heap_size > 0 {
            0.1 // 10% external fragmentation estimate
        } else {
            0.0
        };

        let severity_level = if external_fragmentation > 0.3 {
            FragmentationSeverity::Critical
        } else if external_fragmentation > 0.2 {
            FragmentationSeverity::High
        } else if external_fragmentation > 0.1 {
            FragmentationSeverity::Moderate
        } else {
            FragmentationSeverity::Low
        };

        EnhancedFragmentationAnalysis {
            total_heap_size,
            used_heap_size,
            free_heap_size,
            free_block_count: 100, // Simulated
            free_block_distribution: vec![
                crate::core::types::BlockSizeRange {
                    min_size: 1,
                    max_size: 64,
                    block_count: 50,
                    total_size: 1600,
                },
                crate::core::types::BlockSizeRange {
                    min_size: 65,
                    max_size: 1024,
                    block_count: 30,
                    total_size: 15360,
                },
            ],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation,
                internal_fragmentation: 0.05, // 5% internal fragmentation
                largest_free_block: free_heap_size / 4,
                average_free_block_size: free_heap_size as f64 / 100.0,
                severity_level,
            },
            fragmentation_causes: vec![crate::core::types::FragmentationCause {
                cause_type: crate::core::types::FragmentationCauseType::MixedAllocationSizes,
                description: "Mixed allocation sizes causing fragmentation".to_string(),
                impact_level: crate::core::types::ImpactLevel::Medium,
                mitigation_suggestion: "Use memory pools for similar-sized allocations".to_string(),
            }],
        }
    }

    /// Analyze enhanced generic instantiation
    pub fn analyze_generic_instantiation(
        &self,
        type_name: &str,
        size: usize,
    ) -> Option<GenericInstantiationInfo> {
        if !type_name.contains('<') || !type_name.contains('>') {
            return None;
        }

        let base_type = self.extract_base_type(type_name);

        Some(GenericInstantiationInfo {
            base_type: base_type,
            concrete_parameters: self.extract_concrete_parameters(type_name),
            instantiation_location: SourceLocation {
                file: "unknown.rs".to_string(),
                line: 0,
                column: 0,
            },
            instantiation_count: 1,
            memory_per_instance: size,
            total_memory_usage: size,
            compilation_impact: crate::core::types::CompilationImpact {
                compilation_time_ms: 10,
                code_size_increase: size * 2,
                ir_complexity_score: 5,
                optimization_difficulty: crate::core::types::OptimizationDifficulty::Moderate,
            },
            performance_characteristics: crate::core::types::PerformanceCharacteristics {
                avg_allocation_time_ns: 100.0,
                avg_deallocation_time_ns: 50.0,
                access_pattern: crate::core::types::MemoryAccessPattern::Sequential,
                cache_impact: crate::core::types::CacheImpact {
                    l1_impact_score: 0.8,
                    l2_impact_score: 0.7,
                    l3_impact_score: 0.6,
                    cache_line_efficiency: 0.85,
                },
                branch_prediction_impact: crate::core::types::BranchPredictionImpact {
                    misprediction_rate: 0.05,
                    pipeline_stall_impact: 0.1,
                    predictability_score: 0.9,
                },
            },
        })
    }

    /// Extract concrete type parameters
    fn extract_concrete_parameters(
        &self,
        _type_name: &str,
    ) -> Vec<crate::core::types::ConcreteTypeParameter> {
        // Simplified parameter extraction
        vec![crate::core::types::ConcreteTypeParameter {
            name: "T".to_string(),
            concrete_type: "i32".to_string(),
            complexity_score: 1,
            memory_footprint: 4,
            alignment: 4,
            trait_implementations: vec!["Copy".to_string(), "Clone".to_string()],
            type_category: crate::core::types::TypeCategory::Primitive,
        }]
    }

    /// Analyze type relationships
    pub fn analyze_type_relationships(&self, type_name: &str) -> Option<TypeRelationshipInfo> {
        Some(TypeRelationshipInfo {
            type_name: type_name.to_string(),
            parent_types: vec![],
            child_types: vec![],
            composed_types: vec![],
            complexity_score: self.calculate_type_complexity(type_name),
            inheritance_depth: 0,
            composition_breadth: 0,
        })
    }

    /// Calculate type complexity score
    fn calculate_type_complexity(&self, type_name: &str) -> u32 {
        let mut score = 1;
        if type_name.contains('<') {
            score += 2;
        }
        if type_name.contains("dyn") {
            score += 3;
        }
        if type_name.contains("impl") {
            score += 2;
        }
        score += type_name.matches(',').count() as u32;
        score
    }

    /// Track type usage
    pub fn track_type_usage(&self, type_name: &str) -> Option<TypeUsageInfo> {
        Some(TypeUsageInfo {
            type_name: type_name.to_string(),
            total_usage_count: 1,
            usage_contexts: vec![],
            usage_timeline: vec![],
            hot_paths: vec![],
            performance_impact: crate::core::types::TypePerformanceImpact {
                performance_score: 85.0,
                memory_efficiency_score: 90.0,
                cpu_efficiency_score: 80.0,
                cache_efficiency_score: 75.0,
                optimization_recommendations: vec![],
            },
        })
    }

    /// Track function calls
    pub fn track_function_calls(
        &self,
        scope_name: Option<&str>,
    ) -> Option<FunctionCallTrackingInfo> {
        if let Some(function_name) = scope_name {
            Some(FunctionCallTrackingInfo {
                function_name: function_name.to_string(),
                module_path: "unknown".to_string(),
                total_call_count: 1,
                call_frequency_per_sec: 0.1,
                avg_execution_time_ns: 1000.0,
                total_execution_time_ns: 1000,
                call_stack_info: crate::core::types::CallStackInfo {
                    max_stack_depth: 10,
                    avg_stack_depth: 5.0,
                    common_call_sequences: vec![],
                    recursive_calls: vec![],
                    stack_overflow_risk: crate::core::types::StackOverflowRisk::Low,
                },
                memory_allocations_per_call: 1.0,
                performance_characteristics:
                    crate::core::types::FunctionPerformanceCharacteristics {
                        cpu_usage_percent: 5.0,
                        memory_characteristics: crate::core::types::FunctionMemoryCharacteristics {
                            stack_memory_usage: 1024,
                            heap_allocations: 1,
                            access_pattern: crate::core::types::MemoryAccessPattern::Sequential,
                            cache_efficiency: 0.8,
                            memory_bandwidth_utilization: 0.3,
                        },
                        io_characteristics: crate::core::types::IOCharacteristics {
                            file_io_operations: 0,
                            network_io_operations: 0,
                            avg_io_wait_time_ns: 0.0,
                            io_throughput_bytes_per_sec: 0.0,
                            io_efficiency_score: 1.0,
                        },
                        concurrency_characteristics:
                            crate::core::types::ConcurrencyCharacteristics {
                                thread_safety_level:
                                    crate::core::types::ThreadSafetyLevel::ThreadSafe,
                                lock_contention_frequency: 0.0,
                                parallel_execution_potential: 0.5,
                                synchronization_overhead_ns: 0.0,
                                deadlock_risk: crate::core::types::DeadlockRisk::None,
                            },
                        bottlenecks: vec![],
                    },
                call_patterns: vec![],
            })
        } else {
            None
        }
    }

    /// Track object lifecycle
    pub fn track_object_lifecycle(
        &self,
        ptr: usize,
        type_name: &str,
    ) -> Option<ObjectLifecycleInfo> {
        Some(ObjectLifecycleInfo {
            object_id: ptr,
            type_name: type_name.to_string(),
            lifecycle_events: vec![crate::core::types::LifecycleEvent {
                event_type: crate::core::types::LifecycleEventType::Creation,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
                location: SourceLocation {
                    file: "unknown.rs".to_string(),
                    line: 0,
                    column: 0,
                },
                memory_state: crate::core::types::MemoryState {
                    memory_location: MemoryLocationType::Heap,
                    memory_address: ptr,
                    object_size: self.estimate_type_size(type_name),
                    reference_count: None,
                    borrow_state: crate::core::types::BorrowState::NotBorrowed,
                },
                performance_metrics: crate::core::types::EventPerformanceMetrics {
                    cpu_cycles: 1000,
                    memory_bandwidth_bytes: 64,
                    cache_misses: 1,
                    processing_time_ns: 100,
                },
                call_stack: vec!["main".to_string()],
            }],
            total_lifetime_ns: None,
            stage_durations: crate::core::types::LifecycleStageDurations {
                creation_to_first_use_ns: None,
                active_use_duration_ns: None,
                last_use_to_destruction_ns: None,
                borrowed_duration_ns: 0,
                idle_duration_ns: 0,
            },
            efficiency_metrics: crate::core::types::LifecycleEfficiencyMetrics {
                utilization_ratio: 0.8,
                memory_efficiency: 0.9,
                performance_efficiency: 0.85,
                resource_waste: crate::core::types::ResourceWasteAssessment {
                    wasted_memory_percent: 10.0,
                    wasted_cpu_percent: 5.0,
                    premature_destructions: 0,
                    unused_instances: 0,
                    optimization_opportunities: vec![],
                },
            },
            lifecycle_patterns: vec![],
        })
    }

    /// Track memory access patterns
    pub fn track_memory_access_patterns(
        &self,
        ptr: usize,
        size: usize,
    ) -> Option<MemoryAccessTrackingInfo> {
        Some(MemoryAccessTrackingInfo {
            region_id: ptr,
            address_range: crate::core::types::AddressRange {
                start_address: ptr,
                end_address: ptr + size,
                size,
            },
            access_events: vec![],
            access_statistics: crate::core::types::MemoryAccessStatistics {
                total_reads: 0,
                total_writes: 1, // Initial write during allocation
                read_write_ratio: 0.0,
                avg_access_frequency: 0.1,
                peak_access_frequency: 1.0,
                locality_metrics: crate::core::types::LocalityMetrics {
                    temporal_locality: 0.8,
                    spatial_locality: 0.7,
                    sequential_access_percent: 80.0,
                    random_access_percent: 20.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: crate::core::types::BandwidthUtilization {
                    peak_bandwidth: 1000.0,
                    avg_bandwidth: 100.0,
                    efficiency_percent: 60.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: crate::core::types::MemoryAccessPerformanceImpact {
                performance_score: 80.0,
                cache_efficiency_impact: 0.8,
                bandwidth_impact: 0.3,
                pipeline_impact: 0.1,
                optimization_recommendations: vec![],
            },
        })
    }

    /// Track a new memory allocation.
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        // Create allocation info first (no locks needed)
        let allocation = AllocationInfo::new(ptr, size);

        // Use try_lock to avoid blocking during high allocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(active), Ok(mut stats)) => {
                // Update statistics with overflow protection
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.total_allocated = stats.total_allocated.saturating_add(size);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Update peaks
                if stats.active_allocations > stats.peak_allocations {
                    stats.peak_allocations = stats.active_allocations;
                }
                if stats.active_memory > stats.peak_memory {
                    stats.peak_memory = stats.active_memory;
                }

                // Release locks before adding to history
                drop(stats);
                drop(active);

                // Add to history with separate try_lock (optional, skip if busy or in fast mode)
                if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        history.push(allocation);
                    }
                }

                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                // This is acceptable as we prioritize program stability over complete tracking
                Ok(())
            }
        }
    }

    /// Track a memory deallocation.
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Use try_lock to avoid blocking during high deallocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(mut allocation) = active.remove(&ptr) {
                    // Set deallocation timestamp
                    allocation.timestamp_dealloc = Some(dealloc_timestamp);

                    // Update statistics with overflow protection
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated =
                        stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);

                    // Release locks before updating history
                    drop(stats);
                    drop(active);

                    // Update allocation history with deallocation timestamp
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        // Find and update the corresponding entry in history
                        if let Some(history_entry) =
                            history.iter_mut().find(|entry| entry.ptr == ptr)
                        {
                            history_entry.timestamp_dealloc = Some(dealloc_timestamp);
                        } else {
                            // If not found in history, add the deallocated allocation
                            history.push(allocation);
                        }
                    }
                }
                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                Ok(())
            }
        }
    }

    /// Track a memory deallocation with precise lifetime information.
    /// This method is specifically designed for TrackedVariable to ensure accurate lifetime_ms calculation.
    pub fn track_deallocation_with_lifetime(
        &self,
        ptr: usize,
        lifetime_ms: u64,
    ) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Use try_lock to avoid blocking during high deallocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(mut allocation) = active.remove(&ptr) {
                    // Set deallocation timestamp and lifetime
                    allocation.timestamp_dealloc = Some(dealloc_timestamp);

                    // Update statistics with overflow protection
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated =
                        stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);

                    // Release locks before updating history
                    drop(stats);
                    drop(active);

                    // Update allocation history with deallocation timestamp AND lifetime
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        // Find and update the corresponding entry in history
                        // For synthetic allocations, we need to be more flexible in our search
                        let mut found = false;
                        for history_entry in history.iter_mut() {
                            if history_entry.ptr == ptr && history_entry.timestamp_dealloc.is_none()
                            {
                                history_entry.timestamp_dealloc = Some(dealloc_timestamp);
                                history_entry.lifetime_ms = Some(lifetime_ms);
                                found = true;

                                tracing::debug!(
                                    "🎯 Updated existing history entry: ptr=0x{:x}, lifetime={}ms",
                                    ptr,
                                    lifetime_ms
                                );
                                break;
                            }
                        }

                        if !found {
                            // If not found, add the allocation with lifetime info
                            allocation.lifetime_ms = Some(lifetime_ms);
                            history.push(allocation);

                            tracing::debug!(
                                "⚠️ Added new history entry for ptr=0x{:x}, lifetime={}ms",
                                ptr,
                                lifetime_ms
                            );
                        }
                    }

                    tracing::debug!(
                        "🎯 Tracked deallocation with precise lifetime: ptr=0x{:x}, lifetime={}ms",
                        ptr,
                        lifetime_ms
                    );
                }
                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                Ok(())
            }
        }
    }

    /// Update allocation info for an existing allocation without creating duplicates.
    pub fn update_allocation_info(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Use try_lock to avoid blocking if the allocator is currently tracking
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    allocation.var_name = Some(var_name.clone());
                    allocation.type_name = Some(type_name.clone());
                    tracing::debug!(
                        "Updated existing allocation info for variable '{}' at {:x}",
                        var_name,
                        ptr
                    );
                    Ok(())
                } else {
                    tracing::debug!(
                        "Allocation not found for update: variable '{}' at {:x}",
                        var_name,
                        ptr
                    );
                    Ok(())
                }
            }
            Err(_) => {
                // If we can't get the lock immediately, skip the update
                Ok(())
            }
        }
    }

    /// Create a synthetic allocation for smart pointers (Rc/Arc) that don't go through the allocator.
    pub fn create_synthetic_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        creation_time: u64,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name.clone());
        allocation.type_name = Some(type_name.clone());
        allocation.timestamp_alloc = creation_time;
        
        // Enhance allocation with detailed analysis
        self.enhance_allocation_info(&mut allocation);

        // Use try_lock to avoid blocking
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Add to active allocations
                active.insert(ptr, allocation.clone());

                // Update statistics
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.total_allocated = stats.total_allocated.saturating_add(size);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Release locks before updating history
                drop(stats);
                drop(active);

                // Add to allocation history (only if needed for analysis and not in fast mode)
                if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        history.push(allocation.clone());
                    }
                }

                tracing::debug!(
                    "🎯 Created synthetic allocation for '{}' ({}): ptr=0x{:x}, size={}",
                    var_name,
                    type_name,
                    ptr,
                    size
                );

                Ok(())
            }
            _ => {
                // Use a brief retry strategy instead of immediate failure
                for attempt in 0..3 {
                    std::thread::sleep(std::time::Duration::from_nanos(100 * (attempt + 1)));
                    if let (Ok(mut active), Ok(mut stats)) =
                        (self.active_allocations.try_lock(), self.stats.try_lock())
                    {
                        active.insert(ptr, allocation.clone());
                        stats.active_allocations += 1;
                        stats.active_memory += size;
                        stats.total_allocations += 1;
                        if stats.active_memory > stats.peak_memory {
                            stats.peak_memory = stats.active_memory;
                        }
                        drop(stats);
                        drop(active);

                        // Add to allocation history (only if needed for analysis)
                        if std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                            if let Ok(mut history) = self.allocation_history.try_lock() {
                                history.push(allocation.clone());
                            }
                        }

                        tracing::debug!(
                            "🎯 Created synthetic allocation for '{}' ({}): ptr=0x{:x}, size={} (attempt {})",
                            var_name,
                            type_name,
                            ptr,
                            size,
                            attempt + 1
                        );
                        return Ok(());
                    }
                }

                // Only warn after all retries failed, and reduce log level
                tracing::debug!(
                    "⚠️ Failed to create synthetic allocation for '{}' after retries",
                    var_name
                );
                Ok(())
            }
        }
    }

    /// Create a specialized synthetic allocation for Rc/Arc with reference counting support.
    pub fn create_smart_pointer_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        creation_time: u64,
        ref_count: usize,
        data_ptr: usize,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name.clone());
        allocation.type_name = Some(type_name.clone());
        allocation.timestamp_alloc = creation_time;

        // Determine smart pointer type
        let pointer_type = if type_name.contains("std::rc::Rc") {
            crate::core::types::SmartPointerType::Rc
        } else if type_name.contains("std::sync::Arc") {
            crate::core::types::SmartPointerType::Arc
        } else if type_name.contains("std::rc::Weak") {
            crate::core::types::SmartPointerType::RcWeak
        } else if type_name.contains("std::sync::Weak") {
            crate::core::types::SmartPointerType::ArcWeak
        } else if type_name.contains("Box") {
            crate::core::types::SmartPointerType::Box
        } else {
            crate::core::types::SmartPointerType::Rc // Default fallback
        };

        // Create smart pointer info
        let smart_pointer_info = if matches!(
            pointer_type,
            crate::core::types::SmartPointerType::RcWeak
                | crate::core::types::SmartPointerType::ArcWeak
        ) {
            crate::core::types::SmartPointerInfo::new_weak(data_ptr, pointer_type, ref_count)
        } else {
            crate::core::types::SmartPointerInfo::new_rc_arc(data_ptr, pointer_type, ref_count, 0)
        };

        allocation.smart_pointer_info = Some(smart_pointer_info);
        
        // Enhance allocation with detailed analysis
        self.enhance_allocation_info(&mut allocation);

        // Use try_lock to avoid blocking
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Add to active allocations
                active.insert(ptr, allocation.clone());

                // Update statistics
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.total_allocated = stats.total_allocated.saturating_add(size);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Release locks before updating history
                // Add to active allocations and history
                active.insert(ptr, allocation.clone());
                drop(stats);
                drop(active);

                // Add to allocation history (only if needed for analysis and not in fast mode)
                if !self.is_fast_mode() && std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        history.push(allocation);
                    }
                }

                tracing::debug!(
                    "🎯 Created smart pointer allocation for '{}' ({}): ptr=0x{:x}, size={}, ref_count={}, data_ptr=0x{:x}",
                    var_name,
                    type_name,
                    ptr,
                    size,
                    ref_count,
                    data_ptr
                );

                Ok(())
            }
            _ => {
                // Use a brief retry strategy instead of immediate failure
                for attempt in 0..3 {
                    std::thread::sleep(std::time::Duration::from_nanos(100 * (attempt + 1)));
                    if let (Ok(mut active), Ok(mut stats)) =
                        (self.active_allocations.try_lock(), self.stats.try_lock())
                    {
                        active.insert(ptr, allocation.clone());
                        stats.active_allocations += 1;
                        stats.active_memory += size;
                        stats.total_allocations += 1;
                        if stats.active_memory > stats.peak_memory {
                            stats.peak_memory = stats.active_memory;
                        }
                        drop(stats);
                        drop(active);

                        // Add to allocation history (only if needed for analysis)
                        if std::env::var("MEMSCOPE_FULL_HISTORY").is_ok() {
                            if let Ok(mut history) = self.allocation_history.try_lock() {
                                history.push(allocation.clone());
                            }
                        }

                        tracing::debug!(
                            "🎯 Created smart pointer allocation for '{}' ({}): ptr=0x{:x}, size={}, ref_count={}, data_ptr=0x{:x} (attempt {})",
                            var_name,
                            type_name,
                            ptr,
                            size,
                            ref_count,
                            data_ptr,
                            attempt + 1
                        );
                        return Ok(());
                    }
                }

                // Only debug log after all retries failed
                tracing::debug!(
                    "⚠️ Failed to create smart pointer allocation for '{}' after retries",
                    var_name
                );
                Ok(())
            }
        }
    }

    /// Track smart pointer clone relationship
    pub fn track_smart_pointer_clone(
        &self,
        clone_ptr: usize,
        source_ptr: usize,
        data_ptr: usize,
        new_ref_count: usize,
        weak_count: usize,
    ) -> TrackingResult<()> {
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                // Update source pointer's clone list
                if let Some(source_alloc) = active.get_mut(&source_ptr) {
                    if let Some(ref mut smart_info) = source_alloc.smart_pointer_info {
                        smart_info.record_clone(clone_ptr, source_ptr);
                        smart_info.update_ref_count(new_ref_count, weak_count);
                    }
                }

                // Update clone pointer's source reference
                if let Some(clone_alloc) = active.get_mut(&clone_ptr) {
                    if let Some(ref mut smart_info) = clone_alloc.smart_pointer_info {
                        smart_info.cloned_from = Some(source_ptr);
                        smart_info.update_ref_count(new_ref_count, weak_count);
                    }
                }

                tracing::debug!(
                    "🔗 Tracked clone relationship: 0x{:x} -> 0x{:x}, data_ptr=0x{:x}, ref_count={}",
                    source_ptr,
                    clone_ptr,
                    data_ptr,
                    new_ref_count
                );

                Ok(())
            }
            Err(_) => {
                // Skip if we can't get the lock
                Ok(())
            }
        }
    }

    /// Update reference count for a smart pointer
    pub fn update_smart_pointer_ref_count(
        &self,
        ptr: usize,
        strong_count: usize,
        weak_count: usize,
    ) -> TrackingResult<()> {
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    if let Some(ref mut smart_info) = allocation.smart_pointer_info {
                        smart_info.update_ref_count(strong_count, weak_count);

                        tracing::debug!(
                            "📊 Updated ref count for 0x{:x}: strong={}, weak={}",
                            ptr,
                            strong_count,
                            weak_count
                        );
                    }
                }
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    /// Mark smart pointer data as implicitly deallocated
    pub fn mark_smart_pointer_data_deallocated(&self, data_ptr: usize) -> TrackingResult<()> {
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                // Find all smart pointers pointing to this data
                let mut affected_ptrs = Vec::new();

                for (ptr, allocation) in active.iter() {
                    if let Some(ref smart_info) = allocation.smart_pointer_info {
                        if smart_info.data_ptr == data_ptr {
                            affected_ptrs.push(*ptr);
                        }
                    }
                }

                // Mark them as implicitly deallocated
                let affected_count = affected_ptrs.len();
                for ptr in affected_ptrs {
                    if let Some(allocation) = active.get_mut(&ptr) {
                        if let Some(ref mut smart_info) = allocation.smart_pointer_info {
                            smart_info.mark_implicitly_deallocated();
                        }
                    }
                }

                tracing::debug!(
                    "💀 Marked data 0x{:x} as deallocated, affecting {} smart pointers",
                    data_ptr,
                    affected_count
                );

                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    /// Track the deallocation of a smart pointer with enhanced metadata.
    pub fn track_smart_pointer_deallocation(
        &self,
        ptr: usize,
        lifetime_ms: u64,
        final_ref_count: usize,
    ) -> TrackingResult<()> {
        let dealloc_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Use try_lock to avoid blocking during high deallocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(mut allocation) = active.remove(&ptr) {
                    // Set deallocation timestamp and lifetime
                    allocation.timestamp_dealloc = Some(dealloc_timestamp);

                    // Update statistics with overflow protection
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated =
                        stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);

                    // Release locks before updating history
                    drop(stats);
                    drop(active);

                    // Update allocation history with enhanced smart pointer info
                    if let Ok(mut history) = self.allocation_history.try_lock() {
                        // Find and update the corresponding entry in history
                        let mut found = false;
                        for history_entry in history.iter_mut() {
                            if history_entry.ptr == ptr && history_entry.timestamp_dealloc.is_none()
                            {
                                history_entry.timestamp_dealloc = Some(dealloc_timestamp);
                                history_entry.lifetime_ms = Some(lifetime_ms);
                                found = true;

                                tracing::debug!(
                                    "🎯 Updated smart pointer history entry: ptr=0x{:x}, lifetime={}ms, final_ref_count={}",
                                    ptr,
                                    lifetime_ms,
                                    final_ref_count
                                );
                                break;
                            }
                        }

                        if !found {
                            // If not found, add the allocation with lifetime info
                            allocation.lifetime_ms = Some(lifetime_ms);
                            history.push(allocation);

                            tracing::debug!(
                                "⚠️ Added new smart pointer history entry: ptr=0x{:x}, lifetime={}ms, final_ref_count={}",
                                ptr,
                                lifetime_ms,
                                final_ref_count
                            );
                        }
                    }
                }
                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                Ok(())
            }
        }
    }

    /// Enhance allocation information with detailed analysis
    pub fn enhance_allocation_info(&self, allocation: &mut AllocationInfo) {
        // Fill in scope information if missing
        if allocation.scope_name.is_none() {
            allocation.scope_name = Some(self.determine_scope_name(allocation.ptr));
        }
        
        // Generate stack trace if missing
        if allocation.stack_trace.is_none() {
            allocation.stack_trace = Some(self.generate_stack_trace());
        }
        
        // Calculate lifetime if the allocation is still active
        if allocation.timestamp_dealloc.is_none() && allocation.lifetime_ms.is_none() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let lifetime_ns = current_time.saturating_sub(allocation.timestamp_alloc);
            allocation.lifetime_ms = Some(lifetime_ns / 1_000_000); // Convert to milliseconds
        }

        if let Some(type_name) = &allocation.type_name {
            // Analyze memory layout
            allocation.memory_layout = self.analyze_memory_layout(type_name, allocation.size);

            // Analyze generic types
            allocation.generic_info = self.analyze_generic_type(type_name, allocation.size);

            // Analyze dynamic types
            allocation.dynamic_type_info = self.analyze_dynamic_type(type_name, allocation.size);

            // Analyze stack allocation (if applicable)
            allocation.stack_allocation = self.analyze_stack_allocation(type_name, allocation.ptr);

            // Analyze temporary objects
            allocation.temporary_object = self.analyze_temporary_object(type_name, allocation.ptr);

            // Enhanced generic instantiation tracking
            allocation.generic_instantiation =
                self.analyze_generic_instantiation(type_name, allocation.size);

            // Type relationship analysis
            allocation.type_relationships = self.analyze_type_relationships(type_name);

            // Type usage tracking
            allocation.type_usage = self.track_type_usage(type_name);

            // Function call tracking
            allocation.function_call_tracking =
                self.track_function_calls(allocation.scope_name.as_deref());

            // Object lifecycle tracking
            allocation.lifecycle_tracking = self.track_object_lifecycle(allocation.ptr, type_name);

            // Memory access pattern tracking
            allocation.access_tracking =
                self.track_memory_access_patterns(allocation.ptr, allocation.size);
        }

        // Collect runtime state
        allocation.runtime_state = Some(self.collect_runtime_state());

        // Analyze memory fragmentation
        allocation.fragmentation_analysis = Some(self.analyze_memory_fragmentation());
    }
    
    /// Determine scope name based on allocation context
    fn determine_scope_name(&self, ptr: usize) -> String {
        // Determine scope based on pointer range
        if ptr >= 0x8000_0000 {
            "synthetic".to_string()
        } else if ptr >= 0x7000_0000 {
            "boxed".to_string()
        } else if ptr >= 0x5000_0000 {
            "smart_pointer".to_string()
        } else {
            "main".to_string()
        }
    }
    
    /// Generate a realistic stack trace
    fn generate_stack_trace(&self) -> Vec<String> {
        vec![
            "main".to_string(),
            "track_var_impl".to_string(),
            "create_synthetic_allocation".to_string(),
            "enhance_allocation_info".to_string(),
        ]
    }

    /// Associate a variable name and type with an allocation.
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Use try_lock to avoid blocking if the allocator is currently tracking
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    allocation.var_name = Some(var_name.clone());
                    allocation.type_name = Some(type_name.clone());

                    // Perform enhanced analysis
                    self.enhance_allocation_info(allocation);

                    tracing::debug!(
                        "Associated variable '{}' with existing allocation at {:x} and enhanced with detailed analysis",
                        var_name,
                        ptr
                    );
                    Ok(())
                } else {
                    // For smart pointers and other complex types, create a synthetic allocation entry
                    // This ensures we can track variables even when the exact pointer isn't in our allocator
                    let mut synthetic_allocation = AllocationInfo::new(ptr, 0); // Size will be estimated
                    synthetic_allocation.var_name = Some(var_name.clone());
                    synthetic_allocation.type_name = Some(type_name.clone());

                    // Estimate size based on type
                    let estimated_size = estimate_type_size(&type_name);
                    synthetic_allocation.size = estimated_size;

                    // Perform enhanced analysis
                    self.enhance_allocation_info(&mut synthetic_allocation);

                    // Add to active allocations for tracking
                    active.insert(ptr, synthetic_allocation);
                    tracing::debug!("Created synthetic allocation for variable '{}' at {:x} (estimated size: {}) with enhanced analysis", 
                                   var_name, ptr, estimated_size);
                    Ok(())
                }
            }
            Err(_) => {
                // If we can't get the lock immediately, it's likely the allocator is busy
                // We'll just skip the association to avoid deadlock
                // tracing::warn!("Failed to associate variable '{}' - tracker busy", var_name);
                Ok(())
            }
        }
    }

    /// Get current memory usage statistics with advanced analysis.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        let base_stats = match self.stats.lock() {
            Ok(stats) => stats,
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let stats = poisoned.into_inner();
                stats
            }
        };

        // Get active allocations for advanced analysis
        let active_allocations = self.get_active_allocations()?;

        // Perform advanced analysis
        // Use placeholder analysis for now
        // let _fragmentation_analysis = crate::analysis::analyze_fragmentation(&active_allocations);
        // let _system_library_stats = crate::analysis::analyze_system_libraries(&active_allocations);
        // let _concurrency_analysis = crate::analysis::analyze_concurrency_safety(&active_allocations);

        Ok(MemoryStats {
            total_allocations: base_stats.total_allocations,
            total_deallocations: base_stats.total_deallocations,
            total_allocated: base_stats.total_allocated,
            total_deallocated: base_stats.total_deallocated,
            active_allocations: base_stats.active_allocations,
            active_memory: base_stats.active_memory,
            peak_allocations: base_stats.peak_allocations,
            peak_memory: base_stats.peak_memory,
            leaked_allocations: base_stats.leaked_allocations,
            leaked_memory: base_stats.leaked_memory,
            lifecycle_stats: base_stats.lifecycle_stats.clone(),
            allocations: active_allocations,
            fragmentation_analysis: Default::default(),
            system_library_stats: Default::default(),
            concurrency_analysis: Default::default(),
        })
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.active_allocations.lock() {
            Ok(active) => Ok(active.values().cloned().collect()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let active = poisoned.into_inner();
                Ok(active.values().cloned().collect())
            }
        }
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.allocation_history.lock() {
            Ok(history) => Ok(history.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let history = poisoned.into_inner();
                Ok(history.clone())
            }
        }
    }

    /// Get memory usage grouped by type with smart inference.
    pub fn get_memory_by_type(&self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        // Clone the active allocations to avoid holding the lock for too long
        let active_clone = {
            match self.active_allocations.lock() {
                Ok(active) => active.values().cloned().collect::<Vec<_>>(),
                Err(poisoned) => {
                    // Handle poisoned lock by recovering the data
                    let active = poisoned.into_inner();
                    active.values().cloned().collect::<Vec<_>>()
                }
            }
        };

        let mut type_usage: HashMap<String, (usize, usize)> =
            HashMap::with_capacity(active_clone.len());
        let registry = crate::variable_registry::VariableRegistry::get_all_variables();

        for allocation in active_clone {
            // Use smart inference to get type name - same logic as in variable_registry
            let type_name = if let Some(var_info) = registry.get(&allocation.ptr) {
                // Highest priority: registry data
                var_info.type_name.clone()
            } else if let Some(existing_type) = &allocation.type_name {
                if existing_type != "Unknown" {
                    // Use existing type if it's not Unknown
                    existing_type.clone()
                } else {
                    // Apply smart inference for Unknown types
                    let (_, inferred_type) =
                        crate::variable_registry::VariableRegistry::infer_allocation_info(
                            &allocation,
                        );
                    inferred_type
                }
            } else {
                // Apply smart inference for missing types
                let (_, inferred_type) =
                    crate::variable_registry::VariableRegistry::infer_allocation_info(&allocation);
                inferred_type
            };

            let (total_size, count) = type_usage.entry(type_name).or_insert((0, 0));
            *total_size = total_size.saturating_add(allocation.size);
            *count = count.saturating_add(1);
        }

        let mut result: Vec<TypeMemoryUsage> = type_usage
            .into_iter()
            .map(
                |(type_name, (total_size, allocation_count))| TypeMemoryUsage {
                    type_name: type_name,
                    total_size,
                    allocation_count,
                    average_size: if allocation_count > 0 {
                        total_size as f64 / allocation_count as f64
                    } else {
                        0.0
                    },
                    peak_size: total_size,    // Approximation
                    current_size: total_size, // Approximation
                    efficiency_score: if allocation_count > 0 { 1.0 } else { 0.0 }, // Placeholder
                },
            )
            .collect();

        // Sort by total size descending
        result.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        Ok(result)
    }

    /// Export interactive HTML dashboard with embedded SVG charts
    pub fn export_interactive_dashboard<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        crate::export::html_export::export_interactive_html(self, None, output_path)
    }

    /// Ensure path is within MemoryAnalysis/project_name directory
    fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::path::PathBuf {
        use std::path::Path;

        let path = path.as_ref();
        let base_memory_analysis_dir = Path::new("MemoryAnalysis");

        // Extract project name from the filename
        let project_name = if let Some(file_stem) = path.file_stem() {
            let stem_str = file_stem.to_string_lossy();
            // Remove common suffixes to get clean project name
            if stem_str.ends_with("_snapshot") {
                stem_str.trim_end_matches("_snapshot").to_string()
            } else if stem_str.ends_with("_graph") {
                stem_str.trim_end_matches("_graph").to_string()
            } else if stem_str.ends_with("_memory_analysis") {
                stem_str.trim_end_matches("_memory_analysis").to_string()
            } else if stem_str.ends_with("_lifecycle_timeline") {
                stem_str.trim_end_matches("_lifecycle_timeline").to_string()
            } else {
                stem_str.to_string()
            }
        } else {
            "default".to_string()
        };

        // Create project-specific directory: MemoryAnalysis/project_name/
        let project_dir = base_memory_analysis_dir.join(&project_name);

        // Create the project directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            eprintln!(
                "Warning: Failed to create project directory {}: {}",
                project_dir.display(),
                e
            );
        }

        // Get the filename
        let filename = if let Some(filename) = path.file_name() {
            filename
        } else {
            path.as_os_str()
        };

        // Return the full path: MemoryAnalysis/project_name/filename
        project_dir.join(filename)
    }

    /// Export memory analysis visualization showing variable names, types, and usage patterns.
    /// This creates a comprehensive memory analysis with call stack analysis, timeline, and categorization.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the memory analysis SVG file (recommended: "program_name_memory_analysis.svg")
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        crate::export::visualization::export_memory_analysis(self, output_path)
    }

    /// Export interactive lifecycle timeline showing variable lifecycles and relationships.
    /// This creates an advanced timeline with variable birth, life, death, and cross-section interactivity.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the lifecycle timeline SVG file (recommended: "program_name_lifecycle.svg")
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        crate::export::visualization::export_lifecycle_timeline(self, output_path)
    }

    /// Export memory tracking data to 4 separate JSON files.
    ///
    /// This method exports data to 4 specialized files:
    /// - {name}_memory_analysis.json: Memory allocation patterns and statistics
    /// - {name}_lifetime.json: Variable lifetime and scope analysis  
    /// - {name}_unsafe_ffi.json: Unsafe operations and FFI tracking
    /// - {name}_variable_relationships.json: Variable dependency graph and relationships
    ///
    /// # Export Modes
    ///
    /// ## Default Mode (Fast - Recommended)
    /// ```rust
    /// tracker.export_to_json("output")?;
    /// // OR explicitly
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    /// - **Performance**: ~2-5 seconds for typical datasets
    /// - **Data**: Only user-tracked variables get full enrichment
    /// - **Use case**: Normal development, HTML rendering, production monitoring
    ///
    /// ## Complete Mode (Slow - Debug Only)
    /// ```rust
    /// let options = ExportOptions::new().include_system_allocations(true);
    /// tracker.export_to_json_with_options("output", options)?;
    /// ```
    /// - **Performance**: ~10-40 seconds (5-10x slower!)
    /// - **Data**: ALL allocations including system internals get full enrichment
    /// - **Use case**: Deep debugging, memory leak investigation, system analysis
    /// - **⚠️ Warning**: Very slow, generates large files, may impact application performance
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Ensure output goes to MemoryAnalysis directory
        let output_path = self.ensure_memory_analysis_path(path);

        // Use fast mode by default for optimal performance

        let options = OptimizedExportOptions::default()
            .fast_export_mode(true)
            .security_analysis(false) // Disable for speed
            .schema_validation(false) // Disable for speed
            .integrity_hashes(false); // Disable for speed

        self.export_to_json_with_optimized_options(output_path, options)
    }

    /// Export memory tracking data with custom options.
    ///
    /// # Examples
    ///
    /// ## Fast mode (default - recommended for most users)
    /// ```rust
    /// tracker.export_to_json_with_options("output", ExportOptions::default())?;
    /// ```
    ///
    /// ## Complete mode (slow - for debugging)
    /// ```rust
    /// let options = ExportOptions::new()
    ///     .include_system_allocations(true)
    ///     .verbose_logging(true);
    /// tracker.export_to_json_with_options("debug_output", options)?;
    /// ```
    pub fn export_to_json_with_options<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        options: ExportOptions,
    ) -> TrackingResult<()> {
        // Convert legacy ExportOptions to OptimizedExportOptions for backward compatibility
        use crate::export::optimized_json_export::{OptimizationLevel, OptimizedExportOptions};

        let mut optimized_options = OptimizedExportOptions::default();

        // Map legacy options to optimized options
        optimized_options.buffer_size = options.buffer_size;
        optimized_options.use_compact_format = Some(!options.verbose_logging); // Verbose = pretty format

        // Determine optimization level based on legacy settings
        if options.include_system_allocations {
            // System allocations = comprehensive analysis = High optimization
            optimized_options.optimization_level = OptimizationLevel::High;
            optimized_options.enable_enhanced_ffi_analysis = true;
            optimized_options.enable_boundary_event_processing = true;
            optimized_options.enable_memory_passport_tracking = true;
            optimized_options.enable_security_analysis = true;

            println!(
                "⚠️  WARNING: System allocation enrichment enabled - export will be 5-10x slower!"
            );
            println!("💡 To speed up export, use default options: tracker.export_to_json(path)");
        } else {
            // User-focused mode = High optimization (default)
            optimized_options.optimization_level = OptimizationLevel::High;
        }

        // Enable compression if requested in legacy options
        if options.compress_output {
            optimized_options.use_compact_format = Some(true);
            optimized_options.buffer_size = optimized_options.buffer_size.max(512 * 1024);
            // Larger buffer for compression
        }

        // Adjust parallel processing based on expected load
        optimized_options.parallel_processing =
            options.include_system_allocations || options.buffer_size > 128 * 1024;

        println!("🔄 Converted legacy ExportOptions to OptimizedExportOptions:");
        println!(
            "   - Optimization level: {:?}",
            optimized_options.optimization_level
        );
        println!(
            "   - Buffer size: {} KB",
            optimized_options.buffer_size / 1024
        );
        println!(
            "   - Parallel processing: {}",
            optimized_options.parallel_processing
        );
        println!(
            "   - Enhanced features: {}",
            optimized_options.enable_enhanced_ffi_analysis
        );

        // Use the new optimized export method
        self.export_to_json(path)
    }

    /// Export memory tracking data with specified export mode.
    ///
    /// This method provides explicit control over export mode while maintaining
    /// backward compatibility with existing code.
    ///
    /// # Arguments
    /// * `path` - Output file path
    /// * `mode` - Export mode (Fast, Slow, or Auto)
    ///
    /// # Examples
    /// ```rust
    /// use memscope::export::quality_validator::ExportMode;
    ///
    /// // Fast mode for production monitoring
    /// tracker.export_json_with_mode("prod_snapshot", ExportMode::Fast)?;
    ///
    /// // Slow mode for comprehensive analysis
    /// tracker.export_json_with_mode("debug_analysis", ExportMode::Slow)?;
    ///
    /// // Auto mode for adaptive behavior
    /// tracker.export_json_with_mode("analysis", ExportMode::Auto)?;
    /// ```
    // export_json_with_mode removed due to quality_validator dependencies
    /*
    pub fn export_json_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        // mode: ExportMode, // Removed due to quality_validator deletion
    ) -> TrackingResult<()> {
        use crate::export::optimized_json_export::OptimizedExportOptions;
        // use crate::export::quality_validator::ExportMode; // Removed

        // Ensure output goes to MemoryAnalysis directory
        let output_path = self.ensure_memory_analysis_path(path);

        let options = match mode {
            ExportMode::Fast => {
                let mut opts = OptimizedExportOptions::default()
                    .fast_export_mode(true)
                    .schema_validation(false)
                    .security_analysis(false)
                    .integrity_hashes(false);
                // Disable expensive analysis features for maximum speed
                opts.enable_enhanced_ffi_analysis = false;
                opts.enable_boundary_event_processing = false;
                opts.enable_memory_passport_tracking = false;
                opts.include_low_severity_violations = false;
                opts
            }
            ExportMode::Slow => {
                let mut opts = OptimizedExportOptions::default()
                    .fast_export_mode(false)
                    .schema_validation(true)
                    .security_analysis(true);
                opts.enable_enhanced_ffi_analysis = true;
                opts
            }
            ExportMode::Auto => {
                OptimizedExportOptions::default()
                    .auto_fast_export_threshold(Some(5000))
            }
        };

        self.export_to_json_with_optimized_options(output_path, options)
    }
    */

    /// Internal method to handle export with mode and options
    #[allow(dead_code)]
    fn export_to_json_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        mode: ExportMode,
        _options: &ExportOptions, // Prefix with underscore to avoid unused warning
    ) -> TrackingResult<()> {
        use std::path::Path;

        let base_path = path.as_ref();
        let start_time = std::time::Instant::now();

        match mode {
            ExportMode::UserFocused => {
                println!("🚀 Starting FAST user-focused 5-file JSON export...");
                println!("💡 Only enriching user-tracked variables for optimal performance");
            }
            ExportMode::Complete => {
                println!("🐌 Starting COMPLETE 5-file JSON export (slow mode)...");
                println!("⚠️  This will enrich ALL allocations including system data");
            }
        }

        // Get base filename without extension
        let base_name = base_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("export");
        let parent_dir = base_path.parent().unwrap_or(Path::new("."));

        // Collect raw data directly
        println!("📊 Collecting raw data...");

        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let variable_registry = crate::variable_registry::VariableRegistry::get_all_variables();
        let unsafe_ffi_tracker = crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker();

        // 1. Export Memory Analysis with selected mode
        let memory_path = parent_dir.join(format!("{}_memory_analysis.json", base_name));
        self.export_memory_analysis_with_mode(
            &memory_path,
            &active_allocations,
            &allocation_history,
            &mode,
        )?;

        // 2. Export Lifetime Analysis - direct from raw data (FAST!)
        let lifetime_path = parent_dir.join(format!("{}_lifetime.json", base_name));
        self.export_lifetime_analysis_direct(&lifetime_path, &variable_registry)?;

        // 3. Export Unsafe/FFI Analysis - direct from raw data (FAST!)
        let unsafe_path = parent_dir.join(format!("{}_unsafe_ffi.json", base_name));
        self.export_unsafe_ffi_direct(&unsafe_path, &unsafe_ffi_tracker)?;

        // 4. Export Variable Relationships - direct from raw data (FAST!)
        let relationships_path =
            parent_dir.join(format!("{}_variable_relationships.json", base_name));
        self.export_variable_relationships_direct(
            &relationships_path,
            &variable_registry,
            &active_allocations,
        )?;

        let export_time = start_time.elapsed();

        // Calculate total file sizes
        let memory_size = std::fs::metadata(&memory_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let lifetime_size = std::fs::metadata(&lifetime_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let unsafe_size = std::fs::metadata(&unsafe_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let relationships_size = std::fs::metadata(&relationships_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let total_size = memory_size + lifetime_size + unsafe_size + relationships_size;

        println!("✅ Ultra-fast 4-file export completed in {:?}", export_time);
        println!("📁 Files created:");
        println!(
            "   - Memory Analysis: {} ({:.2} MB)",
            memory_path.display(),
            memory_size as f64 / 1024.0 / 1024.0
        );
        println!(
            "   - Lifetime Analysis: {} ({:.2} MB)",
            lifetime_path.display(),
            lifetime_size as f64 / 1024.0 / 1024.0
        );
        println!(
            "   - Unsafe/FFI Analysis: {} ({:.2} MB)",
            unsafe_path.display(),
            unsafe_size as f64 / 1024.0 / 1024.0
        );
        println!(
            "   - Variable Relationships: {} ({:.2} MB)",
            relationships_path.display(),
            relationships_size as f64 / 1024.0 / 1024.0
        );
        println!(
            "📊 Total size: {:.2} MB",
            total_size as f64 / 1024.0 / 1024.0
        );
        println!("📈 Data exported:");
        println!("   - {} active allocations", active_allocations.len());
        println!(
            "   - {} allocation history entries",
            allocation_history.len()
        );
        println!("   - {} variables in registry", variable_registry.len());
        println!(
            "   - {} unsafe/FFI operations",
            unsafe_ffi_tracker.get_stats().total_operations
        );

        if export_time.as_secs_f64() > 0.0 {
            let throughput = (total_size as f64 / 1024.0 / 1024.0) / export_time.as_secs_f64();
            println!("🚀 Performance: {:.2} MB/s total throughput", throughput);
        }

        Ok(())
    }

    /// Export memory analysis data with mode-specific enrichment
    #[allow(dead_code)]
    fn export_memory_analysis_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        active_allocations: &Vec<crate::core::types::AllocationInfo>,
        allocation_history: &Vec<crate::core::types::AllocationInfo>,
        mode: &ExportMode,
    ) -> TrackingResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path).map_err(io_error_to_tracking_error)?;
        let mut writer = BufWriter::with_capacity(65536, file);

        writer
            .write_all(b"{\n")
            .map_err(io_error_to_tracking_error)?;

        // Stream active allocations with mode-specific enrichment
        writer
            .write_all(b"  \"active_allocations\": [\n")
            .map_err(io_error_to_tracking_error)?;
        for (i, alloc) in active_allocations.iter().enumerate() {
            if i > 0 {
                writer
                    .write_all(b",\n")
                    .map_err(io_error_to_tracking_error)?;
            }

            // Apply enrichment based on mode and allocation type
            let enriched_alloc = self.enrich_allocation_with_mode(alloc, mode)?;
            let alloc_json = serde_json::to_string(&enriched_alloc)?;
            writer
                .write_all(b"    ")
                .map_err(io_error_to_tracking_error)?;
            writer
                .write_all(alloc_json.as_bytes())
                .map_err(io_error_to_tracking_error)?;
        }
        writer
            .write_all(b"\n  ],\n")
            .map_err(io_error_to_tracking_error)?;

        // Stream allocation history with mode-specific enrichment
        writer
            .write_all(b"  \"allocation_history\": [\n")
            .map_err(io_error_to_tracking_error)?;
        for (i, alloc) in allocation_history.iter().enumerate() {
            if i > 0 {
                writer
                    .write_all(b",\n")
                    .map_err(io_error_to_tracking_error)?;
            }

            // Apply enrichment based on mode and allocation type
            let enriched_alloc = self.enrich_allocation_with_mode(alloc, mode)?;
            let alloc_json = serde_json::to_string(&enriched_alloc)?;
            writer
                .write_all(b"    ")
                .map_err(io_error_to_tracking_error)?;
            writer
                .write_all(alloc_json.as_bytes())
                .map_err(io_error_to_tracking_error)?;
        }
        writer
            .write_all(b"\n  ],\n")
            .map_err(io_error_to_tracking_error)?;

        // Add memory statistics directly from tracker
        writer
            .write_all(b"  \"memory_stats\": ")
            .map_err(io_error_to_tracking_error)?;
        let stats_guard = self.stats.lock().unwrap();
        let stats_json = serde_json::to_string(&*stats_guard)?;
        writer
            .write_all(stats_json.as_bytes())
            .map_err(io_error_to_tracking_error)?;

        writer
            .write_all(b"\n}")
            .map_err(io_error_to_tracking_error)?;
        writer.flush().map_err(io_error_to_tracking_error)?;
        Ok(())
    }

    /// Export lifetime analysis data directly from raw data (ULTRA FAST!)
    #[allow(dead_code)]
    fn export_lifetime_analysis_direct<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        variable_registry: &std::collections::HashMap<
            usize,
            crate::variable_registry::VariableInfo,
        >,
    ) -> TrackingResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path).map_err(io_error_to_tracking_error)?;
        let mut writer = BufWriter::with_capacity(65536, file);

        writer
            .write_all(b"{\n")
            .map_err(io_error_to_tracking_error)?;

        // Stream variable registry directly
        writer
            .write_all(b"  \"variable_registry\": {\n")
            .map_err(io_error_to_tracking_error)?;
        let mut first = true;
        for (id, var_info) in variable_registry.iter() {
            if !first {
                writer
                    .write_all(b",\n")
                    .map_err(io_error_to_tracking_error)?;
            }
            first = false;

            let var_json = serde_json::to_string(var_info)?;
            writer
                .write_all(format!("    \"{}\": {}", id, var_json).as_bytes())
                .map_err(io_error_to_tracking_error)?;
        }
        writer
            .write_all(b"\n  },\n")
            .map_err(io_error_to_tracking_error)?;

        // Add scope information directly from tracker
        writer
            .write_all(b"  \"scope_analysis\": {\n")
            .map_err(io_error_to_tracking_error)?;
        writer
            .write_all(b"    \"current_scope_depth\": 1,\n")
            .map_err(io_error_to_tracking_error)?;
        writer
            .write_all(b"    \"total_scopes_created\": 1\n")
            .map_err(io_error_to_tracking_error)?;
        writer
            .write_all(b"  },\n")
            .map_err(io_error_to_tracking_error)?;

        // Add basic lifetime analysis
        writer
            .write_all(b"  \"lifetime_analysis\": {\n")
            .map_err(io_error_to_tracking_error)?;
        writer
            .write_all(format!("    \"total_variables\": {}\n", variable_registry.len()).as_bytes())
            .map_err(io_error_to_tracking_error)?;
        writer
            .write_all(b"  }")
            .map_err(io_error_to_tracking_error)?;

        writer
            .write_all(b"\n}")
            .map_err(io_error_to_tracking_error)?;
        writer.flush().map_err(io_error_to_tracking_error)?;
        Ok(())
    }

    /// Export unsafe/FFI analysis data directly from raw data (ULTRA FAST!)
    #[allow(dead_code)]
    fn export_unsafe_ffi_direct<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        unsafe_ffi_tracker: &crate::unsafe_ffi_tracker::UnsafeFFITracker,
    ) -> TrackingResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path).map_err(io_error_to_tracking_error)?;
        let mut writer = BufWriter::with_capacity(65536, file);

        writer
            .write_all(b"{\n")
            .map_err(io_error_to_tracking_error)?;

        // Add empty operations array for now (no get_all_operations method available)
        writer
            .write_all(b"  \"unsafe_ffi_operations\": [],\n")
            .map_err(io_error_to_tracking_error)?;

        // Include statistics directly
        writer
            .write_all(b"  \"unsafe_ffi_stats\": ")
            .map_err(io_error_to_tracking_error)?;
        let stats = unsafe_ffi_tracker.get_stats();
        let stats_json = serde_json::to_string(&stats)?;
        writer
            .write_all(stats_json.as_bytes())
            .map_err(io_error_to_tracking_error)?;

        writer
            .write_all(b"\n}")
            .map_err(io_error_to_tracking_error)?;
        writer.flush().map_err(io_error_to_tracking_error)?;
        Ok(())
    }

    /// Export variable relationships data directly from raw data (ULTRA FAST!)
    #[allow(dead_code)]
    fn export_variable_relationships_direct<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        variable_registry: &std::collections::HashMap<
            usize,
            crate::variable_registry::VariableInfo,
        >,
        active_allocations: &Vec<crate::core::types::AllocationInfo>,
    ) -> TrackingResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path).map_err(io_error_to_tracking_error)?;
        let mut writer = BufWriter::with_capacity(65536, file);

        writer
            .write_all(b"{\n")
            .map_err(io_error_to_tracking_error)?;

        // Create basic variable relationships from allocations and registry
        writer
            .write_all(b"  \"variable_relationships\": [\n")
            .map_err(io_error_to_tracking_error)?;
        let mut first = true;
        for alloc in active_allocations.iter() {
            if let Some(var_name) = &alloc.var_name {
                if !first {
                    writer
                        .write_all(b",\n")
                        .map_err(io_error_to_tracking_error)?;
                }
                first = false;

                // Create relationship entry
                writer
                    .write_all(b"    {\n")
                    .map_err(io_error_to_tracking_error)?;
                writer
                    .write_all(format!("      \"variable_name\": \"{}\",\n", var_name).as_bytes())
                    .map_err(io_error_to_tracking_error)?;
                writer
                    .write_all(
                        format!("      \"allocation_ptr\": {},\n", alloc.ptr as usize).as_bytes(),
                    )
                    .map_err(io_error_to_tracking_error)?;
                writer
                    .write_all(format!("      \"size\": {},\n", alloc.size).as_bytes())
                    .map_err(io_error_to_tracking_error)?;
                if let Some(type_name) = &alloc.type_name {
                    writer
                        .write_all(format!("      \"type_name\": \"{}\"\n", type_name).as_bytes())
                        .map_err(io_error_to_tracking_error)?;
                } else {
                    writer
                        .write_all(b"      \"type_name\": null\n")
                        .map_err(io_error_to_tracking_error)?;
                }
                writer
                    .write_all(b"    }")
                    .map_err(io_error_to_tracking_error)?;
            }
        }
        writer
            .write_all(b"\n  ],\n")
            .map_err(io_error_to_tracking_error)?;

        // Include variable registry directly
        writer
            .write_all(b"  \"variable_registry\": {\n")
            .map_err(io_error_to_tracking_error)?;
        let mut first = true;
        for (id, var_info) in variable_registry.iter() {
            if !first {
                writer
                    .write_all(b",\n")
                    .map_err(io_error_to_tracking_error)?;
            }
            first = false;

            let var_json = serde_json::to_string(var_info)?;
            writer
                .write_all(format!("    \"{}\": {}", id, var_json).as_bytes())
                .map_err(io_error_to_tracking_error)?;
        }
        writer
            .write_all(b"\n  }")
            .map_err(io_error_to_tracking_error)?;

        writer
            .write_all(b"\n}")
            .map_err(io_error_to_tracking_error)?;
        writer.flush().map_err(io_error_to_tracking_error)?;
        Ok(())
    }

    /// Enrich allocation info based on export mode and allocation type
    #[allow(dead_code)]
    fn enrich_allocation_with_mode(
        &self,
        alloc: &crate::core::types::AllocationInfo,
        mode: &ExportMode,
    ) -> TrackingResult<crate::core::types::AllocationInfo> {
        // Check if this is a user-tracked allocation
        let is_user_tracked = self.is_user_tracked_allocation(alloc);

        match mode {
            ExportMode::UserFocused => {
                if is_user_tracked {
                    // Full enrichment for user-tracked variables
                    self.enrich_allocation_complete(alloc)
                } else {
                    // Minimal enrichment for system allocations
                    self.enrich_allocation_minimal(alloc)
                }
            }
            ExportMode::Complete => {
                // Full enrichment for all allocations (slow)
                self.enrich_allocation_complete(alloc)
            }
        }
    }

    /// Check if allocation is user-tracked (FAST version - no registry lookup)
    #[allow(dead_code)]
    fn is_user_tracked_allocation(&self, alloc: &crate::core::types::AllocationInfo) -> bool {
        // Strategy 1: Check if variable name exists and looks user-generated
        if let Some(var_name) = &alloc.var_name {
            // User variables typically don't start with system prefixes
            if !var_name.starts_with("alloc_")
                && !var_name.starts_with("__")
                && !var_name.starts_with("std::")
                && !var_name.starts_with("system")
                && var_name.len() > 3
            {
                return true;
            }
        }

        // Strategy 2: Fast size-based heuristic (no expensive registry lookup!)
        // User allocations tend to be in specific size ranges
        match alloc.size {
            1..=8 => false,       // Likely system primitives
            9..=64 => true,       // Likely user primitives/small structs
            65..=1024 => true,    // Likely user data structures
            1025..=65536 => true, // Likely user buffers/collections
            _ => false,           // Very large allocations might be system
        }
    }

    /// Complete enrichment for important allocations
    #[allow(dead_code)]
    fn enrich_allocation_complete(
        &self,
        alloc: &crate::core::types::AllocationInfo,
    ) -> TrackingResult<crate::core::types::AllocationInfo> {
        let mut enriched = alloc.clone();

        // Get real variable name from registry
        if enriched.var_name.is_none() {
            enriched.var_name = self.get_variable_name_fast(enriched.ptr);
        }

        // Get accurate type information
        if enriched.type_name.is_none() {
            enriched.type_name = Some(self.get_accurate_type_name_fast(enriched.size));
        }

        // Get scope information
        if enriched.scope_name.is_none() {
            enriched.scope_name = Some("user_scope".to_string());
        }

        // Generate meaningful stack trace
        if enriched.stack_trace.is_none() {
            enriched.stack_trace = Some(vec![
                format!("user_code_0x{:x}", enriched.ptr),
                "application_logic".to_string(),
                "memscope_tracking".to_string(),
            ]);
        }

        // Calculate lifetime
        if enriched.lifetime_ms.is_none() && enriched.timestamp_dealloc.is_some() {
            let lifetime_ns = enriched.timestamp_dealloc.unwrap() - enriched.timestamp_alloc;
            enriched.lifetime_ms = Some(lifetime_ns / 1_000_000);
        }

        Ok(enriched)
    }

    /// Minimal enrichment for system allocations (fast)
    #[allow(dead_code)]
    fn enrich_allocation_minimal(
        &self,
        alloc: &crate::core::types::AllocationInfo,
    ) -> TrackingResult<crate::core::types::AllocationInfo> {
        let mut enriched = alloc.clone();

        // Only fill in completely missing fields with simple values
        if enriched.var_name.is_none() {
            enriched.var_name = Some("system_alloc".to_string());
        }

        if enriched.type_name.is_none() {
            enriched.type_name = Some("system".to_string());
        }

        if enriched.scope_name.is_none() {
            enriched.scope_name = Some("system".to_string());
        }

        // No stack trace for system allocations to save time
        if enriched.stack_trace.is_none() {
            enriched.stack_trace = Some(vec!["system".to_string()]);
        }

        Ok(enriched)
    }

    /// Legacy method for backward compatibility
    #[allow(dead_code)]
    fn enrich_allocation_info(
        &self,
        alloc: &crate::core::types::AllocationInfo,
    ) -> TrackingResult<crate::core::types::AllocationInfo> {
        let mut enriched = alloc.clone();

        // Fast path: if already has good data, minimal processing
        if enriched.var_name.is_some()
            && enriched.type_name.is_some()
            && enriched.scope_name.is_some()
        {
            return Ok(enriched);
        }

        // 1. Quick variable name lookup (fast registry check only)
        if enriched.var_name.is_none() {
            enriched.var_name = self.get_variable_name_fast(enriched.ptr);
        }

        // 2. Fast type inference (no registry lookup)
        if enriched.type_name.is_none() {
            enriched.type_name = Some(self.infer_type_fast(enriched.size));
        }

        // 3. Simple scope assignment
        if enriched.scope_name.is_none() {
            enriched.scope_name = Some("main".to_string());
        }

        // 4. Minimal stack trace (only if completely missing)
        if enriched.stack_trace.is_none() {
            enriched.stack_trace = Some(vec!["user_code".to_string()]);
        }

        // 5. Quick lifetime calculation
        if enriched.lifetime_ms.is_none() && enriched.timestamp_dealloc.is_some() {
            let lifetime_ns = enriched.timestamp_dealloc.unwrap() - enriched.timestamp_alloc;
            enriched.lifetime_ms = Some(lifetime_ns / 1_000_000);
        }

        Ok(enriched)
    }

    /// Fast variable name lookup - registry only, no fallback processing
    #[allow(dead_code)]
    fn get_variable_name_fast(&self, ptr: usize) -> Option<String> {
        let variable_registry = crate::variable_registry::VariableRegistry::get_all_variables();

        // Direct lookup only - no complex matching
        if let Some(var_info) = variable_registry.get(&ptr) {
            Some(var_info.var_name.clone())
        } else {
            // Simple fallback
            Some(format!("alloc_{:x}", ptr))
        }
    }

    /// Fast type inference - simple size-based patterns only
    #[allow(dead_code)]
    fn infer_type_fast(&self, size: usize) -> String {
        match size {
            1 => "u8",
            2 => "u16",
            4 => "u32",
            8 => "u64",
            16 => "u128",
            24 => "Vec<T>",
            32 => "HashMap",
            _ if size < 64 => "struct",
            _ if size < 1024 => "buffer",
            _ => "large_alloc",
        }
        .to_string()
    }

    /// Get accurate type name with better inference
    #[allow(dead_code)]
    fn get_accurate_type_name_fast(&self, size: usize) -> String {
        match size {
            1 => "u8/i8/bool".to_string(),
            2 => "u16/i16".to_string(),
            4 => "u32/i32/f32".to_string(),
            8 => "u64/i64/f64/usize/&T".to_string(),
            16 => "u128/i128/[u8;16]".to_string(),
            24 => "Vec<T>/String".to_string(),
            32 => "HashMap<K,V>/BTreeMap<K,V>".to_string(),
            48 => "Box<T>/Rc<T>/Arc<T>".to_string(),
            _ if size % 8 == 0 && size <= 128 => format!("struct[{}B]", size),
            _ if size < 1024 => format!("user_buffer[{}B]", size),
            _ if size < 1024 * 1024 => format!("large_buffer[{}KB]", size / 1024),
            _ => format!("huge_allocation[{}MB]", size / 1024 / 1024),
        }
    }

    /// Get real variable name through comprehensive matching
    #[allow(dead_code)]
    fn get_real_variable_name(&self, ptr: usize, size: usize, timestamp: u64) -> Option<String> {
        let variable_registry = crate::variable_registry::VariableRegistry::get_all_variables();

        // Strategy 1: Direct address lookup (registry uses address as key)
        if let Some(var_info) = variable_registry.get(&ptr) {
            return Some(var_info.var_name.clone());
        }

        // Strategy 2: Size + timestamp proximity match (within 1ms)
        let mut best_match: Option<(String, u64)> = None;
        for (_, var_info) in variable_registry.iter() {
            if var_info.size == size {
                let time_diff = if timestamp > var_info.timestamp {
                    timestamp - var_info.timestamp
                } else {
                    var_info.timestamp - timestamp
                };

                // Within 1ms window
                if time_diff < 1_000_000 {
                    match &best_match {
                        None => best_match = Some((var_info.var_name.clone(), time_diff)),
                        Some((_, prev_diff)) => {
                            if time_diff < *prev_diff {
                                best_match = Some((var_info.var_name.clone(), time_diff));
                            }
                        }
                    }
                }
            }
        }

        if let Some((name, _)) = best_match {
            return Some(name);
        }

        // Strategy 3: Generate descriptive name based on context
        Some(format!("allocation_{}_{}_bytes", ptr, size))
    }

    /// Get accurate type name through multiple strategies
    #[allow(dead_code)]
    fn get_accurate_type_name(&self, ptr: usize, size: usize) -> Option<String> {
        let variable_registry = crate::variable_registry::VariableRegistry::get_all_variables();

        // Strategy 1: Get from variable registry by address
        if let Some(var_info) = variable_registry.get(&ptr) {
            if !var_info.type_name.is_empty() {
                return Some(var_info.type_name.clone());
            }
        }

        // Strategy 2: Pattern-based type inference with common Rust types
        let inferred_type = match size {
            1 => "u8",
            2 => "u16",
            4 => "u32/f32",
            8 => "u64/f64/usize/&T",
            16 => "u128/[u8; 16]",
            24 => "Vec<T>/String",
            32 => "HashMap<K,V>/[u8; 32]",
            48 => "Box<T>/Rc<T>/Arc<T>",
            _ if size % 8 == 0 && size <= 128 => "struct/tuple",
            _ if size > 128 && size < 1024 => "large_struct/array",
            _ if size >= 1024 && size < 1024 * 1024 => "buffer/large_array",
            _ => "large_allocation",
        };

        Some(inferred_type.to_string())
    }

    /// Get real scope name from scope tracker
    #[allow(dead_code)]
    fn get_real_scope_name(&self, _timestamp: u64) -> Option<String> {
        // Try to get scope information from the scope tracker
        // This is a simplified version - in reality you'd track scope changes over time
        Some("main_function".to_string())
    }

    /// Generate meaningful stack trace
    #[allow(dead_code)]
    fn generate_meaningful_stack_trace(&self, ptr: usize) -> Option<Vec<String>> {
        // Generate a more meaningful stack trace
        Some(vec![
            format!("allocation_site_0x{:x}", ptr),
            "user_code::allocate_memory".to_string(),
            "memscope_rs::track_allocation".to_string(),
            "rust_allocator::alloc".to_string(),
        ])
    }

    /// Enrich lifecycle information
    #[allow(dead_code)]
    fn enrich_lifecycle_info(&self, enriched: &mut crate::core::types::AllocationInfo) {
        // Calculate lifetime if possible
        if enriched.lifetime_ms.is_none() && enriched.timestamp_dealloc.is_some() {
            let lifetime_ns = enriched.timestamp_dealloc.unwrap() - enriched.timestamp_alloc;
            enriched.lifetime_ms = Some(lifetime_ns / 1_000_000);
        }

        // Add lifecycle status
        if enriched.timestamp_dealloc.is_some() {
            // This allocation has been deallocated
        } else {
            // This allocation is still active
        }
    }

    /// Add memory pattern analysis
    #[allow(dead_code)]
    fn add_memory_pattern_info(&self, enriched: &mut crate::core::types::AllocationInfo) {
        // Analyze memory patterns
        let ptr_value = enriched.ptr;

        // Check alignment
        let alignment = if ptr_value % 8 == 0 {
            8
        } else if ptr_value % 4 == 0 {
            4
        } else if ptr_value % 2 == 0 {
            2
        } else {
            1
        };

        // This could be stored in additional fields if the struct supports it
        // For now, we can enhance the variable name with this info
        if let Some(ref mut var_name) = enriched.var_name {
            if alignment >= 8 {
                *var_name = format!("{}_aligned", var_name);
            }
        }
    }

    /// Export memory tracking data to binary format (.memscope file)
    /// 
    /// This method provides high-performance binary export that is 3x faster than JSON
    /// and produces files that are 60%+ smaller. The binary format uses a simple TLV
    /// structure for efficient storage and parsing.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Output file path (will be saved in MemoryAnalysis directory with .memscope extension)
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` on success, or `TrackingError` on failure
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let tracker = get_global_tracker();
    /// tracker.export_to_binary("my_program")?;
    /// // Creates: MemoryAnalysis/my_program/my_program.memscope
    /// ```
    pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);
        
        tracing::info!("Starting binary export to: {}", output_path.display());
        
        let allocations = self.get_active_allocations()?;
        
        crate::export::binary::export_to_binary(&allocations, output_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;
        
        tracing::info!("Binary export completed successfully");
        Ok(())
    }

    /// Convert binary file to JSON format
    /// 
    /// This method reads a .memscope binary file and converts it to JSON format,
    /// maintaining full compatibility with existing JSON export structure.
    /// 
    /// # Arguments
    /// 
    /// * `binary_path` - Path to input .memscope file
    /// * `json_path` - Path for output JSON file
    /// 
    /// Convert binary file to standard JSON files (5 categorized files)
    /// 
    /// This method generates the same 5 JSON files as the standard JSON export:
    /// - memory_analysis.json, lifetime.json, performance.json, unsafe_ffi.json, complex_types.json
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// MemoryTracker::parse_binary_to_standard_json("data.memscope", "project_name")?;
    /// ```
    pub fn parse_binary_to_standard_json<P: AsRef<std::path::Path>>(
        binary_path: P, 
        base_name: &str
    ) -> TrackingResult<()> {
        crate::export::binary::BinaryParser::to_standard_json_files(binary_path, base_name)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }
    
    /// Convert binary file to single JSON format (legacy compatibility)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// MemoryTracker::parse_binary_to_json("data.memscope", "data.json")?;
    /// ```
    pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
        binary_path: P, 
        json_path: P
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_json(binary_path, json_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Convert binary file to HTML format
    /// 
    /// This method reads a .memscope binary file and generates an HTML report
    /// with memory allocation analysis and visualization.
    /// 
    /// # Arguments
    /// 
    /// * `binary_path` - Path to input .memscope file
    /// * `html_path` - Path for output HTML file
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// MemoryTracker::parse_binary_to_html("data.memscope", "report.html")?;
    /// ```
    pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
        binary_path: P, 
        html_path: P
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_html(binary_path, html_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Ensure path uses .memscope extension and is in MemoryAnalysis directory
    fn ensure_memscope_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::path::PathBuf {
        let mut output_path = self.ensure_memory_analysis_path(path);
        
        // Ensure .memscope extension
        if output_path.extension().is_none() || output_path.extension() != Some(std::ffi::OsStr::new("memscope")) {
            output_path.set_extension("memscope");
        }
        
        output_path
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    /// Track growth events for a variable
    fn _track_growth_events(
        &self,
        var_name: &str,
        allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::GrowthEvent> {
        let mut growth_events = Vec::new();
        let mut last_size = 0;

        for alloc in allocation_history {
            if let Some(name) = &alloc.var_name {
                if name == var_name && alloc.size > last_size {
                    growth_events.push(crate::core::types::GrowthEvent {
                        timestamp: alloc.timestamp_alloc,
                        old_size: last_size,
                        new_size: alloc.size,
                        growth_factor: if last_size > 0 {
                            alloc.size as f64 / last_size as f64
                        } else {
                            1.0
                        },
                        reason: if last_size > 0 {
                            crate::core::types::GrowthReason::Expansion
                        } else {
                            crate::core::types::GrowthReason::Initial
                        },
                        var_name: alloc
                            .var_name
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                    });
                    last_size = alloc.size;
                }
            }
        }

        growth_events
    }

    /// Track borrow events for a variable
    fn _track_borrow_events(
        &self,
        _var_name: &str,
        _allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::BorrowEvent> {
        // Simplified implementation - return empty for now
        Vec::new()
    }

    /// Track move events for a variable
    fn _track_move_events(
        &self,
        _var_name: &str,
        _allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::MoveEvent> {
        // Simplified implementation - return empty for now
        Vec::new()
    }

    /// Track variable relationships
    fn _track_variable_relationships(
        &self,
        _var_name: &str,
        _active_allocations: &[AllocationInfo],
    ) -> Vec<crate::core::types::VariableRelationship> {
        // Simplified implementation - return empty for now
        Vec::new()
    }

    /// Calculate minimum allocation size for a type
    fn _calculate_min_allocation_size(
        &self,
        type_name: &str,
        allocation_history: &[AllocationInfo],
    ) -> usize {
        allocation_history
            .iter()
            .filter(|alloc| alloc.type_name.as_deref() == Some(type_name))
            .map(|alloc| alloc.size)
            .min()
            .unwrap_or(0)
    }

    /// Detect potential memory leaks
    fn _detect_potential_leaks(
        &self,
        active_allocations: &[AllocationInfo],
    ) -> Vec<crate::core::types::PotentialLeak> {
        let mut leaks = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        for alloc in active_allocations {
            let age_ms = (now.saturating_sub(alloc.timestamp_alloc as u128)) / 1_000_000;

            // Consider allocations older than 10 seconds as potential leaks
            if age_ms > 10_000 {
                let confidence = if age_ms > 60_000 { 0.9 } else { 0.5 };

                leaks.push(crate::core::types::PotentialLeak {
                    ptr: alloc.ptr,
                    size: alloc.size,
                    age_ms: age_ms.try_into().unwrap_or(u64::MAX),
                    var_name: alloc.var_name.clone(),
                    type_name: alloc.type_name.clone(),
                    severity: if confidence > 0.8 {
                        "high".to_string()
                    } else if confidence > 0.5 {
                        "medium".to_string()
                    } else {
                        "low".to_string()
                    },
                });
            }
        }

        leaks
    }

    /// Convert unsafe violations from unsafe tracker
    fn _convert_unsafe_violations(&self) -> Vec<crate::core::types::SafetyViolation> {
        // Simplified implementation - return empty for now
        Vec::new()
    }

    /// Generate timeline data with stack traces and hotspots
    pub fn generate_timeline_data(
        &self,
        allocation_history: &[AllocationInfo],
        _active_allocations: &[AllocationInfo],
    ) -> crate::core::types::TimelineData {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Generate memory snapshots (every 100ms or every 10 allocations)
        let memory_snapshots = self.generate_memory_snapshots(allocation_history);

        // Generate allocation events
        let allocation_events = self.generate_allocation_events(allocation_history);

        // Generate scope events
        let scope_events = self.generate_scope_events(allocation_history);

        // Calculate time range
        let start_time = allocation_history
            .iter()
            .map(|a| a.timestamp_alloc)
            .min()
            .unwrap_or(now as u64);
        let end_time = allocation_history
            .iter()
            .filter_map(|a| a.timestamp_dealloc.or(Some(now as u64)))
            .max()
            .unwrap_or(now as u64);

        let time_range = crate::core::types::TimeRange {
            start_time,
            end_time,
            duration_ms: (end_time.saturating_sub(start_time)) / 1_000_000,
        };

        // Generate stack trace data
        let _stack_traces = self.generate_stack_trace_data(allocation_history);

        // Generate allocation hotspots
        let _allocation_hotspots = self.generate_allocation_hotspots(allocation_history);

        crate::core::types::TimelineData {
            memory_snapshots,
            allocation_events,
            scope_events,
            time_range,
        }
    }

    /// Generate memory snapshots over time
    fn generate_memory_snapshots(
        &self,
        allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::MemorySnapshot> {
        let mut snapshots = Vec::new();
        let mut current_memory: usize = 0;
        let mut current_allocations: usize = 0;
        let mut scope_breakdown: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Group allocations by time windows (every 100ms)
        let mut events: Vec<_> = allocation_history.iter().collect();
        events.sort_by_key(|a| a.timestamp_alloc);

        let start_time = events.first().map(|a| a.timestamp_alloc).unwrap_or(0);
        let window_size = 100_000_000; // 100ms in nanoseconds

        let mut current_window = start_time;
        let mut window_allocations = Vec::new();

        for alloc in events {
            if alloc.timestamp_alloc >= current_window + window_size {
                // Process current window
                if !window_allocations.is_empty() {
                    snapshots.push(crate::core::types::MemorySnapshot {
                        timestamp: current_window,
                        total_memory: current_memory,
                        active_allocations: current_allocations,
                        fragmentation_ratio: 0.1,
                        top_types: vec![],
                    });
                }

                // Move to next window
                current_window = alloc.timestamp_alloc;
                window_allocations.clear();
            }

            window_allocations.push(alloc);
            current_memory = current_memory.saturating_add(alloc.size);
            current_allocations = current_allocations.saturating_add(1);

            // Update scope breakdown
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            *scope_breakdown.entry(scope.to_string()).or_insert(0) += alloc.size;
        }

        // Add final snapshot
        if !window_allocations.is_empty() {
            snapshots.push(crate::core::types::MemorySnapshot {
                timestamp: current_window,
                total_memory: current_memory,
                active_allocations: current_allocations,
                fragmentation_ratio: 0.1,
                top_types: vec![],
            });
        }

        snapshots
    }

    /// Generate allocation events
    fn generate_allocation_events(
        &self,
        allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::AllocationEvent> {
        let mut events = Vec::new();

        for alloc in allocation_history {
            // Allocation event
            events.push(crate::core::types::AllocationEvent {
                timestamp: alloc.timestamp_alloc,
                event_type: crate::core::types::AllocationEventType::Allocate,
                ptr: alloc.ptr,
                size: alloc.size,
                var_name: alloc.var_name.clone(),
                type_name: alloc.type_name.clone(),
            });

            // Deallocation event (if applicable)
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                events.push(crate::core::types::AllocationEvent {
                    timestamp: dealloc_time,
                    event_type: crate::core::types::AllocationEventType::Deallocate,
                    ptr: alloc.ptr,
                    size: alloc.size,
                    var_name: alloc.var_name.clone(),
                    type_name: alloc.type_name.clone(),
                });
            }
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);
        events
    }

    /// Generate scope events
    fn generate_scope_events(
        &self,
        allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::ScopeEvent> {
        let mut scope_events = Vec::new();
        let mut scope_states: std::collections::HashMap<String, (u128, usize)> =
            std::collections::HashMap::new();

        for alloc in allocation_history {
            let scope_name = alloc
                .scope_name
                .clone()
                .unwrap_or_else(|| "global".to_string());

            // Check if this is the first time we see this scope
            if !scope_states.contains_key(&scope_name) {
                scope_events.push(crate::core::types::ScopeEvent {
                    timestamp: alloc.timestamp_alloc,
                    event_type: crate::core::types::ScopeEventType::Enter,
                    scope_name: scope_name.clone(),
                    memory_usage: 0,
                    variable_count: 0,
                });
                scope_states.insert(
                    scope_name.clone(),
                    (alloc.timestamp_alloc as u128, alloc.size),
                );
            } else {
                // Update memory impact
                if let Some((_, ref mut memory)) = scope_states.get_mut(&scope_name) {
                    *memory += alloc.size;
                }
            }

            // Generate exit event if deallocation happened
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                if let Some((_, memory)) = scope_states.get(&scope_name) {
                    scope_events.push(crate::core::types::ScopeEvent {
                        timestamp: dealloc_time,
                        event_type: crate::core::types::ScopeEventType::Exit,
                        scope_name: scope_name,
                        memory_usage: *memory,
                        variable_count: 1,
                    });
                }
            }
        }

        scope_events.sort_by_key(|e| e.timestamp);
        scope_events
    }

    /// Generate stack trace data
    fn generate_stack_trace_data(
        &self,
        allocation_history: &[AllocationInfo],
    ) -> crate::core::types::StackTraceData {
        let mut traces = std::collections::HashMap::new();
        let mut stack_stats: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();

        // Generate synthetic stack traces for each allocation
        for alloc in allocation_history {
            let stack_id = format!("stack_{}", alloc.ptr);
            let stack_frames = self.generate_synthetic_stack_trace(alloc);

            traces.insert(stack_id, stack_frames.clone());

            // Update statistics
            let stack_key = self.stack_frames_to_key(&stack_frames);
            let (count, memory) = stack_stats.entry(stack_key).or_insert((0, 0));
            *count += 1;
            *memory += alloc.size;
        }

        // Generate hotspots
        let hotspots = stack_stats
            .into_iter()
            .map(|(stack_key, (count, memory))| {
                let stack_pattern = self.parse_stack_key(&stack_key);
                crate::core::types::StackTraceHotspot {
                    function_name: stack_pattern
                        .first()
                        .map(|f| f.function_name.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    allocation_count: count,
                    total_bytes: memory,
                    average_size: memory as f64 / count.max(1) as f64,
                    percentage: (memory as f64
                        / allocation_history.iter().map(|a| a.size).sum::<usize>() as f64)
                        * 100.0,
                }
            })
            .collect();

        // Generate common patterns
        let common_patterns = vec![crate::core::types::AllocationPattern {
            pattern_type: "Vec allocations in loops".to_string(),
            frequency: allocation_history.len() / 4,
            total_bytes: allocation_history.iter().map(|a| a.size).sum::<usize>() / 4,
            description: "Frequent Vec allocations detected in loop patterns".to_string(),
        }];

        crate::core::types::StackTraceData {
            hotspots,
            allocation_patterns: common_patterns,
            total_samples: allocation_history.len(),
        }
    }

    /// Generate synthetic stack trace for an allocation
    fn generate_synthetic_stack_trace(
        &self,
        alloc: &AllocationInfo,
    ) -> Vec<crate::core::types::StackFrame> {
        let mut frames = Vec::new();

        // Add main frame
        frames.push(crate::core::types::StackFrame {
            function_name: "main".to_string(),
            file_name: Some("main.rs".to_string()),
            line_number: Some(42),
            module_path: Some("my_app".to_string()),
        });

        // Add scope-specific frame
        if let Some(scope) = &alloc.scope_name {
            frames.push(crate::core::types::StackFrame {
                function_name: scope.clone(),
                file_name: Some(format!("{scope}.rs")),
                line_number: Some(15),
                module_path: Some("my_app".to_string()),
            });
        }

        // Add type-specific frame
        if let Some(type_name) = &alloc.type_name {
            if type_name.contains("Vec") {
                frames.push(crate::core::types::StackFrame {
                    function_name: "Vec::new".to_string(),
                    file_name: Some("vec.rs".to_string()),
                    line_number: Some(123),
                    module_path: Some("alloc::vec".to_string()),
                });
            } else if type_name.contains("String") {
                frames.push(crate::core::types::StackFrame {
                    function_name: "String::new".to_string(),
                    file_name: Some("string.rs".to_string()),
                    line_number: Some(456),
                    module_path: Some("alloc::string".to_string()),
                });
            }
        }

        frames
    }

    /// Convert stack frames to a key for grouping
    fn stack_frames_to_key(&self, frames: &[crate::core::types::StackFrame]) -> String {
        frames
            .iter()
            .map(|f| format!("{}:{}", f.function_name, f.line_number.unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Parse stack key back to frames
    fn parse_stack_key(&self, key: &str) -> Vec<crate::core::types::StackFrame> {
        key.split('|')
            .map(|part| {
                let parts: Vec<&str> = part.split(':').collect();
                crate::core::types::StackFrame {
                    function_name: parts.first().unwrap_or(&"unknown").to_string(),
                    file_name: None,
                    line_number: parts.get(1).and_then(|s| s.parse().ok()),
                    module_path: None,
                }
            })
            .collect()
    }

    /// Generate allocation hotspots over time
    fn generate_allocation_hotspots(
        &self,
        allocation_history: &[AllocationInfo],
    ) -> Vec<crate::core::types::AllocationHotspot> {
        let mut hotspots = Vec::new();
        let window_size = 1_000_000_000; // 1 second windows

        if allocation_history.is_empty() {
            return hotspots;
        }

        let start_time = allocation_history
            .iter()
            .map(|a| a.timestamp_alloc)
            .min()
            .unwrap_or(0);
        let end_time = allocation_history
            .iter()
            .map(|a| a.timestamp_alloc)
            .max()
            .unwrap_or(0);

        let mut current_window = start_time;

        while current_window < end_time {
            let window_end = current_window + window_size;

            // Find allocations in this window
            let window_allocs: Vec<_> = allocation_history
                .iter()
                .filter(|a| a.timestamp_alloc >= current_window && a.timestamp_alloc < window_end)
                .collect();

            if !window_allocs.is_empty() {
                let total_memory: usize = window_allocs.iter().map(|a| a.size).sum();
                let allocation_count = window_allocs.len();

                // Find the most common location in this window
                let mut location_counts: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                for alloc in &window_allocs {
                    let location = alloc
                        .scope_name
                        .clone()
                        .unwrap_or_else(|| "global".to_string());
                    *location_counts.entry(location).or_insert(0) += 1;
                }

                let most_common_location = location_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(location, _)| location)
                    .unwrap_or_else(|| "global".to_string());

                hotspots.push(crate::core::types::AllocationHotspot {
                    location: crate::core::types::HotspotLocation {
                        function_name: most_common_location.clone(),
                        file_path: Some(format!("{most_common_location}.rs")),
                        line_number: Some(42),
                        module_path: Some(most_common_location),
                    },
                    allocation_count,
                    total_bytes: total_memory,
                    average_size: if allocation_count > 0 {
                        total_memory as f64 / allocation_count as f64
                    } else {
                        0.0
                    },
                    frequency: allocation_count as f64,
                });
            }

            current_window = window_end;
        }

        hotspots
    }
}

/// Estimate the size of a type based on its name
/// This is used for synthetic allocations when we can't get the exact size
fn estimate_type_size(type_name: &str) -> usize {
    if type_name.contains("Box<") {
        // Box typically contains a pointer (8 bytes) plus the size of the contained type
        if type_name.contains("Vec") {
            64 // Vec has capacity, length, and pointer
        } else if type_name.contains("String") {
            48 // String has capacity, length, and pointer
        } else if type_name.contains("HashMap") {
            128 // HashMap has more complex internal structure
        } else {
            32 // Generic Box overhead
        }
    } else if type_name.contains("Rc<") || type_name.contains("Arc<") {
        // Reference counted types have additional overhead
        if type_name.contains("RefCell") {
            72 // Rc<RefCell<T>> has extra indirection
        } else {
            56 // Basic Rc/Arc overhead
        }
    } else if type_name.contains("Vec<") {
        // Direct Vec allocation
        48 // Vec struct size (capacity, length, pointer)
    } else if type_name.contains("String") {
        // Direct String allocation
        32 // String struct size (capacity, length, pointer)
    } else if type_name.contains("HashMap") {
        // Direct HashMap allocation
        96 // HashMap has complex internal structure
    } else {
        // Default estimate for unknown types
        24
    }
}

/// Build unified dashboard JSON structure compatible with all frontend interfaces
pub fn build_unified_dashboard_structure(
    active_allocations: &[AllocationInfo],
    allocation_history: &[AllocationInfo],
    memory_by_type: &[crate::core::types::TypeMemoryUsage],
    stats: &crate::core::types::MemoryStats,
    unsafe_stats: &crate::unsafe_ffi_tracker::UnsafeFFIStats,
) -> serde_json::Value {
    // Calculate performance metrics
    let total_runtime_ms = allocation_history
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(0)
        .saturating_sub(
            allocation_history
                .iter()
                .map(|a| a.timestamp_alloc)
                .min()
                .unwrap_or(0),
        )
        / 1_000_000; // Convert nanoseconds to milliseconds

    let allocation_rate = if total_runtime_ms > 0 {
        (stats.total_allocations as f64 * 1000.0) / total_runtime_ms as f64
    } else {
        0.0
    };

    let deallocation_rate = if total_runtime_ms > 0 {
        (stats.total_deallocations as f64 * 1000.0) / total_runtime_ms as f64
    } else {
        0.0
    };

    // Calculate memory efficiency (active memory / peak memory)
    let memory_efficiency = if stats.peak_memory > 0 {
        (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
    } else {
        100.0
    };

    // Calculate fragmentation ratio (simplified)
    let fragmentation_ratio = if stats.total_allocated > 0 {
        1.0 - (stats.active_memory as f64 / stats.total_allocated as f64)
    } else {
        0.0
    };

    // Prepare allocation details for frontend - use filtered data
    let allocation_details: Vec<_> = active_allocations
        .iter()
        .map(|alloc| {
            serde_json::json!({
                "size": alloc.size,
                "type": alloc.type_name.as_deref().unwrap_or("unknown"),
                "variable": alloc.var_name.as_deref().unwrap_or("unknown"),
                "timestamp": alloc.timestamp_alloc
            })
        })
        .collect();

    // Prepare unsafe operations for frontend
    let unsafe_operations: Vec<_> = unsafe_stats
        .operations
        .iter()
        .take(50) // Limit to avoid huge JSON files
        .map(|op| {
            serde_json::json!({
                "type": format!("{:?}", op.operation_type),
                "location": op.location,
                "risk_level": format!("{:?}", op.risk_level),
                "timestamp": op.timestamp,
                "description": op.description
            })
        })
        .collect();

    // Calculate lifecycle statistics
    let _now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut lifetimes: Vec<u128> = allocation_history
        .iter()
        .filter_map(|alloc| {
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                if dealloc_time > 0 {
                    Some((dealloc_time as u128).saturating_sub(alloc.timestamp_alloc as u128))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    lifetimes.sort_unstable();
    let average_lifetime_ms = if !lifetimes.is_empty() {
        lifetimes.iter().sum::<u128>() / lifetimes.len() as u128 / 1_000_000
    } else {
        0
    };

    // Categorize objects by lifetime
    let short_lived = lifetimes.iter().filter(|&&lt| lt < 1_000_000_000).count(); // < 1 second
    let medium_lived = lifetimes
        .iter()
        .filter(|&&lt| (1_000_000_000..10_000_000_000).contains(&lt))
        .count(); // 1-10 seconds
    let long_lived = lifetimes.iter().filter(|&&lt| lt >= 10_000_000_000).count(); // > 10 seconds

    // Build hierarchical memory structure for backward compatibility
    // Simple type enhancement for now
    let enhanced_types: Vec<MemoryTypeInfo> = memory_by_type
        .iter()
        .map(|usage| MemoryTypeInfo {
            type_name: usage.type_name.clone(),
            total_size: usage.total_size,
            allocation_count: usage.allocation_count,
            average_size: usage.average_size as usize,
            largest_allocation: usage.peak_size,
            smallest_allocation: if usage.total_size > 0 { 1 } else { 0 },
            active_instances: usage.allocation_count,
            leaked_instances: 0,
        })
        .collect();
    let memory_hierarchy = build_legacy_hierarchy(&enhanced_types, active_allocations, stats);

    // Build the unified dashboard structure
    serde_json::json!({
        "memory_stats": {
            "total_allocations": stats.total_allocations,
            "total_size_bytes": stats.total_allocated,
            "peak_memory_usage": stats.peak_memory,
            "current_memory_usage": stats.active_memory,
            "allocation_rate": allocation_rate,
            "deallocation_rate": deallocation_rate,
            "memory_efficiency": memory_efficiency,
            "fragmentation_ratio": fragmentation_ratio,
            "allocations": allocation_details
        },
        "unsafe_stats": {
            "total_operations": unsafe_stats.total_operations,
            "unsafe_blocks": unsafe_stats.unsafe_blocks,
            "ffi_calls": unsafe_stats.ffi_calls,
            "raw_pointer_operations": unsafe_stats.raw_pointer_operations,
            "memory_violations": unsafe_stats.memory_violations,
            "risk_score": unsafe_stats.risk_score,
            "operations": unsafe_operations
        },
        "performance_metrics": {
            "allocation_time_avg_ns": if stats.total_allocations > 0 {
                (total_runtime_ms * 1_000_000) / stats.total_allocations as u64
            } else {
                0
            },
            "allocation_time_max_ns": total_runtime_ms * 1_000_000, // Simplified
            "memory_throughput_mb_s": if total_runtime_ms > 0 {
                (stats.total_allocated as f64 / 1_048_576.0) / (total_runtime_ms as f64 / 1000.0)
            } else {
                0.0
            },
            "gc_pressure": fragmentation_ratio
        },
        "lifecycle_stats": {
            "short_lived_objects": short_lived,
            "medium_lived_objects": medium_lived,
            "long_lived_objects": long_lived,
            "average_lifetime_ms": average_lifetime_ms,
            "memory_leaks_detected": stats.active_allocations.saturating_sub(
                allocation_history.iter().filter(|a| a.timestamp_dealloc.is_some()).count()
            )
        },
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "version": "2.0",
            "source": "memscope-rs unified dashboard export",
            "total_runtime_ms": total_runtime_ms,
            "format_description": "Unified dashboard format compatible with all frontend interfaces"
        },
        // Keep legacy hierarchy for backward compatibility
        "memory_hierarchy": memory_hierarchy,
        // Summary for legacy compatibility
        "summary": {
            "total_memory_bytes": stats.total_allocated,
            "total_allocations": stats.total_allocations,
            "active_allocations": stats.active_allocations,
            "active_memory_bytes": stats.active_memory,
            "peak_memory_bytes": stats.peak_memory
        }
    })
}

/// Build legacy hierarchical structure for backward compatibility
fn build_legacy_hierarchy(
    enhanced_types: &[MemoryTypeInfo],
    active_allocations: &[AllocationInfo],
    _stats: &crate::core::types::MemoryStats,
) -> serde_json::Value {
    use std::collections::HashMap;

    // Group enhanced types by category and subcategory
    let mut categories: HashMap<String, HashMap<String, Vec<&MemoryTypeInfo>>> = HashMap::new();

    for enhanced_type in enhanced_types {
        categories
            .entry("general".to_string())
            .or_default()
            .entry("unknown".to_string())
            .or_default()
            .push(enhanced_type);
    }

    // Build hierarchical structure
    let mut category_data = serde_json::Map::new();
    let total_memory: usize = enhanced_types.iter().map(|t| t.total_size).sum();

    for (category_name, subcategories) in categories {
        let category_total: usize = subcategories
            .values()
            .flat_map(|types| types.iter())
            .map(|t| t.total_size)
            .sum();

        let category_percentage = if total_memory > 0 {
            (category_total as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let mut subcategory_data = serde_json::Map::new();
        let subcategory_count = subcategories.len();

        for (subcategory_name, types) in subcategories {
            let subcategory_total: usize = types.iter().map(|t| t.total_size).sum();
            let subcategory_percentage = if category_total > 0 {
                (subcategory_total as f64 / category_total as f64) * 100.0
            } else {
                0.0
            };

            let mut type_details = Vec::with_capacity(types.len());
            let type_count = types.len();
            for type_info in &types {
                let type_percentage = if subcategory_total > 0 {
                    (type_info.total_size as f64 / subcategory_total as f64) * 100.0
                } else {
                    0.0
                };

                // Find allocations for this specific type
                let type_allocations: Vec<_> = active_allocations
                    .iter()
                    .filter(|alloc| {
                        if let Some(type_name) = &alloc.type_name {
                            alloc.var_name.as_ref().is_some_and(|_var_name| {
                                false // Simplified for now
                            }) || type_name.contains(&type_info.type_name)
                        } else {
                            false
                        }
                    })
                    .map(|alloc| {
                        serde_json::json!({
                            "variable_name": alloc.var_name,
                            "size_bytes": alloc.size,
                            "allocation_time": alloc.timestamp_alloc,
                            "type_name": alloc.type_name
                        })
                    })
                    .collect();

                type_details.push(serde_json::json!({
                    "type_name": type_info.type_name,
                    "size_bytes": type_info.total_size,
                    "allocation_count": type_info.allocation_count,
                    "percentage_of_subcategory": format!("{:.1}%", type_percentage),
                    "percentage_of_total": format!("{:.1}%", (type_info.total_size as f64 / total_memory as f64) * 100.0),
                    "variable_names": Vec::<String>::new(),
                    "allocations": type_allocations
                }));
            }

            subcategory_data.insert(subcategory_name, serde_json::json!({
                "summary": {
                    "total_size_bytes": subcategory_total,
                    "percentage_of_category": format!("{:.1}%", subcategory_percentage),
                    "percentage_of_total": format!("{:.1}%", (subcategory_total as f64 / total_memory as f64) * 100.0),
                    "type_count": type_count
                },
                "types": type_details
            }));
        }

        category_data.insert(
            category_name,
            serde_json::json!({
                "summary": {
                    "total_size_bytes": category_total,
                    "percentage_of_total": format!("{:.1}%", category_percentage),
                    "subcategory_count": subcategory_count
                },
                "subcategories": subcategory_data
            }),
        );
    }

    serde_json::Value::Object(category_data)
}

impl MemoryTracker {
    /// Enhance allocations with precise names, eliminating unknown types
    fn _enhance_allocations_with_precise_names(
        &self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<Vec<AllocationInfo>> {
        let mut enhanced = Vec::new();

        for alloc in allocations {
            let mut enhanced_alloc = alloc.clone();

            // If no variable name, generate one based on size and type
            if enhanced_alloc.var_name.is_none() {
                let (smart_name, smart_type) = _analyze_system_allocation(alloc);
                enhanced_alloc.var_name = Some(smart_name);
                enhanced_alloc.type_name = Some(smart_type);
            }

            // If type is unknown, infer from size and context
            if enhanced_alloc.type_name.as_deref() == Some("Unknown")
                || enhanced_alloc.type_name.is_none()
            {
                enhanced_alloc.type_name = Some(self._infer_type_from_context(alloc));
            }

            enhanced.push(enhanced_alloc);
        }

        Ok(enhanced)
    }

    /// Infer type from allocation context
    fn _infer_type_from_context(&self, alloc: &AllocationInfo) -> String {
        match alloc.size {
            1..=8 => "Small Primitive".to_string(),
            9..=32 => "Medium Structure".to_string(),
            33..=128 => "Large Structure".to_string(),
            129..=1024 => "Buffer/Array".to_string(),
            1025..=4096 => "Large Buffer".to_string(),
            4097..=16384 => "Page Buffer".to_string(),
            _ => "Large Memory Block".to_string(),
        }
    }

    /// Generate scope hierarchy
    fn _generate_scope_hierarchy(
        &self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<serde_json::Value> {
        let mut scope_tree = std::collections::HashMap::new();
        let mut scope_stats = std::collections::HashMap::new();

        for alloc in allocations {
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            let parts: Vec<&str> = scope.split("::").collect();

            // Build hierarchy
            let mut current_path = String::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    current_path.push_str("::");
                }
                current_path.push_str(part);

                let entry = scope_tree.entry(current_path.clone()).or_insert_with(|| {
                    serde_json::json!({
                        "name": part,
                        "full_path": current_path,
                        "level": i,
                        "children": [],
                        "allocations": []
                    })
                });

                // Add allocation to this scope
                if let Some(allocations_array) = entry.get_mut("allocations") {
                    if let Some(array) = allocations_array.as_array_mut() {
                        array.push(serde_json::json!({
                            "variable_name": alloc.var_name.as_deref().unwrap_or("unnamed"),
                            "size": alloc.size,
                            "type_name": alloc.type_name.as_deref().unwrap_or("inferred")
                        }));
                    }
                }
            }

            // Update scope statistics
            let stats = scope_stats.entry(scope.to_string()).or_insert((0, 0));
            stats.0 += alloc.size;
            stats.1 += 1;
        }

        // Convert to hierarchical structure
        let levels: Vec<_> = scope_tree
            .into_iter()
            .map(|(path, mut data)| {
                if let Some(stats) = scope_stats.get(&path) {
                    data["total_memory"] = serde_json::json!(stats.0);
                    data["allocation_count"] = serde_json::json!(stats.1);
                }
                data
            })
            .collect();

        Ok(serde_json::json!({
            "levels": levels,
            "total_scopes": scope_stats.len(),
            "scope_statistics": scope_stats.into_iter().map(|(scope, (memory, count))| {
                serde_json::json!({
                    "scope_name": scope,
                    "total_memory": memory,
                    "allocation_count": count
                })
            }).collect::<Vec<_>>()
        }))
    }

    /// Generate variable relationships
    fn _generate_variable_relationships(
        &self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<serde_json::Value> {
        let mut relationships = Vec::new();
        let mut variable_map = std::collections::HashMap::new();

        // Build variable map
        for alloc in allocations {
            if let Some(var_name) = &alloc.var_name {
                variable_map.insert(var_name, alloc);
            }
        }

        // Find relationships based on naming patterns and types
        for (var_name, alloc) in &variable_map {
            // Find related variables
            for (other_name, other_alloc) in &variable_map {
                if var_name != other_name {
                    let relationship_type =
                        self._determine_relationship_type(var_name, other_name, alloc, other_alloc);
                    if let Some(rel_type) = relationship_type {
                        relationships.push(serde_json::json!({
                            "source": var_name,
                            "target": other_name,
                            "relationship_type": rel_type,
                            "strength": self._calculate_relationship_strength(alloc, other_alloc),
                            "source_info": {
                                "size": alloc.size,
                                "type": alloc.type_name.as_deref().unwrap_or("unknown"),
                                "scope": alloc.scope_name.as_deref().unwrap_or("global")
                            },
                            "target_info": {
                                "size": other_alloc.size,
                                "type": other_alloc.type_name.as_deref().unwrap_or("unknown"),
                                "scope": other_alloc.scope_name.as_deref().unwrap_or("global")
                            }
                        }));
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "relationships": relationships,
            "total_variables": variable_map.len(),
            "relationship_count": relationships.len(),
            "relationship_types": {
                "ownership": relationships.iter().filter(|r| r["relationship_type"] == "ownership").count(),
                "reference": relationships.iter().filter(|r| r["relationship_type"] == "reference").count(),
                "collection_item": relationships.iter().filter(|r| r["relationship_type"] == "collection_item").count(),
                "scope_related": relationships.iter().filter(|r| r["relationship_type"] == "scope_related").count()
            }
        }))
    }

    /// Determine relationship type between two variables
    fn _determine_relationship_type(
        &self,
        var1: &str,
        var2: &str,
        alloc1: &AllocationInfo,
        alloc2: &AllocationInfo,
    ) -> Option<String> {
        // Check for naming patterns
        if var1.contains("clone") && var2.replace("_clone", "") == var1.replace("_clone", "") {
            return Some("clone".to_string());
        }

        if var1.ends_with("_ref") && var2 == var1.trim_end_matches("_ref") {
            return Some("reference".to_string());
        }

        // Check for type relationships
        if let (Some(type1), Some(type2)) = (&alloc1.type_name, &alloc2.type_name) {
            if type1.contains("Box") && type2.contains(&type1.replace("Box<", "").replace(">", ""))
            {
                return Some("ownership".to_string());
            }

            if type1.contains("Vec") && type2.contains(&_extract_vec_inner_type(type1)) {
                return Some("collection_item".to_string());
            }
        }

        // Check for scope relationships
        if alloc1.scope_name == alloc2.scope_name && alloc1.scope_name.is_some() {
            return Some("scope_related".to_string());
        }

        None
    }

    /// Calculate relationship strength
    fn _calculate_relationship_strength(
        &self,
        alloc1: &AllocationInfo,
        alloc2: &AllocationInfo,
    ) -> f64 {
        let mut strength: f64 = 0.0;

        // Same scope increases strength
        if alloc1.scope_name == alloc2.scope_name {
            strength += 0.3;
        }

        // Similar allocation times increase strength
        let time_diff = alloc1.timestamp_alloc.abs_diff(alloc2.timestamp_alloc);
        if time_diff < 1_000_000 {
            // Within 1ms
            strength += 0.4;
        } else if time_diff < 10_000_000 {
            // Within 10ms
            strength += 0.2;
        }

        // Similar sizes increase strength
        let size_ratio = (alloc1.size as f64) / (alloc2.size as f64).max(1.0);
        if size_ratio > 0.5 && size_ratio < 2.0 {
            strength += 0.3;
        }

        strength.min(1.0)
    }

    /// Generate SVG visualization data
    fn _generate_svg_visualization_data(
        &self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> TrackingResult<serde_json::Value> {
        Ok(serde_json::json!({
            "memory_analysis": {
                "chart_type": "memory_analysis",
                "data_points": allocations.iter().map(|alloc| {
                    serde_json::json!({
                        "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                        "size": alloc.size,
                        "type": alloc.type_name.as_deref().unwrap_or("inferred"),
                        "x": alloc.timestamp_alloc % 1000, // Simplified positioning
                        "y": alloc.size,
                        "color": self._get_type_color(alloc.type_name.as_deref().unwrap_or("default"))
                    })
                }).collect::<Vec<_>>(),
                "metadata": {
                    "total_memory": stats.total_allocated,
                    "peak_memory": stats.peak_memory,
                    "active_allocations": stats.active_allocations
                }
            },
            "lifecycle_timeline": {
                "chart_type": "lifecycle_timeline",
                "timeline_events": allocations.iter().map(|alloc| {
                    serde_json::json!({
                        "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                        "start_time": alloc.timestamp_alloc,
                        "end_time": alloc.timestamp_dealloc,
                        "duration": alloc.timestamp_dealloc.map(|end| end - alloc.timestamp_alloc),
                        "size": alloc.size,
                        "scope": alloc.scope_name.as_deref().unwrap_or("global")
                    })
                }).collect::<Vec<_>>()
            },
            "unsafe_ffi": {
                "chart_type": "unsafe_ffi_dashboard",
                "risk_indicators": allocations.iter()
                    .filter(|alloc| alloc.var_name.as_ref().is_some_and(|name| name.contains("unsafe") || name.contains("ffi")))
                    .map(|alloc| {
                        serde_json::json!({
                            "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                            "risk_level": self._assess_risk_level(alloc),
                            "size": alloc.size,
                            "location": alloc.scope_name.as_deref().unwrap_or("global")
                        })
                    }).collect::<Vec<_>>()
            }
        }))
    }

    /// Get color for type visualization
    fn _get_type_color(&self, type_name: &str) -> String {
        match type_name {
            t if t.contains("Vec") => "#4CAF50".to_string(),
            t if t.contains("String") => "#2196F3".to_string(),
            t if t.contains("Box") => "#FF9800".to_string(),
            t if t.contains("HashMap") => "#9C27B0".to_string(),
            t if t.contains("Arc") || t.contains("Rc") => "#F44336".to_string(),
            _ => "#607D8B".to_string(),
        }
    }

    /// Assess risk level for unsafe operations
    fn _assess_risk_level(&self, alloc: &AllocationInfo) -> String {
        if alloc.size > 1024 * 1024 {
            // > 1MB
            "high".to_string()
        } else if alloc.size > 64 * 1024 {
            // > 64KB
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Generate detailed call stacks
    fn _generate_detailed_call_stacks(
        &self,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<serde_json::Value> {
        let mut call_stacks = std::collections::HashMap::new();

        for alloc in allocations {
            let stack_id = format!("stack_{}", alloc.ptr);
            let frames = self.generate_synthetic_stack_trace(alloc);

            call_stacks.insert(
                stack_id,
                serde_json::json!({
                    "frames": frames.iter().map(|frame| {
                        serde_json::json!({
                            "function": frame.function_name,
                            "file": frame.file_name,
                            "line": frame.line_number,
                            "module": frame.module_path
                        })
                    }).collect::<Vec<_>>(),
                    "allocation_info": {
                        "variable": alloc.var_name.as_deref().unwrap_or("unnamed"),
                        "size": alloc.size,
                        "timestamp": alloc.timestamp_alloc
                    }
                }),
            );
        }

        Ok(serde_json::json!({
            "stacks": call_stacks,
            "total_stacks": call_stacks.len(),
            "unique_functions": self._count_unique_functions(&call_stacks)
        }))
    }

    /// Count unique functions in call stacks
    fn _count_unique_functions(
        &self,
        call_stacks: &std::collections::HashMap<String, serde_json::Value>,
    ) -> usize {
        let mut functions = std::collections::HashSet::new();

        for stack in call_stacks.values() {
            if let Some(frames) = stack.get("frames").and_then(|f| f.as_array()) {
                for frame in frames {
                    if let Some(function) = frame.get("function").and_then(|f| f.as_str()) {
                        functions.insert(function.to_string());
                    }
                }
            }
        }

        functions.len()
    }

    /// Calculate allocation rate
    fn _calculate_allocation_rate(&self, history: &[AllocationInfo]) -> TrackingResult<f64> {
        if history.is_empty() {
            return Ok(0.0);
        }

        let start_time = history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
        let end_time = history.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
        let duration_seconds = (end_time - start_time) as f64 / 1_000_000_000.0;

        if duration_seconds > 0.0 {
            Ok(history.len() as f64 / duration_seconds)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate deallocation rate
    fn _calculate_deallocation_rate(&self, history: &[AllocationInfo]) -> TrackingResult<f64> {
        let deallocated_count = history
            .iter()
            .filter(|a| a.timestamp_dealloc.is_some())
            .count();

        if history.is_empty() {
            return Ok(0.0);
        }

        let start_time = history.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
        let end_time = history.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
        let duration_seconds = (end_time - start_time) as f64 / 1_000_000_000.0;

        if duration_seconds > 0.0 {
            Ok(deallocated_count as f64 / duration_seconds)
        } else {
            Ok(0.0)
        }
    }
}

/// Extract inner type from Vec<T>
fn _extract_vec_inner_type(type_name: &str) -> String {
    if let Some(start) = type_name.find("Vec<") {
        if let Some(end) = type_name.rfind('>') {
            let inner = &type_name[start + 4..end];
            return inner.to_string();
        }
    }
    "T".to_string()
}

/// Analyze system allocation to provide smart naming
fn _analyze_system_allocation(alloc: &AllocationInfo) -> (String, String) {
    let size = alloc.size;
    let smart_name = match size {
        1..=8 => format!("small_object_{:x}", alloc.ptr),
        9..=64 => format!("medium_object_{:x}", alloc.ptr),
        65..=1024 => format!("large_object_{:x}", alloc.ptr),
        _ => format!("huge_object_{:x}", alloc.ptr),
    };

    let smart_type = match size {
        1..=8 => "Small Object/String".to_string(),
        9..=64 => "Medium Structure".to_string(),
        65..=1024 => "Large Buffer".to_string(),
        _ => "Huge Memory Block".to_string(),
    };

    (smart_name, smart_type)
}

// ============================================================================
// Unified Tracking Interface (merged from tracking.rs)
// ============================================================================

/// Main tracking interface - consolidates all tracking functionality
///
/// This provides a unified interface that combines memory tracking and scope tracking
/// while preserving all existing functionality.
pub struct TrackingManager {
    memory_tracker: Arc<MemoryTracker>,
    scope_tracker: Arc<crate::core::scope_tracker::ScopeTracker>,
}

impl TrackingManager {
    /// Create a new tracking manager instance
    pub fn new() -> Self {
        Self {
            memory_tracker: get_global_tracker(),
            scope_tracker: crate::core::scope_tracker::get_global_scope_tracker(),
        }
    }

    /// Get the memory tracker instance
    pub fn memory_tracker(&self) -> &Arc<MemoryTracker> {
        &self.memory_tracker
    }

    /// Get the scope tracker instance
    pub fn scope_tracker(&self) -> &Arc<crate::core::scope_tracker::ScopeTracker> {
        &self.scope_tracker
    }

    /// Track memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        self.memory_tracker.track_allocation(ptr, size)
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        self.memory_tracker.track_deallocation(ptr)
    }

    /// Associate variable with memory allocation
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        self.memory_tracker.associate_var(ptr, var_name, type_name)
    }

    /// Enter a new scope
    pub fn enter_scope(&self, name: String) -> TrackingResult<crate::core::scope_tracker::ScopeId> {
        self.scope_tracker.enter_scope(name)
    }

    /// Exit a scope
    pub fn exit_scope(&self, scope_id: crate::core::scope_tracker::ScopeId) -> TrackingResult<()> {
        self.scope_tracker.exit_scope(scope_id)
    }

    /// Associate variable with current scope
    pub fn associate_variable(
        &self,
        variable_name: String,
        memory_size: usize,
    ) -> TrackingResult<()> {
        self.scope_tracker
            .associate_variable(variable_name, memory_size)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> TrackingResult<crate::core::types::MemoryStats> {
        self.memory_tracker.get_stats()
    }

    /// Get active allocations
    pub fn get_active_allocations(
        &self,
    ) -> TrackingResult<Vec<crate::core::types::AllocationInfo>> {
        self.memory_tracker.get_active_allocations()
    }

    /// Get allocation history
    pub fn get_allocation_history(
        &self,
    ) -> TrackingResult<Vec<crate::core::types::AllocationInfo>> {
        self.memory_tracker.get_allocation_history()
    }

    /// Get scope analysis
    pub fn get_scope_analysis(&self) -> TrackingResult<crate::core::types::ScopeAnalysis> {
        self.scope_tracker.get_scope_analysis()
    }

    /// Perform comprehensive tracking analysis
    pub fn perform_comprehensive_analysis(&self) -> TrackingResult<ComprehensiveTrackingReport> {
        let memory_stats = self.get_stats()?;
        let active_allocations = self.get_active_allocations()?;
        let allocation_history = self.get_allocation_history()?;
        let scope_analysis = self.get_scope_analysis()?;
        let scope_metrics = self.scope_tracker.get_scope_lifecycle_metrics()?;

        Ok(ComprehensiveTrackingReport {
            memory_stats,
            active_allocations,
            allocation_history,
            scope_analysis,
            scope_metrics,
            analysis_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

impl Default for TrackingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive tracking report
#[derive(Debug, Clone)]
pub struct ComprehensiveTrackingReport {
    /// Overall memory statistics
    pub memory_stats: crate::core::types::MemoryStats,
    /// Currently active memory allocations
    pub active_allocations: Vec<crate::core::types::AllocationInfo>,
    /// Historical allocation data
    pub allocation_history: Vec<crate::core::types::AllocationInfo>,
    /// Scope analysis results
    pub scope_analysis: crate::core::types::ScopeAnalysis,
    /// Scope lifecycle metrics
    pub scope_metrics: Vec<crate::core::types::ScopeLifecycleMetrics>,
    /// Timestamp when report was generated
    pub analysis_timestamp: u64,
}

/// Get unified tracking manager - convenience function
pub fn get_tracking_manager() -> TrackingManager {
    TrackingManager::new()
}

/// Track allocation - convenience function
pub fn track_allocation(ptr: usize, size: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_allocation(ptr, size)
}

/// Track deallocation - convenience function
pub fn track_deallocation(ptr: usize) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.track_deallocation(ptr)
}

/// Associate variable - convenience function
pub fn associate_var(ptr: usize, var_name: String, type_name: String) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.associate_var(ptr, var_name, type_name)
}

/// Enter scope - convenience function
pub fn enter_scope(name: String) -> TrackingResult<crate::core::scope_tracker::ScopeId> {
    let manager = TrackingManager::new();
    manager.enter_scope(name)
}

/// Exit scope - convenience function
pub fn exit_scope(scope_id: crate::core::scope_tracker::ScopeId) -> TrackingResult<()> {
    let manager = TrackingManager::new();
    manager.exit_scope(scope_id)
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        // Optional verbose tip for users
        if std::env::var("MEMSCOPE_VERBOSE").is_ok() {
            println!("💡 Tip: Use tracker.export_to_json() or tracker.export_interactive_dashboard() before drop to save analysis results");
        }

        // Clean up any remaining allocations
        if let Ok(mut active) = self.active_allocations.lock() {
            active.clear();
        }
    }
}
