use crate::analysis::unknown::types::*;
use crate::capture::types::{AllocationInfo, ImplementationDifficulty};
use std::collections::HashMap;

pub struct UnknownMemoryAnalyzer {
    pub known_system_regions: HashMap<(usize, usize), SystemRegionInfo>,
    pub library_mappings: HashMap<String, LibraryMappingInfo>,
    pub thread_memory_ranges: HashMap<u64, Vec<(usize, usize)>>,
}

impl Default for UnknownMemoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl UnknownMemoryAnalyzer {
    pub fn new() -> Self {
        Self {
            known_system_regions: HashMap::new(),
            library_mappings: HashMap::new(),
            thread_memory_ranges: HashMap::new(),
        }
    }

    pub fn analyze_unknown_regions(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> UnknownMemoryRegionAnalysis {
        let total_memory: usize = allocations.iter().map(|a| a.size).sum();
        let unknown_allocations = self.identify_unknown_allocations(allocations);
        let total_unknown: usize = unknown_allocations.iter().map(|a| a.size).sum();
        let unknown_percentage = (total_unknown as f64 / total_memory as f64) * 100.0;
        let unknown_categories = self.categorize_unknown_regions(&unknown_allocations);
        let potential_causes = self.identify_potential_causes(&unknown_allocations);
        let reduction_strategies = self.generate_reduction_strategies(&unknown_categories);

        UnknownMemoryRegionAnalysis {
            total_unknown_bytes: total_unknown,
            unknown_percentage,
            unknown_categories,
            potential_causes,
            reduction_strategies,
        }
    }

    fn identify_unknown_allocations<'a>(
        &self,
        allocations: &'a [AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_unknown_allocation(alloc))
            .collect()
    }

    fn is_unknown_allocation(&self, allocation: &AllocationInfo) -> bool {
        if self.is_in_stack_region(allocation.ptr) || self.is_in_heap_region(allocation.ptr) {
            return false;
        }

        if self.is_known_system_region(allocation.ptr) {
            return false;
        }

        true
    }

    fn categorize_unknown_regions(
        &self,
        unknown_allocations: &[&AllocationInfo],
    ) -> Vec<UnknownMemoryCategory> {
        let mut categories = Vec::new();

        let mmap_allocations = self.identify_memory_mapped_regions(unknown_allocations);
        if !mmap_allocations.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::MemoryMappedRegions,
                description: "Memory-mapped files, shared memory, and anonymous mappings"
                    .to_string(),
                estimated_size: mmap_allocations.iter().map(|a| a.size).sum(),
                confidence_level: 0.8,
                examples: self.generate_examples(&mmap_allocations, "Memory mapping"),
            });
        }

        let tls_allocations = self.identify_thread_local_storage(unknown_allocations);
        if !tls_allocations.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::ThreadLocalStorage,
                description: "Thread-local storage and thread control blocks".to_string(),
                estimated_size: tls_allocations.iter().map(|a| a.size).sum(),
                confidence_level: 0.7,
                examples: self.generate_examples(&tls_allocations, "Thread-local storage"),
            });
        }

        let lib_allocations = self.identify_library_regions(unknown_allocations);
        if !lib_allocations.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::DynamicLibraryRegions,
                description: "Code and data sections of dynamically loaded libraries".to_string(),
                estimated_size: lib_allocations.iter().map(|a| a.size).sum(),
                confidence_level: 0.9,
                examples: self.generate_examples(&lib_allocations, "Dynamic library"),
            });
        }

        let ffi_allocations = self.identify_ffi_allocations(unknown_allocations);
        if !ffi_allocations.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::ExternalLibraryAllocations,
                description: "Memory allocated by external C/C++ libraries through FFI".to_string(),
                estimated_size: ffi_allocations.iter().map(|a| a.size).sum(),
                confidence_level: 0.6,
                examples: self.generate_examples(&ffi_allocations, "FFI allocation"),
            });
        }

        let system_allocations = self.identify_system_regions(unknown_allocations);
        if !system_allocations.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::SystemReservedRegions,
                description: "Kernel buffers, driver memory, and system caches".to_string(),
                estimated_size: system_allocations.iter().map(|a| a.size).sum(),
                confidence_level: 0.5,
                examples: self.generate_examples(&system_allocations, "System region"),
            });
        }

        let pre_tracking = self.identify_pre_tracking_allocations(unknown_allocations);
        if !pre_tracking.is_empty() {
            categories.push(UnknownMemoryCategory {
                category_type: UnknownRegionType::PreTrackingAllocations,
                description: "Memory allocated before tracking was initialized".to_string(),
                estimated_size: pre_tracking.iter().map(|a| a.size).sum(),
                confidence_level: 0.9,
                examples: self.generate_examples(&pre_tracking, "Pre-tracking"),
            });
        }

        categories
    }

    fn identify_potential_causes(
        &self,
        unknown_allocations: &[&AllocationInfo],
    ) -> Vec<UnknownMemoryCause> {
        let mut causes = Vec::new();

        for allocation in unknown_allocations {
            if self.is_likely_ffi_allocation(allocation) {
                causes.push(UnknownMemoryCause::ForeignFunctionInterface {
                    library_name: self
                        .guess_library_name(allocation)
                        .unwrap_or_else(|| "unknown".to_string()),
                    function_name: None,
                });
            }
        }

        if self.has_memory_mapping_pattern(unknown_allocations) {
            causes.push(UnknownMemoryCause::MemoryMapping {
                mapping_type: MappingType::AnonymousMapping,
                file_path: None,
            });
        }

        if self.has_threading_pattern(unknown_allocations) {
            causes.push(UnknownMemoryCause::ThreadingMemory {
                thread_id: None,
                memory_type: ThreadMemoryType::ThreadStack,
            });
        }

        causes.push(UnknownMemoryCause::InstrumentationGaps {
            gap_type: InstrumentationGapType::EarlyBootstrap,
            description: "Memory allocated during early program initialization".to_string(),
        });

        causes
    }

    fn generate_reduction_strategies(
        &self,
        _categories: &[UnknownMemoryCategory],
    ) -> Vec<UnknownRegionReductionStrategy> {
        vec![
            UnknownRegionReductionStrategy {
                strategy_type: ReductionStrategyType::EnhancedInstrumentation,
                description: "Implement more comprehensive memory tracking hooks".to_string(),
                implementation_steps: vec![
                    "Hook into mmap/munmap system calls".to_string(),
                    "Intercept malloc/free in all loaded libraries".to_string(),
                    "Track thread creation and destruction".to_string(),
                    "Monitor dynamic library loading".to_string(),
                ],
                expected_improvement: 60.0,
                implementation_difficulty: ImplementationDifficulty::Hard,
            },
            UnknownRegionReductionStrategy {
                strategy_type: ReductionStrategyType::FfiCallInterception,
                description: "Intercept and track FFI calls to external libraries".to_string(),
                implementation_steps: vec![
                    "Wrap all extern function calls".to_string(),
                    "Track memory allocations in C libraries".to_string(),
                    "Monitor shared library symbol resolution".to_string(),
                ],
                expected_improvement: 25.0,
                implementation_difficulty: ImplementationDifficulty::Medium,
            },
            UnknownRegionReductionStrategy {
                strategy_type: ReductionStrategyType::MemoryMappingTracking,
                description: "Track memory mapping operations comprehensively".to_string(),
                implementation_steps: vec![
                    "Monitor /proc/self/maps changes".to_string(),
                    "Track mmap/mprotect/munmap calls".to_string(),
                    "Analyze virtual memory layout".to_string(),
                ],
                expected_improvement: 20.0,
                implementation_difficulty: ImplementationDifficulty::Medium,
            },
        ]
    }

    fn is_in_stack_region(&self, _ptr: usize) -> bool {
        false
    }

    fn is_in_heap_region(&self, _ptr: usize) -> bool {
        false
    }

    fn is_known_system_region(&self, ptr: usize) -> bool {
        self.known_system_regions
            .iter()
            .any(|((start, end), _)| ptr >= *start && ptr < *end)
    }

    fn identify_memory_mapped_regions<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_mmap_allocation(alloc))
            .copied()
            .collect()
    }

    fn identify_thread_local_storage<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_tls_allocation(alloc))
            .copied()
            .collect()
    }

    fn identify_library_regions<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_library_allocation(alloc))
            .copied()
            .collect()
    }

    fn identify_ffi_allocations<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_ffi_allocation(alloc))
            .copied()
            .collect()
    }

    fn identify_system_regions<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_system_allocation(alloc))
            .copied()
            .collect()
    }

    fn identify_pre_tracking_allocations<'a>(
        &self,
        allocations: &[&'a AllocationInfo],
    ) -> Vec<&'a AllocationInfo> {
        allocations
            .iter()
            .filter(|alloc| self.is_likely_pre_tracking_allocation(alloc))
            .copied()
            .collect()
    }

    #[allow(clippy::manual_is_multiple_of)]
    fn is_likely_mmap_allocation(&self, allocation: &AllocationInfo) -> bool {
        allocation.size >= 4096 && allocation.ptr % 4096 == 0
    }

    fn is_likely_tls_allocation(&self, allocation: &AllocationInfo) -> bool {
        allocation.size < 1024 && self.is_in_thread_range(allocation.ptr)
    }

    fn is_likely_library_allocation(&self, allocation: &AllocationInfo) -> bool {
        self.library_mappings
            .values()
            .any(|lib| lib.contains_address(allocation.ptr))
    }

    fn is_likely_ffi_allocation(&self, allocation: &AllocationInfo) -> bool {
        allocation.type_name.is_none() && allocation.var_name.is_none()
    }

    fn is_likely_system_allocation(&self, allocation: &AllocationInfo) -> bool {
        allocation.ptr < 0x1000 || allocation.ptr > 0x7fff_0000_0000
    }

    fn is_likely_pre_tracking_allocation(&self, allocation: &AllocationInfo) -> bool {
        allocation.timestamp_alloc < 1000
    }

    fn is_in_thread_range(&self, ptr: usize) -> bool {
        self.thread_memory_ranges.values().any(|ranges| {
            ranges
                .iter()
                .any(|(start, end)| ptr >= *start && ptr < *end)
        })
    }

    fn has_memory_mapping_pattern(&self, allocations: &[&AllocationInfo]) -> bool {
        allocations
            .iter()
            .any(|alloc| self.is_likely_mmap_allocation(alloc))
    }

    fn has_threading_pattern(&self, allocations: &[&AllocationInfo]) -> bool {
        allocations
            .iter()
            .any(|alloc| self.is_likely_tls_allocation(alloc))
    }

    fn guess_library_name(&self, allocation: &AllocationInfo) -> Option<String> {
        for (name, info) in &self.library_mappings {
            if info.contains_address(allocation.ptr) {
                return Some(name.to_string());
            }
        }
        None
    }

    fn generate_examples(
        &self,
        allocations: &[&AllocationInfo],
        origin: &str,
    ) -> Vec<UnknownMemoryExample> {
        allocations
            .iter()
            .take(3)
            .map(|alloc| UnknownMemoryExample {
                address_range: (alloc.ptr, alloc.ptr + alloc.size),
                size: alloc.size,
                suspected_origin: origin.to_string(),
                access_pattern: MemoryAccessPattern::Unknown,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_allocation(ptr: usize, size: usize) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: std::thread::current().id(),
            thread_id_u64: 1,
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            module_path: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
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
            drop_chain_analysis: None,
            stack_ptr: None,
        }
    }

    /// Objective: Verify UnknownMemoryAnalyzer creation with default
    /// Invariants: All internal maps should be empty
    #[test]
    fn test_analyzer_new() {
        let analyzer = UnknownMemoryAnalyzer::new();
        assert!(
            analyzer.known_system_regions.is_empty(),
            "System regions should be empty"
        );
        assert!(
            analyzer.library_mappings.is_empty(),
            "Library mappings should be empty"
        );
        assert!(
            analyzer.thread_memory_ranges.is_empty(),
            "Thread ranges should be empty"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should be equivalent to new()
    #[test]
    fn test_analyzer_default() {
        let analyzer = UnknownMemoryAnalyzer::default();
        assert!(
            analyzer.known_system_regions.is_empty(),
            "Default should create empty analyzer"
        );
    }

    /// Objective: Verify analyze_unknown_regions with empty allocations
    /// Invariants: Should return zero unknown bytes, percentage may be NaN for empty input
    #[test]
    fn test_analyze_empty_allocations() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let result = analyzer.analyze_unknown_regions(&[]);
        assert_eq!(
            result.total_unknown_bytes, 0,
            "Empty input should have zero unknown bytes"
        );
        assert!(
            result.unknown_percentage.is_nan() || result.unknown_percentage == 0.0,
            "Empty input should have NaN or 0% unknown"
        );
    }

    /// Objective: Verify analyze_unknown_regions with single allocation
    /// Invariants: Should categorize unknown allocation correctly
    #[test]
    fn test_analyze_single_allocation() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x10000, 1024)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert!(
            result.total_unknown_bytes > 0,
            "Should detect unknown bytes"
        );
        assert!(
            result.unknown_percentage > 0.0,
            "Should have unknown percentage"
        );
    }

    /// Objective: Verify mmap allocation detection
    /// Invariants: Page-aligned allocations >= 4096 should be detected as mmap
    #[test]
    fn test_mmap_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x1000, 4096)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        let mmap_category = result
            .unknown_categories
            .iter()
            .find(|c| matches!(c.category_type, UnknownRegionType::MemoryMappedRegions));
        assert!(
            mmap_category.is_some(),
            "Should detect memory-mapped region"
        );
    }

    /// Objective: Verify FFI allocation detection
    /// Invariants: Allocations without type_name should be detected as FFI
    #[test]
    fn test_ffi_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x10000, 512)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        let ffi_category = result.unknown_categories.iter().find(|c| {
            matches!(
                c.category_type,
                UnknownRegionType::ExternalLibraryAllocations
            )
        });
        assert!(ffi_category.is_some(), "Should detect FFI allocation");
    }

    /// Objective: Verify system allocation detection
    /// Invariants: Low addresses should be detected as system regions
    #[test]
    fn test_system_region_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x100, 256)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        let system_category = result
            .unknown_categories
            .iter()
            .find(|c| matches!(c.category_type, UnknownRegionType::SystemReservedRegions));
        assert!(system_category.is_some(), "Should detect system region");
    }

    /// Objective: Verify pre-tracking allocation detection
    /// Invariants: Allocations with timestamp < 1000 should be pre-tracking
    #[test]
    fn test_pre_tracking_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let mut alloc = create_test_allocation(0x10000, 1024);
        alloc.timestamp_alloc = 500;
        let result = analyzer.analyze_unknown_regions(&[alloc]);
        let pre_tracking = result
            .unknown_categories
            .iter()
            .find(|c| matches!(c.category_type, UnknownRegionType::PreTrackingAllocations));
        assert!(
            pre_tracking.is_some(),
            "Should detect pre-tracking allocation"
        );
    }

    /// Objective: Verify known system region exclusion
    /// Invariants: Known regions should not be marked as unknown
    #[test]
    fn test_known_system_region_exclusion() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        analyzer.known_system_regions.insert(
            (0x10000, 0x20000),
            SystemRegionInfo {
                region_type: "test".to_string(),
                description: "test region".to_string(),
                read_only: false,
            },
        );
        let allocations = vec![create_test_allocation(0x15000, 1024)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert_eq!(
            result.total_unknown_bytes, 0,
            "Known region should not be unknown"
        );
    }

    /// Objective: Verify library mapping detection
    /// Invariants: Allocations in library range should be detected
    #[test]
    fn test_library_mapping_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        analyzer.library_mappings.insert(
            "test_lib".to_string(),
            LibraryMappingInfo {
                start_address: 0x10000,
                end_address: 0x20000,
                permissions: "r-x".to_string(),
                file_path: "/lib/test.so".to_string(),
            },
        );
        let allocations = vec![create_test_allocation(0x15000, 1024)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        let lib_category = result
            .unknown_categories
            .iter()
            .find(|c| matches!(c.category_type, UnknownRegionType::DynamicLibraryRegions));
        assert!(lib_category.is_some(), "Should detect library allocation");
    }

    /// Objective: Verify thread memory range detection
    /// Invariants: Allocations in thread range should be detected as TLS
    #[test]
    fn test_thread_range_detection() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        analyzer
            .thread_memory_ranges
            .insert(1, vec![(0x10000, 0x20000)]);
        let allocations = vec![create_test_allocation(0x15000, 512)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        let tls_category = result
            .unknown_categories
            .iter()
            .find(|c| matches!(c.category_type, UnknownRegionType::ThreadLocalStorage));
        assert!(tls_category.is_some(), "Should detect TLS allocation");
    }

    /// Objective: Verify reduction strategies are generated
    /// Invariants: Should always return strategies
    #[test]
    fn test_reduction_strategies() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let result = analyzer.analyze_unknown_regions(&[]);
        assert!(
            !result.reduction_strategies.is_empty(),
            "Should generate strategies"
        );
        assert!(
            result.reduction_strategies.iter().any(|s| matches!(
                s.strategy_type,
                ReductionStrategyType::EnhancedInstrumentation
            )),
            "Should include enhanced instrumentation strategy"
        );
    }

    /// Objective: Verify potential causes are identified
    /// Invariants: Should identify causes for unknown allocations
    #[test]
    fn test_potential_causes() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x10000, 512)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert!(
            !result.potential_causes.is_empty(),
            "Should identify potential causes"
        );
    }

    /// Objective: Verify multiple allocations handling
    /// Invariants: Should correctly sum unknown bytes
    #[test]
    fn test_multiple_allocations() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![
            create_test_allocation(0x10000, 1024),
            create_test_allocation(0x20000, 2048),
            create_test_allocation(0x30000, 4096),
        ];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert_eq!(
            result.total_unknown_bytes, 7168,
            "Should sum all unknown bytes"
        );
    }

    /// Objective: Verify LibraryMappingInfo contains_address
    /// Invariants: Should correctly check address bounds
    #[test]
    fn test_library_mapping_contains() {
        let mapping = LibraryMappingInfo {
            start_address: 0x1000,
            end_address: 0x2000,
            permissions: "r-x".to_string(),
            file_path: "/test.so".to_string(),
        };
        assert!(
            mapping.contains_address(0x1000),
            "Start address should be contained"
        );
        assert!(
            mapping.contains_address(0x1500),
            "Middle address should be contained"
        );
        assert!(
            !mapping.contains_address(0x2000),
            "End address should not be contained"
        );
        assert!(
            !mapping.contains_address(0x500),
            "Address before range should not be contained"
        );
    }

    /// Objective: Verify large allocation handling
    /// Invariants: Should handle large sizes without overflow
    #[test]
    fn test_large_allocation() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![create_test_allocation(0x100000, usize::MAX / 4)];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert!(
            result.total_unknown_bytes > 0,
            "Should handle large allocation"
        );
    }

    /// Objective: Verify unknown percentage calculation
    /// Invariants: Percentage should be between 0 and 100
    #[test]
    fn test_percentage_bounds() {
        let mut analyzer = UnknownMemoryAnalyzer::new();
        let allocations = vec![
            create_test_allocation(0x10000, 1024),
            create_test_allocation(0x20000, 2048),
        ];
        let result = analyzer.analyze_unknown_regions(&allocations);
        assert!(
            result.unknown_percentage >= 0.0 && result.unknown_percentage <= 100.0,
            "Percentage should be between 0 and 100"
        );
    }
}
