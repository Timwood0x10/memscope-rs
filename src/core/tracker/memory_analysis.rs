//! Memory layout and container analysis functionality.
//!
//! This module contains methods for analyzing memory layouts, container structures,
//! and calculating efficiency metrics for different data types.

use super::memory_tracker::MemoryTracker;
use crate::core::types::{
    AllocatorStateInfo, CachePerformanceInfo, CapacityUtilization, CodeBloatLevel,
    ContainerAnalysis, ContainerEfficiencyMetrics, ContainerType, CpuUsageInfo, DispatchOverhead,
    DynamicTypeInfo, EnhancedFragmentationAnalysis, FieldLayoutInfo, GenericConstraint,
    GenericTypeInfo, LayoutEfficiency, MemoryLayoutInfo, MemoryPressureInfo, MonomorphizationInfo,
    PaddingAnalysis, PerformanceImpact, ReallocationPatterns, RuntimeStateInfo, TypeErasureInfo,
    TypeParameter, TypeRelationshipInfo, TypeUsageInfo, VTableInfo,
};

impl MemoryTracker {
    /// Analyze memory layout information with enhanced container analysis
    pub fn analyze_memory_layout(&self, type_name: &str, size: usize) -> Option<MemoryLayoutInfo> {
        // Infer memory layout based on type name and size
        let alignment = self.estimate_alignment(type_name, size);
        let field_layout = self.analyze_enhanced_field_layout(type_name, size, alignment);
        let padding_info = self.analyze_padding(&field_layout, size, alignment);
        let layout_efficiency = self.calculate_enhanced_layout_efficiency(
            &field_layout,
            &padding_info,
            size,
            type_name,
        );

        // Perform container-specific analysis
        let container_analysis = self.analyze_container_structure(type_name, size);

        Some(MemoryLayoutInfo {
            total_size: size,
            alignment,
            field_layout,
            padding_info,
            layout_efficiency,
            container_analysis: Some(container_analysis),
        })
    }

    /// Analyze memory fragmentation
    pub fn analyze_memory_fragmentation(&self) -> EnhancedFragmentationAnalysis {
        let stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());

        // Calculate basic fragmentation metrics
        let total_allocated = stats.total_allocated;
        let active_memory = stats.active_memory;
        let peak_memory = stats.peak_memory;

        // Calculate fragmentation ratio
        let fragmentation_ratio = if peak_memory > 0 {
            1.0 - (active_memory as f64 / peak_memory as f64)
        } else {
            0.0
        };

        // Determine severity level
        let severity = if fragmentation_ratio > 0.5 {
            crate::core::types::FragmentationSeverity::High
        } else if fragmentation_ratio > 0.3 {
            crate::core::types::FragmentationSeverity::Moderate
        } else {
            crate::core::types::FragmentationSeverity::Low
        };

        EnhancedFragmentationAnalysis {
            total_heap_size: total_allocated,
            used_heap_size: active_memory,
            free_heap_size: total_allocated.saturating_sub(active_memory),
            free_block_count: 0,                 // Placeholder
            free_block_distribution: Vec::new(), // Placeholder
            fragmentation_metrics: crate::core::types::FragmentationMetrics {
                external_fragmentation: fragmentation_ratio,
                internal_fragmentation: 0.0,  // Placeholder
                largest_free_block: 0,        // Placeholder
                average_free_block_size: 0.0, // Placeholder
                severity_level: severity,
            },
            fragmentation_causes: Vec::new(), // Placeholder
        }
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

    /// Enhanced field layout analysis with container-specific insights
    fn analyze_enhanced_field_layout(
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
                offset: 2 * std::mem::size_of::<usize>(),
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::size_of::<usize>(),
                is_padding: false,
            });

            // Add container-specific analysis
            self.analyze_vec_container_efficiency(type_name, total_size, &mut fields);
        } else if type_name == "String" {
            // String is essentially Vec<u8> with UTF-8 guarantees
            fields.push(FieldLayoutInfo {
                field_name: "ptr".to_string(),
                field_type: "*mut u8".to_string(),
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
                offset: 2 * std::mem::size_of::<usize>(),
                size: std::mem::size_of::<usize>(),
                alignment: std::mem::size_of::<usize>(),
                is_padding: false,
            });

            // Add string-specific analysis
            self.analyze_string_efficiency(total_size, &mut fields);
        } else {
            // Fallback to basic analysis for unknown types
            self.analyze_basic_field_layout(type_name, total_size, alignment, &mut fields);
        }

        fields
    }

    /// Analyze container structure for Vec, HashMap, Box, etc.
    fn analyze_container_structure(&self, type_name: &str, size: usize) -> ContainerAnalysis {
        let container_type = self.classify_container_type(type_name);
        let capacity_utilization = self.analyze_capacity_utilization(&container_type, size);
        let reallocation_patterns = self.detect_reallocation_patterns(&container_type, size);
        let efficiency_metrics = self.calculate_container_efficiency_metrics(
            &container_type,
            size,
            &capacity_utilization,
            &reallocation_patterns,
        );

        ContainerAnalysis {
            container_type,
            capacity_utilization,
            reallocation_patterns,
            efficiency_metrics,
        }
    }

    /// Calculate enhanced layout efficiency with container-specific insights
    fn calculate_enhanced_layout_efficiency(
        &self,
        _fields: &[FieldLayoutInfo],
        _padding_info: &PaddingAnalysis,
        size: usize,
        type_name: &str,
    ) -> LayoutEfficiency {
        // Calculate basic efficiency metrics
        let padding_overhead = self.calculate_padding_overhead(_fields, size);
        let memory_utilization = self.calculate_memory_utilization(size, padding_overhead);

        // Container-specific efficiency adjustments
        let container_efficiency = self.calculate_container_specific_efficiency(type_name, size);

        // Calculate overall efficiency score
        let overall_score = (memory_utilization + container_efficiency) / 2.0;

        LayoutEfficiency {
            memory_utilization,
            cache_friendliness: container_efficiency * 100.0, // Convert to 0-100 scale
            alignment_waste: padding_overhead as usize,
            optimization_potential: if overall_score < 0.5 {
                crate::core::types::OptimizationPotential::Major {
                    potential_savings: (padding_overhead as f64 * 0.8) as usize,
                    suggestions: self.generate_efficiency_recommendations(overall_score, type_name),
                }
            } else if overall_score < 0.8 {
                crate::core::types::OptimizationPotential::Moderate {
                    potential_savings: (padding_overhead as f64 * 0.5) as usize,
                    suggestions: self.generate_efficiency_recommendations(overall_score, type_name),
                }
            } else {
                crate::core::types::OptimizationPotential::None
            },
        }
    }

    /// Analyze padding information
    #[allow(unused)]
    fn analyze_padding(
        &self,
        fields: &[FieldLayoutInfo],
        total_size: usize,
        _alignment: usize,
    ) -> PaddingAnalysis {
        let mut total_padding = 0;
        let mut padding_locations = Vec::new();

        // Calculate padding between fields
        for i in 0..fields.len().saturating_sub(1) {
            let current_end = fields[i].offset + fields[i].size;
            let next_start = fields[i + 1].offset;
            if next_start > current_end {
                let padding_size = next_start - current_end;
                total_padding += padding_size;
                padding_locations.push((current_end, padding_size));
            }
        }

        // Calculate trailing padding
        if let Some(last_field) = fields.last() {
            let last_end = last_field.offset + last_field.size;
            if total_size > last_end {
                let trailing_padding = total_size - last_end;
                total_padding += trailing_padding;
                padding_locations.push((last_end, trailing_padding));
            }
        }

        PaddingAnalysis {
            total_padding_bytes: total_padding,
            padding_locations: padding_locations
                .into_iter()
                .map(|(offset, size)| crate::core::types::PaddingLocation {
                    start_offset: offset,
                    size,
                    reason: crate::core::types::PaddingReason::FieldAlignment,
                })
                .collect(),
            padding_ratio: if total_size > 0 {
                total_padding as f64 / total_size as f64
            } else {
                0.0
            },
            optimization_suggestions: vec![
                "Consider reordering fields by size (largest first)".to_string(),
                "Use #[repr(packed)] for space-critical structs".to_string(),
            ],
        }
    }

    // Private helper methods

    /// Classify container type based on type name
    fn classify_container_type(&self, type_name: &str) -> ContainerType {
        if type_name.contains("Vec<") {
            ContainerType::Vec {
                element_type: "T".to_string(), // Placeholder
                element_size: 8,               // Placeholder
            }
        } else if type_name.contains("HashMap<") {
            ContainerType::HashMap {
                key_type: "K".to_string(),   // Placeholder
                value_type: "V".to_string(), // Placeholder
                key_size: 8,                 // Placeholder
                value_size: 8,               // Placeholder
            }
        } else if type_name.contains("BTreeMap<") {
            ContainerType::BTreeMap {
                key_type: "K".to_string(),   // Placeholder
                value_type: "V".to_string(), // Placeholder
                key_size: 8,                 // Placeholder
                value_size: 8,               // Placeholder
            }
        } else if type_name.contains("Box<") {
            ContainerType::Box {
                boxed_type: "T".to_string(), // Placeholder
                boxed_size: 8,               // Placeholder
            }
        } else if type_name == "String" {
            ContainerType::String
        } else {
            ContainerType::Other {
                type_name: type_name.to_string(),
            }
        }
    }

    /// Analyze capacity utilization for containers
    fn analyze_capacity_utilization(
        &self,
        container_type: &ContainerType,
        size: usize,
    ) -> CapacityUtilization {
        // This is a simplified implementation
        // In a real implementation, you would track actual capacity vs length
        let estimated_utilization = match container_type {
            ContainerType::Vec { .. } | ContainerType::String => {
                // Assume 75% utilization for dynamic arrays
                0.75
            }
            ContainerType::HashMap { .. } => {
                // HashMap typically maintains ~75% load factor
                0.75
            }
            ContainerType::Box { .. } => {
                // Box is always 100% utilized
                1.0
            }
            _ => 0.5, // Default assumption
        };

        CapacityUtilization {
            current_capacity: (size as f64 / estimated_utilization) as usize,
            current_length: size,
            utilization_ratio: estimated_utilization,
            wasted_space: ((1.0 - estimated_utilization) * size as f64) as usize,
            efficiency_assessment: if estimated_utilization > 0.9 {
                crate::core::types::UtilizationEfficiency::Excellent
            } else if estimated_utilization > 0.7 {
                crate::core::types::UtilizationEfficiency::Good
            } else if estimated_utilization > 0.5 {
                crate::core::types::UtilizationEfficiency::Fair
            } else {
                crate::core::types::UtilizationEfficiency::Poor {
                    suggestion: "Consider using a more appropriate container size".to_string(),
                }
            },
        }
    }

    /// Detect reallocation patterns
    fn detect_reallocation_patterns(
        &self,
        container_type: &ContainerType,
        _size: usize,
    ) -> ReallocationPatterns {
        // This is a placeholder implementation
        // In a real implementation, you would track allocation history
        ReallocationPatterns {
            frequency_assessment: match container_type {
                ContainerType::Vec { .. } | ContainerType::String => {
                    crate::core::types::ReallocationFrequency::Moderate
                }
                ContainerType::HashMap { .. } => crate::core::types::ReallocationFrequency::Low,
                ContainerType::Box { .. } => crate::core::types::ReallocationFrequency::None,
                _ => crate::core::types::ReallocationFrequency::Low,
            },
            growth_pattern: crate::core::types::GrowthPattern::Exponential,
            estimated_reallocations: 2, // Placeholder
            optimization_suggestions: vec![
                "Consider pre-allocating capacity if size is known".to_string()
            ],
        }
    }

    /// Calculate container-specific efficiency metrics
    fn calculate_container_efficiency_metrics(
        &self,
        container_type: &ContainerType,
        _size: usize,
        capacity_utilization: &CapacityUtilization,
        _reallocation_patterns: &ReallocationPatterns,
    ) -> ContainerEfficiencyMetrics {
        let access_efficiency = match container_type {
            ContainerType::Vec { .. } | ContainerType::String => 0.95, // O(1) access
            ContainerType::HashMap { .. } => 0.85,                     // Average O(1) access
            ContainerType::BTreeMap { .. } => 0.75,                    // O(log n) access
            ContainerType::Box { .. } => 1.0,                          // Direct access
            _ => 0.5,
        };

        let memory_overhead = match container_type {
            ContainerType::Vec { .. } | ContainerType::String => 0.1, // 10% overhead
            ContainerType::HashMap { .. } => 0.25,                    // 25% overhead for hash table
            ContainerType::BTreeMap { .. } => 0.15, // 15% overhead for tree structure
            ContainerType::Box { .. } => 0.05,      // Minimal overhead
            _ => 0.2,
        };

        ContainerEfficiencyMetrics {
            memory_overhead,
            cache_efficiency: access_efficiency * 100.0,
            access_efficiency: crate::core::types::AccessEfficiency::Sequential, // Placeholder
            health_score: (access_efficiency
                + capacity_utilization.utilization_ratio
                + (1.0 - memory_overhead))
                / 3.0
                * 100.0,
        }
    }

    // Additional helper methods (placeholder implementations)

    fn analyze_vec_container_efficiency(
        &self,
        type_name: &str,
        total_size: usize,
        fields: &mut Vec<FieldLayoutInfo>,
    ) {
        // Extract element type from Vec<T>
        let element_type = if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                &type_name[start + 1..end]
            } else {
                "T"
            }
        } else {
            "T"
        };

        // Estimate element size
        let element_size = self.estimate_type_size(element_type);

        // Add efficiency analysis as a virtual field
        fields.push(FieldLayoutInfo {
            field_name: "efficiency_analysis".to_string(),
            field_type: format!("VecEfficiency<{}>", element_type),
            offset: 0, // Virtual field
            size: 0,   // Virtual field
            alignment: 1,
            is_padding: false,
        });

        // Add capacity utilization insight
        let estimated_capacity = if total_size > 24 {
            // Vec header is ~24 bytes
            (total_size - 24) / element_size.max(1)
        } else {
            0
        };

        if estimated_capacity > 0 {
            fields.push(FieldLayoutInfo {
                field_name: "capacity_analysis".to_string(),
                field_type: format!("EstimatedCapacity: {}", estimated_capacity),
                offset: 0, // Virtual field
                size: 0,   // Virtual field
                alignment: 1,
                is_padding: false,
            });
        }
    }

    fn analyze_string_efficiency(&self, total_size: usize, fields: &mut Vec<FieldLayoutInfo>) {
        // String is essentially Vec<u8>, so similar analysis
        let estimated_capacity = if total_size > 24 {
            total_size - 24 // Subtract Vec header size
        } else {
            0
        };

        fields.push(FieldLayoutInfo {
            field_name: "string_analysis".to_string(),
            field_type: format!("StringCapacity: {} bytes", estimated_capacity),
            offset: 0, // Virtual field
            size: 0,   // Virtual field
            alignment: 1,
            is_padding: false,
        });

        // Add UTF-8 efficiency note
        fields.push(FieldLayoutInfo {
            field_name: "encoding_info".to_string(),
            field_type: "UTF-8 encoded".to_string(),
            offset: 0, // Virtual field
            size: 0,   // Virtual field
            alignment: 1,
            is_padding: false,
        });
    }

    fn analyze_basic_field_layout(
        &self,
        type_name: &str,
        total_size: usize,
        alignment: usize,
        fields: &mut Vec<FieldLayoutInfo>,
    ) {
        // For unknown types, create a basic single-field layout
        fields.push(FieldLayoutInfo {
            field_name: "data".to_string(),
            field_type: type_name.to_string(),
            offset: 0,
            size: total_size,
            alignment,
            is_padding: false,
        });

        // Add alignment padding if necessary
        let aligned_size = (total_size + alignment - 1) & !(alignment - 1);
        if aligned_size > total_size {
            fields.push(FieldLayoutInfo {
                field_name: "tail_padding".to_string(),
                field_type: "padding".to_string(),
                offset: total_size,
                size: aligned_size - total_size,
                alignment: 1,
                is_padding: true,
            });
        }
    }

    fn calculate_padding_overhead(&self, fields: &[FieldLayoutInfo], size: usize) -> f64 {
        let total_padding: usize = fields
            .iter()
            .filter(|field| field.is_padding)
            .map(|field| field.size)
            .sum();

        if size > 0 {
            total_padding as f64 / size as f64
        } else {
            0.0
        }
    }

    fn calculate_memory_utilization(&self, size: usize, padding_overhead: f64) -> f64 {
        if size > 0 {
            1.0 - padding_overhead
        } else {
            0.0
        }
    }

    fn calculate_container_specific_efficiency(&self, type_name: &str, size: usize) -> f64 {
        match type_name {
            name if name.contains("Vec<") => {
                // Vec efficiency depends on capacity utilization
                if size > 24 {
                    0.85 // Good efficiency for dynamic arrays
                } else {
                    0.6 // Lower efficiency for small vectors
                }
            }
            name if name.contains("HashMap<") => {
                // HashMap efficiency depends on load factor
                if size > 48 {
                    0.75 // Reasonable efficiency for hash tables
                } else {
                    0.5 // Lower efficiency for small hash maps
                }
            }
            name if name.contains("Box<") => 1.0, // Box is always efficient
            "String" => {
                if size > 24 {
                    0.9 // Good efficiency for strings
                } else {
                    0.7 // Lower efficiency for small strings
                }
            }
            _ => 0.8, // Default efficiency
        }
    }

    fn generate_efficiency_recommendations(&self, score: f64, type_name: &str) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score < 0.5 {
            recommendations.push(
                "Consider significant restructuring for better memory efficiency".to_string(),
            );
        }

        if type_name.contains("Vec<") {
            if score < 0.8 {
                recommendations.push(
                    "Consider pre-allocating Vec capacity with Vec::with_capacity()".to_string(),
                );
                recommendations
                    .push("Use Vec::shrink_to_fit() to reduce unused capacity".to_string());
            }
        } else if type_name.contains("HashMap<") {
            if score < 0.7 {
                recommendations
                    .push("Consider pre-sizing HashMap with HashMap::with_capacity()".to_string());
                recommendations.push(
                    "Evaluate if BTreeMap might be more efficient for your use case".to_string(),
                );
            }
        } else if type_name == "String" {
            if score < 0.8 {
                recommendations.push(
                    "Consider using String::with_capacity() for known string sizes".to_string(),
                );
                recommendations.push("Use &str instead of String when possible".to_string());
            }
        }

        if score < 0.6 {
            recommendations
                .push("Consider using #[repr(packed)] for space-critical structs".to_string());
            recommendations.push(
                "Reorder struct fields by size (largest first) to minimize padding".to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Memory layout is already well optimized".to_string());
        }

        recommendations
    }

    /// Estimate type size based on type name
    pub(crate) fn estimate_type_size(&self, type_name: &str) -> usize {
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

    fn extract_base_type(&self, type_name: &str) -> String {
        if let Some(pos) = type_name.find('<') {
            type_name[..pos].to_string()
        } else {
            type_name.to_string()
        }
    }

    fn extract_trait_name(&self, type_name: &str) -> String {
        if type_name.contains("dyn ") {
            if let Some(start) = type_name.find("dyn ") {
                let after_dyn = &type_name[start + 4..];
                if let Some(end) = after_dyn.find('>') {
                    after_dyn[..end].trim().to_string()
                } else {
                    after_dyn.trim().to_string()
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
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

    /// Track type usage with enhanced statistics collection
    pub fn track_type_usage(&self, type_name: &str) -> Option<TypeUsageInfo> {
        // Get active allocations to analyze type usage patterns
        let active_allocations = match self.get_active_allocations() {
            Ok(allocations) => allocations,
            Err(_) => return None, // Skip if lock is contended
        };

        // Calculate type usage statistics from current allocations
        let mut type_count = 0u64;
        let mut total_size = 0usize;
        let mut allocation_timestamps = Vec::new();
        let mut allocation_sizes = Vec::new();

        // Analyze current allocations for this type
        for allocation in &active_allocations {
            if let Some(ref alloc_type) = allocation.type_name {
                if alloc_type == type_name {
                    type_count += 1;
                    total_size += allocation.size;
                    allocation_timestamps.push(allocation.timestamp_alloc);
                    allocation_sizes.push(allocation.size);
                }
            }
        }

        // If no allocations found for this type, return basic info
        if type_count == 0 {
            return Some(TypeUsageInfo {
                type_name: type_name.to_string(),
                total_usage_count: 1,
                usage_contexts: vec![],
                usage_timeline: vec![],
                hot_paths: vec![],
                performance_impact: self.calculate_basic_performance_impact(type_name),
            });
        }

        // Create usage timeline from allocation timestamps
        let usage_timeline = self.create_usage_timeline(&allocation_timestamps, &allocation_sizes);

        // Analyze usage contexts based on type characteristics
        let usage_contexts = self.analyze_usage_contexts(type_name, type_count, total_size);

        // Calculate performance impact based on usage patterns
        let performance_impact = self.calculate_type_performance_impact(
            type_name,
            type_count,
            total_size,
            &allocation_sizes,
        );

        Some(TypeUsageInfo {
            type_name: type_name.to_string(),
            total_usage_count: type_count,
            usage_contexts,
            usage_timeline,
            hot_paths: vec![], // Hot path analysis would require more complex profiling
            performance_impact,
        })
    }

    /// Generate optimization recommendations based on type characteristics
    fn generate_optimization_recommendations(
        &self,
        type_name: &str,
    ) -> Vec<crate::core::types::OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match type_name {
            "String" => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type:
                        crate::core::types::RecommendationType::DataStructureChange,
                    priority: crate::core::types::Priority::Medium,
                    description: "Consider using &str when possible to avoid allocations"
                        .to_string(),
                    expected_improvement: 15.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::Medium,
                    description: "Use String::with_capacity() when final size is known".to_string(),
                    expected_improvement: 20.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
            }
            name if name.starts_with("Vec<") => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::High,
                    description: "Use Vec::with_capacity() when size is predictable".to_string(),
                    expected_improvement: 25.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type:
                        crate::core::types::RecommendationType::DataStructureChange,
                    priority: crate::core::types::Priority::Low,
                    description: "Consider using SmallVec for small collections".to_string(),
                    expected_improvement: 10.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
                });
            }
            name if name.starts_with("HashMap<") => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::High,
                    description: "Use HashMap::with_capacity() for better performance".to_string(),
                    expected_improvement: 30.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type:
                        crate::core::types::RecommendationType::DataStructureChange,
                    priority: crate::core::types::Priority::Medium,
                    description:
                        "Consider using FxHashMap for better performance with integer keys"
                            .to_string(),
                    expected_improvement: 15.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
                });
            }
            name if name.starts_with("Box<") => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::Low,
                    description: "Box is efficient for heap allocation of single values"
                        .to_string(),
                    expected_improvement: 5.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
            }
            name if name.starts_with("Rc<") => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type:
                        crate::core::types::RecommendationType::DataStructureChange,
                    priority: crate::core::types::Priority::Medium,
                    description: "Consider Arc<> for thread-safe reference counting".to_string(),
                    expected_improvement: 0.0, // No performance improvement, just thread safety
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::High,
                    description: "Be aware of potential reference cycles".to_string(),
                    expected_improvement: 0.0, // Prevents memory leaks
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
                });
            }
            name if name.starts_with("Arc<") => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type:
                        crate::core::types::RecommendationType::DataStructureChange,
                    priority: crate::core::types::Priority::Low,
                    description: "Arc has atomic overhead - use Rc if single-threaded".to_string(),
                    expected_improvement: 10.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
                });
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::Medium,
                    description: "Consider weak references to break cycles".to_string(),
                    expected_improvement: 0.0, // Prevents memory leaks
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
                });
            }
            _ => {
                recommendations.push(crate::core::types::OptimizationRecommendation {
                    recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                    priority: crate::core::types::Priority::Low,
                    description: "Consider the memory layout and access patterns".to_string(),
                    expected_improvement: 5.0,
                    implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
                });
            }
        }

        recommendations
    }

    /// help functions
    /// Calculate performance impact based on actual usage patterns
    fn calculate_type_performance_impact(
        &self,
        type_name: &str,
        usage_count: u64,
        total_size: usize,
        allocation_sizes: &[usize],
    ) -> crate::core::types::TypePerformanceImpact {
        let avg_size = if usage_count > 0 {
            total_size as f64 / usage_count as f64
        } else {
            0.0
        };

        // Calculate size variance to detect allocation patterns
        let size_variance = if allocation_sizes.len() > 1 {
            let mean = avg_size;
            let variance = allocation_sizes
                .iter()
                .map(|&size| (size as f64 - mean).powi(2))
                .sum::<f64>()
                / allocation_sizes.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // Base scores on type characteristics
        let mut base_scores = self.calculate_basic_performance_impact(type_name);

        // Adjust scores based on usage patterns

        // High usage count might indicate performance bottlenecks
        if usage_count > 1000 {
            base_scores.performance_score *= 0.9; // Reduce performance score
            base_scores.cpu_efficiency_score *= 0.95;
        }

        // Large average size might indicate memory inefficiency
        if avg_size > 1024.0 * 1024.0 {
            // > 1MB
            base_scores.memory_efficiency_score *= 0.8;
            base_scores.cache_efficiency_score *= 0.7;
        }

        // High size variance might indicate inefficient allocation patterns
        if size_variance > avg_size * 0.5 {
            base_scores.memory_efficiency_score *= 0.9;
        }

        // Add specific recommendations based on analysis
        let mut recommendations = base_scores.optimization_recommendations;

        if usage_count > 100 && avg_size < 64.0 {
            recommendations.push(crate::core::types::OptimizationRecommendation {
                recommendation_type: crate::core::types::RecommendationType::MemoryPooling,
                priority: crate::core::types::Priority::High,
                description:
                    "Consider using object pooling for small, frequently allocated objects"
                        .to_string(),
                expected_improvement: 25.0,
                implementation_difficulty: crate::core::types::ImplementationDifficulty::Medium,
            });
        }

        if size_variance > avg_size {
            recommendations.push(crate::core::types::OptimizationRecommendation {
                recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                priority: crate::core::types::Priority::High,
                description:
                    "Consider pre-allocating with estimated capacity to reduce reallocations"
                        .to_string(),
                expected_improvement: 30.0,
                implementation_difficulty: crate::core::types::ImplementationDifficulty::Easy,
            });
        }

        if total_size > 10 * 1024 * 1024 {
            // > 10MB total
            recommendations.push(crate::core::types::OptimizationRecommendation {
                recommendation_type: crate::core::types::RecommendationType::MemoryLayout,
                priority: crate::core::types::Priority::Critical,
                description:
                    "Consider memory usage optimization - this type uses significant memory"
                        .to_string(),
                expected_improvement: 40.0,
                implementation_difficulty: crate::core::types::ImplementationDifficulty::Hard,
            });
        }

        crate::core::types::TypePerformanceImpact {
            performance_score: base_scores.performance_score,
            memory_efficiency_score: base_scores.memory_efficiency_score,
            cpu_efficiency_score: base_scores.cpu_efficiency_score,
            cache_efficiency_score: base_scores.cache_efficiency_score,
            optimization_recommendations: recommendations,
        }
    }

    /// Calculate basic performance impact for types with no allocation data
    fn calculate_basic_performance_impact(
        &self,
        type_name: &str,
    ) -> crate::core::types::TypePerformanceImpact {
        // Provide reasonable defaults based on type characteristics
        let (perf_score, mem_score, cpu_score, cache_score) = match type_name {
            "String" => (80.0, 85.0, 90.0, 75.0),
            name if name.starts_with("Vec<") => (85.0, 90.0, 85.0, 80.0),
            name if name.starts_with("HashMap<") => (75.0, 80.0, 70.0, 65.0),
            name if name.starts_with("Box<") => (90.0, 95.0, 95.0, 90.0),
            name if name.starts_with("Rc<") || name.starts_with("Arc<") => (85.0, 85.0, 80.0, 85.0),
            _ => (85.0, 90.0, 80.0, 75.0), // Default scores
        };

        crate::core::types::TypePerformanceImpact {
            performance_score: perf_score,
            memory_efficiency_score: mem_score,
            cpu_efficiency_score: cpu_score,
            cache_efficiency_score: cache_score,
            optimization_recommendations: self.generate_optimization_recommendations(type_name),
        }
    }

    /// Create usage timeline from allocation timestamps and sizes
    fn create_usage_timeline(
        &self,
        timestamps: &[u64],
        sizes: &[usize],
    ) -> Vec<crate::core::types::UsageTimePoint> {
        if timestamps.is_empty() {
            return vec![];
        }

        // Group allocations by time windows (e.g., 1 second intervals)
        let mut timeline = Vec::new();
        let window_size_ns = 1_000_000_000; // 1 second in nanoseconds

        if let (Some(&first_ts), Some(&last_ts)) = (timestamps.first(), timestamps.last()) {
            let mut current_window = first_ts;

            while current_window <= last_ts {
                let window_end = current_window + window_size_ns;

                // Count allocations and total memory in this window
                let mut usage_count = 0u32;
                let mut memory_usage = 0usize;

                for (i, &ts) in timestamps.iter().enumerate() {
                    if ts >= current_window && ts < window_end {
                        usage_count += 1;
                        memory_usage += sizes[i];
                    }
                }

                if usage_count > 0 {
                    timeline.push(crate::core::types::UsageTimePoint {
                        timestamp: current_window,
                        usage_count,
                        memory_usage,
                        performance_snapshot: crate::core::types::PerformanceSnapshot {
                            cpu_usage: 0.0, // Would need system profiling
                            memory_usage: memory_usage as f64,
                            cache_hit_rate: 0.95, // Estimated
                            throughput: usage_count as f64,
                        },
                    });
                }

                current_window = window_end;
            }
        }

        timeline
    }

    /// Analyze usage contexts based on type characteristics
    fn analyze_usage_contexts(
        &self,
        type_name: &str,
        usage_count: u64,
        total_size: usize,
    ) -> Vec<crate::core::types::UsageContext> {
        let mut contexts = Vec::new();

        // Infer context based on type name patterns
        let context_type = if type_name.starts_with("Vec<") {
            crate::core::types::ContextType::LocalVariable
        } else if type_name.starts_with("HashMap<") || type_name.starts_with("BTreeMap<") {
            crate::core::types::ContextType::LocalVariable
        } else if type_name.starts_with("Box<")
            || type_name.starts_with("Rc<")
            || type_name.starts_with("Arc<")
        {
            crate::core::types::ContextType::LocalVariable
        } else if type_name == "String" || type_name.starts_with("&str") {
            crate::core::types::ContextType::LocalVariable
        } else if type_name.contains("Future") || type_name.contains("async") {
            crate::core::types::ContextType::AsyncContext
        } else {
            crate::core::types::ContextType::LocalVariable
        };

        // Calculate performance metrics for this context
        let avg_size = if usage_count > 0 {
            total_size as f64 / usage_count as f64
        } else {
            0.0
        };
        let allocation_frequency = usage_count as f64; // Simplified

        contexts.push(crate::core::types::UsageContext {
            context_type,
            location: "unknown".to_string(), // Would need stack trace analysis
            frequency: usage_count as u32,
            performance_metrics: crate::core::types::ContextPerformanceMetrics {
                avg_execution_time_ns: avg_size * 10.0, // Rough estimate
                allocation_frequency,
                cache_miss_rate: self.estimate_cache_miss_rate(type_name, avg_size),
                branch_misprediction_rate: 0.05, // Default estimate
            },
        });

        contexts
    }

    /// Estimate cache miss rate based on type and size characteristics
    fn estimate_cache_miss_rate(&self, type_name: &str, avg_size: f64) -> f64 {
        // Cache line size is typically 64 bytes
        let cache_line_size = 64.0;

        if avg_size <= cache_line_size {
            0.05 // Low miss rate for small objects
        } else if avg_size <= cache_line_size * 4.0 {
            0.15 // Medium miss rate
        } else if type_name.starts_with("Vec<") || type_name.starts_with("HashMap<") {
            0.25 // Higher miss rate for large containers
        } else {
            0.35 // High miss rate for large objects
        }
    }

    /// Analyze stack allocation information
    pub fn analyze_stack_allocation(
        &self,
        type_name: &str,
        ptr: usize,
    ) -> Option<crate::core::types::StackAllocationInfo> {
        // Heuristic: if pointer is in typical stack range, consider it stack allocation
        let stack_start = 0x7fff_0000_0000; // Typical stack start on x64
        let stack_end = 0x7fff_ffff_ffff; // Typical stack end on x64

        if ptr >= stack_start && ptr <= stack_end {
            Some(crate::core::types::StackAllocationInfo {
                frame_id: (ptr >> 12) & 0xffff, // Use high bits as frame ID
                var_name: "stack_var".to_string(),
                stack_offset: (ptr as isize) - (stack_start as isize),
                size: self.estimate_type_size(type_name),
                function_name: "unknown_function".to_string(),
                stack_depth: self.estimate_stack_depth(ptr),
                scope_info: crate::core::types::StackScopeInfo {
                    scope_type: crate::core::types::ScopeType::Function,
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
}
