//! Integration Validator - Validates all enhanced components work together
//!
//! This module provides comprehensive validation of all enhanced components
//! to ensure they work correctly together and comply with requirement.md

use crate::core::types::TrackingResult;
use crate::core::enhanced_call_stack_normalizer::{
    get_global_enhanced_call_stack_normalizer,
};
use crate::core::edge_case_handler::{
    get_global_edge_case_handler, EdgeCaseType, EdgeCaseSeverity
};
use crate::core::comprehensive_data_deduplicator::{
    get_global_data_deduplicator,
};
use crate::analysis::enhanced_ffi_function_resolver::{
    get_global_enhanced_ffi_resolver,
};
use crate::analysis::unsafe_ffi_tracker::StackFrame;
use std::collections::HashMap;

/// Integration validation results
#[derive(Debug)]
pub struct ValidationResults {
    pub call_stack_normalizer_ok: bool,
    pub edge_case_handler_ok: bool,
    pub data_deduplicator_ok: bool,
    pub ffi_resolver_ok: bool,
    pub integration_ok: bool,
    pub performance_ok: bool,
    pub memory_usage_ok: bool,
}

/// Comprehensive integration validator
pub struct IntegrationValidator;

impl IntegrationValidator {
    /// Run complete validation suite
    pub fn validate_all() -> TrackingResult<ValidationResults> {
        tracing::info!("🧪 Starting comprehensive integration validation");
        
        let mut results = ValidationResults {
            call_stack_normalizer_ok: false,
            edge_case_handler_ok: false,
            data_deduplicator_ok: false,
            ffi_resolver_ok: false,
            integration_ok: false,
            performance_ok: false,
            memory_usage_ok: false,
        };

        // Test individual components
        results.call_stack_normalizer_ok = Self::test_call_stack_normalizer()?;
        results.edge_case_handler_ok = Self::test_edge_case_handler()?;
        results.data_deduplicator_ok = Self::test_data_deduplicator()?;
        results.ffi_resolver_ok = Self::test_ffi_resolver()?;

        // Test integration
        results.integration_ok = Self::test_component_integration()?;
        
        // Test performance
        results.performance_ok = Self::test_performance()?;
        
        // Test memory usage
        results.memory_usage_ok = Self::test_memory_usage()?;

        let all_ok = results.call_stack_normalizer_ok
            && results.edge_case_handler_ok
            && results.data_deduplicator_ok
            && results.ffi_resolver_ok
            && results.integration_ok
            && results.performance_ok
            && results.memory_usage_ok;

        if all_ok {
            tracing::info!("✅ All integration tests passed");
        } else {
            tracing::error!("❌ Some integration tests failed: {:?}", results);
        }

        Ok(results)
    }

    /// Test enhanced call stack normalizer
    fn test_call_stack_normalizer() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing enhanced call stack normalizer");
        
        let normalizer = get_global_enhanced_call_stack_normalizer();
        
        // Test basic normalization
        let frames = vec![
            StackFrame {
                function_name: "test_function".to_string(),
                file_name: Some("test.rs".to_string()),
                line_number: Some(42),
                is_unsafe: false,
            }
        ];

        let id1 = normalizer.normalize_call_stack(&frames)?;
        let id2 = normalizer.normalize_call_stack(&frames)?;
        
        // Should deduplicate
        if id1 != id2 {
            tracing::error!("Call stack normalization failed: different IDs for same stack");
            return Ok(false);
        }

        // Test retrieval
        let retrieved = normalizer.get_call_stack(id1)?;
        if retrieved.len() != 1 {
            tracing::error!("Call stack retrieval failed: wrong frame count");
            return Ok(false);
        }

        // Test statistics
        let stats = normalizer.get_stats()?;
        if stats.total_processed == 0 {
            tracing::error!("Call stack statistics not working");
            return Ok(false);
        }

        tracing::info!("✅ Enhanced call stack normalizer tests passed");
        Ok(true)
    }

    /// Test edge case handler
    fn test_edge_case_handler() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing edge case handler");
        
        let handler = get_global_edge_case_handler();
        
        // Test edge case handling
        let context = HashMap::new();
        let case_id = handler.handle_edge_case(
            EdgeCaseType::NullPointerAccess,
            EdgeCaseSeverity::High,
            "Test null pointer access".to_string(),
            context,
        )?;

        if case_id == 0 {
            tracing::error!("Edge case handling failed: invalid case ID");
            return Ok(false);
        }

        // Test retrieval
        let case = handler.get_edge_case(case_id)?;
        if case.case_type != EdgeCaseType::NullPointerAccess {
            tracing::error!("Edge case retrieval failed: wrong case type");
            return Ok(false);
        }

        // Test statistics
        let stats = handler.get_stats()?;
        if stats.total_cases_detected == 0 {
            tracing::error!("Edge case statistics not working");
            return Ok(false);
        }

        tracing::info!("✅ Edge case handler tests passed");
        Ok(true)
    }

    /// Test comprehensive data deduplicator
    fn test_data_deduplicator() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing comprehensive data deduplicator");
        
        let deduplicator = get_global_data_deduplicator();
        
        // Test string deduplication
        let test_string = "test string for deduplication";
        let ref1 = deduplicator.deduplicate_string(test_string)?;
        let ref2 = deduplicator.deduplicate_string(test_string)?;
        
        if ref1.hash != ref2.hash {
            tracing::error!("String deduplication failed: different hashes");
            return Ok(false);
        }

        let retrieved = deduplicator.get_string(&ref1)?;
        if *retrieved != test_string {
            tracing::error!("String retrieval failed: content mismatch");
            return Ok(false);
        }

        // Test stack trace deduplication
        let frames = vec![
            StackFrame {
                function_name: "test_function".to_string(),
                file_name: Some("test.rs".to_string()),
                line_number: Some(42),
                is_unsafe: false,
            }
        ];

        let stack_ref1 = deduplicator.deduplicate_stack_trace(&frames)?;
        let stack_ref2 = deduplicator.deduplicate_stack_trace(&frames)?;
        
        if stack_ref1.hash != stack_ref2.hash {
            tracing::error!("Stack trace deduplication failed: different hashes");
            return Ok(false);
        }

        // Test statistics
        let stats = deduplicator.get_stats()?;
        if stats.total_operations == 0 {
            tracing::error!("Data deduplicator statistics not working");
            return Ok(false);
        }

        tracing::info!("✅ Comprehensive data deduplicator tests passed");
        Ok(true)
    }

    /// Test enhanced FFI function resolver
    fn test_ffi_resolver() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing enhanced FFI function resolver");
        
        let resolver = get_global_enhanced_ffi_resolver();
        
        // Test function resolution
        let resolved = resolver.resolve_function("malloc", Some("libc"))?;
        if resolved.function_name != "malloc" {
            tracing::error!("FFI function resolution failed: wrong function name");
            return Ok(false);
        }

        if resolved.library_name != "libc" {
            tracing::error!("FFI function resolution failed: wrong library name");
            return Ok(false);
        }

        // Test pattern matching
        let pthread_resolved = resolver.resolve_function("pthread_create", None)?;
        if pthread_resolved.library_name != "libpthread" {
            tracing::error!("FFI pattern matching failed: wrong library");
            return Ok(false);
        }

        // Test statistics
        let stats = resolver.get_stats()?;
        if stats.total_attempts == 0 {
            tracing::error!("FFI resolver statistics not working");
            return Ok(false);
        }

        tracing::info!("✅ Enhanced FFI function resolver tests passed");
        Ok(true)
    }

    /// Test component integration
    fn test_component_integration() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing component integration");
        
        // Test that components work together
        let normalizer = get_global_enhanced_call_stack_normalizer();
        let deduplicator = get_global_data_deduplicator();
        let handler = get_global_edge_case_handler();
        
        // Create test data
        let frames = vec![
            StackFrame {
                function_name: "integrated_test_function".to_string(),
                file_name: Some("integration_test.rs".to_string()),
                line_number: Some(100),
                is_unsafe: false,
            }
        ];

        // Test normalization + deduplication
        let stack_id = normalizer.normalize_call_stack(&frames)?;
        let dedup_ref = deduplicator.deduplicate_stack_trace(&frames)?;
        
        // Both should work independently
        let normalized_frames = normalizer.get_call_stack(stack_id)?;
        let deduplicated_frames = deduplicator.get_stack_trace(&dedup_ref)?;
        
        if normalized_frames.len() != deduplicated_frames.len() {
            tracing::error!("Integration test failed: frame count mismatch");
            return Ok(false);
        }

        // Test edge case handling during integration
        let context = HashMap::new();
        let case_id = handler.handle_edge_case(
            EdgeCaseType::IntegerOverflow,
            EdgeCaseSeverity::Medium,
            "Integration test edge case".to_string(),
            context,
        )?;

        if case_id == 0 {
            tracing::error!("Integration edge case handling failed");
            return Ok(false);
        }

        tracing::info!("✅ Component integration tests passed");
        Ok(true)
    }

    /// Test performance characteristics
    fn test_performance() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing performance characteristics");
        
        let start_time = std::time::Instant::now();
        
        // Perform a series of operations to test performance
        let normalizer = get_global_enhanced_call_stack_normalizer();
        let deduplicator = get_global_data_deduplicator();
        
        for i in 0..1000 {
            let frames = vec![
                StackFrame {
                    function_name: format!("perf_test_function_{}", i % 10),
                    file_name: Some("perf_test.rs".to_string()),
                    line_number: Some(i as u32),
                    is_unsafe: false,
                }
            ];

            // These should be fast due to deduplication
            let _stack_id = normalizer.normalize_call_stack(&frames)?;
            let _dedup_ref = deduplicator.deduplicate_stack_trace(&frames)?;
        }

        let elapsed = start_time.elapsed();
        
        // Should complete in reasonable time (less than 1 second for 1000 operations)
        if elapsed.as_secs() > 1 {
            tracing::error!("Performance test failed: took too long ({:?})", elapsed);
            return Ok(false);
        }

        tracing::info!("✅ Performance tests passed (completed in {:?})", elapsed);
        Ok(true)
    }

    /// Test memory usage characteristics
    fn test_memory_usage() -> TrackingResult<bool> {
        tracing::info!("🧪 Testing memory usage characteristics");
        
        // Test that deduplication actually saves memory
        let deduplicator = get_global_data_deduplicator();
        
        let test_string = "This is a test string that should be deduplicated to save memory";
        
        // Create many references to the same string
        for _ in 0..100 {
            let _dedup_ref = deduplicator.deduplicate_string(test_string)?;
        }

        let stats = deduplicator.get_stats()?;
        
        // Should have high deduplication rate
        if stats.cache_hit_rate < 0.9 {
            tracing::error!("Memory usage test failed: low cache hit rate ({})", stats.cache_hit_rate);
            return Ok(false);
        }

        // Should show memory savings
        if stats.memory_saved_bytes == 0 {
            tracing::error!("Memory usage test failed: no memory savings reported");
            return Ok(false);
        }

        tracing::info!("✅ Memory usage tests passed (saved {} bytes)", stats.memory_saved_bytes);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_validation() {
        let results = IntegrationValidator::validate_all().expect("Validation should not fail");
        
        assert!(results.call_stack_normalizer_ok, "Call stack normalizer should work");
        assert!(results.edge_case_handler_ok, "Edge case handler should work");
        assert!(results.data_deduplicator_ok, "Data deduplicator should work");
        assert!(results.ffi_resolver_ok, "FFI resolver should work");
        assert!(results.integration_ok, "Integration should work");
        assert!(results.performance_ok, "Performance should be acceptable");
        assert!(results.memory_usage_ok, "Memory usage should be optimized");
    }
}