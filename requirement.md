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

