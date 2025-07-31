//! Performance comparison (placeholder)

use super::performance_benchmark::BenchmarkResult;

/// Performance comparison between different methods
pub struct PerformanceComparison {
    results: Vec<BenchmarkResult>,
}

impl PerformanceComparison {
    /// Create a new performance comparison
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    /// Add a benchmark result
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }
    
    /// Get the fastest result
    pub fn fastest(&self) -> Option<&BenchmarkResult> {
        self.results.iter().min_by_key(|r| r.duration)
    }
    
    /// Get the slowest result
    pub fn slowest(&self) -> Option<&BenchmarkResult> {
        self.results.iter().max_by_key(|r| r.duration)
    }
    
    /// Generate comparison report
    pub fn generate_report(&self) -> String {
        if self.results.is_empty() {
            return "No benchmark results available".to_string();
        }
        
        let mut report = String::new();
        report.push_str("Performance Comparison Report\n");
        report.push_str("============================\n\n");
        
        for result in &self.results {
            report.push_str(&format!(
                "{}: {:?} ({:.2} items/sec)\n",
                result.name,
                result.duration,
                result.throughput
            ));
        }
        
        if let Some(fastest) = self.fastest() {
            report.push_str(&format!("\nFastest: {}\n", fastest.name));
        }
        
        report
    }
}