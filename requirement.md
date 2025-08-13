## 重点

1. **English-only comments** - 所有代码注释必须是英文
2. **7:3 code-to-comment ratio** - 保持适当的文档化比例
3. **Unified error handling** - 统一错误处理系统
4. **No locks, unwrap, or clone** - 禁止使用锁、unwrap和clone，使用有意义的错误来代替unwrap。
5. **Simple architecture** - 保持架构简洁，专注核心功能
6. **Zero functionality impact** - 禁止影响任何现有功能，特别是数据获取、JSON/binary/HTML导出
7. **Meaningful names** - 所有目录和文件必须有描述性的有意义名称
8. **Use make check** - 禁止使用cargo check，必须使用make check检查完整日志
9.**Use tracking** -  禁止使用println! 使用tracking 来显示日志。
10. 所有的改动基于v5-pre branch，
11. 禁止使用没有意义的变量名字和函数名字。
12. 禁止影响当前json file 的输出内容。
13.禁止产生技术债务。也就说这个task 必须完成binary---html的优化工作，而不是留下任何一个TODO。
14. 代码应该精简，而不是很冗余，比如说能用match 就不要用if else，要符合rust的编码规范。
15. 要求0 error，0 warning
16. 架构一定要简单，代码要精简，有简短的代码，完成复杂的需求。
17. 测试代码一定要有意义，测试程序中的核心功能，且保证所有测试必须通过，以及测试运行时间短。
18. 对于新增的功能，测试要做到全面。
19. 要保证输出的html，和没改动之前的json一致啊，也就是说binary中生存的json file 是5个，要和MemoryAnalysis/binary_demo_example/*.json 一致
20. 严禁创建乱七八糟的test files
21. 严禁影响到其他的功能，尤其是full-binary ---> json 的导出时间。
