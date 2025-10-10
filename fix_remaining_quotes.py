#!/usr/bin/env python3

# 读取文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'r') as f:
    content = f.read()

# 修复引号问题
content = content.replace('"<div class="absolute inset-0 rounded-full" "', '"<div class=\"absolute inset-0 rounded-full\" "')

# 写回文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'w') as f:
    f.write(content)

print("修复了引号问题")