use crate::analysis::closure::types::*;
use crate::capture::types::AllocationInfo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

static GLOBAL_CLOSURE_ANALYZER: OnceLock<Arc<ClosureAnalyzer>> = OnceLock::new();

pub fn get_global_closure_analyzer() -> Arc<ClosureAnalyzer> {
    GLOBAL_CLOSURE_ANALYZER
        .get_or_init(|| Arc::new(ClosureAnalyzer::new()))
        .clone()
}

pub struct ClosureAnalyzer {
    closures: Mutex<HashMap<usize, ClosureInfo>>,
    capture_events: Mutex<Vec<CaptureEvent>>,
    lifetime_graph: Mutex<LifetimeGraph>,
}

impl ClosureAnalyzer {
    pub fn new() -> Self {
        Self {
            closures: Mutex::new(HashMap::new()),
            capture_events: Mutex::new(Vec::new()),
            lifetime_graph: Mutex::new(LifetimeGraph::new()),
        }
    }

    pub fn register_closure(&self, closure_ptr: usize, captures: Vec<CaptureInfo>) {
        let closure_info = ClosureInfo {
            ptr: closure_ptr,
            captures: captures.clone(),
            creation_timestamp: current_timestamp(),
            thread_id: format!("{:?}", std::thread::current().id()),
            call_site: capture_call_site(),
            memory_footprint: self.calculate_closure_footprint(&captures),
            optimization_potential: self.analyze_optimization_potential(&captures),
        };

        for capture in &captures {
            let event = CaptureEvent {
                closure_ptr,
                captured_var: capture.clone(),
                event_type: CaptureEventType::Captured,
                timestamp: current_timestamp(),
            };

            if let Ok(mut events) = self.capture_events.lock() {
                events.push(event);
            }
        }

        if let Ok(mut graph) = self.lifetime_graph.lock() {
            graph.add_closure_relationships(closure_ptr, &captures);
        }

        if let Ok(mut closures) = self.closures.lock() {
            closures.insert(closure_ptr, closure_info);
        }
    }

    pub fn track_closure_drop(&self, closure_ptr: usize) {
        if let Ok(mut closures) = self.closures.lock() {
            if let Some(closure_info) = closures.get_mut(&closure_ptr) {
                for capture in &closure_info.captures {
                    let event = CaptureEvent {
                        closure_ptr,
                        captured_var: capture.clone(),
                        event_type: CaptureEventType::Released,
                        timestamp: current_timestamp(),
                    };

                    if let Ok(mut events) = self.capture_events.lock() {
                        events.push(event);
                    }
                }
            }
            closures.remove(&closure_ptr);
        }

        if let Ok(mut graph) = self.lifetime_graph.lock() {
            graph.remove_closure(closure_ptr);
        }
    }

    pub fn analyze_closure_patterns(
        &self,
        allocations: &[AllocationInfo],
    ) -> ClosureAnalysisReport {
        let mut detected_closures = Vec::new();
        let mut capture_statistics = CaptureStatistics::default();

        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                if self.is_closure_type(type_name) {
                    if let Some(analysis) = self.analyze_closure_allocation(allocation) {
                        detected_closures.push(analysis);
                    }
                }
            }
        }

        if let Ok(closures) = self.closures.lock() {
            capture_statistics = self.calculate_capture_statistics(&closures);
        }

        let optimization_suggestions = self.generate_optimization_suggestions(&detected_closures);
        let lifetime_analysis = self.analyze_capture_lifetimes();

        ClosureAnalysisReport {
            detected_closures,
            capture_statistics,
            optimization_suggestions,
            lifetime_analysis,
            analysis_timestamp: current_timestamp(),
        }
    }

    fn is_closure_type(&self, type_name: &str) -> bool {
        type_name.contains("closure")
            || type_name.contains("{{closure}}")
            || type_name.starts_with("fn(")
            || type_name.contains("dyn Fn")
            || type_name.contains("impl Fn")
    }

    fn analyze_closure_allocation(&self, allocation: &AllocationInfo) -> Option<DetectedClosure> {
        let type_name = allocation.type_name.as_ref()?;

        Some(DetectedClosure {
            ptr: allocation.ptr,
            type_name: type_name.clone(),
            size: allocation.size,
            estimated_captures: self.estimate_captures_from_size(allocation.size),
            closure_type: self.classify_closure_type(type_name),
            creation_context: CreationContext {
                scope_name: allocation.scope_name.clone(),
                thread_id: format!("{:?}", allocation.thread_id),
                timestamp: allocation.timestamp_alloc,
            },
            memory_impact: self.assess_memory_impact(allocation.size),
        })
    }

    fn estimate_captures_from_size(&self, size: usize) -> usize {
        if size <= 8 {
            0
        } else if size <= 32 {
            2
        } else if size <= 128 {
            8
        } else {
            size / 16
        }
    }

    fn classify_closure_type(&self, type_name: &str) -> ClosureType {
        if type_name.contains("FnOnce") {
            ClosureType::FnOnce
        } else if type_name.contains("FnMut") {
            ClosureType::FnMut
        } else if type_name.contains("Fn") {
            ClosureType::Fn
        } else {
            ClosureType::Unknown
        }
    }

    fn assess_memory_impact(&self, size: usize) -> MemoryImpact {
        match size {
            0..=16 => MemoryImpact::Minimal,
            17..=64 => MemoryImpact::Low,
            65..=256 => MemoryImpact::Medium,
            257..=1024 => MemoryImpact::High,
            _ => MemoryImpact::VeryHigh,
        }
    }

    fn calculate_closure_footprint(&self, captures: &[CaptureInfo]) -> ClosureFootprint {
        let total_size = captures.iter().map(|c| c.size).sum();
        let by_value_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue)
            .count();
        let by_ref_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByReference)
            .count();
        let by_mut_ref_count = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByMutableReference)
            .count();

        ClosureFootprint {
            total_size,
            capture_count: captures.len(),
            by_value_count,
            by_ref_count,
            by_mut_ref_count,
            estimated_heap_usage: self.estimate_heap_usage(captures),
        }
    }

    fn estimate_heap_usage(&self, captures: &[CaptureInfo]) -> usize {
        captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue)
            .filter(|c| self.is_heap_allocated_type(&c.var_type))
            .map(|c| c.size)
            .sum()
    }

    fn is_heap_allocated_type(&self, type_name: &str) -> bool {
        type_name.contains("Vec")
            || type_name.contains("String")
            || type_name.contains("HashMap")
            || type_name.contains("Box")
            || type_name.contains("Arc")
            || type_name.contains("Rc")
    }

    fn analyze_optimization_potential(&self, captures: &[CaptureInfo]) -> OptimizationPotential {
        let mut suggestions = Vec::new();
        let mut potential_savings = 0;

        for capture in captures {
            if capture.mode == CaptureMode::ByValue && capture.size > 64 {
                suggestions.push(format!(
                    "Consider capturing '{}' by reference instead of by value to save {} bytes",
                    capture.var_name, capture.size
                ));
                potential_savings += capture.size;
            }
        }

        let mut_captures = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByMutableReference)
            .count();
        if mut_captures > captures.len() / 2 {
            suggestions.push("Consider if all mutable captures are necessary".to_string());
        }

        let heap_captures = captures
            .iter()
            .filter(|c| c.mode == CaptureMode::ByValue && self.is_heap_allocated_type(&c.var_type))
            .count();

        if heap_captures > 0 {
            suggestions
                .push("Consider using move semantics for heap-allocated captures".to_string());
        }

        OptimizationPotential {
            level: if potential_savings > 256 {
                OptimizationLevel::High
            } else if potential_savings > 64 {
                OptimizationLevel::Medium
            } else if !suggestions.is_empty() {
                OptimizationLevel::Low
            } else {
                OptimizationLevel::None
            },
            potential_savings,
            suggestions,
        }
    }

    fn calculate_capture_statistics(
        &self,
        closures: &HashMap<usize, ClosureInfo>,
    ) -> CaptureStatistics {
        let total_closures = closures.len();
        let total_captures = closures.values().map(|c| c.captures.len()).sum();

        let mut by_mode = HashMap::new();
        let mut by_type = HashMap::new();
        let mut total_memory = 0;

        for closure in closures.values() {
            total_memory += closure.memory_footprint.total_size;

            for capture in &closure.captures {
                *by_mode.entry(capture.mode.clone()).or_insert(0) += 1;
                *by_type.entry(capture.var_type.clone()).or_insert(0) += 1;
            }
        }

        let avg_captures_per_closure = if total_closures > 0 {
            total_captures as f64 / total_closures as f64
        } else {
            0.0
        };

        CaptureStatistics {
            total_closures,
            total_captures,
            avg_captures_per_closure,
            total_memory_usage: total_memory,
            captures_by_mode: by_mode,
            captures_by_type: by_type,
        }
    }

    fn generate_optimization_suggestions(
        &self,
        closures: &[DetectedClosure],
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        let high_memory_closures = closures
            .iter()
            .filter(|c| matches!(c.memory_impact, MemoryImpact::High | MemoryImpact::VeryHigh))
            .count();

        if high_memory_closures > 0 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Memory,
                priority: SuggestionPriority::High,
                description: format!(
                    "Found {high_memory_closures} closures with high memory usage",
                ),
                recommendation: "Consider reducing capture size or using references".to_string(),
                estimated_impact: "20-50% memory reduction".to_string(),
            });
        }

        let fnonce_count = closures
            .iter()
            .filter(|c| c.closure_type == ClosureType::FnOnce)
            .count();

        if fnonce_count > closures.len() / 2 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Performance,
                priority: SuggestionPriority::Medium,
                description: "Many FnOnce closures detected".to_string(),
                recommendation: "Consider if Fn or FnMut traits would be more appropriate"
                    .to_string(),
                estimated_impact: "Improved reusability".to_string(),
            });
        }

        suggestions
    }

    fn analyze_capture_lifetimes(&self) -> LifetimeAnalysis {
        if let Ok(graph) = self.lifetime_graph.lock() {
            graph.analyze_lifetimes()
        } else {
            LifetimeAnalysis::default()
        }
    }
}

impl Default for ClosureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn capture_call_site() -> String {
    format!("{}:{}", file!(), line!())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    /// Objective: Verify ClosureAnalyzer creation with default values
    /// Invariants: New analyzer should have empty collections
    #[test]
    fn test_closure_analyzer_creation() {
        let analyzer = ClosureAnalyzer::new();

        let closures = analyzer.closures.lock().unwrap();
        let events = analyzer.capture_events.lock().unwrap();

        assert!(closures.is_empty(), "New analyzer should have no closures");
        assert!(
            events.is_empty(),
            "New analyzer should have no capture events"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same as new()
    #[test]
    fn test_closure_analyzer_default() {
        let analyzer = ClosureAnalyzer::default();

        let closures = analyzer.closures.lock().unwrap();
        assert!(
            closures.is_empty(),
            "Default analyzer should have no closures"
        );
    }

    /// Objective: Verify register_closure functionality
    /// Invariants: Should add closure and capture events correctly
    #[test]
    fn test_register_closure() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: None,
        }];

        analyzer.register_closure(0x5000, captures);

        let closures = analyzer.closures.lock().unwrap();
        assert!(
            closures.contains_key(&0x5000),
            "Should contain registered closure"
        );

        let events = analyzer.capture_events.lock().unwrap();
        assert_eq!(events.len(), 1, "Should have one capture event");
        assert_eq!(
            events[0].event_type,
            CaptureEventType::Captured,
            "Event should be Captured"
        );
    }

    /// Objective: Verify register_closure with multiple captures
    /// Invariants: Should create capture events for each capture
    #[test]
    fn test_register_closure_multiple_captures() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "a".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "b".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "c".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByMutableReference,
                var_type: "Vec<u8>".to_string(),
                size: 24,
                lifetime_bound: None,
            },
        ];

        analyzer.register_closure(0x6000, captures);

        let events = analyzer.capture_events.lock().unwrap();
        assert_eq!(events.len(), 3, "Should have three capture events");
    }

    /// Objective: Verify track_closure_drop functionality
    /// Invariants: Should remove closure and create release events
    #[test]
    fn test_track_closure_drop() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![CaptureInfo {
            var_name: "x".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "i32".to_string(),
            size: 4,
            lifetime_bound: None,
        }];

        analyzer.register_closure(0x5000, captures);
        analyzer.track_closure_drop(0x5000);

        let closures = analyzer.closures.lock().unwrap();
        assert!(
            !closures.contains_key(&0x5000),
            "Should not contain dropped closure"
        );

        let events = analyzer.capture_events.lock().unwrap();
        assert_eq!(events.len(), 2, "Should have captured and released events");
        assert_eq!(
            events[1].event_type,
            CaptureEventType::Released,
            "Second event should be Released"
        );
    }

    /// Objective: Verify track_closure_drop for non-existent closure
    /// Invariants: Should handle gracefully without error
    #[test]
    fn test_track_closure_drop_nonexistent() {
        let analyzer = ClosureAnalyzer::new();

        analyzer.track_closure_drop(0xdead);

        let closures = analyzer.closures.lock().unwrap();
        assert!(closures.is_empty(), "Should still be empty");
    }

    /// Objective: Verify is_closure_type detection
    /// Invariants: Should correctly identify closure types
    #[test]
    fn test_is_closure_type() {
        let analyzer = ClosureAnalyzer::new();

        assert!(
            analyzer.is_closure_type("closure"),
            "Should detect 'closure'"
        );
        assert!(
            analyzer.is_closure_type("{{closure}}"),
            "Should detect '{{closure}}'"
        );
        assert!(
            analyzer.is_closure_type("fn() -> i32"),
            "Should detect 'fn('"
        );
        assert!(
            analyzer.is_closure_type("dyn Fn()"),
            "Should detect 'dyn Fn'"
        );
        assert!(
            analyzer.is_closure_type("impl FnMut()"),
            "Should detect 'impl Fn'"
        );
        assert!(
            !analyzer.is_closure_type("String"),
            "Should not detect 'String'"
        );
        assert!(
            !analyzer.is_closure_type("Vec<u8>"),
            "Should not detect 'Vec<u8>'"
        );
    }

    /// Objective: Verify estimate_captures_from_size
    /// Invariants: Should estimate based on closure size
    #[test]
    fn test_estimate_captures_from_size() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(
            analyzer.estimate_captures_from_size(0),
            0,
            "Size 0 should estimate 0 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(8),
            0,
            "Size <= 8 should estimate 0 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(16),
            2,
            "Size 9-32 should estimate 2 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(32),
            2,
            "Size 32 should estimate 2 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(64),
            8,
            "Size 33-128 should estimate 8 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(128),
            8,
            "Size 128 should estimate 8 captures"
        );
        assert_eq!(
            analyzer.estimate_captures_from_size(256),
            16,
            "Size > 128 should estimate size/16 captures"
        );
    }

    /// Objective: Verify classify_closure_type
    /// Invariants: Should correctly classify Fn/FnMut/FnOnce
    #[test]
    fn test_classify_closure_type() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(
            analyzer.classify_closure_type("FnOnce"),
            ClosureType::FnOnce,
            "Should classify FnOnce"
        );
        assert_eq!(
            analyzer.classify_closure_type("impl FnOnce"),
            ClosureType::FnOnce,
            "Should classify impl FnOnce"
        );
        assert_eq!(
            analyzer.classify_closure_type("FnMut"),
            ClosureType::FnMut,
            "Should classify FnMut"
        );
        assert_eq!(
            analyzer.classify_closure_type("dyn FnMut"),
            ClosureType::FnMut,
            "Should classify dyn FnMut"
        );
        assert_eq!(
            analyzer.classify_closure_type("Fn"),
            ClosureType::Fn,
            "Should classify Fn"
        );
        assert_eq!(
            analyzer.classify_closure_type("dyn Fn"),
            ClosureType::Fn,
            "Should classify dyn Fn"
        );
        assert_eq!(
            analyzer.classify_closure_type("SomeType"),
            ClosureType::Unknown,
            "Should classify unknown"
        );
    }

    /// Objective: Verify assess_memory_impact
    /// Invariants: Should correctly assess impact levels
    #[test]
    fn test_assess_memory_impact() {
        let analyzer = ClosureAnalyzer::new();

        assert_eq!(
            analyzer.assess_memory_impact(0),
            MemoryImpact::Minimal,
            "Size 0 should be Minimal"
        );
        assert_eq!(
            analyzer.assess_memory_impact(16),
            MemoryImpact::Minimal,
            "Size 16 should be Minimal"
        );
        assert_eq!(
            analyzer.assess_memory_impact(32),
            MemoryImpact::Low,
            "Size 32 should be Low"
        );
        assert_eq!(
            analyzer.assess_memory_impact(64),
            MemoryImpact::Low,
            "Size 64 should be Low"
        );
        assert_eq!(
            analyzer.assess_memory_impact(128),
            MemoryImpact::Medium,
            "Size 128 should be Medium"
        );
        assert_eq!(
            analyzer.assess_memory_impact(256),
            MemoryImpact::Medium,
            "Size 256 should be Medium"
        );
        assert_eq!(
            analyzer.assess_memory_impact(512),
            MemoryImpact::High,
            "Size 512 should be High"
        );
        assert_eq!(
            analyzer.assess_memory_impact(1024),
            MemoryImpact::High,
            "Size 1024 should be High"
        );
        assert_eq!(
            analyzer.assess_memory_impact(2048),
            MemoryImpact::VeryHigh,
            "Size 2048 should be VeryHigh"
        );
    }

    /// Objective: Verify calculate_closure_footprint
    /// Invariants: Should correctly calculate footprint metrics
    #[test]
    fn test_calculate_closure_footprint() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "a".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "b".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "String".to_string(),
                size: 8,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "c".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByMutableReference,
                var_type: "Vec<u8>".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        let footprint = analyzer.calculate_closure_footprint(&captures);

        assert_eq!(footprint.total_size, 20, "Total size should be 20");
        assert_eq!(footprint.capture_count, 3, "Capture count should be 3");
        assert_eq!(footprint.by_value_count, 1, "By value count should be 1");
        assert_eq!(footprint.by_ref_count, 1, "By ref count should be 1");
        assert_eq!(
            footprint.by_mut_ref_count, 1,
            "By mut ref count should be 1"
        );
    }

    /// Objective: Verify is_heap_allocated_type
    /// Invariants: Should correctly identify heap types
    #[test]
    fn test_is_heap_allocated_type() {
        let analyzer = ClosureAnalyzer::new();

        assert!(
            analyzer.is_heap_allocated_type("Vec<u8>"),
            "Vec should be heap type"
        );
        assert!(
            analyzer.is_heap_allocated_type("String"),
            "String should be heap type"
        );
        assert!(
            analyzer.is_heap_allocated_type("HashMap<K, V>"),
            "HashMap should be heap type"
        );
        assert!(
            analyzer.is_heap_allocated_type("Box<T>"),
            "Box should be heap type"
        );
        assert!(
            analyzer.is_heap_allocated_type("Arc<T>"),
            "Arc should be heap type"
        );
        assert!(
            analyzer.is_heap_allocated_type("Rc<T>"),
            "Rc should be heap type"
        );
        assert!(
            !analyzer.is_heap_allocated_type("i32"),
            "i32 should not be heap type"
        );
        assert!(
            !analyzer.is_heap_allocated_type("&str"),
            "&str should not be heap type"
        );
    }

    /// Objective: Verify estimate_heap_usage
    /// Invariants: Should sum sizes of heap-allocated by-value captures
    #[test]
    fn test_estimate_heap_usage() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "vec".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "Vec<u8>".to_string(),
                size: 24,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "s".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByValue,
                var_type: "String".to_string(),
                size: 24,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "num".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "ref_vec".to_string(),
                var_ptr: 0x4000,
                mode: CaptureMode::ByReference,
                var_type: "Vec<u8>".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        let heap_usage = analyzer.estimate_heap_usage(&captures);

        assert_eq!(heap_usage, 48, "Should sum Vec and String sizes (24+24)");
    }

    /// Objective: Verify analyze_optimization_potential with large by-value captures
    /// Invariants: Should suggest reference capture for large values
    #[test]
    fn test_analyze_optimization_potential_large_value() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![CaptureInfo {
            var_name: "large_data".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "[u8; 128]".to_string(),
            size: 128,
            lifetime_bound: None,
        }];

        let potential = analyzer.analyze_optimization_potential(&captures);

        assert_eq!(
            potential.level,
            OptimizationLevel::Medium,
            "Large capture should have Medium optimization level"
        );
        assert_eq!(
            potential.potential_savings, 128,
            "Potential savings should be 128"
        );
        assert!(
            potential
                .suggestions
                .iter()
                .any(|s| s.contains("reference")),
            "Should suggest reference capture"
        );
    }

    /// Objective: Verify analyze_optimization_potential with high savings
    /// Invariants: Should have High optimization level for >256 bytes
    #[test]
    fn test_analyze_optimization_potential_high_savings() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "data1".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "[u8; 128]".to_string(),
                size: 128,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "data2".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByValue,
                var_type: "[u8; 128]".to_string(),
                size: 128,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "data3".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByValue,
                var_type: "[u8; 128]".to_string(),
                size: 128,
                lifetime_bound: None,
            },
        ];

        let potential = analyzer.analyze_optimization_potential(&captures);

        assert_eq!(
            potential.level,
            OptimizationLevel::High,
            "Savings > 256 should have High optimization level"
        );
        assert_eq!(
            potential.potential_savings, 384,
            "Potential savings should be 384 (128 * 3)"
        );
    }

    /// Objective: Verify analyze_optimization_potential with many mutable captures
    /// Invariants: Should suggest reviewing mutable captures
    #[test]
    fn test_analyze_optimization_potential_many_mutable() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "a".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByMutableReference,
                var_type: "i32".to_string(),
                size: 8,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "b".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByMutableReference,
                var_type: "i32".to_string(),
                size: 8,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "c".to_string(),
                var_ptr: 0x3000,
                mode: CaptureMode::ByReference,
                var_type: "i32".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        let potential = analyzer.analyze_optimization_potential(&captures);

        assert!(
            potential.suggestions.iter().any(|s| s.contains("mutable")),
            "Should suggest reviewing mutable captures"
        );
    }

    /// Objective: Verify analyze_optimization_potential with heap captures
    /// Invariants: Should suggest move semantics
    #[test]
    fn test_analyze_optimization_potential_heap_captures() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![CaptureInfo {
            var_name: "vec".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByValue,
            var_type: "Vec<u8>".to_string(),
            size: 24,
            lifetime_bound: None,
        }];

        let potential = analyzer.analyze_optimization_potential(&captures);

        assert!(
            potential
                .suggestions
                .iter()
                .any(|s| s.contains("move") || s.contains("heap")),
            "Should suggest move semantics for heap captures"
        );
    }

    /// Objective: Verify analyze_optimization_potential with no issues
    /// Invariants: Should have None optimization level
    #[test]
    fn test_analyze_optimization_potential_none() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![CaptureInfo {
            var_name: "small".to_string(),
            var_ptr: 0x1000,
            mode: CaptureMode::ByReference,
            var_type: "i32".to_string(),
            size: 8,
            lifetime_bound: None,
        }];

        let potential = analyzer.analyze_optimization_potential(&captures);

        assert_eq!(
            potential.level,
            OptimizationLevel::None,
            "Small reference capture should have None optimization level"
        );
        assert_eq!(
            potential.potential_savings, 0,
            "Potential savings should be 0"
        );
    }

    /// Objective: Verify calculate_capture_statistics
    /// Invariants: Should correctly aggregate statistics
    #[test]
    fn test_calculate_capture_statistics() {
        let analyzer = ClosureAnalyzer::new();

        let mut closures = HashMap::new();

        closures.insert(
            0x1000,
            ClosureInfo {
                ptr: 0x1000,
                captures: vec![
                    CaptureInfo {
                        var_name: "a".to_string(),
                        var_ptr: 0x1000,
                        mode: CaptureMode::ByValue,
                        var_type: "i32".to_string(),
                        size: 4,
                        lifetime_bound: None,
                    },
                    CaptureInfo {
                        var_name: "b".to_string(),
                        var_ptr: 0x2000,
                        mode: CaptureMode::ByReference,
                        var_type: "String".to_string(),
                        size: 8,
                        lifetime_bound: None,
                    },
                ],
                creation_timestamp: 1000,
                thread_id: "main".to_string(),
                call_site: "test.rs:1".to_string(),
                memory_footprint: ClosureFootprint {
                    total_size: 12,
                    capture_count: 2,
                    by_value_count: 1,
                    by_ref_count: 1,
                    by_mut_ref_count: 0,
                    estimated_heap_usage: 0,
                },
                optimization_potential: OptimizationPotential {
                    level: OptimizationLevel::None,
                    potential_savings: 0,
                    suggestions: vec![],
                },
            },
        );

        let stats = analyzer.calculate_capture_statistics(&closures);

        assert_eq!(stats.total_closures, 1, "Should have 1 closure");
        assert_eq!(stats.total_captures, 2, "Should have 2 captures");
        assert_eq!(stats.avg_captures_per_closure, 2.0, "Average should be 2.0");
        assert_eq!(stats.total_memory_usage, 12, "Total memory should be 12");
    }

    /// Objective: Verify calculate_capture_statistics with empty closures
    /// Invariants: Should return zero statistics
    #[test]
    fn test_calculate_capture_statistics_empty() {
        let analyzer = ClosureAnalyzer::new();

        let closures = HashMap::new();
        let stats = analyzer.calculate_capture_statistics(&closures);

        assert_eq!(stats.total_closures, 0, "Should have 0 closures");
        assert_eq!(stats.total_captures, 0, "Should have 0 captures");
        assert_eq!(stats.avg_captures_per_closure, 0.0, "Average should be 0.0");
    }

    /// Objective: Verify generate_optimization_suggestions with high memory closures
    /// Invariants: Should suggest memory optimization
    #[test]
    fn test_generate_optimization_suggestions_high_memory() {
        let analyzer = ClosureAnalyzer::new();

        let closures = vec![DetectedClosure {
            ptr: 0x1000,
            type_name: "closure".to_string(),
            size: 1024,
            estimated_captures: 10,
            closure_type: ClosureType::Fn,
            creation_context: CreationContext {
                scope_name: None,
                thread_id: "main".to_string(),
                timestamp: 1000,
            },
            memory_impact: MemoryImpact::High,
        }];

        let suggestions = analyzer.generate_optimization_suggestions(&closures);

        assert!(
            suggestions
                .iter()
                .any(|s| s.category == OptimizationCategory::Memory),
            "Should have memory suggestion"
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.priority == SuggestionPriority::High),
            "Should have high priority suggestion"
        );
    }

    /// Objective: Verify generate_optimization_suggestions with many FnOnce closures
    /// Invariants: Should suggest Fn/FnMut alternatives
    #[test]
    fn test_generate_optimization_suggestions_many_fnonce() {
        let analyzer = ClosureAnalyzer::new();

        let closures = vec![
            DetectedClosure {
                ptr: 0x1000,
                type_name: "FnOnce".to_string(),
                size: 64,
                estimated_captures: 2,
                closure_type: ClosureType::FnOnce,
                creation_context: CreationContext {
                    scope_name: None,
                    thread_id: "main".to_string(),
                    timestamp: 1000,
                },
                memory_impact: MemoryImpact::Low,
            },
            DetectedClosure {
                ptr: 0x2000,
                type_name: "FnOnce".to_string(),
                size: 64,
                estimated_captures: 2,
                closure_type: ClosureType::FnOnce,
                creation_context: CreationContext {
                    scope_name: None,
                    thread_id: "main".to_string(),
                    timestamp: 1000,
                },
                memory_impact: MemoryImpact::Low,
            },
            DetectedClosure {
                ptr: 0x3000,
                type_name: "Fn".to_string(),
                size: 64,
                estimated_captures: 2,
                closure_type: ClosureType::Fn,
                creation_context: CreationContext {
                    scope_name: None,
                    thread_id: "main".to_string(),
                    timestamp: 1000,
                },
                memory_impact: MemoryImpact::Low,
            },
        ];

        let suggestions = analyzer.generate_optimization_suggestions(&closures);

        assert!(
            suggestions.iter().any(|s| s.description.contains("FnOnce")),
            "Should mention FnOnce closures"
        );
    }

    /// Objective: Verify analyze_capture_lifetimes
    /// Invariants: Should return lifetime analysis
    #[test]
    fn test_analyze_capture_lifetimes() {
        let analyzer = ClosureAnalyzer::new();

        let captures = vec![
            CaptureInfo {
                var_name: "a".to_string(),
                var_ptr: 0x1000,
                mode: CaptureMode::ByValue,
                var_type: "i32".to_string(),
                size: 4,
                lifetime_bound: None,
            },
            CaptureInfo {
                var_name: "b".to_string(),
                var_ptr: 0x2000,
                mode: CaptureMode::ByReference,
                var_type: "String".to_string(),
                size: 8,
                lifetime_bound: None,
            },
        ];

        analyzer.register_closure(0x5000, captures);
        let analysis = analyzer.analyze_capture_lifetimes();

        assert_eq!(
            analysis.total_relationships, 1,
            "Should have 1 relationship"
        );
    }

    /// Objective: Verify get_global_closure_analyzer singleton
    /// Invariants: Should return same instance
    #[test]
    fn test_global_closure_analyzer_singleton() {
        let analyzer1 = get_global_closure_analyzer();
        let analyzer2 = get_global_closure_analyzer();

        assert!(
            Arc::ptr_eq(&analyzer1, &analyzer2),
            "Should return same instance"
        );
    }

    /// Objective: Verify current_timestamp returns valid value
    /// Invariants: Timestamp should be positive
    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0, "Timestamp should be positive");
    }

    /// Objective: Verify capture_call_site returns valid string
    /// Invariants: Should contain file and line info
    #[test]
    fn test_capture_call_site() {
        let site = capture_call_site();
        assert!(!site.is_empty(), "Call site should not be empty");
        assert!(
            site.contains(".rs"),
            "Call site should contain .rs extension"
        );
    }

    /// Objective: Verify analyze_closure_patterns with allocations
    /// Invariants: Should detect closures in allocations
    #[test]
    fn test_analyze_closure_patterns_with_allocations() {
        let analyzer = ClosureAnalyzer::new();

        let allocations = vec![AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("closure_var".to_string()),
            type_name: Some("closure".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: thread::current().id(),
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
        }];

        let report = analyzer.analyze_closure_patterns(&allocations);

        assert_eq!(report.detected_closures.len(), 1, "Should detect 1 closure");
        assert!(report.analysis_timestamp > 0, "Should have timestamp");
    }

    /// Objective: Verify analyze_closure_patterns with non-closure allocations
    /// Invariants: Should not detect non-closure types
    #[test]
    fn test_analyze_closure_patterns_non_closure() {
        let analyzer = ClosureAnalyzer::new();

        let allocations = vec![AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("string_var".to_string()),
            type_name: Some("String".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: thread::current().id(),
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
        }];

        let report = analyzer.analyze_closure_patterns(&allocations);

        assert_eq!(
            report.detected_closures.len(),
            0,
            "Should not detect non-closure"
        );
    }

    /// Objective: Verify analyze_closure_allocation
    /// Invariants: Should correctly analyze closure allocation
    #[test]
    fn test_analyze_closure_allocation() {
        let analyzer = ClosureAnalyzer::new();

        let allocation = AllocationInfo {
            ptr: 0x1000,
            size: 128,
            var_name: Some("my_closure".to_string()),
            type_name: Some("dyn FnMut()".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: thread::current().id(),
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
        };

        let detected = analyzer.analyze_closure_allocation(&allocation);

        assert!(detected.is_some(), "Should detect closure");
        let closure = detected.unwrap();
        assert_eq!(closure.ptr, 0x1000, "Pointer should match");
        assert_eq!(
            closure.closure_type,
            ClosureType::FnMut,
            "Should classify as FnMut"
        );
        assert_eq!(
            closure.memory_impact,
            MemoryImpact::Medium,
            "Size 128 should be Medium impact"
        );
    }

    /// Objective: Verify concurrent access to ClosureAnalyzer
    /// Invariants: Should handle concurrent operations safely
    #[test]
    fn test_concurrent_access() {
        let analyzer = Arc::new(ClosureAnalyzer::new());
        let mut handles = vec![];

        for i in 0..5 {
            let analyzer_clone = analyzer.clone();
            let handle = thread::spawn(move || {
                let captures = vec![CaptureInfo {
                    var_name: format!("var{}", i),
                    var_ptr: 0x1000 + i,
                    mode: CaptureMode::ByValue,
                    var_type: "i32".to_string(),
                    size: 4,
                    lifetime_bound: None,
                }];
                analyzer_clone.register_closure(0x5000 + i, captures);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let closures = analyzer.closures.lock().unwrap();
        assert_eq!(closures.len(), 5, "Should have 5 closures from 5 threads");
    }
}
