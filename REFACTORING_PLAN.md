# MemScope-RS 重构执行计划

## 🎯 重构目标

基于现有的 `memscope-rs-refactoring.md` 文档，制定详细的执行计划，确保：
- ✅ 不删除任何核心代码
- ✅ 不修改任何输出格式（SVG/JSON）
- ✅ 不影响现有功能
- ✅ 保持完全向后兼容
- ✅ 整合新增的 `export_enhanced_json` 功能

## 📋 当前项目状态分析

### 现有文件结构
```
src/
├── lib.rs                      # 入口文件
├── main.rs                     # 主程序
├── types.rs                    # 1326行，过于庞大 ⚠️
├── tracker.rs                  # 核心跟踪功能 + 新增 export_enhanced_json
├── scope_tracker.rs            # 作用域跟踪
├── allocator.rs               # 内存分配器
├── visualization.rs           # 基础可视化
├── advanced_charts.rs         # 高级图表
├── advanced_analysis.rs       # 高级分析
├── export_enhanced.rs         # 增强导出
├── html_export.rs             # HTML导出
├── optimized_html_export.rs   # 优化HTML导出
├── report_generator.rs        # 报告生成
├── unsafe_ffi_tracker.rs      # Unsafe/FFI跟踪
├── unsafe_ffi_visualization.rs # Unsafe/FFI可视化
├── thread_utils.rs            # 线程工具
└── utils.rs                   # 工具函数
```

### 现有输出功能（必须保持）
1. **JSON输出**:
   - `export_to_json()` - 标准JSON格式
   - `export_enhanced_json()` - 新增的增强JSON格式
2. **SVG输出**:
   - `export_memory_analysis()` - memoryAnalysis.svg
   - `export_lifecycle_timeline()` - lifecycleTimeline.svg
   - `export_unsafe_ffi_dashboard()` - unsafe_ffi_dashboard.svg
3. **HTML输出**:
   - `export_html_dashboard()` - HTML仪表板

## 🗂️ 重构后的目标结构

```
src/
├── lib.rs                    # 入口文件（保持不变）
├── main.rs                   # 主程序（保持不变）
├── allocator.rs             # 内存分配器（保持不变）
├── thread_utils.rs          # 线程工具（保持不变）
├── utils.rs                 # 工具函数（保持不变）
├── tracking.rs              # 合并：tracker.rs + scope_tracker.rs
├── visualization.rs         # 合并：visualization.rs + advanced_charts.rs + unsafe_ffi_visualization.rs
├── analysis.rs              # 合并：advanced_analysis.rs + unsafe_ffi_tracker.rs
├── export.rs                # 合并：export_enhanced.rs + html_export.rs + optimized_html_export.rs + report_generator.rs
└── types/                   # 拆分 types.rs
    ├── mod.rs               # 模块声明
    ├── core.rs              # 核心类型和错误
    ├── allocation.rs        # 分配相关类型
    ├── visualization.rs     # 可视化类型
    └── analysis.rs          # 分析类型
```

## 🚀 执行阶段

### 阶段1: 拆分类型系统 (types.rs → types/)
**目标**: 将1326行的types.rs拆分为模块化结构
**风险**: 低 - 只是重新组织，不改变逻辑
**预计时间**: 1-2小时

#### 步骤1.1: 创建types目录结构
- [ ] 创建 `src/types/` 目录
- [ ] 创建 `src/types/mod.rs`
- [ ] 创建 `src/types/core.rs`
- [ ] 创建 `src/types/allocation.rs`
- [ ] 创建 `src/types/visualization.rs`
- [ ] 创建 `src/types/analysis.rs`

#### 步骤1.2: 迁移核心类型
- [ ] 移动错误类型到 `core.rs`
- [ ] 移动基础trait到 `core.rs`
- [ ] 移动配置类型到 `core.rs`

#### 步骤1.3: 迁移分配相关类型
- [ ] 移动 `AllocationInfo` 到 `allocation.rs`
- [ ] 移动 `MemoryStats` 到 `allocation.rs`
- [ ] 移动分配相关枚举到 `allocation.rs`

#### 步骤1.4: 迁移可视化类型
- [ ] 移动图表类型到 `visualization.rs`
- [ ] 移动SVG相关类型到 `visualization.rs`
- [ ] 移动HTML相关类型到 `visualization.rs`

#### 步骤1.5: 迁移分析类型
- [ ] 移动分析结果类型到 `analysis.rs`
- [ ] 移动统计类型到 `analysis.rs`
- [ ] 移动unsafe/FFI类型到 `analysis.rs`

#### 步骤1.6: 更新导入
- [ ] 更新 `lib.rs` 中的导入
- [ ] 更新所有源文件中的类型导入
- [ ] 确保编译通过

#### 步骤1.7: 验证阶段1
- [ ] 运行所有测试
- [ ] 运行所有示例
- [ ] 验证JSON输出格式不变
- [ ] 验证SVG输出格式不变
- [ ] 验证HTML输出格式不变

### 阶段2: 合并导出功能 (export.rs)
**目标**: 整合所有导出相关功能，包括新的export_enhanced_json
**风险**: 中等 - 涉及核心输出功能
**预计时间**: 2-3小时

#### 步骤2.1: 创建export.rs
- [ ] 创建 `src/export.rs`
- [ ] 从 `tracker.rs` 复制 `export_to_json` 和 `export_enhanced_json`
- [ ] 从 `export_enhanced.rs` 复制所有导出函数
- [ ] 从 `html_export.rs` 复制HTML导出函数
- [ ] 从 `optimized_html_export.rs` 复制优化导出函数
- [ ] 从 `report_generator.rs` 复制报告生成函数

#### 步骤2.2: 整理导出接口
- [ ] 保持所有现有函数签名不变
- [ ] 确保 `export_enhanced_json` 功能完整
- [ ] 添加统一的导出trait（如果需要）

#### 步骤2.3: 更新tracker.rs
- [ ] 在 `tracker.rs` 中保留导出函数的调用接口
- [ ] 将实际实现委托给 `export.rs`
- [ ] 确保向后兼容性

#### 步骤2.4: 更新导入和依赖
- [ ] 更新 `lib.rs` 导出
- [ ] 更新相关模块的导入
- [ ] 确保编译通过

#### 步骤2.5: 验证阶段2
- [ ] 运行所有测试
- [ ] 运行所有示例
- [ ] 特别验证 `export_enhanced_json` 功能
- [ ] 验证所有JSON/SVG/HTML输出格式不变
- [ ] 验证向后兼容性

### 阶段3: 合并可视化功能 (visualization.rs)
**目标**: 整合所有可视化相关功能
**风险**: 中等 - 涉及SVG生成
**预计时间**: 2-3小时

#### 步骤3.1: 扩展visualization.rs
- [ ] 将 `advanced_charts.rs` 的功能合并到 `visualization.rs`
- [ ] 将 `unsafe_ffi_visualization.rs` 的功能合并到 `visualization.rs`
- [ ] 保持所有现有函数和接口

#### 步骤3.2: 组织可视化模块
- [ ] 按功能分组：内存分析、生命周期、unsafe/FFI
- [ ] 保持三个主要SVG输出功能：
  - memoryAnalysis.svg
  - lifecycleTimeline.svg  
  - unsafe_ffi_dashboard.svg

#### 步骤3.3: 更新依赖
- [ ] 更新相关模块的导入
- [ ] 确保编译通过

#### 步骤3.4: 验证阶段3
- [ ] 运行所有测试
- [ ] 验证三个主要SVG输出格式完全不变
- [ ] 验证可视化质量和内容

### 阶段4: 合并分析功能 (analysis.rs)
**目标**: 整合所有分析相关功能
**风险**: 中等 - 涉及核心分析逻辑
**预计时间**: 2-3小时

#### 步骤4.1: 创建analysis.rs
- [ ] 合并 `advanced_analysis.rs` 功能
- [ ] 合并 `unsafe_ffi_tracker.rs` 功能
- [ ] 保持所有分析算法不变

#### 步骤4.2: 组织分析模块
- [ ] 内存分析功能
- [ ] 性能分析功能
- [ ] Unsafe/FFI分析功能
- [ ] 统计分析功能

#### 步骤4.3: 验证阶段4
- [ ] 运行所有测试
- [ ] 验证分析结果准确性
- [ ] 验证JSON输出中的分析数据

### 阶段5: 合并跟踪功能 (tracking.rs)
**目标**: 整合核心跟踪功能
**风险**: 高 - 涉及核心功能
**预计时间**: 3-4小时

#### 步骤5.1: 创建tracking.rs
- [ ] 合并 `tracker.rs` 和 `scope_tracker.rs`
- [ ] 保持所有跟踪逻辑不变
- [ ] 保持 `export_enhanced_json` 功能

#### 步骤5.2: 保持接口兼容
- [ ] 确保 `get_global_tracker()` 功能不变
- [ ] 确保 `track_var!` 宏功能不变
- [ ] 确保所有公共API不变

#### 步骤5.3: 验证阶段5
- [ ] 运行所有测试
- [ ] 运行所有示例
- [ ] 验证跟踪精度和性能

### 阶段6: 清理和优化
**目标**: 清理重构过程中的临时文件和重复代码
**风险**: 低
**预计时间**: 1小时

#### 步骤6.1: 清理旧文件
- [ ] 确认所有功能都已迁移
- [ ] 删除空的旧文件
- [ ] 更新 `Cargo.toml` 如果需要

#### 步骤6.2: 最终验证
- [ ] 运行完整测试套件
- [ ] 运行所有示例
- [ ] 验证所有输出格式
- [ ] 验证性能没有退化

## 🔍 验证检查清单

每个阶段完成后必须验证：

### 功能验证
- [ ] 所有现有测试通过
- [ ] 所有示例程序正常运行
- [ ] `track_var!` 宏正常工作
- [ ] `get_global_tracker()` 正常工作

### 输出验证
- [ ] `export_to_json()` 输出格式不变
- [ ] `export_enhanced_json()` 功能完整
- [ ] memoryAnalysis.svg 格式和内容不变
- [ ] lifecycleTimeline.svg 格式和内容不变
- [ ] unsafe_ffi_dashboard.svg 格式和内容不变
- [ ] HTML仪表板功能不变

### 性能验证
- [ ] 编译时间没有显著增加
- [ ] 运行时性能没有退化
- [ ] 内存使用没有增加

### 兼容性验证
- [ ] 所有公共API保持不变
- [ ] 用户代码无需修改
- [ ] 文档和示例仍然有效

## 🚨 回滚策略

如果任何阶段出现问题：
1. 立即停止当前阶段
2. 使用 `git checkout` 回滚到阶段开始前的状态
3. 分析问题原因
4. 调整计划后重新开始

## 📝 执行记录

- [x] 阶段1完成时间: 2025-01-17 11:15 - Types模块重构完成
- [x] 阶段2完成时间: 2025-01-17 11:16 - Export模块整合完成
- [x] 阶段3完成时间: 2025-01-17 11:20 - 可视化模块整合完成
- [x] 阶段4完成时间: 2025-01-17 11:26 - 分析模块整合完成
- [x] 阶段5完成时间: 2025-01-17 11:29 - 跟踪模块整合完成
- [ ] 阶段6完成时间: ___________

## 🎯 成功标准

重构成功的标准：
1. ✅ 所有现有功能保持不变
2. ✅ 所有输出格式保持不变
3. ✅ 新增的 `export_enhanced_json` 功能正常
4. ✅ 代码结构更清晰，维护性更好
5. ✅ 编译和运行性能没有退化
6. ✅ 完全向后兼容

---

**重要提醒**: 每个步骤执行前都要创建git提交点，确保可以随时回滚！