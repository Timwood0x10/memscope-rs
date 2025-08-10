# Binary Export Enhancement Tasks

## 要求

1. **English-only comments** - 所有代码注释必须是英文
2. **7:3 code-to-comment ratio** - 保持适当的文档化比例
3. **Unified error handling** - 统一错误处理系统
4. **No locks, unwrap, or clone** - 禁止使用锁、unwrap和clone，使用有意义的错误来代替unwrap。
5. **Simple architecture** - 保持架构简洁，专注核心功能
6. **Zero functionality impact** - 禁止影响任何现有功能，特别是数据获取、JSON/binary/HTML导出
7. **Meaningful names** - 所有目录和文件必须有描述性的有意义名称
8. **Use make check** - 禁止使用cargo check，必须使用make check检查完整日志
9.**Use tracking** -  禁止使用println! 使用tracking 来显示日志。
10. 所有的改动基于v5-pre branch，确定没问题了，再进行分支合并。
11. 禁止使用没有意义的变量名字和函数名字。
12. 禁止影响当前json file 的输出内容。
13.禁止产生技术债务。也就说这个task 必须完成binary---josn的优化工作，而不是留下任何一个TODO。
14. 代码应该精简，而不是很冗余，比如说能用match 就不要用if else，要符合rust的编码规范。
15. 要求 0 error，0 warning
16. 架构一定要简单，代码要精简，有简短的代码，完成复杂的需求。
17. 测试代码一定要有意义，测试程序中的核心功能，且保证所有测试必须通过，以及测试运行时间短。
18.对于新增的功能，测试要做到全面。
19.要保证输出的json，和没改动之前的json一致啊，也就是说binary中生存的json file 是5个，要和MemoryAnalysis/binary_demo_example/*.json 一致
20.严禁创建乱七八糟的test files
21. **Full-binary严禁null字段**: full-binary模式下不能出现null字段，既然是全部数据，自然不能出现模糊的null值，也不能是unknown。

## 🎯 目标

1. 提供用户选择：export_user_binary vs export_full_binary
2. **user_binary**: 只保存user variables，文件小（几KB），和系统分配无关
3. **full_binary**: 保存全部数据（用户+系统），文件大（上百KB），就是当前binary的信息
4. **重点优化full_binary解析**: 因为数据量大，需要集成现有优化方案

## 📋 Task List

### Task 1: 添加Binary导出模式选择

**文件**: `src/core/tracker/memory_tracker.rs`

- [x] 添加 `BinaryExportMode` 枚举 - 定义UserOnly和Full两种模式
- [x] 添加 `export_user_binary()` 方法 - 只导出user variables（几KB小文件，无系统分配）
- [x] 添加 `export_full_binary()` 方法 - 导出全部数据（上百KB大文件，当前binary的完整信息）
- [x] 增强 `export_to_binary()` 方法 - 添加可选的模式参数，更加灵活：
  - [x] `export_to_binary_with_mode(path, mode)` - 支持选择user/full模式
  - [x] 底层调用两种方式：严格过滤（user）vs 宽松过滤（full）

### Task 2: 重点优化Full Binary解析（复用现有优化组件）

**文件**: `src/export/binary/parser.rs`

- [ ] 保持 `parse_user_binary_to_json()` 使用现有简单 `reader.read_all()` 策略
- [ ] **重点实现** `parse_full_binary_to_json()` 复用以下现有优化组件：
  - [ ] **按需读取**: 集成 `SelectiveBinaryReader` + `BinaryIndex` 进行快速定位
  - [ ] **流式写入**: 集成 `StreamingJsonWriter` + `StreamingFieldProcessor` 恒定内存
  - [ ] **智能选择**: 集成 `SelectiveJsonExporter` 自适应处理策略
  - [ ] **批量处理**: 使用 `BatchProcessor` 优化大数据集处理
  - [ ] **字段解析**: 复用 `FieldParser` 进行选择性字段提取
  - [ ] **错误恢复**: 集成 `ErrorRecovery` 处理损坏数据
  - [ ] **缓存优化**: 使用 `Cache` + `StringTable` 减少重复数据
  - [ ] **过滤引擎**: 集成 `FilterEngine` 进行高效数据筛选

### Task 3: 扩展Binary格式头部

**文件**: `src/export/binary/mod.rs`

- [ ] 在binary头部添加导出模式标识 (user_only vs full)
- [ ] 添加allocation计数信息 (total, user_count, system_count)
- [ ] 实现自动检测binary类型的功能

### Task 4: Full Binary专用优化（JSON格式一致性保证）

**文件**: `src/export/binary/parser.rs` (修改现有文件，不新建)

- [ ] **JSON格式一致性**: 确保user/full两种模式生成的JSON文件：
  - [ ] 文件命名一致：`{base_name}_memory_analysis.json` 等5个文件
  - [ ] 字段结构一致：相同的JSON schema和字段名
  - [ ] 区别仅在数据量：user模式数据少，full模式数据多
- [ ] **严格禁止Null字段**: 在full-binary模式下严格禁止出现null字段
  - [ ] 既然是全部数据(full)，不能出现模糊的null值
  - [ ] 所有字段必须有明确的值，使用默认值替代null
  - [ ] 确保数据完整性和一致性
- [ ] **性能目标**: 确保full binary解析<300ms（通过复用现有优化组件）

### Task 5: 更新示例展示差异

**文件**: `examples/`

- [ ] 更新 `enhanced_simple_showcase.rs` 展示user_binary（简单快速）
- [ ] 更新 `complex_lifecycle_showcase_binary.rs` 展示full_binary（优化处理大数据）
- [ ] 添加性能对比示例，展示两种模式的差异

## 🏗️ 差异化架构设计

### User Binary（保持简单）

```rust
// 轻量级处理，只有用户变量，文件小（几KB）
user_variables_only -> user_binary -> simple_read_all() -> JSON
```

### Full Binary（重点优化）

```rust
// 重度优化，全部数据（用户+系统），文件大（上百KB）
all_allocations -> full_binary -> optimized_streaming_read() -> JSON
```

### 核心接口

```rust
// Binary export mode enumeration
pub enum BinaryExportMode {
    UserOnly,  // 严格过滤，只有用户变量
    Full,      // 宽松过滤，全部数据
}

impl MemoryTracker {
    pub fn export_to_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()>  // 默认full模式
    pub fn export_to_binary_with_mode<P: AsRef<Path>>(&self, path: P, mode: BinaryExportMode) -> TrackingResult<()>  // 灵活选择
    pub fn export_user_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()>  // 严格过滤
    pub fn export_full_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()>  // 宽松过滤
}

impl BinaryParser {
    pub fn parse_user_binary_to_json<P: AsRef<Path>>(binary_path: P, base_name: &str) -> Result<(), BinaryExportError>  // 简单
    pub fn parse_full_binary_to_json<P: AsRef<Path>>(binary_path: P, base_name: &str) -> Result<(), BinaryExportError>  // 优化
}
```

## 📊 预期效果对比

### User Binary

- **数据内容**: 只有用户变量（var_name.is_some()）
- **文件大小**: 几KB（小文件）
- **JSON文件**: 5个文件，字段结构相同，数据量少
- **处理策略**: 简单直接，使用现有策略
- **性能**: 已经足够快

### Full Binary  

- **数据内容**: 全部数据（用户+系统分配，当前binary的完整信息）
- **文件大小**: 上百KB（大文件）
- **JSON文件**: 5个文件，字段结构相同，数据量大
- **处理策略**: 复用现有优化组件（重点优化对象）
- **性能目标**: <300ms（通过复用8个现有优化组件达成）

### JSON格式一致性保证

- **文件命名**: 两种模式生成相同的5个JSON文件名
- **字段结构**: 完全相同的JSON schema和字段名
- **唯一区别**: 数据量大小（user少，full多）

## 🔄 实施重点

1. **P0**: Task 1 (添加接口)
2. **P0**: Task 2 (重点优化full_binary解析)
3. **P1**: Task 3-4 (格式和优化)
4. **P2**: Task 5 (示例)

---
**核心思路**:

- user_binary = 只有用户变量，几KB小文件，简单处理
- full_binary = 全部数据（当前binary信息），上百KB大文件，重度优化
- 避免过度工程化，针对性解决问题
