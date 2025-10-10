#!/usr/bin/env python3

# 读取文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'r') as f:
    lines = f.readlines()

# 修复每一行的问题
fixed_lines = []
for line in lines:
    # 修复HTML属性中的引号问题
    if '<div class="absolute inset-0 rounded-full"' in line:
        line = line.replace('<div class="absolute inset-0 rounded-full"', '<div class="absolute inset-0 rounded-full"')
    
    # 修复style属性
    if '"style=""' in line:
        line = line.replace('"style=""', '"style="')
    
    # 修复单引号字符串为双引号
    if "console.log('" in line:
        line = line.replace("console.log('", 'console.log("')
        line = line.replace("',", '",')
    
    if "', variableGroups.length, '" in line:
        line = line.replace("', variableGroups.length, '", '", variableGroups.length, "')
    
    if "addEventListener('click'" in line:
        line = line.replace("addEventListener('click'", 'addEventListener("click"')
    
    if "querySelector('span')" in line:
        line = line.replace("querySelector('span')", 'querySelector("span")')
    
    if "className = '" in line:
        line = line.replace("className = '", 'className = "')
        line = line.replace("';", '";')
    
    fixed_lines.append(line)

# 写回文件
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'w') as f:
    f.writelines(fixed_lines)

print("修复了所有引号问题")