# 🎉 统一API使用指南

## 新的清晰API（推荐使用）

### 导入
```rust
use memscope_rs::export::{MemScopeExporter, export_json, export_binary, export_html, export_auto};
```

### 使用方式

#### 1. 最简单的方式 - 全局函数
```rust
// JSON导出 - 最常用
export_json(tracker, "output.json")?;

// 二进制导出 - 最高效
export_binary(tracker, "output.memscope")?;

// HTML导出 - 最直观
export_html(tracker, "output.html")?;

// 智能导出 - 自动选择最佳格式
export_auto(tracker, "output")?;
```

#### 2. 面向对象方式
```rust
let exporter = MemScopeExporter::new(tracker);

exporter.export_json("output.json")?;
exporter.export_binary("output.memscope")?;
exporter.export_html("output.html")?;
exporter.export_auto("output")?;
```

## 兼容API（继续支持）

```rust
// 这些API继续工作，保证向后兼容
export_user_variables_json(allocations, stats, "output")?;  ✅
export_user_variables_binary(allocations, stats, "output")?; ✅
```

## API对比

### 改进前（混乱）
```rust
// 用户不知道选哪个，有58个导出函数！
export_to_json()?;
export_json_fast()?;
export_optimized_json_files()?;
export_comprehensive_analysis_optimized()?;
export_user_variables_json()?;
export_enhanced_json_with_validation()?;
// ... 还有52个其他函数
```

### 改进后（清晰）
```rust
// 只有4个核心方法，简单明了！
export_json(tracker, "output")?;     // JSON格式
export_binary(tracker, "output")?;   // 二进制格式  
export_html(tracker, "output")?;     // HTML格式
export_auto(tracker, "output")?;     // 智能选择
```

## 性能对比

- `export_json()` - 可读性最好，适合小数据
- `export_binary()` - 性能最好，适合大数据
- `export_html()` - 可视化最好，适合分析
- `export_auto()` - 智能选择，适合不确定的场景

## 迁移指南

### 现有代码（无需修改）
```rust
// 这些代码继续工作
export_user_variables_json(allocations, stats, "output")?;  ✅
tracker.export_user_binary("output")?;                      ✅
tracker.export_full_binary("output")?;                      ✅
```

### 新代码（推荐使用）
```rust
// 新代码使用更简单的API
export_json(tracker, "output")?;  // 替代复杂的导出函数
```