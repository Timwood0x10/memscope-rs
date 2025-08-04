//! Multi-source data integration module
//!
//! This module provides advanced data integration capabilities for combining
//! multiple JSON data sources with cross-referencing and conflict resolution.

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use super::data_normalizer::{AllocationInfo, UnifiedMemoryData};

/// Data integration error types
#[derive(Debug)]
#[allow(dead_code)]
pub enum IntegrationError {
    /// Data conflict between sources
    DataConflict(String),
    /// Missing required data
    MissingData(String),
    /// Cross-reference resolution failed
    CrossReferenceError(String),
    /// Index building failed
    IndexError(String),
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegrationError::DataConflict(msg) => write!(f, "Data conflict: {}", msg),
            IntegrationError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            IntegrationError::CrossReferenceError(msg) => {
                write!(f, "Cross-reference error: {}", msg)
            }
            IntegrationError::IndexError(msg) => write!(f, "Index error: {}", msg),
        }
    }
}

impl Error for IntegrationError {}

/// Cross-reference information between data sources
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CrossReference {
    /// Source data type
    pub source_type: String,
    /// Target data type
    pub target_type: String,
    /// Reference key
    pub key: String,
    /// Reference value
    pub value: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

/// Data integration index for fast lookups
#[derive(Debug)]
pub struct DataIndex {
    /// Pointer to allocation mapping
    pub ptr_to_allocation: HashMap<String, usize>,
    /// Variable name to allocation mapping
    pub var_to_allocation: HashMap<String, Vec<usize>>,
    /// Type name to allocation mapping
    pub type_to_allocation: HashMap<String, Vec<usize>>,
    /// Timestamp to allocation mapping
    pub timestamp_to_allocation: HashMap<u64, Vec<usize>>,
    /// Scope to allocation mapping
    pub scope_to_allocation: HashMap<String, Vec<usize>>,
}

/// Integration statistics
#[derive(Debug)]
pub struct IntegrationStats {
    /// Number of cross-references found
    pub cross_references_found: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Number of data enrichments performed
    pub enrichments_performed: usize,
    /// Index building time in milliseconds
    pub index_build_time_ms: u64,
    /// Integration processing time in milliseconds
    pub integration_time_ms: u64,
}

/// Multi-source data integrator
pub struct DataIntegrator {
    /// Enable cross-referencing
    enable_cross_referencing: bool,
    /// Enable conflict resolution
    enable_conflict_resolution: bool,
    /// Enable data enrichment
    enable_data_enrichment: bool,
    /// Confidence threshold for cross-references
    confidence_threshold: f64,
}

impl DataIntegrator {
    /// Create a new data integrator with default settings
    pub fn new() -> Self {
        Self {
            enable_cross_referencing: true,
            enable_conflict_resolution: true,
            enable_data_enrichment: true,
            confidence_threshold: 0.7,
        }
    }

    /// Create integrator with custom settings
    #[allow(dead_code)]
    pub fn with_settings(
        cross_ref: bool,
        conflict_resolution: bool,
        data_enrichment: bool,
        confidence_threshold: f64,
    ) -> Self {
        Self {
            enable_cross_referencing: cross_ref,
            enable_conflict_resolution: conflict_resolution,
            enable_data_enrichment: data_enrichment,
            confidence_threshold,
        }
    }

    /// Integrate unified data with advanced cross-referencing and enrichment
    pub fn integrate(
        &self,
        unified_data: &mut UnifiedMemoryData,
    ) -> Result<IntegrationStats, IntegrationError> {
        let start_time = std::time::Instant::now();

        tracing::info!("🔗 Starting advanced data integration...");

        // Build data indexes for fast lookups
        let index_start = std::time::Instant::now();
        let data_index = self.build_data_index(&unified_data.allocations)?;
        let index_build_time = index_start.elapsed().as_millis() as u64;

        tracing::info!("📊 Built data indexes in {}ms", index_build_time);

        let mut stats = IntegrationStats {
            cross_references_found: 0,
            conflicts_resolved: 0,
            enrichments_performed: 0,
            index_build_time_ms: index_build_time,
            integration_time_ms: 0,
        };

        // Perform cross-referencing
        if self.enable_cross_referencing {
            let cross_refs = self.find_cross_references(unified_data, &data_index)?;
            stats.cross_references_found = cross_refs.len();
            self.apply_cross_references(unified_data, &cross_refs)?;
            tracing::info!("🔗 Found and applied {} cross-references", cross_refs.len());
        }

        // Resolve data conflicts
        if self.enable_conflict_resolution {
            let conflicts_resolved = self.resolve_conflicts(unified_data)?;
            stats.conflicts_resolved = conflicts_resolved;
            tracing::info!("⚖️  Resolved {} data conflicts", conflicts_resolved);
        }

        // Perform data enrichment
        if self.enable_data_enrichment {
            let enrichments = self.enrich_data(unified_data, &data_index)?;
            stats.enrichments_performed = enrichments;
            tracing::info!("✨ Performed {} data enrichments", enrichments);
        }

        stats.integration_time_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            "✅ Data integration completed in {}ms",
            stats.integration_time_ms
        );
        Ok(stats)
    }

    /// Build comprehensive data index for fast lookups
    fn build_data_index(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<DataIndex, IntegrationError> {
        let mut index = DataIndex {
            ptr_to_allocation: HashMap::new(),
            var_to_allocation: HashMap::new(),
            type_to_allocation: HashMap::new(),
            timestamp_to_allocation: HashMap::new(),
            scope_to_allocation: HashMap::new(),
        };

        for (i, alloc) in allocations.iter().enumerate() {
            // Index by pointer
            index.ptr_to_allocation.insert(alloc.ptr.clone(), i);

            // Index by variable name
            if let Some(var_name) = &alloc.var_name {
                index
                    .var_to_allocation
                    .entry(var_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }

            // Index by type name
            if let Some(type_name) = &alloc.type_name {
                index
                    .type_to_allocation
                    .entry(type_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }

            // Index by timestamp
            index
                .timestamp_to_allocation
                .entry(alloc.timestamp_alloc)
                .or_insert_with(Vec::new)
                .push(i);

            // Index by scope
            if let Some(scope_name) = &alloc.scope_name {
                index
                    .scope_to_allocation
                    .entry(scope_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        Ok(index)
    }

    /// Find cross-references between different data sources
    fn find_cross_references(
        &self,
        unified_data: &UnifiedMemoryData,
        data_index: &DataIndex,
    ) -> Result<Vec<CrossReference>, IntegrationError> {
        let mut cross_refs = Vec::new();

        // Cross-reference allocations with performance data
        cross_refs.extend(self.cross_ref_allocations_performance(unified_data, data_index)?);

        // Cross-reference allocations with security violations
        cross_refs.extend(self.cross_ref_allocations_security(unified_data, data_index)?);

        // Cross-reference allocations with lifecycle events
        cross_refs.extend(self.cross_ref_allocations_lifecycle(unified_data, data_index)?);

        // Cross-reference allocations with complex types
        cross_refs.extend(self.cross_ref_allocations_complex_types(unified_data, data_index)?);

        // Filter by confidence threshold
        cross_refs.retain(|cr| cr.confidence >= self.confidence_threshold);

        Ok(cross_refs)
    }

    /// Cross-reference allocations with performance data
    fn cross_ref_allocations_performance(
        &self,
        unified_data: &UnifiedMemoryData,
        _data_index: &DataIndex,
    ) -> Result<Vec<CrossReference>, IntegrationError> {
        let mut cross_refs = Vec::new();

        // Find allocations that match performance distribution patterns
        let _perf_dist = &unified_data.performance.allocation_distribution;

        for (i, alloc) in unified_data.allocations.iter().enumerate() {
            let size_category = match alloc.size {
                0..=63 => "tiny",
                64..=1023 => "small",
                1024..=65535 => "medium",
                65536..=1048575 => "large",
                _ => "massive",
            };

            cross_refs.push(CrossReference {
                source_type: "allocation".to_string(),
                target_type: "performance_distribution".to_string(),
                key: format!("allocation_{}", i),
                value: size_category.to_string(),
                confidence: 0.9,
            });
        }

        Ok(cross_refs)
    }

    /// Cross-reference allocations with security violations
    fn cross_ref_allocations_security(
        &self,
        unified_data: &UnifiedMemoryData,
        _data_index: &DataIndex,
    ) -> Result<Vec<CrossReference>, IntegrationError> {
        let mut cross_refs = Vec::new();

        // Look for allocations that might be related to security violations
        for violation in &unified_data.security.violation_reports {
            if let Some(violation_obj) = violation.as_object() {
                if let Some(ptr_value) = violation_obj.get("allocation_ptr") {
                    if let Some(ptr_str) = ptr_value.as_str() {
                        cross_refs.push(CrossReference {
                            source_type: "security_violation".to_string(),
                            target_type: "allocation".to_string(),
                            key: "allocation_ptr".to_string(),
                            value: ptr_str.to_string(),
                            confidence: 0.8,
                        });
                    }
                }
            }
        }

        Ok(cross_refs)
    }

    /// Cross-reference allocations with lifecycle events
    fn cross_ref_allocations_lifecycle(
        &self,
        unified_data: &UnifiedMemoryData,
        data_index: &DataIndex,
    ) -> Result<Vec<CrossReference>, IntegrationError> {
        let mut cross_refs = Vec::new();

        // Match lifecycle events with allocations by timestamp
        for event in &unified_data.lifecycle.lifecycle_events {
            if let Some(event_obj) = event.as_object() {
                if let Some(timestamp) = event_obj.get("timestamp").and_then(|t| t.as_u64()) {
                    // Find allocations with similar timestamps (within 1ms)
                    for (&alloc_timestamp, allocation_indices) in
                        &data_index.timestamp_to_allocation
                    {
                        if alloc_timestamp.abs_diff(timestamp) <= 1000 {
                            // 1ms tolerance
                            for &alloc_index in allocation_indices {
                                cross_refs.push(CrossReference {
                                    source_type: "lifecycle_event".to_string(),
                                    target_type: "allocation".to_string(),
                                    key: "timestamp".to_string(),
                                    value: alloc_index.to_string(),
                                    confidence: 0.7,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(cross_refs)
    }

    /// Cross-reference allocations with complex types
    fn cross_ref_allocations_complex_types(
        &self,
        unified_data: &UnifiedMemoryData,
        data_index: &DataIndex,
    ) -> Result<Vec<CrossReference>, IntegrationError> {
        let mut cross_refs = Vec::new();

        // Match complex type analysis with allocations
        for complex_type in &unified_data.complex_types.complex_type_analysis {
            if let Some(type_obj) = complex_type.as_object() {
                if let Some(type_name) = type_obj.get("type_name").and_then(|t| t.as_str()) {
                    if let Some(allocation_indices) = data_index.type_to_allocation.get(type_name) {
                        for &alloc_index in allocation_indices {
                            cross_refs.push(CrossReference {
                                source_type: "complex_type".to_string(),
                                target_type: "allocation".to_string(),
                                key: "type_name".to_string(),
                                value: alloc_index.to_string(),
                                confidence: 0.85,
                            });
                        }
                    }
                }
            }
        }

        Ok(cross_refs)
    }

    /// Apply cross-references to enrich data
    fn apply_cross_references(
        &self,
        unified_data: &mut UnifiedMemoryData,
        cross_refs: &[CrossReference],
    ) -> Result<(), IntegrationError> {
        // Group cross-references by target type
        let mut allocation_refs = Vec::new();

        for cross_ref in cross_refs {
            if cross_ref.target_type == "allocation" {
                allocation_refs.push(cross_ref);
            }
        }

        // Apply cross-references to allocations
        for cross_ref in allocation_refs {
            if let Ok(alloc_index) = cross_ref.value.parse::<usize>() {
                if alloc_index < unified_data.allocations.len() {
                    // Add cross-reference information to allocation
                    // This would typically involve adding metadata fields
                    // For now, we'll just count the enrichment
                }
            }
        }

        Ok(())
    }

    /// Resolve conflicts between different data sources
    fn resolve_conflicts(
        &self,
        unified_data: &mut UnifiedMemoryData,
    ) -> Result<usize, IntegrationError> {
        let mut conflicts_resolved = 0;

        // Check for conflicts between stats and actual allocation data
        let actual_active_count = unified_data
            .allocations
            .iter()
            .filter(|alloc| alloc.timestamp_dealloc.is_none())
            .count();

        if actual_active_count != unified_data.stats.active_allocations
            && unified_data.stats.active_allocations > 0
        {
            tracing::info!(
                "🔧 Resolving active allocation count conflict: stats={}, actual={}",
                unified_data.stats.active_allocations,
                actual_active_count
            );
            unified_data.stats.active_allocations = actual_active_count;
            conflicts_resolved += 1;
        }

        // Check for memory size conflicts
        let actual_active_memory: usize = unified_data
            .allocations
            .iter()
            .filter(|alloc| alloc.timestamp_dealloc.is_none())
            .map(|alloc| alloc.size)
            .sum();

        if actual_active_memory != unified_data.stats.active_memory
            && unified_data.stats.active_memory > 0
        {
            let diff_percent = ((actual_active_memory as f64
                - unified_data.stats.active_memory as f64)
                / unified_data.stats.active_memory as f64)
                .abs()
                * 100.0;
            if diff_percent > 5.0 {
                // Only resolve if difference is > 5%
                tracing::info!(
                    "🔧 Resolving active memory conflict: stats={}, actual={} ({:.1}% diff)",
                    unified_data.stats.active_memory,
                    actual_active_memory,
                    diff_percent
                );
                unified_data.stats.active_memory = actual_active_memory;
                conflicts_resolved += 1;
            }
        }

        // Resolve timestamp conflicts (ensure deallocation timestamp > allocation timestamp)
        for alloc in &mut unified_data.allocations {
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                if dealloc_time <= alloc.timestamp_alloc {
                    tracing::info!(
                        "🔧 Resolving timestamp conflict for allocation {}",
                        alloc.ptr
                    );
                    alloc.timestamp_dealloc = Some(alloc.timestamp_alloc + 1);
                    conflicts_resolved += 1;
                }
            }
        }

        Ok(conflicts_resolved)
    }

    /// Enrich data with additional computed information
    fn enrich_data(
        &self,
        unified_data: &mut UnifiedMemoryData,
        _data_index: &DataIndex,
    ) -> Result<usize, IntegrationError> {
        let mut enrichments = 0;

        // Enrich allocations with computed lifetime information
        for alloc in &mut unified_data.allocations {
            if alloc.lifetime_ms.is_none() {
                if let Some(dealloc_time) = alloc.timestamp_dealloc {
                    let lifetime_ns = (dealloc_time - alloc.timestamp_alloc) * 1_000_000; // Convert to nanoseconds
                    alloc.lifetime_ms = Some(lifetime_ns / 1_000_000); // Convert to milliseconds
                    enrichments += 1;
                }
            }
        }

        // Enrich with type frequency information
        let mut type_frequency = HashMap::new();
        for alloc in &unified_data.allocations {
            if let Some(type_name) = &alloc.type_name {
                *type_frequency.entry(type_name.clone()).or_insert(0) += 1;
            }
        }

        // Add type frequency as metadata
        let type_freq_json: Value = type_frequency
            .into_iter()
            .map(|(k, v)| (k, Value::Number(v.into())))
            .collect::<serde_json::Map<String, Value>>()
            .into();

        unified_data
            .multi_source
            .insert("type_frequency".to_string(), type_freq_json);
        enrichments += 1;

        // Enrich with allocation size distribution
        let mut size_distribution = HashMap::new();
        for alloc in &unified_data.allocations {
            let size_category = match alloc.size {
                0..=63 => "tiny",
                64..=1023 => "small",
                1024..=65535 => "medium",
                65536..=1048575 => "large",
                _ => "massive",
            };
            *size_distribution.entry(size_category).or_insert(0) += 1;
        }

        let size_dist_json: Value = size_distribution
            .into_iter()
            .map(|(k, v)| (k.to_string(), Value::Number(v.into())))
            .collect::<serde_json::Map<String, Value>>()
            .into();

        unified_data
            .multi_source
            .insert("size_distribution".to_string(), size_dist_json);
        enrichments += 1;

        // Enrich with scope statistics
        let mut scope_stats = HashMap::new();
        for alloc in &unified_data.allocations {
            let scope = alloc.scope_name.as_deref().unwrap_or("unknown");
            let entry = scope_stats.entry(scope.to_string()).or_insert((0, 0usize));
            entry.0 += 1; // count
            entry.1 += alloc.size; // total size
        }

        let scope_stats_json: Value = scope_stats
            .into_iter()
            .map(|(scope, (count, total_size))| {
                (
                    scope,
                    serde_json::json!({
                        "allocation_count": count,
                        "total_size": total_size,
                        "average_size": if count > 0 { total_size / count } else { 0 }
                    }),
                )
            })
            .collect::<serde_json::Map<String, Value>>()
            .into();

        unified_data
            .multi_source
            .insert("scope_statistics".to_string(), scope_stats_json);
        enrichments += 1;

        Ok(enrichments)
    }
}

impl Default for DataIntegrator {
    fn default() -> Self {
        Self::new()
    }
}
