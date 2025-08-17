# 数据收集策略与代码质量分析报告

## 📋 概述

本报告全面分析了memscope项目中的数据收集策略、接口设计以及代码质量指标。

## 🔍 1. 数据收集策略分析

### 1.1 导出接口 (Export Interfaces)

**总计**: 76 个导出接口

#### pub fn export_to_json_optimized<P: AsRef<std::path::Path>>(
- **文件**: `src/lib.rs`
- **行号**: 1246
- **代码上下文**:
```rust
    1243: 
    1244: impl MemoryTracker {
    1245:     /// Export tracking data with complex type optimization (separate files for better performance)
>>> 1246:     pub fn export_to_json_optimized<P: AsRef<std::path::Path>>(
    1247:         &self,
    1248:         path: P,
    1249:     ) -> TrackingResult<crate::export::complex_type_export::ComplexTypeExportResult> {
```

#### fn export_final_snapshot(base_path: &str) -> TrackingResult<()> {
- **文件**: `src/lib.rs`
- **行号**: 1525
- **代码上下文**:
```rust
    1522: }
    1523: 
    1524: /// Export final memory snapshot with complete lifecycle data
>>> 1525: fn export_final_snapshot(base_path: &str) -> TrackingResult<()> {
    1526:     let tracker = get_global_tracker();
    1527: 
    1528:     // Force a final garbage collection attempt to capture any remaining deallocations
```

#### pub fn export_partial(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
- **文件**: `src/core/error.rs`
- **行号**: 135
- **代码上下文**:
```rust
     132:     }
     133: 
     134:     /// Create an export error with partial success
>>>  135:     pub fn export_partial(format: impl Into<Arc<str>>, message: impl Into<Arc<str>>) -> Self {
     136:         Self::Export {
     137:             format: format.into(),
     138:             message: message.into(),
```

#### pub fn export_to_json(&self, export_data: &LifecycleExportData) -> serde_json::Result<String> {
- **文件**: `src/core/lifecycle_summary.rs`
- **行号**: 452
- **代码上下文**:
```rust
     449:     }
     450: 
     451:     /// Export lifecycle data to JSON string
>>>  452:     pub fn export_to_json(&self, export_data: &LifecycleExportData) -> serde_json::Result<String> {
     453:         serde_json::to_string_pretty(export_data)
     454:     }
     455: }
```

#### pub fn export_to_json(&self) -> serde_json::Result<String> {
- **文件**: `src/core/ownership_history.rs`
- **行号**: 351
- **代码上下文**:
```rust
     348:     }
     349: 
     350:     /// Export ownership history to JSON format
>>>  351:     pub fn export_to_json(&self) -> serde_json::Result<String> {
     352:         let export_data = OwnershipHistoryExport {
     353:             summaries: self.ownership_summaries.clone(),
     354:             detailed_events: self.ownership_events.clone(),
```

#### pub fn export_ownership_history(&self) -> Result<String, String> {
- **文件**: `src/core/tracker/allocation_tracking.rs`
- **行号**: 705
- **代码上下文**:
```rust
     702:     }
     703: 
     704:     /// Export ownership history to JSON
>>>  705:     pub fn export_ownership_history(&self) -> Result<String, String> {
     706:         if let Ok(ownership_history) = self.ownership_history.try_lock() {
     707:             ownership_history.export_to_json().map_err(|e| e.to_string())
     708:         } else {
```

#### pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 203
- **代码上下文**:
```rust
     200:     ///
     201:     /// # Arguments
     202:     /// * `path` - Output filename for the memory analysis SVG file (recommended: "program_name_memory_analysis.svg")
>>>  203:     pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
     204:         let output_path = self.ensure_memory_analysis_path(path);
     205:         crate::export::visualization::export_memory_analysis(self, output_path)
     206:     }
```

#### pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 237
- **代码上下文**:
```rust
     234:     /// tracker.export_to_binary("my_program")?;
     235:     /// // Creates: MemoryAnalysis/my_program.memscope
     236:     /// ```
>>>  237:     pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
     238:         // Maintain compatibility by defaulting to user-only export
     239:         self.export_user_binary(path)
     240:     }
```

#### pub fn export_to_binary_with_mode<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 260
- **代码上下文**:
```rust
     257:     /// // Export all data (large, complete)
     258:     /// tracker.export_to_binary_with_mode("my_program_full", BinaryExportMode::Full)?;
     259:     /// ```
>>>  260:     pub fn export_to_binary_with_mode<P: AsRef<std::path::Path>>(
     261:         &self,
     262:         path: P,
     263:         mode: BinaryExportMode,
```

#### pub fn export_user_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 291
- **代码上下文**:
```rust
     288:     /// tracker.export_user_binary("my_program_user")?;
     289:     /// // Creates: MemoryAnalysis/my_program_user.memscope (user variables only)
     290:     /// ```
>>>  291:     pub fn export_user_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
     292:         let output_path = self.ensure_memscope_path(path);
     293: 
     294:         tracing::info!("Starting user binary export to: {}", output_path.display());
```

#### pub fn export_full_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 335
- **代码上下文**:
```rust
     332:     /// tracker.export_full_binary("my_program_full")?;
     333:     /// // Creates: MemoryAnalysis/my_program_full.memscope
     334:     /// ```
>>>  335:     pub fn export_full_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
     336:         let output_path = self.ensure_memscope_path(path);
     337: 
     338:         tracing::info!("Starting full binary export to: {}", output_path.display());
```

#### pub fn export_binary_to_html<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 437
- **代码上下文**:
```rust
     434:     }
     435: 
     436:     /// Alias for parse_binary_to_html for backward compatibility
>>>  437:     pub fn export_binary_to_html<P: AsRef<std::path::Path>>(
     438:         binary_path: P,
     439:         html_path: P,
     440:     ) -> TrackingResult<()> {
```

#### pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 450
- **代码上下文**:
```rust
     447:     ///
     448:     /// # Arguments
     449:     /// * `path` - Output filename for the lifecycle timeline SVG file (recommended: "program_name_lifecycle.svg")
>>>  450:     pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
     451:         &self,
     452:         path: P,
     453:     ) -> TrackingResult<()> {
```

#### pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/export_json.rs`
- **行号**: 39
- **代码上下文**:
```rust
      36:     /// - **Data**: ALL allocations including system internals
      37:     /// - **Use case**: Deep debugging, memory leak investigation, system analysis
      38:     /// - **⚠️ Warning**: Very slow, generates large files, may impact application performance
>>>   39:     pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
      40:         // Ensure output goes to MemoryAnalysis directory
      41:         let output_path = self.ensure_memory_analysis_path(path);
      42: 
```

#### pub fn export_to_json_with_options<P: AsRef<Path>>(
- **文件**: `src/core/tracker/export_json.rs`
- **行号**: 72
- **代码上下文**:
```rust
      69:     ///     .verbose_logging(true);
      70:     /// tracker.export_to_json_with_options("debug_output", options)?;
      71:     /// ```
>>>   72:     pub fn export_to_json_with_options<P: AsRef<Path>>(
      73:         &self,
      74:         path: P,
      75:         options: crate::core::tracker::config::ExportOptions,
```

#### fn export_to_json_with_optimized_options_internal<P: AsRef<Path>>(
- **文件**: `src/core/tracker/export_json.rs`
- **行号**: 138
- **代码上下文**:
```rust
     135:     }
     136: 
     137:     /// Internal method to handle export with optimized options
>>>  138:     fn export_to_json_with_optimized_options_internal<P: AsRef<Path>>(
     139:         &self,
     140:         path: P,
     141:         options: OptimizedExportOptions,
```

#### pub fn export_interactive_dashboard<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/export_html.rs`
- **行号**: 47
- **代码上下文**:
```rust
      44:     /// HTML export is generally fast (1-3 seconds) as it focuses on visualization
      45:     /// rather than comprehensive data processing. The file size depends on the
      46:     /// amount of tracking data but is typically 1-10MB.
>>>   47:     pub fn export_interactive_dashboard<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
      48:         let output_path = self.ensure_memory_analysis_path(path);
      49: 
      50:         // Delegate to the specialized HTML export module
```

#### pub fn export_interactive_dashboard_with_ffi<P: AsRef<Path>>(
- **文件**: `src/core/tracker/export_html.rs`
- **行号**: 75
- **代码上下文**:
```rust
      72:     ///     Some(&unsafe_tracker)
      73:     /// )?;
      74:     /// ```
>>>   75:     pub fn export_interactive_dashboard_with_ffi<P: AsRef<Path>>(
      76:         &self,
      77:         path: P,
      78:         unsafe_ffi_tracker: Option<&crate::analysis::unsafe_ffi_tracker::UnsafeFFITracker>,
```

#### pub fn export_html_summary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/core/tracker/export_html.rs`
- **行号**: 108
- **代码上下文**:
```rust
     105:     /// - **Key metrics**: Focus on most important memory statistics
     106:     /// - **Executive summary**: High-level insights and recommendations
     107:     /// - **Quick loading**: Optimized for fast viewing and sharing
>>>  108:     pub fn export_html_summary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
     109:         let output_path = self.ensure_memory_analysis_path(path);
     110: 
     111:         // Generate summary data
```

#### pub fn export_memory_analysis<P: AsRef<Path>>(
- **文件**: `src/export/visualization.rs`
- **行号**: 20
- **代码上下文**:
```rust
      17: };
      18: 
      19: /// Export memory analysis visualization showing variable names, types, and usage
>>>   20: pub fn export_memory_analysis<P: AsRef<Path>>(
      21:     tracker: &MemoryTracker,
      22:     path: P,
      23: ) -> TrackingResult<()> {
```

#### pub fn export_lifecycle_timeline<P: AsRef<Path>>(
- **文件**: `src/export/visualization.rs`
- **行号**: 55
- **代码上下文**:
```rust
      52: }
      53: 
      54: /// Export interactive lifecycle timeline showing variable lifecycles and relationships
>>>   55: pub fn export_lifecycle_timeline<P: AsRef<Path>>(
      56:     tracker: &MemoryTracker,
      57:     path: P,
      58: ) -> TrackingResult<()> {
```

#### fn export_scope_analysis_json(
- **文件**: `src/export/visualization.rs`
- **行号**: 719
- **代码上下文**:
```rust
     716: }
     717: 
     718: /// Export complete scope analysis to JSON file
>>>  719: fn export_scope_analysis_json(
     720:     all_scopes: &HashMap<String, Vec<&AllocationInfo>>,
     721:     displayed_scopes: &[(String, Vec<&AllocationInfo>)],
     722: ) -> TrackingResult<()> {
```

#### pub fn export_unsafe_ffi_dashboard<P: AsRef<Path>>(
- **文件**: `src/export/visualization.rs`
- **行号**: 1348
- **代码上下文**:
```rust
    1345: }
    1346: 
    1347: /// Export comprehensive unsafe/FFI memory analysis to dedicated SVG
>>> 1348: pub fn export_unsafe_ffi_dashboard<P: AsRef<Path>>(
    1349:     tracker: &UnsafeFFITracker,
    1350:     path: P,
    1351: ) -> TrackingResult<()> {
```

#### pub fn export_comprehensive_analysis_optimized<P: AsRef<Path>>(
- **文件**: `src/export/complex_type_export.rs`
- **行号**: 150
- **代码上下文**:
```rust
     147: }
     148: 
     149: /// Export comprehensive analysis with complex type separation
>>>  150: pub fn export_comprehensive_analysis_optimized<P: AsRef<Path>>(
     151:     report: &ComprehensiveAnalysisReport,
     152:     allocations: &[AllocationInfo],
     153:     base_path: P,
```

#### fn export_json_data<T: Serialize>(
- **文件**: `src/export/complex_type_export.rs`
- **行号**: 311
- **代码上下文**:
```rust
     308: }
     309: 
     310: /// Export JSON data with configuration options
>>>  311: fn export_json_data<T: Serialize>(
     312:     data: &T,
     313:     path: &Path,
     314:     config: &ComplexTypeExportConfig,
```

#### pub fn export_to_json_fast<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 635
- **代码上下文**:
```rust
     632:     /// - Uses parallel shard processing for large datasets
     633:     /// - Automatically switches to fast export coordinator when beneficial
     634:     /// - Reduces export time by 60-80% for complex programs
>>>  635:     pub fn export_to_json_fast<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
     636:         let options = OptimizedExportOptions::with_optimization_level(OptimizationLevel::Low)
     637:             .parallel_processing(true)
     638:             .streaming_writer(false)
```

#### pub fn export_to_json_comprehensive<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 659
- **代码上下文**:
```rust
     656:     /// // Comprehensive export for security audit
     657:     /// tracker.export_to_json_comprehensive("security_audit")?;
     658:     /// ```
>>>  659:     pub fn export_to_json_comprehensive<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
     660:         let options = OptimizedExportOptions::with_optimization_level(OptimizationLevel::Maximum)
     661:             .security_analysis(true)
     662:             .adaptive_optimization(true);
```

#### pub fn export_to_json_with_optimized_options<P: AsRef<Path>>(
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 803
- **代码上下文**:
```rust
     800:     ///     .schema_validation(true);
     801:     /// tracker.export_to_json_with_options("output/analysis", options)?;
     802:     /// ```
>>>  803:     pub fn export_to_json_with_optimized_options<P: AsRef<Path>>(
     804:         &self,
     805:         base_path: P,
     806:         options: OptimizedExportOptions,
```

#### pub fn export_optimized_json_files<P: AsRef<Path>>(&self, base_path: P) -> TrackingResult<()> {
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 1449
- **代码上下文**:
```rust
    1446: /// Ultra-fast export implementation (legacy methods for backward compatibility)
    1447: impl MemoryTracker {
    1448:     /// Optimized export to standard 4 JSON files (replaces export_separated_json_simple)
>>> 1449:     pub fn export_optimized_json_files<P: AsRef<Path>>(&self, base_path: P) -> TrackingResult<()> {
    1450:         let options = OptimizedExportOptions::default();
    1451:         self.export_optimized_json_files_with_options(base_path, options)
    1452:     }
```

#### pub fn export_optimized_json_files_with_complex_types<P: AsRef<Path>>(
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 1455
- **代码上下文**:
```rust
    1452:     }
    1453: 
    1454:     /// Export to 5 JSON files including complex types analysis
>>> 1455:     pub fn export_optimized_json_files_with_complex_types<P: AsRef<Path>>(
    1456:         &self,
    1457:         base_path: P,
    1458:     ) -> TrackingResult<()> {
```

#### pub fn export_optimized_json_files_with_options<P: AsRef<Path>>(
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 1468
- **代码上下文**:
```rust
    1465:     }
    1466: 
    1467:     /// Optimized export to standard 4 JSON files with custom options
>>> 1468:     pub fn export_optimized_json_files_with_options<P: AsRef<Path>>(
    1469:         &self,
    1470:         base_path: P,
    1471:         options: OptimizedExportOptions,
```

#### pub fn export_extensible_json_files<P: AsRef<Path>>(
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 1547
- **代码上下文**:
```rust
    1544:     }
    1545: 
    1546:     /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
>>> 1547:     pub fn export_extensible_json_files<P: AsRef<Path>>(
    1548:         &self,
    1549:         base_path: P,
    1550:         file_types: &[JsonFileType],
```

#### pub fn export_extensible_json_files_with_options<P: AsRef<Path>>(
- **文件**: `src/export/optimized_json_export.rs`
- **行号**: 1557
- **代码上下文**:
```rust
    1554:     }
    1555: 
    1556:     /// A generic export method reserved for future expansion. can easily add a 5th and 6th JSON file
>>> 1557:     pub fn export_extensible_json_files_with_options<P: AsRef<Path>>(
    1558:         &self,
    1559:         base_path: P,
    1560:         file_types: &[JsonFileType],
```

#### pub fn export_fast<P: AsRef<Path>>(
- **文件**: `src/export/export_modes.rs`
- **行号**: 33
- **代码上下文**:
```rust
      30: }
      31: 
      32: /// Fast Future: pure export, no validation
>>>   33: pub fn export_fast<P: AsRef<Path>>(
      34:     output_path: P,
      35: ) -> Pin<Box<dyn Future<Output = FastExportResult> + Send>> {
      36:     let path = output_path.as_ref().to_path_buf();
```

#### pub fn export_with_validation<P: AsRef<Path>>(
- **文件**: `src/export/export_modes.rs`
- **行号**: 58
- **代码上下文**:
```rust
      55: }
      56: 
      57: /// Export with validation: export first, then validate
>>>   58: pub fn export_with_validation<P: AsRef<Path>>(
      59:     output_path: P,
      60: ) -> Pin<Box<dyn Future<Output = NormalExportResult> + Send>> {
      61:     let path = output_path.as_ref().to_path_buf();
```

#### pub async fn export_without_validation<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 250
- **代码上下文**:
```rust
     247:     }
     248: 
     249:     /// Execute export without validation (for both fast and normal modes)
>>>  250:     pub async fn export_without_validation<P: AsRef<Path>>(
     251:         &mut self,
     252:         output_path: P,
     253:     ) -> TrackingResult<CompleteExportStats> {
```

#### pub async fn export_with_mode<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 292
- **代码上下文**:
```rust
     289:     }
     290: 
     291:     /// Export with mode-specific behavior and optional deferred validation
>>>  292:     pub async fn export_with_mode<P: AsRef<Path>>(
     293:         &mut self,
     294:         output_path: P,
     295:     ) -> TrackingResult<(CompleteExportStats, Option<DeferredValidation>)> {
```

#### async fn export_with_inline_validation<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 366
- **代码上下文**:
```rust
     363:     }
     364: 
     365:     /// Export with inline validation (for slow mode)
>>>  366:     async fn export_with_inline_validation<P: AsRef<Path>>(
     367:         &mut self,
     368:         output_path: P,
     369:     ) -> TrackingResult<CompleteExportStats> {
```

#### pub fn export_fast<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 457
- **代码上下文**:
```rust
     454:     }
     455: 
     456:     /// Execute fast export
>>>  457:     pub fn export_fast<P: AsRef<Path>>(
     458:         &mut self,
     459:         output_path: P,
     460:     ) -> TrackingResult<CompleteExportStats> {
```

#### pub fn export_fast_with_progress<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 465
- **代码上下文**:
```rust
     462:     }
     463: 
     464:     /// Execute fast export with progress monitoring
>>>  465:     pub fn export_fast_with_progress<P: AsRef<Path>>(
     466:         &mut self,
     467:         output_path: P,
     468:         progress_callback: Option<ProgressCallback>,
```

#### pub fn export_fast<P: AsRef<Path>>(output_path: P) -> TrackingResult<CompleteExportStats> {
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 1158
- **代码上下文**:
```rust
    1155: }
    1156: 
    1157: /// Convenience function: Export to specified path
>>> 1158: pub fn export_fast<P: AsRef<Path>>(output_path: P) -> TrackingResult<CompleteExportStats> {
    1159:     let mut coordinator = FastExportCoordinator::default();
    1160:     coordinator.export_fast(output_path)
    1161: }
```

#### pub fn export_fast_with_config<P: AsRef<Path>>(
- **文件**: `src/export/fast_export_coordinator.rs`
- **行号**: 1164
- **代码上下文**:
```rust
    1161: }
    1162: 
    1163: /// Convenience function: Export with custom config
>>> 1164: pub fn export_fast_with_config<P: AsRef<Path>>(
    1165:     output_path: P,
    1166:     config: FastExportConfig,
    1167: ) -> TrackingResult<CompleteExportStats> {
```

#### pub fn export_interactive_html<P: AsRef<Path>>(
- **文件**: `src/export/html_export.rs`
- **行号**: 18
- **代码上下文**:
```rust
      15: const JS_CONTENT: &str = include_str!("../../templates/script.js");
      16: 
      17: /// Export comprehensive interactive HTML report
>>>   18: pub fn export_interactive_html<P: AsRef<Path>>(
      19:     tracker: &MemoryTracker,
      20:     unsafe_ffi_tracker: Option<&UnsafeFFITracker>,
      21:     path: P,
```

#### pub fn export_enhanced_svg<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
- **文件**: `src/export/export_enhanced.rs`
- **行号**: 520
- **代码上下文**:
```rust
     517: }
     518: 
     519: /// Enhanced SVG export with comprehensive visualization
>>>  520: pub fn export_enhanced_svg<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<()> {
     521:     let path = path.as_ref();
     522: 
     523:     // Create parent directories if needed
```

#### pub fn export_efficiency(&self) -> f64 {
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 115
- **代码上下文**:
```rust
     112:     }
     113: 
     114:     /// Calculate export efficiency (files per second)
>>>  115:     pub fn export_efficiency(&self) -> f64 {
     116:         if self.total_export_time_us == 0 {
     117:             0.0
     118:         } else {
```

#### pub fn export_to_json_selective<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 192
- **代码上下文**:
```rust
     189:     }
     190: 
     191:     /// Export a single binary file to JSON with selective fields
>>>  192:     pub fn export_to_json_selective<P: AsRef<Path>, Q: AsRef<Path>>(
     193:         &mut self,
     194:         binary_path: P,
     195:         json_path: Q,
```

#### pub fn export_multiple_json_types<P: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 316
- **代码上下文**:
```rust
     313:     }
     314: 
     315:     /// Export multiple binary files to JSON in parallel
>>>  316:     pub fn export_multiple_json_types<P: AsRef<Path>>(
     317:         &mut self,
     318:         binary_files: &[(P, P)], // (binary_path, json_path) pairs
     319:         requested_fields: &HashSet<AllocationField>,
```

#### pub fn export_memory_analysis_json<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 358
- **代码上下文**:
```rust
     355:     }
     356: 
     357:     /// Export to memory_analysis.json format (compatible with existing format)
>>>  358:     pub fn export_memory_analysis_json<P: AsRef<Path>, Q: AsRef<Path>>(
     359:         &mut self,
     360:         binary_path: P,
     361:         json_path: Q,
```

#### pub fn export_lifetime_json<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 381
- **代码上下文**:
```rust
     378:     }
     379: 
     380:     /// Export to lifetime.json format (compatible with existing format)
>>>  381:     pub fn export_lifetime_json<P: AsRef<Path>, Q: AsRef<Path>>(
     382:         &mut self,
     383:         binary_path: P,
     384:         json_path: Q,
```

#### pub fn export_performance_json<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 460
- **代码上下文**:
```rust
     457:     }
     458: 
     459:     /// Export to performance.json format (compatible with existing format)
>>>  460:     pub fn export_performance_json<P: AsRef<Path>, Q: AsRef<Path>>(
     461:         &mut self,
     462:         binary_path: P,
     463:         json_path: Q,
```

#### pub fn export_unsafe_ffi_json<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 481
- **代码上下文**:
```rust
     478:     }
     479: 
     480:     /// Export to unsafe_ffi.json format (compatible with existing format)
>>>  481:     pub fn export_unsafe_ffi_json<P: AsRef<Path>, Q: AsRef<Path>>(
     482:         &mut self,
     483:         binary_path: P,
     484:         json_path: Q,
```

#### pub fn export_complex_types_json<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 566
- **代码上下文**:
```rust
     563:     }
     564: 
     565:     /// Export to complex_types.json format (compatible with existing format)
>>>  566:     pub fn export_complex_types_json<P: AsRef<Path>, Q: AsRef<Path>>(
     567:         &mut self,
     568:         binary_path: P,
     569:         json_path: Q,
```

#### pub fn export_all_standard_json_types<P: AsRef<Path>, Q: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 649
- **代码上下文**:
```rust
     646:     }
     647: 
     648:     /// Export all 5 JSON types in the standard format (compatible with existing output)
>>>  649:     pub fn export_all_standard_json_types<P: AsRef<Path>, Q: AsRef<Path>>(
     650:         &mut self,
     651:         binary_path: P,
     652:         output_dir: Q,
```

#### pub fn export_with_auto_field_selection<P: AsRef<Path>>(
- **文件**: `src/export/binary/selective_json_exporter.rs`
- **行号**: 682
- **代码上下文**:
```rust
     679:     }
     680: 
     681:     /// Export with automatic field selection based on file analysis
>>>  682:     pub fn export_with_auto_field_selection<P: AsRef<Path>>(
     683:         &mut self,
     684:         binary_path: P,
     685:         json_path: P,
```

#### pub fn export_to_binary<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 129
- **代码上下文**:
```rust
     126: use std::path::Path;
     127: 
     128: /// Export allocation information to binary format with default configuration
>>>  129: pub fn export_to_binary<P: AsRef<Path>>(
     130:     allocations: &[AllocationInfo],
     131:     path: P,
     132: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_to_binary_with_mode<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 137
- **代码上下文**:
```rust
     134: }
     135: 
     136: /// Export allocation information to binary format with enhanced header
>>>  137: pub fn export_to_binary_with_mode<P: AsRef<Path>>(
     138:     allocations: &[AllocationInfo],
     139:     path: P,
     140:     export_mode: BinaryExportMode,
```

#### pub fn export_to_binary_with_config<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 166
- **代码上下文**:
```rust
     163: }
     164: 
     165: /// Export allocation information to binary format with custom configuration
>>>  166: pub fn export_to_binary_with_config<P: AsRef<Path>>(
     167:     allocations: &[AllocationInfo],
     168:     path: P,
     169:     config: &BinaryExportConfig,
```

#### pub fn export_binary_to_html_dashboard<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 389
- **代码上下文**:
```rust
     386: /// # Ok(())
     387: /// # }
     388: /// ```
>>>  389: pub fn export_binary_to_html_dashboard<P: AsRef<Path>>(
     390:     binary_path: P,
     391:     _output_path: P,
     392:     project_name: &str,
```

#### pub fn export_binary<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 136
- **代码上下文**:
```rust
     133: /// - Intelligent buffer management
     134: /// - Zero impact on existing JSON-only performance
     135: /// - Shared data reading to avoid duplicate I/O
>>>  136: pub fn export_binary<P: AsRef<Path>>(
     137:     binary_path: P,
     138:     base_name: &str,
     139:     format: BinaryOutputFormat,
```

#### pub fn export_binary_optimized<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 145
- **代码上下文**:
```rust
     142: }
     143: 
     144: /// **[OPTIMIZED IMPLEMENTATION]** Internal optimized binary export implementation
>>>  145: pub fn export_binary_optimized<P: AsRef<Path>>(
     146:     binary_path: P,
     147:     base_name: &str,
     148:     format: BinaryOutputFormat,
```

#### pub fn export_binary_with_format<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 241
- **代码上下文**:
```rust
     238: }
     239: 
     240: /// **[BACKWARD COMPATIBILITY]** Legacy function that maintains existing API
>>>  241: pub fn export_binary_with_format<P: AsRef<Path>>(
     242:     binary_path: P,
     243:     base_name: &str,
     244:     format: BinaryOutputFormat,
```

#### fn export_json_optimized<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 253
- **代码上下文**:
```rust
     250: /// **[ULTRA-FAST JSON EXPORT]** Use existing JSON generation without modifications
     251: /// This preserves the performance of the existing binary-to-JSON pipeline
     252: /// References the same optimized approach used in parse_full_binary_to_json
>>>  253: fn export_json_optimized<P: AsRef<Path>>(
     254:     binary_path: P,
     255:     base_name: &str,
     256:     _config: &BinaryExportConfig,
```

#### fn export_html_optimized<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 265
- **代码上下文**:
```rust
     262: }
     263: 
     264: /// **[OPTIMIZED HTML EXPORT]** Enhanced HTML generation with streaming and batching
>>>  265: fn export_html_optimized<P: AsRef<Path>>(
     266:     binary_path: P,
     267:     output_path: P,
     268:     project_name: &str,
```

#### fn export_html_filtered<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 375
- **代码上下文**:
```rust
     372: 
     373: 
     374: /// **[FILTERED HTML EXPORT]** Generate HTML with user/system data filtering for optimal performance
>>>  375: fn export_html_filtered<P: AsRef<Path>>(
     376:     binary_path: P,
     377:     output_path: P,
     378:     project_name: &str,
```

#### fn export_both_formats_parallel<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 490
- **代码上下文**:
```rust
     487: /// - Parallel JSON and HTML generation
     488: /// - Optimized I/O with large buffers
     489: /// - Direct streaming writes without intermediate allocations
>>>  490: fn export_both_formats_parallel<P: AsRef<Path>>(
     491:     binary_path: P,
     492:     base_name: &str,
     493:     config: &BinaryExportConfig,
```

#### pub fn export_binary_to_json<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 623
- **代码上下文**:
```rust
     620: 
     621: /// **[MAIN API]** Export to JSON only (preserves existing ultra-fast performance)
     622: /// Uses the same optimized approach as parse_full_binary_to_json
>>>  623: pub fn export_binary_to_json<P: AsRef<Path>>(
     624:     binary_path: P,
     625:     base_name: &str,
     626: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_binary_to_dashboard<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 670
- **代码上下文**:
```rust
     667: /// let stats = export_binary_to_dashboard("data.bin", "my_project", options)?;
     668: /// # Ok::<(), Box<dyn std::error::Error>>(())
     669: /// ```
>>>  670: pub fn export_binary_to_dashboard<P: AsRef<Path>>(
     671:     binary_path: P,
     672:     project_name: &str,
     673:     options: DashboardOptions,
```

#### pub fn export_binary_to_html<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 697
- **代码上下文**:
```rust
     694: 
     695: /// **[MAIN API]** Export to HTML only with ultra-fast optimizations (user data only)
     696: /// Uses shared data approach to match JSON performance, generates lightweight HTML
>>>  697: pub fn export_binary_to_html<P: AsRef<Path>>(
     698:     binary_path: P,
     699:     base_name: &str,
     700: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_binary_to_html_system<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 712
- **代码上下文**:
```rust
     709: 
     710: /// **[MAIN API]** Export to HTML with system data only
     711: /// Generates HTML dashboard with system allocations (no var_name)
>>>  712: pub fn export_binary_to_html_system<P: AsRef<Path>>(
     713:     binary_path: P,
     714:     base_name: &str,
     715: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_binary_to_html_both<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 721
- **代码上下文**:
```rust
     718: 
     719: /// **[MAIN API]** Export to both user and system HTML dashboards
     720: /// Generates two separate HTML files for better performance and usability
>>>  721: pub fn export_binary_to_html_both<P: AsRef<Path>>(
     722:     binary_path: P,
     723:     base_name: &str,
     724: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_binary_to_both<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 730
- **代码上下文**:
```rust
     727: 
     728: /// **[MAIN API]** Export to both JSON and HTML with parallel processing
     729: /// Uses shared data loading and parallel generation for maximum efficiency
>>>  730: pub fn export_binary_to_both<P: AsRef<Path>>(
     731:     binary_path: P,
     732:     base_name: &str,
     733: ) -> Result<(), BinaryExportError> {
```

#### pub fn export_binary_with_config<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 738
- **代码上下文**:
```rust
     735: }
     736: 
     737: /// Export with custom configuration for advanced users
>>>  738: pub fn export_binary_with_config<P: AsRef<Path>>(
     739:     binary_path: P,
     740:     base_name: &str,
     741:     format: BinaryOutputFormat,
```

#### fn export_html_with_shared_data_filtered(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 839
- **代码上下文**:
```rust
     836: 
     837: 
     838: /// **[ULTRA-FAST HTML GENERATION WITH FILTERING]** Generate HTML using shared data with user/system filtering
>>>  839: fn export_html_with_shared_data_filtered(
     840:     allocations: &[crate::core::types::AllocationInfo],
     841:     output_path: &std::path::Path,
     842:     project_name: &str,
```

#### fn export_binary_to_html_embedded_impl<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 1171
- **代码上下文**:
```rust
    1168: // ============================================================================
    1169: 
    1170: /// Implementation for embedded format (backward compatible)
>>> 1171: fn export_binary_to_html_embedded_impl<P: AsRef<Path>>(
    1172:     binary_path: P,
    1173:     project_name: &str,
    1174:     options: &DashboardOptions,
```

#### fn export_binary_to_html_lightweight_impl<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 1209
- **代码上下文**:
```rust
    1206: }
    1207: 
    1208: /// Implementation for lightweight format (HTML + separate JSON files)
>>> 1209: fn export_binary_to_html_lightweight_impl<P: AsRef<Path>>(
    1210:     binary_path: P,
    1211:     project_name: &str,
    1212:     options: &DashboardOptions,
```

#### fn export_binary_to_html_progressive_impl<P: AsRef<Path>>(
- **文件**: `src/export/binary/html_export.rs`
- **行号**: 1227
- **代码上下文**:
```rust
    1224: }
    1225: 
    1226: /// Implementation for progressive format (HTML + lazy-loaded JSON)
>>> 1227: fn export_binary_to_html_progressive_impl<P: AsRef<Path>>(
    1228:     binary_path: P,
    1229:     project_name: &str,
    1230:     options: &DashboardOptions,
```

### 1.2 解析接口 (Parse Interfaces)

**总计**: 23 个解析接口

#### pub fn parse_binary_to_standard_json<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 390
- **代码上下文**:
```rust
     387:     /// ```text
     388:     /// MemoryTracker::parse_binary_to_standard_json("data.memscope", "project_name")?;
     389:     /// ```
>>>  390:     pub fn parse_binary_to_standard_json<P: AsRef<std::path::Path>>(
     391:         binary_path: P,
     392:         base_name: &str,
     393:     ) -> TrackingResult<()> {
```

#### pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 405
- **代码上下文**:
```rust
     402:     /// ```text
     403:     /// MemoryTracker::parse_binary_to_json("data.memscope", "data.json")?;
     404:     /// ```
>>>  405:     pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
     406:         binary_path: P,
     407:         json_path: P,
     408:     ) -> TrackingResult<()> {
```

#### pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
- **文件**: `src/core/tracker/memory_tracker.rs`
- **行号**: 428
- **代码上下文**:
```rust
     425:     /// ```text
     426:     /// MemoryTracker::parse_binary_to_html("data.memscope", "report.html")?;
     427:     /// ```
>>>  428:     pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
     429:         binary_path: P,
     430:         html_path: P,
     431:     ) -> TrackingResult<()> {
```

#### pub fn parse_generic_parameters(type_name: &str) -> (String, Vec<String>) {
- **文件**: `src/analysis/generic_analysis.rs`
- **行号**: 352
- **代码上下文**:
```rust
     349: }
     350: 
     351: /// Parse generic type parameters from a type name
>>>  352: pub fn parse_generic_parameters(type_name: &str) -> (String, Vec<String>) {
     353:     if let Some(start) = type_name.find('<') {
     354:         if let Some(end) = type_name.rfind('>') {
     355:             let base_type = type_name[..start].to_string();
```

#### fn parse_allocation_record(&mut self) -> Result<AllocationInfo, BinaryExportError> {
- **文件**: `src/export/binary/selective_reader.rs`
- **行号**: 1151
- **代码上下文**:
```rust
    1148:     }
    1149: 
    1150:     /// Parse a single allocation record from the current position
>>> 1151:     fn parse_allocation_record(&mut self) -> Result<AllocationInfo, BinaryExportError> {
    1152:         // For the initial implementation, we'll load all allocations once and cache them
    1153:         // This is not the most memory-efficient approach, but it's simple and correct
    1154:         if self.allocation_cache.is_empty() {
```

#### pub fn parse_binary_to_html_direct<P: AsRef<Path>>(
- **文件**: `src/export/binary/binary_html_export.rs`
- **行号**: 145
- **代码上下文**:
```rust
     142: /// println!("Conversion completed in {}ms", stats.total_export_time_ms);
     143: /// # Ok::<(), Box<dyn std::error::Error>>(())
     144: /// ```
>>>  145: pub fn parse_binary_to_html_direct<P: AsRef<Path>>(
     146:     binary_path: P,
     147:     html_path: P,
     148:     project_name: &str,
```

#### pub fn parse_binary_to_html_with_config<P: AsRef<Path>>(
- **文件**: `src/export/binary/binary_html_export.rs`
- **行号**: 162
- **代码上下文**:
```rust
     159: ///
     160: /// This function allows fine-tuned control over the conversion process
     161: /// with custom configuration options.
>>>  162: pub fn parse_binary_to_html_with_config<P: AsRef<Path>>(
     163:     binary_path: P,
     164:     html_path: P,
     165:     project_name: &str,
```

#### pub fn parse_binary_to_html_auto<P: AsRef<Path>>(
- **文件**: `src/export/binary/binary_html_export.rs`
- **行号**: 418
- **代码上下文**:
```rust
     415: ///
     416: /// This function automatically detects the best conversion strategy based on
     417: /// file characteristics and system resources.
>>>  418: pub fn parse_binary_to_html_auto<P: AsRef<Path>>(
     419:     binary_path: P,
     420:     html_path: P,
     421:     project_name: &str,
```

#### fn parse_generic_parameters(&self, params_str: &str) -> Vec<String> {
- **文件**: `src/export/binary/complex_type_analyzer.rs`
- **行号**: 300
- **代码上下文**:
```rust
     297:     }
     298: 
     299:     /// Parse generic parameters from a parameter string
>>>  300:     fn parse_generic_parameters(&self, params_str: &str) -> Vec<String> {
     301:         let mut parameters = Vec::new();
     302:         let mut current_param = String::new();
     303:         let mut bracket_depth = 0;
```

#### pub fn parse_selective_fields<R: Read + Seek>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 151
- **代码上下文**:
```rust
     148:     }
     149: 
     150:     /// Parse only the requested fields from a binary record
>>>  151:     pub fn parse_selective_fields<R: Read + Seek>(
     152:         &mut self,
     153:         reader: &mut R,
     154:         requested_fields: &HashSet<AllocationField>,
```

#### pub fn parse_full_allocation<R: Read + Seek>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 188
- **代码上下文**:
```rust
     185:     }
     186: 
     187:     /// Parse an allocation record with all fields (for compatibility)
>>>  188:     pub fn parse_full_allocation<R: Read + Seek>(
     189:         &mut self,
     190:         reader: &mut R,
     191:     ) -> Result<AllocationInfo, BinaryExportError> {
```

#### fn parse_basic_fields<R: Read>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 220
- **代码上下文**:
```rust
     217:     // Private helper methods
     218: 
     219:     /// Parse basic fields (always present)
>>>  220:     fn parse_basic_fields<R: Read>(
     221:         &mut self,
     222:         reader: &mut R,
     223:         requested_fields: &HashSet<AllocationField>,
```

#### fn parse_optional_fields<R: Read>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 257
- **代码上下文**:
```rust
     254:     }
     255: 
     256:     /// Parse optional fields
>>>  257:     fn parse_optional_fields<R: Read>(
     258:         &mut self,
     259:         reader: &mut R,
     260:         requested_fields: &HashSet<AllocationField>,
```

#### fn parse_advanced_fields<R: Read + Seek>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 347
- **代码上下文**:
```rust
     344:     }
     345: 
     346:     /// Parse advanced fields (may not be present in all records)
>>>  347:     fn parse_advanced_fields<R: Read + Seek>(
     348:         &mut self,
     349:         reader: &mut R,
     350:         requested_fields: &HashSet<AllocationField>,
```

#### fn parse_optional_string<R: Read>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 391
- **代码上下文**:
```rust
     388:     }
     389: 
     390:     /// Parse an optional string field
>>>  391:     fn parse_optional_string<R: Read>(
     392:         &mut self,
     393:         reader: &mut R,
     394:     ) -> Result<Option<String>, BinaryExportError> {
```

#### fn parse_optional_string_vec<R: Read>(
- **文件**: `src/export/binary/field_parser.rs`
- **行号**: 404
- **代码上下文**:
```rust
     401:     }
     402: 
     403:     /// Parse an optional string vector field
>>>  404:     fn parse_optional_string_vec<R: Read>(
     405:         &mut self,
     406:         reader: &mut R,
     407:     ) -> Result<Option<Vec<String>>, BinaryExportError> {
```

#### pub fn parse_binary_to_json<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 200
- **代码上下文**:
```rust
     197: }
     198: 
     199: /// Convert binary file to JSON format
>>>  200: pub fn parse_binary_to_json<P: AsRef<Path>>(
     201:     binary_path: P,
     202:     json_path: P,
     203: ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_binary_to_html<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 208
- **代码上下文**:
```rust
     205: }
     206: 
     207: /// Convert binary file to HTML format
>>>  208: pub fn parse_binary_to_html<P: AsRef<Path>>(
     209:     binary_path: P,
     210:     html_path: P,
     211: ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_binary_auto<P: AsRef<Path>>(
- **文件**: `src/export/binary/mod.rs`
- **行号**: 408
- **代码上下文**:
```rust
     405: /// # Ok(())
     406: /// # }
     407: /// ```
>>>  408: pub fn parse_binary_auto<P: AsRef<Path>>(
     409:     binary_path: P,
     410:     base_name: &str,
     411: ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_user_binary_to_json<P: AsRef<Path>>(
- **文件**: `src/export/binary/parser.rs`
- **行号**: 225
- **代码上下文**:
```rust
     222: 
     223:     /// Parse user binary to JSON using BinaryReader for consistency and performance
     224:     /// Now uses the same BinaryReader approach as full binary parsing for consistent performance
>>>  225:     pub fn parse_user_binary_to_json<P: AsRef<Path>>(
     226:         binary_path: P,
     227:         base_name: &str,
     228:     ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_full_binary_to_json<P: AsRef<Path>>(
- **文件**: `src/export/binary/parser.rs`
- **行号**: 262
- **代码上下文**:
```rust
     259:     /// - 直接调用优化的generate_*_json方法 (避免复杂的SelectiveJsonExporter)
     260:     /// - 并行生成5个JSON文件 (Task 7.1)
     261:     /// - 目标: <300ms性能，无null字段，JSON格式一致
>>>  262:     pub fn parse_full_binary_to_json<P: AsRef<Path>>(
     263:         binary_path: P,
     264:         base_name: &str,
     265:     ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_full_binary_to_json_with_existing_optimizations<P: AsRef<Path>>(
- **文件**: `src/export/binary/parser.rs`
- **行号**: 340
- **代码上下文**:
```rust
     337:     /// **[Task 23]** Ultra-fast binary to JSON conversion using existing optimizations
     338:     ///
     339:     /// This method provides the same ultra-fast performance as v5-draft
>>>  340:     pub fn parse_full_binary_to_json_with_existing_optimizations<P: AsRef<Path>>(
     341:         binary_path: P,
     342:         base_name: &str,
     343:     ) -> Result<(), BinaryExportError> {
```

#### pub fn parse_binary_to_json_with_index<P: AsRef<Path>>(
- **文件**: `src/export/binary/parser.rs`
- **行号**: 1252
- **代码上下文**:
```rust
    1249:     ///
    1250:     /// This is the core high-performance interface that uses BinaryIndex for direct data access,
    1251:     /// avoiding the overhead of loading all allocations into memory.
>>> 1252:     pub fn parse_binary_to_json_with_index<P: AsRef<Path>>(
    1253:         binary_path: P,
    1254:         base_name: &str,
    1255:     ) -> Result<(), BinaryExportError> {
```

### 1.3 数据收集方法 (Collection Methods)

**总计**: 347 个数据收集方法

#### fn get_current_state(&self) -> TypeStateInfo;
- **文件**: `src/advanced_types.rs`
- **行号**: 157

#### fn get_performance_info(&self) -> PerformanceInfo;
- **文件**: `src/advanced_types.rs`
- **行号**: 163

#### pub fn get_type_category(type_name: &str) -> Option<AdvancedTypeCategory> {
- **文件**: `src/advanced_types.rs`
- **行号**: 562

#### fn get_heap_ptr(&self) -> Option<usize>;
- **文件**: `src/lib.rs`
- **行号**: 58

#### fn get_type_name(&self) -> &'static str;
- **文件**: `src/lib.rs`
- **行号**: 61

#### fn get_size_estimate(&self) -> usize;
- **文件**: `src/lib.rs`
- **行号**: 64

#### fn get_ref_count(&self) -> usize {
- **文件**: `src/lib.rs`
- **行号**: 67

#### fn get_data_ptr(&self) -> usize {
- **文件**: `src/lib.rs`
- **行号**: 72

#### fn get_internal_allocations(&self, _var_name: &str) -> Vec<(usize, String)> {
- **文件**: `src/lib.rs`
- **行号**: 77

#### fn get_advanced_type_info(&self) -> Option<crate::advanced_types::AdvancedTypeInfo> {
- **文件**: `src/lib.rs`
- **行号**: 92

... 还有 337 个方法

### 1.4 过滤策略 (Filter Strategies)

**总计**: 326 个过滤策略

- **文件**: `src/lib.rs:39`
  ```rust
  pub use core::tracker::memory_tracker::BinaryExportMode;
  ```

- **文件**: `src/variable_registry.rs:442`
  ```rust
  .filter_map(|a| a["lifetime_ms"].as_u64())
  ```

- **文件**: `src/variable_registry.rs:447`
  ```rust
  .filter(|a| a["is_active"].as_bool().unwrap_or(false))
  ```

- **文件**: `src/variable_registry.rs:452`
  ```rust
  .filter(|a| a["timestamp_dealloc"].is_null() == false)
  ```

- **文件**: `src/variable_registry.rs:458`
  ```rust
  .filter_map(|a| a["lifetime_ms"].as_u64())
  ```

- **文件**: `src/variable_registry.rs:463`
  ```rust
  .filter(|a| a["is_active"].as_bool().unwrap_or(false))
  ```

- **文件**: `src/variable_registry.rs:468`
  ```rust
  .filter(|a| a["timestamp_dealloc"].is_null() == false)
  ```

- **文件**: `src/variable_registry.rs:523`
  ```rust
  .filter_map(|a| a["timestamp_dealloc"].as_u64())
  ```

- **文件**: `src/variable_registry.rs:528`
  ```rust
  .filter_map(|a| a["timestamp_dealloc"].as_u64())
  ```

- **文件**: `src/variable_registry.rs:534`
  ```rust
  .filter(|a| a["timestamp_dealloc"].is_null())
  ```

- **文件**: `src/variable_registry.rs:539`
  ```rust
  .filter(|a| a["timestamp_dealloc"].is_null())
  ```

- **文件**: `src/variable_registry.rs:774`
  ```rust
  .filter(|alloc| alloc.size >= 8)
  ```

- **文件**: `src/variable_registry.rs:783`
  ```rust
  .filter(|alloc| alloc.size >= 8)
  ```

- **文件**: `src/variable_registry.rs:884`
  ```rust
  .filter(|v| {
  ```

- **文件**: `src/core/performance_analysis.rs:163`
  ```rust
  let critical_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Critical)).count();
  ```

### 1.5 Binary导出模式 (Binary Export Modes)

**总计**: 42 个模式定义/使用

- **文件**: `src/lib.rs:39`
  ```rust
  pub use core::tracker::memory_tracker::BinaryExportMode;
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:19`
  ```rust
  pub enum BinaryExportMode {
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:28`
  ```rust
  impl Default for BinaryExportMode {
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:31`
  ```rust
  BinaryExportMode::UserOnly
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:255`
  ```rust
  /// tracker.export_to_binary_with_mode("my_program_user", BinaryExportMode::UserOnly)?;
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:258`
  ```rust
  /// tracker.export_to_binary_with_mode("my_program_full", BinaryExportMode::Full)?;
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:263`
  ```rust
  mode: BinaryExportMode,
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:266`
  ```rust
  BinaryExportMode::UserOnly => {
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:270`
  ```rust
  BinaryExportMode::Full => {
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:313`
  ```rust
  crate::export::binary::format::BinaryExportMode::UserOnly,
  ```

- **文件**: `src/core/tracker/memory_tracker.rs:352`
  ```rust
  crate::export::binary::format::BinaryExportMode::Full,
  ```

- **文件**: `src/export/binary/format.rs:27`
  ```rust
  pub enum BinaryExportMode {
  ```

- **文件**: `src/export/binary/format.rs:34`
  ```rust
  impl From<u8> for BinaryExportMode {
  ```

- **文件**: `src/export/binary/format.rs:37`
  ```rust
  0 => BinaryExportMode::UserOnly,
  ```

- **文件**: `src/export/binary/format.rs:38`
  ```rust
  1 => BinaryExportMode::Full,
  ```

- **文件**: `src/export/binary/format.rs:39`
  ```rust
  _ => BinaryExportMode::UserOnly, // Default fallback
  ```

- **文件**: `src/export/binary/format.rs:83`
  ```rust
  export_mode: BinaryExportMode,
  ```

- **文件**: `src/export/binary/format.rs:107`
  ```rust
  export_mode: BinaryExportMode::UserOnly as u8,
  ```

- **文件**: `src/export/binary/format.rs:135`
  ```rust
  pub fn get_export_mode(&self) -> BinaryExportMode {
  ```

- **文件**: `src/export/binary/format.rs:136`
  ```rust
  BinaryExportMode::from(self.export_mode)
  ```

- **文件**: `src/export/binary/format.rs:141`
  ```rust
  self.get_export_mode() == BinaryExportMode::UserOnly
  ```

- **文件**: `src/export/binary/format.rs:146`
  ```rust
  self.get_export_mode() == BinaryExportMode::Full
  ```

- **文件**: `src/export/binary/format.rs:356`
  ```rust
  let header = FileHeader::new(100, BinaryExportMode::Full, 60, 40);
  ```

- **文件**: `src/export/binary/format.rs:362`
  ```rust
  assert_eq!(header.get_export_mode(), BinaryExportMode::Full);
  ```

- **文件**: `src/export/binary/format.rs:372`
  ```rust
  let header = FileHeader::new(42, BinaryExportMode::UserOnly, 42, 0);
  ```

- **文件**: `src/export/binary/format.rs:385`
  ```rust
  assert_eq!(header.get_export_mode(), BinaryExportMode::UserOnly);
  ```

- **文件**: `src/export/binary/format.rs:392`
  ```rust
  assert_eq!(BinaryExportMode::from(0), BinaryExportMode::UserOnly);
  ```

- **文件**: `src/export/binary/format.rs:393`
  ```rust
  assert_eq!(BinaryExportMode::from(1), BinaryExportMode::Full);
  ```

- **文件**: `src/export/binary/format.rs:394`
  ```rust
  assert_eq!(BinaryExportMode::from(255), BinaryExportMode::UserOnly); // Default fallback
  ```

- **文件**: `src/export/binary/mod.rs:88`
  ```rust
  pub use format::{BinaryExportMode, FileHeader, FORMAT_VERSION, MAGIC_BYTES};
  ```

- **文件**: `src/export/binary/mod.rs:140`
  ```rust
  export_mode: BinaryExportMode,
  ```

- **文件**: `src/export/binary/mod.rs:219`
  ```rust
  pub export_mode: BinaryExportMode,
  ```

- **文件**: `src/export/binary/mod.rs:237`
  ```rust
  self.export_mode == BinaryExportMode::UserOnly
  ```

- **文件**: `src/export/binary/mod.rs:242`
  ```rust
  self.export_mode == BinaryExportMode::Full
  ```

- **文件**: `src/export/binary/mod.rs:248`
  ```rust
  BinaryExportMode::UserOnly => format!(
  ```

- **文件**: `src/export/binary/mod.rs:253`
  ```rust
  BinaryExportMode::Full => format!(
  ```

- **文件**: `src/export/binary/mod.rs:266`
  ```rust
  BinaryExportMode::UserOnly => "Simple processing (small file, user data only)",
  ```

- **文件**: `src/export/binary/mod.rs:267`
  ```rust
  BinaryExportMode::Full => "Optimized processing (large file, comprehensive data)",
  ```

- **文件**: `src/export/binary/mod.rs:427`
  ```rust
  BinaryExportMode::UserOnly => {
  ```

- **文件**: `src/export/binary/mod.rs:431`
  ```rust
  BinaryExportMode::Full => {
  ```

- **文件**: `src/export/binary/writer.rs:7`
  ```rust
  AdvancedMetricsHeader, BinaryExportMode, FileHeader, MetricsBitmapFlags, ALLOCATION_RECORD_TYPE,
  ```

- **文件**: `src/export/binary/writer.rs:108`
  ```rust
  export_mode: BinaryExportMode,
  ```

## 🚨 2. 代码质量分析

### 2.1 Unwrap使用统计

**总计**: 1125 个unwrap使用
**涉及文件**: 109 个

#### 按文件统计 (Top 10):
| 文件 | Unwrap数量 |
|------|-----------|
| `src/export/optimized_json_export.rs` | 73 |
| `src/export/binary/parser.rs` | 57 |
| `src/cli/commands/html_from_json/data_normalizer.rs` | 56 |
| `src/export/binary/selective_reader.rs` | 48 |
| `src/export/binary/reader.rs` | 34 |
| `src/export/binary/cache.rs` | 32 |
| `src/export/analysis_engine.rs` | 29 |
| `src/export/binary/streaming_json_writer.rs` | 28 |
| `src/export/binary/integration_test_variable_relationships.rs` | 27 |
| `src/analysis/memory_passport_tracker.rs` | 26 |

### 2.2 Clone使用统计

**总计**: 1600 个clone使用
**涉及文件**: 111 个

#### 按文件统计 (Top 10):
| 文件 | Clone数量 |
|------|-----------|
| `src/core/types/mod.rs` | 221 |
| `src/enhanced_types.rs` | 92 |
| `src/analysis/unsafe_ffi_tracker.rs` | 89 |
| `src/export/binary/variable_relationship_analyzer.rs` | 59 |
| `src/export/quality_validator.rs` | 45 |
| `src/export/batch_processor.rs` | 43 |
| `src/analysis/enhanced_memory_analysis.rs` | 40 |
| `src/analysis/closure_analysis.rs` | 36 |
| `src/advanced_types.rs` | 35 |
| `src/analysis/lifecycle_analysis.rs` | 33 |

### 2.3 Lock使用统计

**总计**: 414 个lock使用
**涉及文件**: 36 个

#### 按文件统计 (Top 10):
| 文件 | Lock数量 |
|------|-----------|
| `src/analysis/unsafe_ffi_tracker.rs` | 41 |
| `src/core/tracker/allocation_tracking.rs` | 27 |
| `src/core/call_stack_normalizer.rs` | 24 |
| `src/analysis/memory_passport_tracker.rs` | 21 |
| `src/analysis/async_analysis.rs` | 20 |
| `src/export/optimized_json_export.rs` | 20 |
| `src/cli/commands/html_from_json/debug_logger.rs` | 18 |
| `src/analysis/borrow_analysis.rs` | 17 |
| `src/core/targeted_optimizations.rs` | 15 |
| `src/core/smart_optimization.rs` | 15 |

## 🎯 3. 总结和建议

### 3.1 数据收集策略总结

- **导出接口**: 76 个
- **解析接口**: 23 个
- **数据收集方法**: 347 个
- **过滤策略**: 326 个

### 3.2 代码质量建议

- **Unwrap优化**: 发现 1125 个unwrap使用，建议使用更安全的错误处理
- **Clone优化**: 发现 1600 个clone使用，建议评估是否可以使用引用
- **Lock优化**: 发现 414 个lock使用，建议评估并发性能

### 3.3 优先处理建议

1. **优先处理unwrap最多的文件**: `src/export/optimized_json_export.rs` (73 个)
2. **优先处理clone最多的文件**: `src/core/types/mod.rs` (221 个)
3. **优先处理lock最多的文件**: `src/analysis/unsafe_ffi_tracker.rs` (41 个)