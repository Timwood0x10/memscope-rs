#!/usr/bin/env python3
import re

# Read the file
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'r') as f:
    lines = f.readlines()

# Process each line to fix quote issues
fixed_lines = []
for line in lines:
    # Skip lines that are already correct or don't need fixing
    if 'const gradientStyle' in line or 'varDiv.innerHTML' in line:
        fixed_lines.append(line)
        continue
    
    # Replace single quotes around HTML strings with double quotes
    # First, handle complete HTML strings
    if "' <" in line and ">' +" in line:
        line = line.replace("'", '"')
    # Handle closing tags
    elif "' </" in line and ">' +" in line:
        line = line.replace("'", '"')
    # Handle style attributes
    elif "' style=\"" in line:
        line = line.replace("'", '"')
    # Handle title attributes
    elif "' title=\"" in line:
        line = line.replace("'", '"')
    # Handle other attributes
    elif re.match(r"\s*' [^=]+\" [^']+'", line):
        line = line.replace("'", '"')
    
    fixed_lines.append(line)

# Write back
with open('/Users/scc/code/rustcode/memscope-rs/src/cli/commands/html_from_json/direct_json_template.rs', 'w') as f:
    f.writelines(fixed_lines)

print("Fixed all HTML strings")