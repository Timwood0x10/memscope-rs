# First Memory Analysis

Learn how to generate, interpret, and act on your first memory analysis report with memscope-rs.

## 🎯 Quick Start Analysis

### Step 1: Create a Simple Program

```rust
// src/main.rs
use memscope_rs::{init, track_var, get_global_tracker};
use std::rc::Rc;

fn main() {
    // Initialize tracking
    init();
    
    // Create and track different types of data
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);
    
    let text = String::from("Hello, memory analysis!");
    track_var!(text);
    
    let boxed_data = Box::new(vec![10, 20, 30]);
    track_var!(boxed_data);
    
    let shared_data = Rc::new(String::from("Shared between clones"));
    track_var!(shared_data);
    
    let shared_clone = Rc::clone(&shared_data);
    track_var!(shared_clone);
    
    // Use the data normally
    println!("Numbers: {:?}", numbers);
    println!("Text: {}", text);
    println!("Boxed: {:?}", *boxed_data);
    println!("Shared: {}", *shared_data);
    println!("Rc count: {}", Rc::strong_count(&shared_data));
    
    // Generate analysis report
    let tracker = get_global_tracker();
    
    // Export to multiple formats
    tracker.export_to_json("first_analysis").unwrap();
    tracker.export_memory_analysis("first_analysis.svg").unwrap();
    
    println!("✅ Analysis complete! Check MemoryAnalysis/ directory");
}
```

### Step 2: Run and Generate Reports

```bash
# Run the program
cargo run

# Check generated files
ls MemoryAnalysis/first_analysis/
# You'll see:
# - first_analysis_memory_analysis.json
# - first_analysis_lifetime.json
# - first_analysis_performance.json
# - first_analysis_unsafe_ffi.json
# - first_analysis_complex_types.json
# - first_analysis.svg
```

### Step 3: Generate Interactive HTML Report

```bash
# Generate enhanced HTML report
make html DIR=MemoryAnalysis/first_analysis BASE=first_analysis

# Open the report
open memory_report.html
```

## 📊 Understanding the Results

### Console Output Analysis

```
Numbers: [1, 2, 3, 4, 5]
Text: Hello, memory analysis!
Boxed: [10, 20, 30]
Shared: Shared between clones
Rc count: 2
✅ Analysis complete! Check MemoryAnalysis/ directory
```

**What this tells us**:
- All variables work normally after tracking ✅
- Rc reference count is 2 (original + clone) ✅
- No runtime performance impact ✅

### JSON Report Structure

The main analysis file (`first_analysis_memory_analysis.json`) contains:

```json
{
  "metadata": {
    "export_timestamp": 1691234567890,
    "total_allocations": 5,
    "active_allocations": 5,
    "export_version": "0.1.4"
  },
  "memory_stats": {
    "active_allocations": 5,
    "active_memory": 156,
    "total_allocations": 5,
    "peak_memory": 156,
    "total_deallocations": 0
  },
  "allocations": [
    {
      "ptr": 140712345678912,
      "size": 40,
      "var_name": "numbers",
      "type_name": "Vec<i32>",
      "is_leaked": false
    },
    {
      "ptr": 140712345678952,
      "size": 25,
      "var_name": "text",
      "type_name": "String",
      "is_leaked": false
    }
    // ... more allocations
  ]
}
```

### Key Metrics Explained

| Metric | Value | Meaning |
|--------|-------|---------|
| `active_allocations` | 5 | Currently tracked variables |
| `active_memory` | 156 bytes | Total memory in use |
| `total_allocations` | 5 | Total allocations made |
| `peak_memory` | 156 bytes | Maximum memory used |
| `total_deallocations` | 0 | Variables still active |

## 🔍 Detailed Analysis

### Memory Distribution by Type

From the analysis, you can see:

```
Vec<i32>: 40 bytes (numbers)
String: 25 bytes (text) 
Vec<i32>: 24 bytes (boxed_data contents)
String: ~30 bytes (shared_data contents)
Rc overhead: ~37 bytes (reference counting)
```

### Smart Pointer Analysis

The Rc tracking shows:
- **Original Rc**: Points to shared string data
- **Clone Rc**: Shares the same data pointer
- **Reference count**: 2 (tracked automatically)
- **Memory efficiency**: Data shared, not duplicated

### Lifecycle Analysis

From `first_analysis_lifetime.json`:
- All variables created in main function
- No deallocations yet (program just ended)
- All allocations have similar timestamps
- No memory leaks detected

## 📈 SVG Visualization

The generated `first_analysis.svg` shows:

1. **Memory Usage Timeline** - When allocations occurred
2. **Type Distribution** - Pie chart of memory by type
3. **Allocation Sizes** - Bar chart of individual allocations
4. **Reference Relationships** - Rc/Arc sharing visualization

## 🌐 HTML Interactive Dashboard

The HTML report (`memory_report.html`) provides:

### Overview Section
- **Memory Summary** - Key statistics at a glance
- **Allocation Timeline** - Interactive timeline chart
- **Type Distribution** - Clickable pie chart

### Detailed Analysis
- **Allocation Table** - Sortable, filterable list
- **Smart Pointer Graph** - Reference relationship visualization
- **Performance Metrics** - Timing and efficiency data

### Interactive Features
- **Click on charts** - Drill down into specific data
- **Filter by type** - Focus on specific allocation types
- **Search variables** - Find specific tracked variables
- **Export data** - Download filtered results

## 🎯 Interpreting Results

### Healthy Memory Patterns

✅ **Good signs in your analysis**:
- `total_deallocations` matches expected cleanup
- No memory leaks (`is_leaked: false`)
- Reasonable allocation sizes
- Smart pointer reference counts make sense
- Peak memory is reasonable for your program

### Potential Issues to Watch

⚠️ **Warning signs**:
- High number of small allocations (fragmentation)
- Large peak memory vs. active memory (inefficiency)
- Memory leaks (`is_leaked: true`)
- Unexpected reference counts
- Very long-lived temporary allocations

### Example Analysis

From our first analysis:
```
✅ 5 allocations, all active (no premature cleanup)
✅ 156 bytes total (reasonable for our data)
✅ No memory leaks detected
✅ Rc reference count = 2 (expected: original + clone)
✅ All variables have meaningful names
```

## 🔧 Improving Your Analysis

### Add More Tracking

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    init();
    
    // Track function-level allocations
    let result = process_large_data();
    track_var!(result);
    
    // Track temporary allocations
    {
        let temp_buffer = vec![0; 10000];
        track_var!(temp_buffer);
        // temp_buffer dropped here
    }
    
    // Track collections
    let mut map = std::collections::HashMap::new();
    for i in 0..100 {
        map.insert(i, format!("value_{}", i));
    }
    track_var!(map);
    
    // Generate comprehensive report
    let tracker = get_global_tracker();
    tracker.export_to_json("comprehensive_analysis").unwrap();
}

fn process_large_data() -> Vec<i32> {
    let data = vec![1; 50000];
    track_var!(data);
    
    data.into_iter().map(|x| x * 2).collect()
}
```

### Compare Different Scenarios

```rust
// Scenario 1: Many small allocations
fn many_small_allocations() {
    for i in 0..1000 {
        let small_vec = vec![i];
        track_var!(small_vec);
    }
}

// Scenario 2: Few large allocations  
fn few_large_allocations() {
    let large_vec = vec![0; 100000];
    track_var!(large_vec);
}

fn main() {
    init();
    
    many_small_allocations();
    // Export and compare
    get_global_tracker().export_to_json("many_small").unwrap();
    
    // Reset and try different approach
    few_large_allocations();
    get_global_tracker().export_to_json("few_large").unwrap();
}
```

## 📋 Analysis Checklist

After generating your first analysis, check:

- [ ] ✅ All expected variables are tracked
- [ ] ✅ Memory sizes look reasonable
- [ ] ✅ No unexpected memory leaks
- [ ] ✅ Smart pointer reference counts are correct
- [ ] ✅ HTML report opens and displays correctly
- [ ] ✅ SVG visualization shows expected patterns
- [ ] ✅ JSON data contains detailed allocation info

## 🚀 Next Steps

Now that you've completed your first analysis:

1. **[Memory Analysis Guide](../user-guide/memory-analysis.md)** - Learn advanced analysis techniques
2. **[Export Formats](../user-guide/export-formats.md)** - Understand different output formats
3. **[CLI Tools](../user-guide/cli-tools.md)** - Automate report generation
4. **[Tracking Macros](../user-guide/tracking-macros.md)** - Master all tracking options

## 💡 Key Insights

From your first analysis, you learned:

- **Memory tracking is invisible** - No impact on program behavior
- **Multiple output formats** - JSON data, SVG charts, HTML dashboard
- **Detailed information** - Variable names, types, sizes, relationships
- **Smart pointer tracking** - Automatic reference count monitoring
- **Interactive analysis** - HTML dashboard for deep exploration
- **Production ready** - Safe to use in real applications

Your first analysis is complete! Use these insights to optimize your Rust programs. 🎯