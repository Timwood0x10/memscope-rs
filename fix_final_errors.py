#!/usr/bin/env python3

# 读取文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'r') as f:
    content = f.read()

# 修复第8209行的问题
content = content.replace(
    '                    "<div class="absolute inset-0 rounded-full" " +',
    '                    "<div class=\"absolute inset-0 rounded-full\" " +'
)

# 修复第8274行
content = content.replace(
    'console.log("📊 Collapsed lifecycle to show first", maxInitialRows, \'variables\');',
    'console.log("📊 Collapsed lifecycle to show first", maxInitialRows, "variables");'
)

# 修复第8278行
content = content.replace(
    'console.log("✅ Lifecycle toggle button initialized successfully\');',
    'console.log("✅ Lifecycle toggle button initialized successfully");'
)

# 写回文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'w') as f:
    f.write(content)

print("修复了剩余的错误")