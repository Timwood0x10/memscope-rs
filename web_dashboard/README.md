# 🦀 memscope-rs Web Dashboard

Interactive web interface for Rust memory analysis with unsafe/FFI monitoring.

## 🚀 Quick Start

1. **Generate memory analysis data:**
   ```bash
   # Run any example to generate JSON data
   cargo run --example unsafe_ffi_demo
   cargo run --example complex_lifecycle_showcase
   ```

2. **Start web server:**
   ```bash
   cd web_dashboard
   python3 -m http.server 8000
   ```

3. **Open dashboard:**
   ```
   http://localhost:8000
   ```

## 📊 Features

### Memory Analysis
- **KPI Dashboard**: 6 key performance indicators with circular progress bars
- **Memory Timeline**: Real-time memory usage trends
- **Performance Metrics**: Allocation rates, memory turnover, fragmentation
- **Memory Hierarchy**: Tree view of allocation structure

### Unsafe & FFI Analysis
- **Risk Assessment**: Safety violations and boundary crossings
- **Allocation Tracking**: Unsafe Rust and FFI memory allocations
- **Security Monitoring**: Real-time safety violation detection

## 🔄 Data Sources

- **Sample Data**: Built-in demonstration data
- **Upload JSON**: Manual file upload
- **Auto-detect JSON**: Automatically loads available JSON files
- **Unsafe/FFI Analysis**: Focus on unsafe code and FFI calls
- **Complex Lifecycle**: Multi-layered memory hierarchy analysis

## 📁 Core Files

```
web_dashboard/
├── index.html          # Main dashboard interface
├── dashboard.js        # Core functionality and data handling
├── styles.css          # Professional styling
├── data.json          # Current analysis data
└── *.json             # Additional analysis datasets
```

## 🎯 Usage

1. Select data source from dropdown menu
2. Navigate between "Memory Analysis" and "Unsafe & FFI" tabs
3. Interact with charts and metrics
4. Upload custom JSON files for analysis

## 📈 Data Flow

```
Rust Program → JSON Export → Web Dashboard → Interactive Analysis
```

The system automatically transforms memscope-rs JSON exports into interactive visualizations.