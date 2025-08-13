# Extending Analysis Features

Advanced guide for extending and customizing memscope-rs analysis capabilities.

## 🎯 Objectives

- Create custom analyzers
- Extend existing analysis features
- Integrate third-party tools
- Develop specialized analysis plugins

## 🔧 Custom Analyzers

### Basic Analyzer Framework

```rust
use memscope_rs::{AllocationInfo, AnalysisResult};

pub trait CustomAnalyzer {
    type Output;
    
    fn analyze(&self, allocations: &[AllocationInfo]) -> AnalysisResult<Self::Output>;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

pub struct FragmentationAnalyzer {
    threshold: f64,
}

impl FragmentationAnalyzer {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}

impl CustomAnalyzer for FragmentationAnalyzer {
    type Output = FragmentationReport;
    
    fn analyze(&self, allocations: &[AllocationInfo]) -> AnalysisResult<Self::Output> {
        let mut total_allocated = 0;
        let mut total_gaps = 0;
        let mut gap_count = 0;
        
        // Analyze memory fragmentation
        for window in allocations.windows(2) {
            let current = &window[0];
            let next = &window[1];
            
            total_allocated += current.size;
            
            if let (Some(current_end), Some(next_start)) = 
                (current.ptr.checked_add(current.size), Some(next.ptr)) {
                if next_start > current_end {
                    let gap = next_start - current_end;
                    total_gaps += gap;
                    gap_count += 1;
                }
            }
        }
        
        let fragmentation_ratio = if total_allocated > 0 {
            total_gaps as f64 / total_allocated as f64
        } else {
            0.0
        };
        
        Ok(FragmentationReport {
            fragmentation_ratio,
            total_gaps,
            gap_count,
            is_fragmented: fragmentation_ratio > self.threshold,
        })
    }
    
    fn name(&self) -> &str {
        "FragmentationAnalyzer"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
}

#[derive(Debug)]
pub struct FragmentationReport {
    pub fragmentation_ratio: f64,
    pub total_gaps: usize,
    pub gap_count: usize,
    pub is_fragmented: bool,
}
```

### Advanced Pattern Analyzer

```rust
use std::collections::HashMap;

pub struct PatternAnalyzer {
    min_pattern_length: usize,
    min_frequency: usize,
}

impl PatternAnalyzer {
    pub fn new(min_pattern_length: usize, min_frequency: usize) -> Self {
        Self {
            min_pattern_length,
            min_frequency,
        }
    }
    
    fn detect_allocation_patterns(&self, allocations: &[AllocationInfo]) -> Vec<AllocationPattern> {
        let mut patterns = HashMap::new();
        
        // Detect allocation size patterns
        for window in allocations.windows(self.min_pattern_length) {
            let sizes: Vec<usize> = window.iter().map(|a| a.size).collect();
            let pattern_key = format!("{:?}", sizes);
            
            *patterns.entry(pattern_key.clone()).or_insert(0) += 1;
        }
        
        // Filter frequent patterns
        patterns.into_iter()
            .filter(|(_, count)| *count >= self.min_frequency)
            .map(|(pattern, frequency)| AllocationPattern {
                pattern,
                frequency,
                impact: self.calculate_impact(frequency),
            })
            .collect()
    }
    
    fn calculate_impact(&self, frequency: usize) -> PatternImpact {
        match frequency {
            f if f > 100 => PatternImpact::High,
            f if f > 50 => PatternImpact::Medium,
            _ => PatternImpact::Low,
        }
    }
}

#[derive(Debug)]
pub struct AllocationPattern {
    pub pattern: String,
    pub frequency: usize,
    pub impact: PatternImpact,
}

#[derive(Debug)]
pub enum PatternImpact {
    Low,
    Medium,
    High,
}
```

## 🔌 Plugin System

### Analysis Plugin Interface

```rust
use std::any::Any;
use std::collections::HashMap;

pub trait AnalysisPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn analyze(&self, data: &AnalysisContext) -> Box<dyn Any>;
    fn dependencies(&self) -> Vec<&str> { vec![] }
}

pub struct AnalysisContext {
    pub allocations: Vec<AllocationInfo>,
    pub metadata: HashMap<String, String>,
    pub previous_results: HashMap<String, Box<dyn Any>>,
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn AnalysisPlugin>>,
    execution_order: Vec<String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            execution_order: Vec::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Box<dyn AnalysisPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name.clone(), plugin);
        self.resolve_dependencies();
    }
    
    fn resolve_dependencies(&mut self) {
        // Simplified dependency resolution
        let mut resolved = Vec::new();
        let mut remaining: Vec<_> = self.plugins.keys().cloned().collect();
        
        while !remaining.is_empty() {
            let mut progress = false;
            
            remaining.retain(|name| {
                let plugin = &self.plugins[name];
                let deps = plugin.dependencies();
                
                if deps.iter().all(|dep| resolved.contains(&dep.to_string())) {
                    resolved.push(name.clone());
                    progress = true;
                    false
                } else {
                    true
                }
            });
            
            if !progress {
                panic!("Circular dependency or missing dependency");
            }
        }
        
        self.execution_order = resolved;
    }
    
    pub fn run_analysis(&self, context: &mut AnalysisContext) -> HashMap<String, Box<dyn Any>> {
        let mut results = HashMap::new();
        
        for plugin_name in &self.execution_order {
            if let Some(plugin) = self.plugins.get(plugin_name) {
                let result = plugin.analyze(context);
                results.insert(plugin_name.clone(), result);
                context.previous_results.insert(plugin_name.clone(), 
                    results[plugin_name].as_ref() as *const dyn Any as *mut dyn Any);
            }
        }
        
        results
    }
}
```

### Example Plugin Implementation

```rust
pub struct MemoryLeakPlugin {
    threshold_ms: u64,
}

impl MemoryLeakPlugin {
    pub fn new(threshold_ms: u64) -> Self {
        Self { threshold_ms }
    }
}

impl AnalysisPlugin for MemoryLeakPlugin {
    fn name(&self) -> &str {
        "MemoryLeakDetector"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn analyze(&self, context: &AnalysisContext) -> Box<dyn Any> {
        let mut potential_leaks = Vec::new();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        for allocation in &context.allocations {
            if let Some(creation_time) = allocation.creation_time {
                let lifetime = current_time - creation_time;
                if lifetime > self.threshold_ms {
                    potential_leaks.push(PotentialLeak {
                        ptr: allocation.ptr,
                        size: allocation.size,
                        lifetime_ms: lifetime,
                        var_name: allocation.var_name.clone(),
                    });
                }
            }
        }
        
        Box::new(MemoryLeakReport {
            potential_leaks,
            threshold_ms: self.threshold_ms,
        })
    }
}

#[derive(Debug)]
pub struct PotentialLeak {
    pub ptr: usize,
    pub size: usize,
    pub lifetime_ms: u64,
    pub var_name: Option<String>,
}

#[derive(Debug)]
pub struct MemoryLeakReport {
    pub potential_leaks: Vec<PotentialLeak>,
    pub threshold_ms: u64,
}
```

## 🔗 Third-party Tool Integration

### Valgrind Integration

```rust
use std::process::Command;
use std::fs;

pub struct ValgrindIntegration {
    valgrind_path: String,
    output_file: String,
}

impl ValgrindIntegration {
    pub fn new(valgrind_path: String) -> Self {
        Self {
            valgrind_path,
            output_file: "valgrind_output.xml".to_string(),
        }
    }
    
    pub fn run_with_valgrind(&self, program: &str, args: &[&str]) -> Result<ValgrindReport, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&self.valgrind_path);
        cmd.arg("--tool=memcheck")
           .arg("--xml=yes")
           .arg(&format!("--xml-file={}", self.output_file))
           .arg(program)
           .args(args);
        
        let output = cmd.output()?;
        
        if output.status.success() {
            let xml_content = fs::read_to_string(&self.output_file)?;
            self.parse_valgrind_output(&xml_content)
        } else {
            Err("Valgrind execution failed".into())
        }
    }
    
    fn parse_valgrind_output(&self, xml: &str) -> Result<ValgrindReport, Box<dyn std::error::Error>> {
        // Simplified XML parsing
        let leak_count = xml.matches("<kind>Leak_").count();
        let error_count = xml.matches("<error>").count();
        
        Ok(ValgrindReport {
            memory_leaks: leak_count,
            memory_errors: error_count,
            raw_output: xml.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct ValgrindReport {
    pub memory_leaks: usize,
    pub memory_errors: usize,
    pub raw_output: String,
}
```

### Perf Integration

```rust
pub struct PerfIntegration {
    perf_path: String,
}

impl PerfIntegration {
    pub fn new() -> Self {
        Self {
            perf_path: "perf".to_string(),
        }
    }
    
    pub fn profile_memory(&self, program: &str, args: &[&str]) -> Result<PerfReport, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&self.perf_path);
        cmd.arg("stat")
           .arg("-e")
           .arg("cache-misses,cache-references,page-faults")
           .arg(program)
           .args(args);
        
        let output = cmd.output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        self.parse_perf_output(&stderr)
    }
    
    fn parse_perf_output(&self, output: &str) -> Result<PerfReport, Box<dyn std::error::Error>> {
        let mut cache_misses = 0;
        let mut cache_references = 0;
        let mut page_faults = 0;
        
        for line in output.lines() {
            if line.contains("cache-misses") {
                cache_misses = self.extract_number(line);
            } else if line.contains("cache-references") {
                cache_references = self.extract_number(line);
            } else if line.contains("page-faults") {
                page_faults = self.extract_number(line);
            }
        }
        
        Ok(PerfReport {
            cache_misses,
            cache_references,
            page_faults,
            cache_miss_rate: if cache_references > 0 {
                cache_misses as f64 / cache_references as f64
            } else {
                0.0
            },
        })
    }
    
    fn extract_number(&self, line: &str) -> u64 {
        line.split_whitespace()
            .next()
            .and_then(|s| s.replace(",", "").parse().ok())
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct PerfReport {
    pub cache_misses: u64,
    pub cache_references: u64,
    pub page_faults: u64,
    pub cache_miss_rate: f64,
}
```

## 📊 Custom Report Generation

### Report Generator

```rust
use serde_json::Value;

pub struct CustomReportGenerator {
    template: String,
    formatters: HashMap<String, Box<dyn Fn(&Value) -> String>>,
}

impl CustomReportGenerator {
    pub fn new(template: String) -> Self {
        let mut formatters: HashMap<String, Box<dyn Fn(&Value) -> String>> = HashMap::new();
        
        // Add default formatters
        formatters.insert("bytes".to_string(), Box::new(|v| {
            if let Some(bytes) = v.as_u64() {
                format_bytes(bytes)
            } else {
                "N/A".to_string()
            }
        }));
        
        formatters.insert("percentage".to_string(), Box::new(|v| {
            if let Some(ratio) = v.as_f64() {
                format!("{:.2}%", ratio * 100.0)
            } else {
                "N/A".to_string()
            }
        }));
        
        Self {
            template,
            formatters,
        }
    }
    
    pub fn generate_report(&self, data: &Value) -> String {
        let mut result = self.template.clone();
        
        // Simplified template replacement
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{}}}", key);
                let formatted_value = self.format_value(value);
                result = result.replace(&placeholder, &formatted_value);
            }
        }
        
        result
    }
    
    fn format_value(&self, value: &Value) -> String {
        // Choose appropriate formatter based on value type
        match value {
            Value::Number(n) if n.is_u64() => {
                let bytes = n.as_u64().unwrap();
                if bytes > 1024 {
                    format_bytes(bytes)
                } else {
                    bytes.to_string()
                }
            }
            Value::Number(n) if n.is_f64() => {
                let ratio = n.as_f64().unwrap();
                if ratio <= 1.0 {
                    format!("{:.2}%", ratio * 100.0)
                } else {
                    format!("{:.2}", ratio)
                }
            }
            _ => value.to_string(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}
```

## 🎉 Summary

Extending analysis features enables you to:
- Create specialized analyzers
- Integrate existing tools
- Develop plugin systems
- Generate custom reports