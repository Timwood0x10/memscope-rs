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
    "<call_site_placeholder>".to_string()
}
