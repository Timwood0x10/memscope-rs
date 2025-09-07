# 提升测试覆盖率 

当前项目中的测试覆盖率太低了，提升一下 

1. 对于缺少test的模块，以test mod的形式进行添加到文件内，编码风格参考./aim/requirement.md。禁止创建任何的rs file

2. 工作流程是先定位覆盖率低的模块，然后阅读模块中的代码，制定合适的test case，并且每编写一个模块就测试一次，测试通过之后，再进行下一个模块，并且test case 一定要有意义，且符合当前模块，不能写一些无意义的test case，禁止出现滥竽充数的test case，质量必须高且有意义。

2.1 慎用get_global_tracker()，以及 `TrackingManager::new()` 尤其是在运行多个test case 的时候会导致死锁。必须想法子规避。


3.
Filename                                                     Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
advanced_trackable_macro.rs                                      415                 2    99.52%          21                 0   100.00%         255                 0   100.00%           0                 0         -
advanced_types.rs                                                890                61    93.15%          35                 1    97.14%         831               143    82.79%           0                 0         -
analysis/async_analysis.rs                                       391                93    76.21%          22                 6    72.73%         292                71    75.68%           0                 0         -
analysis/borrow_analysis.rs                                      350               120    65.71%          25                 5    80.00%         241                80    66.80%           0                 0         -
analysis/circular_reference.rs                                   789               109    86.19%          21                 2    90.48%         691                77    88.86%           0                 0         -
analysis/closure_analysis.rs                                    1261                32    97.46%          79                 2    97.47%         850                21    97.53%           0                 0         -
analysis/enhanced_ffi_function_resolver.rs                       866               334    61.43%          36                 9    75.00%         554               197    64.44%           0                 0         -
analysis/enhanced_memory_analysis.rs                            1670               190    88.62%          98                 4    95.92%        1402               163    88.37%           0                 0         -
analysis/ffi_function_resolver.rs                                642               144    77.57%          44                10    77.27%         511                92    82.00%           0                 0         -
analysis/generic_analysis.rs                                     324               158    51.23%          34                21    38.24%         226               119    47.35%           0                 0         -
analysis/lifecycle_analysis.rs                                  1133                48    95.76%          66                 4    93.94%         717                31    95.68%           0                 0         -
analysis/memory_passport_tracker.rs                              642               141    78.04%          40                14    65.00%         476               134    71.85%           0                 0         -
analysis/mod.rs                                                  501                 3    99.40%          37                 1    97.30%         307                 3    99.02%           0                 0         -
analysis/safety_analyzer.rs                                     1330               149    88.80%          65                 7    89.23%        1075               123    88.56%           0                 0         -
analysis/security_violation_analyzer.rs                         1313                64    95.13%          65                 0   100.00%         959                33    96.56%           0                 0         -
analysis/unknown_memory_regions.rs                               877                55    93.73%          67                 4    94.03%         596                35    94.13%           0                 0         -
analysis/unsafe_ffi_tracker.rs                                  2331               625    73.19%          90                21    76.67%        1735               522    69.91%           0                 0         -
analysis/variable_relationships.rs                              1170                70    94.02%          55                 1    98.18%         840                40    95.24%           0                 0         -
bin/allocation_count_diagnostic.rs                               358               190    46.93%          12                 5    58.33%         248               132    46.77%           0                 0         -
bin/core_performance_test.rs                                     610               215    64.75%          20                 2    90.00%         438               145    66.89%           0                 0         -
bin/establish_baseline.rs                                        243                31    87.24%           6                 1    83.33%         118                10    91.53%           0                 0         -
bin/large_active_allocations.rs                                  214                81    62.15%           8                 2    75.00%         107                44    58.88%           0                 0         -
bin/lifecycle_analysis.rs                                        287               158    44.95%          11                 6    45.45%         173               103    40.46%           0                 0         -
bin/performance_only_benchmark.rs                                581               244    58.00%          14                 2    85.71%         342               114    66.67%           0                 0         -
bin/run_benchmark.rs                                             115                40    65.22%           7                 1    85.71%         100                19    81.00%           0                 0         -
bin/simple_benchmark.rs                                          534               208    61.05%          14                 2    85.71%         310               101    67.42%           0                 0         -
cli/commands/analyze.rs                                          961               468    51.30%          40                17    57.50%         553               298    46.11%           0                 0         -
cli/commands/generate_report.rs                                  530               111    79.06%          19                 3    84.21%         292                59    79.79%           0                 0         -
cli/commands/html_from_json/data_integrator.rs                  1291                42    96.75%          68                 1    98.53%         846                17    97.99%           0                 0         -
cli/commands/html_from_json/data_normalizer.rs                  1140               236    79.30%         115                61    46.96%         633               101    84.04%           0                 0         -
cli/commands/html_from_json/debug_logger.rs                      536               205    61.75%          39                 9    76.92%         380               148    61.05%           0                 0         -
cli/commands/html_from_json/direct_json_template.rs             1768              1768     0.00%         136               136     0.00%        1124              1124     0.00%           0                 0         -
cli/commands/html_from_json/error_handler.rs                     899               752    16.35%          34                21    38.24%         566               435    23.14%           0                 0         -
cli/commands/html_from_json/json_file_discovery.rs               297                95    68.01%          17                 3    82.35%         201                51    74.63%           0                 0         -
cli/commands/html_from_json/large_file_optimizer.rs              433               177    59.12%          20                 5    75.00%         295               117    60.34%           0                 0         -
cli/commands/html_from_json/mod.rs                               871               871     0.00%          23                23     0.00%         597               597     0.00%           0                 0         -
cli/commands/test.rs                                             266               129    51.50%          17                 5    70.59%         151                71    52.98%           0                 0         -
core/adaptive_hashmap.rs                                         299                70    76.59%          17                 4    76.47%         168                50    70.24%           0                 0         -
core/allocation_adapter.rs                                       359               126    64.90%          66                34    48.48%         275               104    62.18%           0                 0         -
core/allocator.rs                                                369                34    90.79%          33                 3    90.91%         243                15    93.83%           0                 0         -
core/atomic_stats.rs                                             661                10    98.49%          54                 0   100.00%         451                 3    99.33%           0                 0         -
core/bounded_memory_stats.rs                                     426               127    70.19%          35                11    68.57%         410                99    75.85%           0                 0         -
core/call_stack_normalizer.rs                                    558                91    83.69%          36                 8    77.78%         393                88    77.61%           0                 0         -
core/clone_monitor.rs                                            171                14    91.81%          16                 0   100.00%         113                11    90.27%           0                 0         -
core/clone_optimizer.rs                                          676                 0   100.00%          35                 0   100.00%         377                 0   100.00%           0                 0         -
core/clone_utils.rs                                              288                 5    98.26%          19                 0   100.00%         169                 2    98.82%           0                 0         -
core/comprehensive_data_deduplicator.rs                         1524               198    87.01%          63                 8    87.30%         901               118    86.90%           0                 0         -
core/edge_case_handler.rs                                        466               134    71.24%          37                13    64.86%         354               102    71.19%           0                 0         -
core/enhanced_call_stack_normalizer.rs                           466               182    60.94%          35                16    54.29%         341               144    57.77%           0                 0         -
core/enhanced_pointer_extractor.rs                               255                99    61.18%          25                12    52.00%         231               102    55.84%           0                 0         -
core/enhanced_type_inference.rs                                  557               152    72.71%          19                 4    78.95%         372               123    66.94%           0                 0         -
core/error.rs                                                    773                60    92.24%          41                 2    95.12%         489                39    92.02%           0                 0         -
core/error_adapter.rs                                            262               140    46.56%          11                 0   100.00%         136                65    52.21%           0                 0         -
core/fast_data_deduplicator.rs                                   339                10    97.05%          21                 2    90.48%         183                 7    96.17%           0                 0         -
core/integration_validator.rs                                    814               208    74.45%          29                 3    89.66%         485               118    75.67%           0                 0         -
core/lifecycle_summary.rs                                        554                30    94.58%          33                 0   100.00%         449                14    96.88%           0                 0         -
core/optimized_locks.rs                                          637                42    93.41%          50                 4    92.00%         413                31    92.49%           0                 0         -
core/optimized_tracker.rs                                        366                 8    97.81%          24                 0   100.00%         217                16    92.63%           0                 0         -
core/optimized_types.rs                                          567               193    65.96%          50                13    74.00%         347                89    74.35%           0                 0         -
core/ownership_history.rs                                        432                29    93.29%          26                 2    92.31%         355                34    90.42%           0                 0         -
core/safe_operations.rs                                          228                11    95.18%          22                 3    86.36%         131                18    86.26%           0                 0         -
core/scope_tracker.rs                                            640                88    86.25%          35                 7    80.00%         360                74    79.44%           0                 0         -
core/sharded_locks.rs                                            334               111    66.77%          35                17    51.43%         221                85    61.54%           0                 0         -
core/shared_types.rs                                             488                 0   100.00%          50                 0   100.00%         301                 0   100.00%           0                 0         -
core/simple_mutex.rs                                             118                 0   100.00%          12                 0   100.00%          72                 0   100.00%           0                 0         -
core/smart_optimization.rs                                       183                88    51.91%          20                 9    55.00%         126                56    55.56%           0                 0         -
core/string_pool.rs                                              571                11    98.07%          35                 2    94.29%         296                 9    96.96%           0                 0         -
core/string_pool_monitor.rs                                      337                16    95.25%          27                 0   100.00%         350                20    94.29%           0                 0         -
core/targeted_optimizations.rs                                   236                70    70.34%          23                 9    60.87%         158                43    72.78%           0                 0         -
core/test_optimized_locks.rs                                     106                 6    94.34%           6                 0   100.00%          58                 3    94.83%           0                 0         -
core/threshold_batch_processor.rs                                270                76    71.85%          21                 4    80.95%         193                44    77.20%           0                 0         -
core/tracker/allocation_tracking.rs                              752               532    29.26%          32                23    28.12%         569               413    27.42%           0                 0         -
core/tracker/config.rs                                           242                 5    97.93%          17                 0   100.00%         168                 0   100.00%           0                 0         -
core/tracker/export_html.rs                                      341                41    87.98%          19                 4    78.95%         334                18    94.61%           0                 0         -
core/tracker/export_json.rs                                     1111               800    27.99%          58                43    25.86%         798               566    29.07%           0                 0         -
core/tracker/global_functions.rs                                 166                42    74.70%          16                 6    62.50%          80                23    71.25%           0                 0         -
core/tracker/memory_analysis.rs                                 2000               481    75.95%          75                 8    89.33%        1474               412    72.05%           0                 0         -
core/tracker/memory_tracker.rs                                  1334               235    82.38%          90                20    77.78%         964               215    77.70%           0                 0         -
core/tracker/tracking_manager.rs                                 194                72    62.89%          22                11    50.00%         219                58    73.52%           0                 0         -
core/types/mod.rs                                                724               181    75.00%          48                 2    95.83%         617                67    89.14%           0                 0         -
core/unwrap_safe.rs                                              609               102    83.25%          63                11    82.54%         431                59    86.31%           0                 0         -
enhanced_types.rs                                                670                10    98.51%          64                 0   100.00%         545                 0   100.00%           0                 0         -
export/adaptive_performance.rs                                   848                68    91.98%          56                 2    96.43%         582                50    91.41%           0                 0         -
export/analysis_engine.rs                                        822               641    22.02%          32                21    34.38%         533               396    25.70%           0                 0         -
export/api.rs                                                    306               206    32.68%          27                18    33.33%         230               154    33.04%           0                 0         -
export/batch_processor.rs                                       1393               138    90.09%          77                20    74.03%         965                74    92.33%           0                 0         -
export/binary/batch_processor.rs                                 632               362    42.72%          47                17    63.83%         435               235    45.98%           0                 0         -
export/binary/binary_html_export.rs                              389               306    21.34%          15                 6    60.00%         303               198    34.65%           0                 0         -
export/binary/binary_html_writer.rs                              524               175    66.60%          35                14    60.00%         412               141    65.78%           0                 0         -
export/binary/binary_template_engine.rs                         1141               171    85.01%          56                13    76.79%        1099               117    89.35%           0                 0         -
export/binary/cache.rs                                           798                87    89.10%          47                 9    80.85%         502                45    91.04%           0                 0         -
export/binary/complex_type_analyzer.rs                           610                27    95.57%          28                 4    85.71%         412                13    96.84%           0                 0         -
export/binary/config.rs                                          344               126    63.37%          46                22    52.17%         361               124    65.65%           0                 0         -
export/binary/error.rs                                            67                32    52.24%           6                 2    66.67%          41                20    51.22%           0                 0         -
export/binary/error_recovery.rs                                  352               137    61.08%          27                 7    74.07%         301               111    63.12%           0                 0         -
export/binary/ffi_safety_analyzer.rs                             811                30    96.30%          44                 4    90.91%         579                15    97.41%           0                 0         -
export/binary/field_parser.rs                                    665               323    51.43%          36                16    55.56%         489               248    49.28%           0                 0         -
export/binary/filter_engine.rs                                   643               176    72.63%          40                 6    85.00%         484               135    72.11%           0                 0         -
export/binary/format.rs                                          383                11    97.13%          35                 2    94.29%         290                 8    97.24%           0                 0         -
export/binary/html_converter.rs                                  536               192    64.18%          14                 6    57.14%         568               272    52.11%           0                 0         -
export/binary/html_export.rs                                    1009               718    28.84%          42                28    33.33%         658               437    33.59%           0                 0         -
export/binary/index.rs                                           345                17    95.07%          27                 2    92.59%         231                14    93.94%           0                 0         -
export/binary/index_builder.rs                                   679               128    81.15%          29                 9    68.97%         442               109    75.34%           0                 0         -
export/binary/integration_test_complex_types.rs                  264                 0   100.00%          12                 0   100.00%         163                 0   100.00%           0                 0         -
export/binary/integration_test_ffi_safety.rs                     349                 3    99.14%          18                 0   100.00%         245                 0   100.00%           0                 0         -
export/binary/integration_test_template_resources.rs             364                 5    98.63%          10                 0   100.00%         233                 5    97.85%           0                 0         -
export/binary/integration_test_variable_relationships.rs         708                 1    99.86%          41                 0   100.00%         577                 0   100.00%           0                 0         -
export/binary/memory_layout_serialization.rs                     571               303    46.94%          29                18    37.93%         324               154    52.47%           0                 0         -
export/binary/mod.rs                                             194               127    34.54%          13                 9    30.77%         129                89    31.01%           0                 0         -
export/binary/parser.rs                                         3292              2759    16.19%          65                56    13.85%        1547              1318    14.80%           0                 0         -
export/binary/reader.rs                                         1037               343    66.92%          30                 5    83.33%         573               162    71.73%           0                 0         -
export/binary/selective_json_exporter.rs                         884               730    17.42%          49                29    40.82%         588               452    23.13%           0                 0         -
export/binary/selective_reader.rs                               1576               374    76.27%          85                30    64.71%        1133               265    76.61%           0                 0         -
export/binary/serializable.rs                                    741               524    29.28%          52                34    34.62%         370               251    32.16%           0                 0         -
export/binary/smart_pointer_serialization.rs                     315                44    86.03%          13                 2    84.62%         195                 9    95.38%           0                 0         -
export/binary/streaming_field_processor.rs                       635               171    73.07%          60                19    68.33%         522               126    75.86%           0                 0         -
export/binary/streaming_json_writer.rs                          1732               700    59.58%          67                17    74.63%        1214               407    66.47%           0                 0         -
export/binary/string_table.rs                                    559               150    73.17%          37                10    72.97%         352                99    71.88%           0                 0         -
export/binary/template_resource_manager.rs                       645                65    89.92%          32                 7    78.12%         376                38    89.89%           0                 0         -
export/binary/variable_relationship_analyzer.rs                 1001                56    94.41%          54                 4    92.59%         852                26    96.95%           0                 0         -
export/binary/writer.rs                                         1098               324    70.49%          41                 8    80.49%         619               123    80.13%           0                 0         -
export/complex_type_export.rs                                    360               360     0.00%          22                22     0.00%         240               240     0.00%           0                 0         -
export/config_optimizer.rs                                        66                66     0.00%           6                 6     0.00%          67                67     0.00%           0                 0         -
export/data_localizer.rs                                         391               252    35.55%          30                18    40.00%         310               206    33.55%           0                 0         -
export/enhanced_json_exporter.rs                                 337               221    34.42%          19                11    42.11%         283               174    38.52%           0                 0         -
export/error_handling.rs                                         523               279    46.65%          33                12    63.64%         463               241    47.95%           0                 0         -
export/error_recovery.rs                                         551               242    56.08%          29                10    65.52%         563               259    54.00%           0                 0         -
export/export_enhanced.rs                                       3949              3119    21.02%          59                46    22.03%        2433              1889    22.36%           0                 0         -
export/export_modes.rs                                           129               129     0.00%          14                14     0.00%          90                90     0.00%           0                 0         -
export/fast_export_coordinator.rs                               1022               848    17.03%          57                39    31.58%         817               670    17.99%           0                 0         -
export/high_speed_buffered_writer.rs                             515                52    89.90%          27                 5    81.48%         308                28    90.91%           0                 0         -
export/html_export.rs                                            489               211    56.85%          20                12    40.00%         423               110    74.00%           0                 0         -
export/lifecycle_exporter.rs                                     165               165     0.00%          10                10     0.00%         112               112     0.00%           0                 0         -
export/optimized_json_export.rs                                 2453              2453     0.00%          88                88     0.00%        1669              1669     0.00%           0                 0         -
export/parallel_shard_processor.rs                               425                43    89.88%          25                 3    88.00%         299                26    91.30%           0                 0         -
export/progress_monitor.rs                                       445               104    76.63%          35                 8    77.14%         318                68    78.62%           0                 0         -
export/quality_validator.rs                                     1903              1879     1.26%         114               108     5.26%        1637              1587     3.05%           0                 0         -
export/schema_validator.rs                                       772               221    71.37%          41                10    75.61%         558               156    72.04%           0                 0         -
export/streaming_json_writer.rs                                  548               451    17.70%          32                20    37.50%         327               226    30.89%           0                 0         -
export/system_optimizer.rs                                       644               644     0.00%          27                27     0.00%         469               469     0.00%           0                 0         -
export/visualization.rs                                         2504              2204    11.98%          55                45    18.18%        1425              1247    12.49%           0                 0         -
lib.rs                                                          1749               790    54.83%         141                55    60.99%        1105               584    47.15%           0                 0         -
main.rs                                                          448               175    60.94%          20                 5    75.00%         277               152    45.13%           0                 0         -
utils.rs                                                        1469               221    84.96%          53                 3    94.34%         737                76    89.69%           0                 0         -
variable_registry.rs                                            1611               238    85.23%          78                19    75.64%         997               122    87.76%           0                 0         -
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                         105032             39476    62.42%        5594              1807    67.70%       72225             26035    63.95%           0                 0         -

这是 cargo llvm-cov 运行之后的覆盖率的结果，建议你按照覆盖率低的模块进行优化， 步骤则是，先定位模块，然后仔细阅读模块的源码和设计的理念，之后按照你的理解，编写合适的 test mod 并且保证test mod 可以全部通过（当个文件，以及多个文件混合，都要通过）全部通过之后，转移到下一个模块，直到完成覆盖率优化。



/// 这是我的事情，和你没关系啊，。先不要管他。
/// export svg 部分也是要修改的，因为导出的json 字段变换了，这部分也要进行相应的变化。