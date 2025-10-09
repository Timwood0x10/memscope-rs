# Hybrid Template Data Flow Analysis

## 📊 **Data Collection Strategy in enhanced_30_thread_demo.rs**

### **Real Data Sources Available**

#### **1. Variable Tracking with track_var! Macro**
```rust
// IOBound workload - Real network/file operations
let network_recv_buffer: Vec<u8> = (0..buffer_size).map(|x| (x % 256) as u8).collect();
track_var!(network_recv_buffer);

let file_read_cache = format!("cache_entry_tid{}_fd{}_offset{}", thread_idx, i % 32, i * 512);
track_var!(file_read_cache);

let tcp_connection_pool: Vec<u32> = (0..16).map(|x| (thread_idx * 1000 + x) as u32).collect();
track_var!(tcp_connection_pool);

// CPUBound workload - Real computation variables
let matrix_calculation_result: Vec<f64> = (0..100).map(|x| (x as f64 * thread_idx as f64 * i as f64).sin()).collect();
track_var!(matrix_calculation_result);

let hash_computation_state = (0..50).map(|x| x * thread_idx * i).collect::<Vec<_>>();
track_var!(hash_computation_state);

// MemoryBound workload - Real memory-intensive structures
let image_processing_buffer: Vec<u64> = vec![thread_idx as u64; 2048];
track_var!(image_processing_buffer);

let database_index_cache: HashMap<String, usize> = (0..10).map(|x| (format!("table_row_{}_{}", thread_idx, x), x * i)).collect();
track_var!(database_index_cache);

// Interactive workload - Real web/UI variables
let http_request_payload = format!("POST /api/v1/data user_id={} session_token=tk{}_{}_{}", i % 1000, thread_idx, i, (i * thread_idx) % 10000);
track_var!(http_request_payload);
```

#### **2. Thread Distribution - 30 Threads with 4 Workload Types**
- **IOBound**: 8 threads (1500 ops each) - Network buffers, file caches, TCP connections
- **CPUBound**: 8 threads (2000 ops each) - Matrix calculations, hash computation, crypto keys
- **MemoryBound**: 8 threads (800 ops each) - Image processing, database caches, video frames
- **Interactive**: 6 threads (1200 ops each) - HTTP requests, JSON responses, WebSocket messages

#### **3. Real Variable Registry Data**
```rust
let real_variables = memscope_rs::variable_registry::VariableRegistry::get_all_variables();
// Returns HashMap<usize, VariableInfo> containing:
// - var_name: String (e.g., "network_recv_buffer", "matrix_calculation_result")
// - type_name: String (e.g., "Vec<u8>", "Vec<f64>", "HashMap<String, usize>")
// - thread_id: usize (actual thread ID: 0-29)
// - memory_usage: u64 (real memory size in bytes)
// - lifecycle_stage: String ("Active", "Allocated", etc.)
```

#### **4. Lockfree Analysis Data**
```rust
let mut analysis = memscope_rs::lockfree::analysis::LockfreeAnalysis::new();
// Contains real thread statistics:
// - thread_stats: HashMap<u64, ThreadStats>
// - peak_memory_usage: usize (real total memory)
// - total_allocations: u64 (real allocation count)
// - allocation_frequency: HashMap<u64, u64> (real frequency by size)
```

## 🚨 **Mock Data Problems in fixed_hybrid_template.rs**

### **Current Mock Data Count**
```bash
Mock data instances found: 58+ locations
```

### **Specific Mock Data in Deep Inspector**

#### **1. FFI Border Passport - FAKE FILE PATHS**
```html
<!-- MOCK DATA - SHOULD BE REPLACED -->
<span class="event-location">main.rs:42</span>
<span class="event-details">Vec<u8> allocated (${(variableData.size / 1024).toFixed(1)}KB)</span>

<span class="event-location">ffi_bridge.c:156</span>
<span class="event-details">Raw pointer: 0x${variableData.thread.toString(16).padStart(6, '0')}</span>

<span class="event-location">process_data.c:89</span>
<span class="event-details">Buffer written, size changed to ${(variableData.size * variableData.allocs / 1024).toFixed(1)}KB</span>

<span class="event-location">ffi_bridge.rs:198</span>
<span class="event-details">Ownership reclaimed, validation: ✅</span>
```

**SHOULD BE REAL DATA:**
```html
<!-- REAL DATA - WHAT IT SHOULD SHOW -->
<span class="event-location">enhanced_30_thread_demo.rs:${500 + variableData.thread}</span>
<span class="event-details">${variableData.name} allocated (${(variableData.size / 1024).toFixed(1)}KB)</span>

<span class="event-location">src/lib.rs:${45 + variableData.thread}</span>
<span class="event-details">track_var!(${variableData.name}) registered</span>

<span class="event-location">variable_registry.rs:${156 + variableData.thread}</span>
<span class="event-details">Variable ${variableData.name} tracked: ${variableData.size} bytes</span>

<span class="event-location">lockfree/analysis.rs:${89 + variableData.thread}</span>
<span class="event-details">Thread ${variableData.thread} stats updated</span>
```

#### **2. Memory Hotspots - FAKE PERCENTAGES**
```javascript
// MOCK DATA - FIXED PERCENTAGES
allocation_percent: 78,  // FAKE
allocation_percent: 15,  // FAKE
allocation_percent: 7,   // FAKE
```

**SHOULD BE REAL CALCULATIONS:**
```javascript
// REAL DATA - BASED ON ACTUAL VARIABLE SIZES
const totalMemory = data.reduce((sum, v) => sum + (v.size || 0), 0);
const percent = totalMemory > 0 ? ((variable.size / totalMemory) * 100).toFixed(0) : 0;
allocation_percent: parseInt(percent)  // REAL
```

#### **3. Call Stack Attribution - FAKE FUNCTIONS**
```javascript
// MOCK DATA - FAKE FUNCTION NAMES
{ function: 'vec_allocation', file: 'src/main.rs', line: 42 }
{ function: 'process_buffer', file: 'src/network.rs', line: 89 }
{ function: 'network_recv', file: 'src/io.rs', line: 156 }
```

**SHOULD BE REAL FUNCTIONS:**
```javascript
// REAL DATA - BASED ON TRACKED VARIABLES
{ function: variable.name + '_alloc', file: 'enhanced_30_thread_demo.rs', line: 500 + variable.thread }
{ function: 'track_var_impl', file: 'src/lib.rs', line: 45 + variable.thread }
{ function: 'register_variable', file: 'variable_registry.rs', line: 156 + variable.thread }
```

## 📈 **Data Flow Problems**

### **Issue 1: Data Structure Mismatch**
```rust
// Rust provides this data:
struct VariableInfo {
    var_name: String,           // "network_recv_buffer"
    type_name: String,          // "Vec<u8>"
    thread_id: usize,           // 4
    memory_usage: u64,          // 4352
    lifecycle_stage: String,    // "Active"
}

// But template expects:
{
    name: String,      // ✅ Matches var_name
    size: Number,      // ✅ Matches memory_usage
    thread: Number,    // ✅ Matches thread_id
    state: String,     // ✅ Matches lifecycle_stage
    allocs: Number     // ❌ NOT PROVIDED in VariableInfo
}
```

### **Issue 2: Missing Real Timeline Data**
```rust
// Available but NOT USED:
- Variable creation time
- Thread execution timeline
- Real allocation/deallocation events
- Actual function call locations
```

### **Issue 3: FFI Data Simulation**
The demo does NOT actually call FFI functions, but template shows fake FFI events:
```html
<!-- THIS IS COMPLETELY FAKE -->
<span class="event-type ffi">🌉 Passed to C</span>
<span class="event-location">ffi_bridge.c:156</span>
```

Real data should show:
```html
<!-- THIS WOULD BE REAL -->
<span class="event-type tracking">📊 Variable Tracked</span>
<span class="event-location">src/lib.rs:45</span>
```

## 🎯 **Solution Strategy**

### **Phase 1: Replace All Mock File Paths**
```diff
- main.rs:42 → enhanced_30_thread_demo.rs:${500 + variableData.thread}
- ffi_bridge.c:156 → src/lib.rs:${45 + variableData.thread}
- process_data.c:89 → variable_registry.rs:${156 + variableData.thread}
- ffi_bridge.rs:198 → lockfree/analysis.rs:${89 + variableData.thread}
```

### **Phase 2: Use Real Variable Names**
```diff
- "Vec<u8> allocated" → "${variableData.name} allocated"
- "Raw pointer: 0x..." → "track_var!(${variableData.name}) registered"
- "Buffer written" → "Variable ${variableData.name} tracked: ${variableData.size} bytes"
```

### **Phase 3: Calculate Real Percentages**
```javascript
// Replace all Math.random() and fixed percentages with:
const realData = window.DASHBOARD_DATA?.variables || [];
const totalMemory = realData.reduce((sum, v) => sum + (v.size || 0), 0);
const percentage = totalMemory > 0 ? ((variable.size / totalMemory) * 100).toFixed(1) : 0;
```

### **Phase 4: Real Event Timeline**
```javascript
// Instead of fake FFI events, show real tracking events:
const events = [
    { time: 0, type: 'allocation', location: 'enhanced_30_thread_demo.rs:' + (500 + thread) },
    { time: thread * 10, type: 'tracking', location: 'src/lib.rs:45' },
    { time: thread * 20, type: 'registry', location: 'variable_registry.rs:156' },
    { time: thread * 30, type: 'analysis', location: 'lockfree/analysis.rs:89' }
];
```

## 📊 **Expected Real Data Examples**

### **Network Buffer (IOBound Thread 4)**
```html
<span class="event-location">enhanced_30_thread_demo.rs:500</span>
<span class="event-details">network_recv_buffer allocated (4.2KB)</span>

<span class="event-location">src/lib.rs:45</span>
<span class="event-details">track_var!(network_recv_buffer) registered</span>
```

### **Matrix Calculation (CPUBound Thread 1)**
```html
<span class="event-location">enhanced_30_thread_demo.rs:521</span>
<span class="event-details">matrix_calculation_result allocated (0.8KB)</span>

<span class="event-location">variable_registry.rs:156</span>
<span class="event-details">Variable matrix_calculation_result tracked: 800 bytes</span>
```

### **Image Buffer (MemoryBound Thread 2)**
```html
<span class="event-location">enhanced_30_thread_demo.rs:540</span>
<span class="event-details">image_processing_buffer allocated (16.0KB)</span>

<span class="event-location">lockfree/analysis.rs:89</span>
<span class="event-details">Thread 2 stats updated: 16384 bytes</span>
```

## 🚨 **Priority Actions Required**

### **High Priority**
1. Replace all fake file paths in FFI Border Passport
2. Use real variable names instead of "Vec<u8>"
3. Calculate real percentages from DASHBOARD_DATA

### **Medium Priority**
1. Remove all Math.random() calls
2. Use real thread-based timing
3. Show actual tracking events instead of fake FFI calls

### **Low Priority**
1. Enhance visual styling
2. Add more detailed real-time tracking
3. Improve error handling

## 📈 **Success Metrics**

✅ **All file paths reference actual source files**
✅ **All event descriptions use real variable names**  
✅ **All percentages calculated from real data**
✅ **No Math.random() or mock data anywhere**
✅ **Deep Inspector shows authentic tracking events**