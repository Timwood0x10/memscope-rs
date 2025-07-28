# 🔧 MemScope-RS Troubleshooting Guide

This guide helps you resolve common issues with MemScope-RS HTML dashboard generation.

## 📋 Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Common Issues](#common-issues)
- [Data Loading Problems](#data-loading-problems)
- [Performance Problems](#performance-problems)
- [Template and JavaScript Issues](#template-and-javascript-issues)
- [Advanced Troubleshooting](#advanced-troubleshooting)

## 🚀 Quick Diagnostics

### Run Deployment Verification
```bash
# Run comprehensive deployment test
./scripts/verify-deployment.sh

# Test unified architecture consistency
./scripts/test-unified-architecture.sh
```

### Check System Requirements
```bash
# Verify Rust version (requires 1.70+)
rustc --version

# Check if project builds
make check

# Verify examples work
make run-basic
```

## 🐛 Common Issues

### Issue: "No JSON files found in directory"

**Symptoms:**
- Error message: "No JSON files found in MemoryAnalysis/"
- HTML generation fails
- Empty or missing data directory

**Solutions:**
1. **Generate test data first:**
   ```bash
   make run-basic
   # This creates MemoryAnalysis/ directory with sample data
   ```

2. **Check directory contents:**
   ```bash
   ls -la MemoryAnalysis/
   # Should show .json files
   ```

3. **Use custom directory:**
   ```bash
   make html DIR=path/to/your/json/files
   ```

4. **Verify JSON file format:**
   ```bash
   # Check if files are valid JSON
   jq . MemoryAnalysis/snapshot_memory_analysis_memory_analysis.json
   ```

### Issue: "Failed to build project"

**Symptoms:**
- Compilation errors
- Missing dependencies
- Rust version conflicts

**Solutions:**
1. **Update Rust:**
   ```bash
   rustup update
   ```

2. **Clean and rebuild:**
   ```bash
   make clean
   make release
   ```

3. **Check dependencies:**
   ```bash
   cargo check --all-targets --all-features
   ```

### Issue: "HTML file generated but appears broken"

**Symptoms:**
- HTML file exists but shows errors in browser
- JavaScript console errors
- Missing visualizations

**Solutions:**
1. **Check HTML file size:**
   ```bash
   ls -lh memory_report.html
   # Should be > 100KB for typical data
   ```

2. **Validate HTML content:**
   ```bash
   grep -q "Memory & FFI Snapshot Analysis" memory_report.html
   echo $? # Should output 0
   ```

3. **Check for JavaScript errors:**
   - Open browser developer tools (F12)
   - Look for errors in Console tab
   - Check Network tab for failed requests

4. **Regenerate with verbose output:**
   ```bash
   make html-only DIR=MemoryAnalysis OUTPUT=debug_report.html
   ```

## 📊 Data Loading Problems

### Issue: "Data loading failed" or "Cache errors"

**Symptoms:**
- Dashboard shows "Loading..." indefinitely
- Console errors about failed data loading
- Cache-related error messages

**Solutions:**
1. **Clear browser cache:**
   - Hard refresh: Ctrl+F5 (Windows/Linux) or Cmd+Shift+R (Mac)
   - Or clear browser cache manually

2. **Check data file integrity:**
   ```bash
   # Validate all JSON files
   for file in MemoryAnalysis/*.json; do
       echo "Checking $file..."
       jq . "$file" > /dev/null || echo "Invalid JSON: $file"
   done
   ```

3. **Test with minimal data:**
   ```bash
   # Create minimal test data
   mkdir test_minimal
   echo '{"stats": {"total_allocations": 1}}' > test_minimal/snapshot_memory_analysis_memory_analysis.json
   make html-only DIR=test_minimal OUTPUT=minimal_test.html
   ```

4. **Enable debug mode:**
   - Open browser developer tools
   - Check Console for detailed error messages
   - Look for network request failures

### Issue: "Partial data loading warnings"

**Symptoms:**
- Yellow warning banner about partial data
- Some dashboard sections empty
- Missing visualizations

**Solutions:**
1. **Check which files are missing:**
   ```bash
   # Expected files:
   ls MemoryAnalysis/snapshot_memory_analysis_*.json
   ls MemoryAnalysis/snapshot_unsafe_ffi.json
   ```

2. **Generate complete dataset:**
   ```bash
   # Run different examples to generate more data
   make run-lifecycle
   make run-complex-lifecycle-showcase
   ```

3. **Accept partial data:**
   - The dashboard is designed to work with partial data
   - Missing sections will show appropriate messages
   - Core functionality remains available


## ⚡ Performance Problems

### Issue: "Slow HTML generation"

**Symptoms:**
- HTML generation takes very long
- High memory usage during generation
- System becomes unresponsive

**Solutions:**
1. **Check data size:**
   ```bash
   du -sh MemoryAnalysis/
   # Large datasets (>100MB) may be slow
   ```

2. **Use release build:**
   ```bash
   make release
   # Ensure using optimized binary
   ```

3. **Reduce data size:**
   ```bash
   # Create smaller test dataset
   head -n 1000 large_file.json > smaller_file.json
   ```

4. **Monitor system resources:**
   ```bash
   # Check memory usage during generation
   top -p $(pgrep memscope-rs)
   ```

### Issue: "Dashboard loads slowly in browser"

**Symptoms:**
- Long loading times
- Browser becomes unresponsive
- High memory usage in browser

**Solutions:**
1. **Check HTML file size:**
   ```bash
   ls -lh memory_report.html
   # Files >10MB may be slow to load
   ```

2. **Optimize data processing:**
   ```bash
   # Use optimized HTML generation
   make html-only
   ```

3. **Enable browser performance monitoring:**
   - Open Developer Tools → Performance tab
   - Record page load to identify bottlenecks

4. **Reduce data complexity:**
   - Use smaller datasets for testing
   - Filter data before generation

## 🔧 Template and JavaScript Issues

### Issue: "JavaScript errors in console"

**Symptoms:**
- Console shows JavaScript errors
- Dashboard functionality broken
- Visualizations not working

**Solutions:**
1. **Check template files exist:**
   ```bash
   ls -la templates/
   # Should show dashboard.html, script.js, etc.
   ```

2. **Validate JavaScript syntax:**
   ```bash
   # Check for syntax errors
   node -c templates/script.js
   node -c templates/data-loader.js
   ```

3. **Test with minimal JavaScript:**
   - Disable browser extensions
   - Try in incognito/private mode
   - Test in different browser

4. **Check for missing dependencies:**
   - Ensure Chart.js and other libraries load
   - Check Network tab for failed CDN requests

### Issue: "Charts or visualizations not displaying"

**Symptoms:**
- Empty chart containers
- Missing graphs
- Layout issues

**Solutions:**
1. **Check Chart.js loading:**
   ```javascript
   // In browser console
   console.log(typeof Chart);
   // Should output "function"
   ```

2. **Verify data format:**
   ```javascript
   // In browser console
   console.log(window.UNIFIED_DATA);
   // Should show data structure
   ```

3. **Test with sample data:**
   - Use minimal test dataset
   - Check if charts work with simple data

## 🔍 Advanced Troubleshooting

### Enable Debug Logging

1. **Add debug output to templates:**
   ```javascript
   // Add to templates/script.js
   console.log('Debug: Data loaded', data);
   console.log('Debug: Processing step X');
   ```

2. **Use browser debugging:**
   - Set breakpoints in JavaScript
   - Step through data processing
   - Inspect variable values

### Analyze Generated HTML

1. **Extract embedded data:**
   ```bash
   # Extract JSON data from HTML
   grep -o 'const UNIFIED_DATA = {.*};' memory_report.html > extracted_data.js
   ```

2. **Validate data structure:**
   ```bash
   # Check data completeness
   node -e "
   const fs = require('fs');
   const data = fs.readFileSync('extracted_data.js', 'utf8');
   const json = data.replace('const UNIFIED_DATA = ', '').replace(';', '');
   const parsed = JSON.parse(json);
   console.log('Data keys:', Object.keys(parsed));
   console.log('Allocations count:', parsed.memory_analysis?.allocations?.length || 0);
   "
   ```

### Performance Profiling

1. **Profile HTML generation:**
   ```bash
   # Time the generation process
   time make html-only DIR=MemoryAnalysis OUTPUT=profile_test.html
   ```

2. **Memory usage analysis:**
   ```bash
   # Monitor memory during generation
   /usr/bin/time -v ./target/release/memscope-rs html-from-json --input-dir MemoryAnalysis --output profile_test.html --base-name snapshot
   ```

### Create Minimal Reproduction

1. **Generate minimal test case:**
   ```bash
   # Create minimal data
   mkdir minimal_repro
   echo '{
     "stats": {"total_allocations": 1, "active_memory": 1024},
     "allocations": [{"id": 1, "size": 1024, "type_name": "test"}]
   }' > minimal_repro/snapshot_memory_analysis_memory_analysis.json
   
   # Test with minimal data
   make html-only DIR=minimal_repro OUTPUT=minimal_repro.html
   ```

2. **Gradually add complexity:**
   - Add more data fields one by one
   - Identify which field causes issues
   - Report specific problem

## 📞 Getting Help

If you're still experiencing issues:

1. **Check existing issues:**
   - Search GitHub issues for similar problems
   - Look for recent bug reports

2. **Gather diagnostic information:**
   ```bash
   # System info
   uname -a
   rustc --version
   cargo --version
   
   # Project info
   git log --oneline -5
   ls -la MemoryAnalysis/
   
   # Error logs
   make html-only 2>&1 | tee error.log
   ```

3. **Create issue report:**
   - Include system information
   - Provide error logs
   - Describe steps to reproduce
   - Attach minimal test case if possible

## 🎯 Prevention Tips

1. **Regular testing:**
   ```bash
   # Run verification regularly
   ./scripts/verify-deployment.sh
   ```

2. **Keep data clean:**
   - Validate JSON files regularly
   - Remove corrupted data files
   - Monitor data size growth

3. **Update dependencies:**
   ```bash
   cargo update
   make check
   ```

4. **Monitor performance:**
   - Track HTML generation times
   - Monitor memory usage
   - Test with various data sizes

---

**Need more help?** Check the [README.md](README.md) for additional documentation or create an issue on GitHub with your specific problem and diagnostic information.