//! Advanced memory analysis functions for fragmentation, system libraries, and concurrency

use crate::types::*;
use std::collections::HashMap;

/// Analyze memory fragmentation patterns
pub fn analyze_fragmentation(allocations: &[AllocationInfo]) -> FragmentationAnalysis {
    let mut analysis = FragmentationAnalysis::default();
    
    if allocations.is_empty() {
        return analysis;
    }
    
    // Sort allocations by address for fragmentation analysis
    let mut sorted_allocs: Vec<_> = allocations.iter().collect();
    sorted_allocs.sort_by_key(|a| a.ptr);
    
    // Calculate basic metrics
    analysis.smallest_allocation = allocations.iter().map(|a| a.size).min().unwrap_or(0);
    analysis.largest_free_block = calculate_largest_free_block(&sorted_allocs);
    
    // Analyze memory holes
    analysis.memory_holes = find_memory_holes(&sorted_allocs);
    analysis.total_fragments = analysis.memory_holes.len();
    
    // Calculate size distribution
    analysis.size_distribution = calculate_size_distribution(allocations);
    
    // Calculate fragmentation ratios
    let total_memory: usize = allocations.iter().map(|a| a.size).sum();
    let total_gaps: usize = analysis.memory_holes.iter().map(|h| h.size).sum();
    
    if total_memory > 0 {
        analysis.external_fragmentation = total_gaps as f64 / total_memory as f64;
        analysis.fragmentation_ratio = analysis.external_fragmentation;
    }
    
    // Calculate alignment waste
    analysis.alignment_waste = calculate_alignment_waste(allocations);
    
    // Internal fragmentation (estimated based on allocation patterns)
    analysis.internal_fragmentation = estimate_internal_fragmentation(allocations);
    
    analysis
}

/// Analyze system library usage patterns
pub fn analyze_system_libraries(allocations: &[AllocationInfo]) -> SystemLibraryStats {
    let mut stats = SystemLibraryStats::default();
    
    for alloc in allocations {
        classify_library_allocation(alloc, &mut stats);
    }
    
    // Calculate averages and finalize stats
    finalize_library_stats(&mut stats);
    
    stats
}

/// Analyze concurrency safety patterns
pub fn analyze_concurrency_safety(allocations: &[AllocationInfo]) -> ConcurrencyAnalysis {
    let mut analysis = ConcurrencyAnalysis::default();
    
    // Group allocations by thread
    let mut thread_allocations: HashMap<String, Vec<&AllocationInfo>> = HashMap::new();
    for alloc in allocations {
        thread_allocations.entry(alloc.thread_id.clone()).or_default().push(alloc);
    }
    
    // Analyze each allocation for concurrency patterns
    for alloc in allocations {
        analyze_allocation_concurrency(alloc, &mut analysis);
    }
    
    // Detect concurrency patterns
    analysis.concurrency_patterns = detect_concurrency_patterns(&thread_allocations);
    
    // Assess data race risks
    analysis.data_race_risks = assess_data_race_risks(allocations);
    
    // Calculate risk scores
    analysis.deadlock_risk_score = calculate_deadlock_risk(&analysis);
    analysis.lock_contention_risk = assess_lock_contention_risk(&analysis);
    
    analysis
}

// Helper functions for fragmentation analysis

fn calculate_largest_free_block(sorted_allocs: &[&AllocationInfo]) -> usize {
    if sorted_allocs.len() < 2 {
        return 0;
    }
    
    let mut largest_gap = 0;
    for i in 0..sorted_allocs.len() - 1 {
        let current_end = sorted_allocs[i].ptr + sorted_allocs[i].size;
        let next_start = sorted_allocs[i + 1].ptr;
        
        if next_start > current_end {
            let gap = next_start - current_end;
            largest_gap = largest_gap.max(gap);
        }
    }
    
    largest_gap
}

fn find_memory_holes(sorted_allocs: &[&AllocationInfo]) -> Vec<MemoryHole> {
    let mut holes = Vec::new();
    
    for i in 0..sorted_allocs.len().saturating_sub(1) {
        let current_end = sorted_allocs[i].ptr + sorted_allocs[i].size;
        let next_start = sorted_allocs[i + 1].ptr;
        
        if next_start > current_end {
            let hole_size = next_start - current_end;
            
            // Classify hole type
            let hole_type = if hole_size % 8 == 0 {
                "alignment_padding".to_string()
            } else if hole_size > 1024 {
                "freed_space".to_string()
            } else {
                "gap".to_string()
            };
            
            holes.push(MemoryHole {
                start_address: current_end,
                size: hole_size,
                duration_ms: 0, // Would need timeline data to calculate
                hole_type,
                cause: "Memory layout fragmentation".to_string(),
            });
        }
    }
    
    holes
}

fn calculate_size_distribution(allocations: &[AllocationInfo]) -> HashMap<String, usize> {
    let mut distribution = HashMap::new();
    
    for alloc in allocations {
        let category = match alloc.size {
            0..=64 => "tiny (0-64B)",
            65..=256 => "small (65-256B)",
            257..=1024 => "medium (257B-1KB)",
            1025..=4096 => "large (1-4KB)",
            4097..=16384 => "very_large (4-16KB)",
            16385..=65536 => "huge (16-64KB)",
            _ => "massive (>64KB)",
        };
        
        *distribution.entry(category.to_string()).or_insert(0) += 1;
    }
    
    distribution
}

fn calculate_alignment_waste(allocations: &[AllocationInfo]) -> usize {
    allocations.iter()
        .map(|alloc| {
            // Estimate alignment waste (assuming 8-byte alignment)
            let aligned_size = (alloc.size + 7) & !7;
            aligned_size - alloc.size
        })
        .sum()
}

fn estimate_internal_fragmentation(allocations: &[AllocationInfo]) -> f64 {
    // Estimate based on allocation patterns
    let total_allocated: usize = allocations.iter().map(|a| a.size).sum();
    let estimated_used: usize = allocations.iter()
        .map(|a| {
            // Assume 90% efficiency for most allocations
            (a.size as f64 * 0.9) as usize
        })
        .sum();
    
    if total_allocated > 0 {
        1.0 - (estimated_used as f64 / total_allocated as f64)
    } else {
        0.0
    }
}

// Helper functions for system library analysis

fn classify_library_allocation(alloc: &AllocationInfo, stats: &mut SystemLibraryStats) {
    // Use type name and backtrace to classify
    let type_name = alloc.type_name.as_deref().unwrap_or("");
    let var_name = alloc.var_name.as_deref().unwrap_or("");
    
    // Check for standard collections
    if type_name.contains("HashMap") || type_name.contains("BTreeMap") || 
       type_name.contains("HashSet") || type_name.contains("Vec") {
        update_library_usage(&mut stats.std_collections, alloc, "collections");
    }
    // Check for async runtime
    else if type_name.contains("tokio") || var_name.contains("async") || var_name.contains("tokio") {
        update_library_usage(&mut stats.async_runtime, alloc, "tokio_runtime");
    }
    // Check for network I/O
    else if var_name.contains("net_") || var_name.contains("tcp_") || var_name.contains("udp_") {
        update_library_usage(&mut stats.network_io, alloc, "network");
    }
    // Check for file system
    else if var_name.contains("fs_") || var_name.contains("file_") {
        update_library_usage(&mut stats.file_system, alloc, "filesystem");
    }
    // Check for serialization
    else if var_name.contains("json_") || var_name.contains("serde") {
        update_library_usage(&mut stats.serialization, alloc, "json_serde");
    }
    // Check for regex
    else if var_name.contains("regex") {
        update_library_usage(&mut stats.regex_engine, alloc, "regex");
    }
    // Check for crypto
    else if var_name.contains("crypto") || var_name.contains("hash") {
        update_library_usage(&mut stats.crypto_security, alloc, "crypto");
    }
    // Default to unknown system
    else if alloc.var_name.is_none() || var_name.starts_with("[SYS]") {
        update_library_usage(&mut stats.unknown_system, alloc, "unknown");
    }
}

fn update_library_usage(usage: &mut LibraryUsage, alloc: &AllocationInfo, category: &str) {
    usage.allocation_count += 1;
    usage.total_bytes += alloc.size;
    usage.peak_bytes = usage.peak_bytes.max(alloc.size);
    
    *usage.categories.entry(category.to_string()).or_insert(0) += alloc.size;
    
    if let Some(var_name) = &alloc.var_name {
        if !usage.hotspot_functions.contains(var_name) && usage.hotspot_functions.len() < 10 {
            usage.hotspot_functions.push(var_name.clone());
        }
    }
}

fn finalize_library_stats(stats: &mut SystemLibraryStats) {
    // Calculate averages for each library
    let libraries = [
        &mut stats.std_collections,
        &mut stats.async_runtime,
        &mut stats.network_io,
        &mut stats.file_system,
        &mut stats.serialization,
        &mut stats.regex_engine,
        &mut stats.crypto_security,
        &mut stats.database,
        &mut stats.graphics_ui,
        &mut stats.http_stack,
        &mut stats.compression,
        &mut stats.logging,
        &mut stats.unknown_system,
    ];
    
    for library in libraries {
        if library.allocation_count > 0 {
            library.average_size = library.total_bytes as f64 / library.allocation_count as f64;
        }
    }
}

// Helper functions for concurrency analysis

fn analyze_allocation_concurrency(alloc: &AllocationInfo, analysis: &mut ConcurrencyAnalysis) {
    let type_name = alloc.type_name.as_deref().unwrap_or("");
    let var_name = alloc.var_name.as_deref().unwrap_or("");
    
    // Check for thread-safe types
    if type_name.contains("Arc") || var_name.contains("arc_") {
        analysis.arc_shared += alloc.size;
        analysis.shared_memory_bytes += alloc.size;
        analysis.thread_safety_allocations += 1;
    } else if type_name.contains("Mutex") || var_name.contains("mutex_") {
        analysis.mutex_protected += alloc.size;
        analysis.thread_safety_allocations += 1;
    } else if type_name.contains("Rc") || var_name.contains("rc_") {
        analysis.rc_shared += alloc.size;
    } else if var_name.contains("channel_") {
        analysis.channel_buffers += alloc.size;
        analysis.thread_safety_allocations += 1;
    } else if var_name.contains("thread_local") {
        analysis.thread_local_storage += alloc.size;
    } else if var_name.contains("atomic_") {
        analysis.atomic_operations += alloc.size;
        analysis.thread_safety_allocations += 1;
    }
}

fn detect_concurrency_patterns(thread_allocations: &HashMap<String, Vec<&AllocationInfo>>) -> Vec<ConcurrencyPattern> {
    let mut patterns = Vec::new();
    
    // Producer-consumer pattern detection
    if thread_allocations.len() >= 2 {
        let total_channel_memory: usize = thread_allocations.values()
            .flat_map(|allocs| allocs.iter())
            .filter(|alloc| alloc.var_name.as_deref().unwrap_or("").contains("channel_"))
            .map(|alloc| alloc.size)
            .sum();
            
        if total_channel_memory > 0 {
            patterns.push(ConcurrencyPattern {
                pattern_type: "producer_consumer".to_string(),
                thread_count: thread_allocations.len(),
                memory_usage: total_channel_memory,
                safety_level: "safe".to_string(),
                performance_impact: if total_channel_memory > 1024 * 1024 { "high" } else { "medium" }.to_string(),
                locations: vec!["Multiple threads with channel communication".to_string()],
            });
        }
    }
    
    // Shared state pattern detection
    let shared_memory: usize = thread_allocations.values()
        .flat_map(|allocs| allocs.iter())
        .filter(|alloc| {
            let var_name = alloc.var_name.as_deref().unwrap_or("");
            var_name.contains("arc_") || var_name.contains("mutex_")
        })
        .map(|alloc| alloc.size)
        .sum();
        
    if shared_memory > 0 {
        patterns.push(ConcurrencyPattern {
            pattern_type: "shared_state".to_string(),
            thread_count: thread_allocations.len(),
            memory_usage: shared_memory,
            safety_level: "safe".to_string(),
            performance_impact: if shared_memory > 512 * 1024 { "high" } else { "low" }.to_string(),
            locations: vec!["Shared memory via Arc/Mutex".to_string()],
        });
    }
    
    patterns
}

fn assess_data_race_risks(allocations: &[AllocationInfo]) -> Vec<DataRaceRisk> {
    let mut risks = Vec::new();
    
    // Look for potentially unsafe patterns
    for alloc in allocations {
        let var_name = alloc.var_name.as_deref().unwrap_or("");
        
        // Rc in multi-threaded context is risky
        if alloc.type_name.as_deref().unwrap_or("").contains("Rc") && 
           !alloc.thread_id.contains("main") {
            risks.push(DataRaceRisk {
                risk_type: "rc_multithreaded".to_string(),
                memory_address: alloc.ptr,
                severity: "high".to_string(),
                description: "Rc used in multi-threaded context".to_string(),
                suggested_fix: "Use Arc instead of Rc for thread safety".to_string(),
            });
        }
        
        // Large shared memory without proper synchronization
        if alloc.size > 1024 * 1024 && var_name.contains("shared") && 
           !var_name.contains("mutex") && !var_name.contains("arc") {
            risks.push(DataRaceRisk {
                risk_type: "large_unsynchronized".to_string(),
                memory_address: alloc.ptr,
                severity: "medium".to_string(),
                description: "Large memory allocation without synchronization".to_string(),
                suggested_fix: "Consider using Arc<Mutex<T>> for thread-safe access".to_string(),
            });
        }
    }
    
    risks
}

fn calculate_deadlock_risk(analysis: &ConcurrencyAnalysis) -> f64 {
    let mut risk_score: f64 = 0.0;
    
    // Multiple mutexes increase deadlock risk
    if analysis.mutex_protected > 0 {
        risk_score += 0.3;
    }
    
    // Complex concurrency patterns increase risk
    if analysis.concurrency_patterns.len() > 2 {
        risk_score += 0.2;
    }
    
    // High contention increases risk
    if analysis.lock_contention_risk == "high" || analysis.lock_contention_risk == "critical" {
        risk_score += 0.4;
    }
    
    risk_score.min(1.0f64)
}

fn assess_lock_contention_risk(analysis: &ConcurrencyAnalysis) -> String {
    let total_sync_memory = analysis.mutex_protected + analysis.arc_shared;
    let sync_ratio = if analysis.shared_memory_bytes > 0 {
        total_sync_memory as f64 / analysis.shared_memory_bytes as f64
    } else {
        0.0
    };
    
    if sync_ratio > 0.8 {
        "critical".to_string()
    } else if sync_ratio > 0.6 {
        "high".to_string()
    } else if sync_ratio > 0.3 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}