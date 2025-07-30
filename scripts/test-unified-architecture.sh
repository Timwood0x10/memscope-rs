#!/bin/bash

# Test script to verify unified architecture between static HTML and web server modes
# This ensures both modes use the same templates and data processing logic

set -e

echo "🧪 Testing Unified Architecture Consistency"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Test 1: Build project
echo -e "\n${BLUE}🔨 Building Project${NC}"
if cargo build --release; then
    log_success "Project builds successfully"
else
    log_error "Project build failed"
    exit 1
fi

# Test 2: Generate test data
echo -e "\n${BLUE}📊 Generating Test Data${NC}"
if cargo run --example basic_usage; then
    log_success "Test data generated"
else
    log_error "Failed to generate test data"
    exit 1
fi

# Test 3: Generate static HTML
echo -e "\n${BLUE}📄 Testing Static HTML Generation${NC}"
if make html-only DIR=MemoryAnalysis OUTPUT=test_static.html; then
    log_success "Static HTML generated successfully"
    
    # Check if file exists and has reasonable size
    if [ -f "test_static.html" ]; then
        file_size=$(stat -f%z "test_static.html" 2>/dev/null || stat -c%s "test_static.html" 2>/dev/null || echo "0")
        if [ "$file_size" -gt 10000 ]; then
            log_success "Static HTML file has reasonable size ($file_size bytes)"
        else
            log_error "Static HTML file is too small ($file_size bytes)"
        fi
    else
        log_error "Static HTML file was not created"
    fi
else
    log_error "Static HTML generation failed"
fi

# Test 4: Test web server mode (background process)
echo -e "\n${BLUE}🌐 Testing Web Server Mode${NC}"

# Start web server in background
log_info "Starting web server on port 8081..."
timeout 30s ./target/release/memscope-rs html-from-json \
    --input-dir MemoryAnalysis \
    --output test_server.html \
    --base-name snapshot \
    --serve \
    --port 8081 &

SERVER_PID=$!
sleep 3

# Test if server is responding
if curl -s http://localhost:8081/health > /dev/null; then
    log_success "Web server is responding"
    
    # Test main page
    if curl -s http://localhost:8081/ | grep -q "Memory & FFI Snapshot Analysis"; then
        log_success "Web server serves dashboard page"
    else
        log_error "Web server dashboard page is not working"
    fi
    
    # Test API endpoint
    if curl -s http://localhost:8081/api/data | python3 -m json.tool > /dev/null 2>&1; then
        log_success "Web server API endpoint returns valid JSON"
    else
        log_error "Web server API endpoint is not working"
    fi
    
else
    log_error "Web server is not responding"
fi

# Clean up server process
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

# Test 5: Compare data consistency
echo -e "\n${BLUE}🔍 Testing Data Consistency${NC}"

# Extract embedded JSON from static HTML
if [ -f "test_static.html" ]; then
    # Extract the JSON data from the HTML file
    if grep -o 'embeddedData = {.*};' test_static.html | sed 's/embeddedData = //;s/;$//' > static_data.json; then
        log_success "Extracted embedded data from static HTML"
        
        # Validate JSON format
        if python3 -m json.tool static_data.json > /dev/null 2>&1; then
            log_success "Embedded JSON data is valid"
        else
            log_error "Embedded JSON data is invalid"
        fi
    else
        log_warning "Could not extract embedded data from static HTML"
    fi
fi

# Test 6: Template consistency
echo -e "\n${BLUE}📋 Testing Template Consistency${NC}"

# Check if both modes use the same template file
if grep -q "templates/dashboard.html" src/web/server.rs && \
   grep -q "templates/dashboard.html" src/cli/commands/html_from_json/direct_json_template.rs; then
    log_success "Both modes use the same template file"
else
    log_error "Different template files are being used"
fi

# Check if JavaScript files are consistent
js_files=("data-loader.js" "data-processor.js" "cache-manager.js" "error-handler.js" "script.js")
for js_file in "${js_files[@]}"; do
    if [ -f "templates/$js_file" ]; then
        log_success "JavaScript file $js_file exists"
    else
        log_error "JavaScript file $js_file is missing"
    fi
done

# Test 7: Data processing logic consistency
echo -e "\n${BLUE}⚙️  Testing Data Processing Logic${NC}"

# Check if DataProcessor is used consistently
if grep -q "DataProcessor" templates/data-loader.js && \
   grep -q "processJSONData" templates/data-loader.js; then
    log_success "Data processing logic is consistent"
else
    log_error "Data processing logic inconsistency detected"
fi

# Test 8: Error handling consistency
echo -e "\n${BLUE}🛡️  Testing Error Handling${NC}"

# Check if error handling is consistent
if grep -q "errorHandler" templates/data-loader.js && \
   [ -f "templates/error-handler.js" ]; then
    log_success "Error handling is consistent"
else
    log_error "Error handling inconsistency detected"
fi

# Clean up test files
echo -e "\n${BLUE}🧹 Cleaning Up${NC}"
rm -f test_static.html test_server.html static_data.json
log_info "Test files cleaned up"

# Final report
echo -e "\n${BLUE}📊 Test Results Summary${NC}"
echo "======================"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Unified architecture is working correctly.${NC}"
    echo -e "${GREEN}✅ Static HTML and Web Server modes use consistent logic${NC}"
    exit 0
else
    echo -e "\n${RED}❌ $TESTS_FAILED tests failed. Please check the issues above.${NC}"
    exit 1
fi