# Makefile for memscope-rs - Rust Memory Analysis Toolkit
# Author: memscope-rs development team
# Description: Build, test, and deployment automation

# Variables
CARGO := cargo
PROJECT_NAME := memscope-rs
VERSION := $(shell grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
TARGET_DIR := target
DOCS_DIR := target/doc
COVERAGE_DIR := target/coverage

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

# Default target
.PHONY: all
all: check test

# Help target
.PHONY: help
help:
	@echo "$(BLUE)memscope-rs Makefile - Available targets:$(NC)"
	@echo ""
	@echo "$(GREEN)Building:$(NC)"
	@echo "  build          - Build the project in debug mode"
	@echo "  release        - Build the project in release mode"
	@echo "  check          - Check the project for errors"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@echo "  test           - Run all tests (with reduced logging)"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-stress    - Run stress tests"
	@echo "  test-safety    - Run safety tests"
	@echo "  test-performance - Run performance tests"
	@echo "  test-edge      - Run edge case tests"
	@echo "  test-fast      - Run fast tests (unit tests in release mode)"
	@echo "  test-quiet     - Run all tests quietly (no logs)"
	@echo "  test-verbose   - Run tests with verbose output"
	@echo ""
	@echo "$(GREEN)Quality Assurance:$(NC)"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  clippy         - Run clippy linter"
	@echo "  clippy-fix     - Run clippy with automatic fixes"
	@echo "  audit          - Security audit dependencies"
	@echo ""
	@echo "$(GREEN)Documentation:$(NC)"
	@echo "  doc            - Generate documentation"
	@echo "  doc-open       - Generate and open documentation"
	@echo "  doc-check      - Check documentation"
	@echo ""
	@echo "$(GREEN)Examples:$(NC)"
	@echo "  run-basic      - Run basic usage example"
	@echo "  run-ownership  - Run ownership patterns demo"
	@echo "  run-unsafe-ffi - Run unsafe/FFI safety demonstration"
	@echo "  run-improved-tracking - Run improved tracking showcase"
	@echo "  run-speed-test - Run speed test example"
	@echo "  run-lifecycle  - Run lifecycle example"
	@echo "  run-main       - Run main application"
	@echo "  run-memory-stress - Run memory stress test example"
	@echo "  run-complex-lifecycle-showcase - Run complex lifecycle showcase example"
	@echo ""
	@echo "$(GREEN)Binary Tools:$(NC)"
	@echo "  run-benchmark  - Run comprehensive performance benchmarks"
	@echo "  run-simple-benchmark - Run simple benchmark testing"
	@echo "  run-core-performance - Run core performance evaluation"
	@echo "  run-performance-only - Run performance-only benchmark"
	@echo "  run-lifecycle-analysis - Run lifecycle analysis tool"
	@echo "  run-allocation-diagnostic - Run allocation count diagnostics"
	@echo "  run-large-allocations - Run large active allocations analysis"
	@echo "  run-test-validation - Run test mode validation"
	@echo ""
	@echo "$(GREEN)HTML Reports (Enhanced):$(NC)"
	@echo "  html           - Generate HTML report"
	@echo "                   Usage: make html [DIR=path] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]"
	@echo "  html-only      - Generate HTML report only"
	@echo "                   Usage: make html-only [DIR=path] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]"
	@echo "  html-clean     - Clean generated HTML files"
	@echo "  html-help      - Show detailed HTML command usage and examples"
	@echo ""
	@echo "$(GREEN)Demonstrations:$(NC)"
	@echo "  demo           - Quick demonstration workflow (build + basic example + HTML)"
	@echo "  demo-all       - Comprehensive feature demonstration"
	@echo "  perf-demo      - Performance evaluation workflow"
	@echo ""
	@echo "$(GREEN)CI/CD:$(NC)"
	@echo "  ci             - Run full CI pipeline locally"
	@echo "  pre-commit     - Run pre-commit checks"
	@echo "  coverage       - Generate test coverage report (tarpaulin)"
	@echo "  coverage-llvm  - Generate LLVM coverage report (HTML)"
	@echo "  coverage-summary - Generate LLVM coverage summary (console)"
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@echo "  update         - Update dependencies"
	@echo "  outdated       - Check for outdated dependencies"
	@echo "  tree           - Show dependency tree"
	@echo "  bloat          - Analyze binary size"
	@echo "$(YELLOW)Available targets:$(NC)"
	@echo ""
	@echo "  $(GREEN)test-all$(NC)               - Run all tests"
	@echo "  $(GREEN)benchmark$(NC)          - Run performance benchmarks"
	@echo "  $(GREEN)benchmark-main$(NC)     - Run only main (realistic) benchmarks"
	@echo "  $(GREEN)full-report$(NC)        - Generate comprehensive project report"
	@echo "  $(GREEN)validate$(NC)           - Full validation (CI + examples + docs)"
	@echo ""
	@echo "$(YELLOW)Development targets:$(NC)"
	@echo "  $(GREEN)dev-setup$(NC)          - Setup development environment"
	@echo "  $(GREEN)quick-test$(NC)         - Run quick tests (lib only)"
	@echo "  $(GREEN)watch$(NC)              - Watch for changes and run tests"

# Building targets
.PHONY: build
build:
	@echo "$(BLUE)Building $(PROJECT_NAME) in debug mode...$(NC)"
	$(CARGO) build

.PHONY: release
release:
	@echo "$(BLUE)Building $(PROJECT_NAME) in release mode...$(NC)"
	$(CARGO) build --release --bin memscope-rs

.PHONY: check
check:
	@echo "$(BLUE)Checking $(PROJECT_NAME) for errors...$(NC)"
	$(CARGO) check --all-targets --all-features

.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf $(COVERAGE_DIR)
	rm -f *.json *.svg
	@echo "$(GREEN)Clean completed$(NC)"

# Testing targets
.PHONY: test
test:
	@echo "$(BLUE)Running all tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --all --features test -- --test-threads=1

.PHONY: test-unit
test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --lib --features test -- --test-threads=1

.PHONY: test-integration
test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test comprehensive_integration_test --features test -- --test-threads=1

.PHONY: test-stress
test-stress:
	@echo "$(BLUE)Running stress tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test stress_test --features test -- --test-threads=1

.PHONY: test-safety
test-safety:
	@echo "$(BLUE)Running safety tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test safety_test --features test -- --test-threads=1

.PHONY: test-performance
test-performance:
	@echo "$(BLUE)Running performance tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test performance_test --release --features test -- --test-threads=1

.PHONY: test-edge
test-edge:
	@echo "$(BLUE)Running edge case tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test edge_cases_test --features test -- --test-threads=1

.PHONY: test-comprehensive
test-comprehensive:
	@echo "$(BLUE)Running comprehensive integration tests...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=error $(CARGO) test --test comprehensive_integration_test --features test -- --test-threads=1

.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running all tests with verbose output...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=debug $(CARGO) test --all --verbose --features test -- --test-threads=1

.PHONY: test-fast
test-fast:
	@echo "$(BLUE)Running fast tests (unit tests only)...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=off $(CARGO) test --lib --release --features test -- --test-threads=1

.PHONY: test-quiet
test-quiet:
	@echo "$(BLUE)Running all tests quietly...$(NC)"
	MEMSCOPE_TEST_MODE=1 RUST_LOG=off $(CARGO) test --all --quiet --features test -- --test-threads=1

# Quality assurance targets
.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	@echo "$(BLUE)Checking code formatting...$(NC)"
	$(CARGO) fmt -- --check

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy linter...$(NC)"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: clippy-fix
clippy-fix:
	@echo "$(BLUE)Running clippy with automatic fixes...$(NC)"
	$(CARGO) clippy --fix --all-targets --all-features

.PHONY: audit
audit:
	@echo "$(BLUE)Running security audit...$(NC)"
	$(CARGO) audit

# Documentation targets
.PHONY: doc
doc:
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --no-deps --all-features

.PHONY: doc-open
doc-open:
	@echo "$(BLUE)Generating and opening documentation...$(NC)"
	$(CARGO) doc --no-deps --all-features --open

.PHONY: doc-check
doc-check:
	@echo "$(BLUE)Checking documentation...$(NC)"
	$(CARGO) doc --no-deps --all-features --document-private-items

# Example targets
.PHONY: run-basic
run-basic:
	@echo "$(BLUE)Running basic usage example...$(NC)"
	$(CARGO) run --example basic_usage

.PHONY: run-ownership
run-ownership:
	@echo "$(BLUE)Running ownership patterns demo...$(NC)"
	$(CARGO) run --example ownership_demo

.PHONY: run-unsafe-ffi
run-unsafe-ffi:
	@echo "$(BLUE)Running unsafe/FFI safety demonstration...$(NC)"
	$(CARGO) run --example unsafe_ffi_demo

.PHONY: run-improved-tracking
run-improved-tracking:
	@echo "$(BLUE)Running improved tracking showcase...$(NC)"
	$(CARGO) run --example improved_tracking_showcase

.PHONY: run-speed-test
run-speed-test:
	@echo "$(BLUE)Running speed test example...$(NC)"
	$(CARGO) run --example speed_test

.PHONY: run-memory-stress
run-memory-stress:
	@echo "$(BLUE)Running memory stress test example...$(NC)"
	$(CARGO) run --example memory_stress_test

.PHONY: run-lifecycle
run-lifecycle:
	@echo "$(BLUE)Running lifecycle example...$(NC)"
	$(CARGO) run --example lifecycles_simple

.PHONY: run-main
run-main:
	@echo "$(BLUE)Running main application...$(NC)"
	$(CARGO) run

.PHONY: run-complex-lifecycle-showcase
run-complex-lifecycle-showcase:
	@echo "$(BLUE)Running complex lifecycle showcase example...$(NC)"
	$(CARGO) run --example complex_lifecycle_showcase

# Binary tools targets
.PHONY: run-benchmark
run-benchmark:
	@echo "$(BLUE)Running comprehensive performance benchmarks...$(NC)"
	$(CARGO) run --bin run_benchmark

.PHONY: run-simple-benchmark
run-simple-benchmark:
	@echo "$(BLUE)Running simple benchmark testing...$(NC)"
	$(CARGO) run --bin simple_benchmark

.PHONY: run-core-performance
run-core-performance:
	@echo "$(BLUE)Running core performance evaluation...$(NC)"
	$(CARGO) run --bin core_performance_test

.PHONY: run-performance-only
run-performance-only:
	@echo "$(BLUE)Running performance-only benchmark...$(NC)"
	$(CARGO) run --bin performance_only_benchmark

.PHONY: run-lifecycle-analysis
run-lifecycle-analysis:
	@echo "$(BLUE)Running lifecycle analysis tool...$(NC)"
	$(CARGO) run --bin lifecycle_analysis

.PHONY: run-allocation-diagnostic
run-allocation-diagnostic:
	@echo "$(BLUE)Running allocation count diagnostics...$(NC)"
	$(CARGO) run --bin allocation_count_diagnostic

.PHONY: run-large-allocations
run-large-allocations:
	@echo "$(BLUE)Running large active allocations analysis...$(NC)"
	$(CARGO) run --bin large_active_allocations

.PHONY: run-test-validation
run-test-validation:
	@echo "$(BLUE)Running test mode validation...$(NC)"
	$(CARGO) run --bin test_mode_specific_validation

# CI/CD targets
.PHONY: ci
ci: clean check fmt-check clippy test doc
	@echo "$(GREEN)✅ Full CI pipeline completed successfully!$(NC)"

.PHONY: pre-commit
pre-commit: fmt clippy test-unit
	@echo "$(GREEN)✅ Pre-commit checks completed!$(NC)"

.PHONY: coverage coverage-llvm coverage-summary
coverage:
	@echo "$(BLUE)Generating test coverage report...$(NC)"
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		mkdir -p $(COVERAGE_DIR); \
		$(CARGO) tarpaulin --out Html --output-dir $(COVERAGE_DIR); \
		echo "$(GREEN)Coverage report generated in $(COVERAGE_DIR)/tarpaulin-report.html$(NC)"; \
	else \
		echo "$(YELLOW)cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin$(NC)"; \
	fi

coverage-llvm:
	@echo "$(BLUE)Generating LLVM coverage report...$(NC)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		mkdir -p $(COVERAGE_DIR); \
		$(CARGO) llvm-cov --html --output-dir $(COVERAGE_DIR) --lib --tests -- --test-thread=1; \
		echo "$(GREEN)LLVM coverage report generated in $(COVERAGE_DIR)/index.html$(NC)"; \
	else \
		echo "$(YELLOW)cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov$(NC)"; \
	fi

coverage-summary:
	@echo "$(BLUE)Generating LLVM coverage summary...$(NC)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		$(CARGO) llvm-cov  --summary-only  --lib --tests -- --test-threads=1; \
	else \
		echo "$(YELLOW)cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov$(NC)"; \
	fi

# Maintenance targets
.PHONY: update
update:
	@echo "$(BLUE)Updating dependencies...$(NC)"
	$(CARGO) update

.PHONY: outdated
outdated:
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		$(CARGO) outdated; \
	else \
		echo "$(YELLOW)cargo-outdated not installed. Install with: cargo install cargo-outdated$(NC)"; \
	fi

.PHONY: tree
tree:
	@echo "$(BLUE)Showing dependency tree...$(NC)"
	$(CARGO) tree

.PHONY: bloat
bloat:
	@echo "$(BLUE)Analyzing binary size...$(NC)"
	@if command -v cargo-bloat >/dev/null 2>&1; then \
		$(CARGO) bloat --release; \
	else \
		echo "$(YELLOW)cargo-bloat not installed. Install with: cargo install cargo-bloat$(NC)"; \
	fi

# Benchmark targets
.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) test --release --test performance_test

# Installation targets
.PHONY: install
install:
	@echo "$(BLUE)Installing $(PROJECT_NAME)...$(NC)"
	$(CARGO) install --path .

.PHONY: uninstall
uninstall:
	@echo "$(YELLOW)Uninstalling $(PROJECT_NAME)...$(NC)"
	$(CARGO) uninstall $(PROJECT_NAME)

# Development setup
.PHONY: setup-dev
setup-dev:
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@echo "Installing useful cargo tools..."
	@cargo install cargo-audit cargo-outdated cargo-tarpaulin cargo-bloat 2>/dev/null || true
	@echo "$(GREEN)Development environment setup complete!$(NC)"

# Quick development workflow
.PHONY: dev
dev: fmt clippy test-unit
	@echo "$(GREEN)✅ Quick development check completed!$(NC)"

# Release preparation
.PHONY: prepare-release
prepare-release: clean ci
	@echo "$(BLUE)Preparing release v$(VERSION)...$(NC)"
	@echo "$(GREEN)✅ Release preparation completed for v$(VERSION)!$(NC)"
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. Update CHANGELOG.md"
	@echo "  2. Commit changes: git commit -am 'Prepare release v$(VERSION)'"
	@echo "  3. Tag release: git tag v$(VERSION)"
	@echo "  4. Push: git push origin main --tags"

# Show project info
.PHONY: info
info:
	@echo "$(BLUE)Project Information:$(NC)"
	@echo "  Name: $(PROJECT_NAME)"
	@echo "  Version: $(VERSION)"
	@echo "  Rust version: $(shell rustc --version)"
	@echo "  Cargo version: $(shell cargo --version)"
	@echo "  Target directory: $(TARGET_DIR)"
	@echo "  Documentation: $(DOCS_DIR)"

# HTML Report Generation
# Usage: make html [DIR=path/to/json/files] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]
.PHONY: html
html: release
	$(eval INPUT_DIR := $(or $(DIR),MemoryAnalysis/basic_usage))
	$(eval OUTPUT_FILE := $(or $(OUTPUT),memory_report.html))
	$(eval BASE_NAME := $(or $(BASE),snapshot))
	$(eval VERBOSE_FLAG := $(if $(VERBOSE),--verbose,))
	$(eval DEBUG_FLAG := $(if $(DEBUG),--debug,))
	$(eval PERFORMANCE_FLAG := $(if $(PERFORMANCE),--performance,))
	@echo "$(BLUE)Generating HTML report (Enhanced)...$(NC)"
	@echo "$(BLUE)Input directory: $(INPUT_DIR)$(NC)"
	@echo "$(BLUE)Output file: $(OUTPUT_FILE)$(NC)"
	@echo "$(BLUE)Base name: $(BASE_NAME)$(NC)"
	@if [ -n "$(VERBOSE)" ]; then echo "$(BLUE)Verbose mode: enabled$(NC)"; fi
	@if [ -n "$(DEBUG)" ]; then echo "$(BLUE)Debug mode: enabled$(NC)"; fi
	@if [ -n "$(PERFORMANCE)" ]; then echo "$(BLUE)Performance mode: enabled$(NC)"; fi
	@if [ ! -d "$(INPUT_DIR)" ]; then \
		echo "$(YELLOW)Directory $(INPUT_DIR) not found...$(NC)"; \
		if [ "$(INPUT_DIR)" = "MemoryAnalysis/basic_usage" ] || [ "$(INPUT_DIR)" = "MemoryAnalysis" ]; then \
			echo "$(YELLOW)Running basic example to generate data...$(NC)"; \
			$(CARGO) run --example basic_usage; \
		else \
			echo "$(RED)Error: Directory $(INPUT_DIR) does not exist!$(NC)"; \
			echo "$(YELLOW)Please create the directory or run: make html DIR=existing_directory$(NC)"; \
			exit 1; \
		fi \
	fi
	@echo "$(GREEN)Scanning $(INPUT_DIR) for JSON files...$(NC)"
	@json_count=$$(find "$(INPUT_DIR)" -name "*.json" -type f | wc -l); \
	if [ $$json_count -eq 0 ]; then \
		echo "$(YELLOW)No JSON files found in $(INPUT_DIR)$(NC)"; \
		if [ "$(INPUT_DIR)" = "MemoryAnalysis" ]; then \
			echo "$(YELLOW)Running basic example to generate data...$(NC)"; \
			$(CARGO) run --example basic_usage; \
		else \
			echo "$(RED)Error: No JSON files found in $(INPUT_DIR)!$(NC)"; \
			exit 1; \
		fi \
	else \
		echo "$(GREEN)Found $$json_count JSON files in $(INPUT_DIR)$(NC)"; \
	fi
	@echo "$(GREEN)Generating HTML report from $(INPUT_DIR)/ directory...$(NC)"
	./target/release/memscope-rs html-from-json --input-dir "$(INPUT_DIR)" --output "$(OUTPUT_FILE)" --base-name "$(BASE_NAME)" $(VERBOSE_FLAG) $(DEBUG_FLAG) $(PERFORMANCE_FLAG)
	@echo "$(GREEN)✅ HTML report generated: $(OUTPUT_FILE)$(NC)"

# Usage: make html-only [DIR=path/to/json/files] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]
.PHONY: html-only
html-only: release
	$(eval INPUT_DIR := $(or $(DIR),MemoryAnalysis/basic_usage))
	$(eval OUTPUT_FILE := $(or $(OUTPUT),memory_report.html))
	$(eval BASE_NAME := $(or $(BASE),snapshot))
	$(eval VERBOSE_FLAG := $(if $(VERBOSE),--verbose,))
	$(eval DEBUG_FLAG := $(if $(DEBUG),--debug,))
	$(eval PERFORMANCE_FLAG := $(if $(PERFORMANCE),--performance,))
	@echo "$(BLUE)Generating HTML report (Enhanced)...$(NC)"
	@echo "$(BLUE)Input directory: $(INPUT_DIR)$(NC)"
	@echo "$(BLUE)Output file: $(OUTPUT_FILE)$(NC)"
	@echo "$(BLUE)Base name: $(BASE_NAME)$(NC)"
	@if [ -n "$(VERBOSE)" ]; then echo "$(BLUE)Verbose mode: enabled$(NC)"; fi
	@if [ -n "$(DEBUG)" ]; then echo "$(BLUE)Debug mode: enabled$(NC)"; fi
	@if [ -n "$(PERFORMANCE)" ]; then echo "$(BLUE)Performance mode: enabled$(NC)"; fi
	@if [ ! -d "$(INPUT_DIR)" ]; then \
		echo "$(YELLOW)Directory $(INPUT_DIR) not found...$(NC)"; \
		if [ "$(INPUT_DIR)" = "MemoryAnalysis/basic_usage" ] || [ "$(INPUT_DIR)" = "MemoryAnalysis" ]; then \
			echo "$(YELLOW)Running basic example to generate data...$(NC)"; \
			$(CARGO) run --example basic_usage; \
		else \
			echo "$(RED)Error: Directory $(INPUT_DIR) does not exist!$(NC)"; \
			echo "$(YELLOW)Please create the directory or run: make html-only DIR=existing_directory$(NC)"; \
			exit 1; \
		fi \
	fi
	@echo "$(GREEN)Scanning $(INPUT_DIR) for JSON files...$(NC)"
	@json_count=$$(find "$(INPUT_DIR)" -name "*.json" -type f | wc -l); \
	if [ $$json_count -eq 0 ]; then \
		echo "$(YELLOW)No JSON files found in $(INPUT_DIR)$(NC)"; \
		if [ "$(INPUT_DIR)" = "MemoryAnalysis" ]; then \
			echo "$(YELLOW)Running basic example to generate data...$(NC)"; \
			$(CARGO) run --example basic_usage; \
		else \
			echo "$(RED)Error: No JSON files found in $(INPUT_DIR)!$(NC)"; \
			exit 1; \
		fi \
	else \
		echo "$(GREEN)Found $$json_count JSON files in $(INPUT_DIR)$(NC)"; \
	fi
	./target/release/memscope-rs html-from-json --input-dir "$(INPUT_DIR)" --output "$(OUTPUT_FILE)" --base-name "$(BASE_NAME)" $(VERBOSE_FLAG) $(DEBUG_FLAG) $(PERFORMANCE_FLAG)
	@echo "$(GREEN)✅ HTML report generated: $(OUTPUT_FILE)$(NC)"
	@echo "$(BLUE)Open $(OUTPUT_FILE) in your browser to view the interactive report$(NC)"

# Enhanced HTML generation shortcuts
.PHONY: html-verbose html-debug html-performance html-validate
html-verbose: release
	@echo "$(BLUE)Generating HTML report with verbose output...$(NC)"
	$(MAKE) html VERBOSE=1

html-debug: release
	@echo "$(BLUE)Generating HTML report with debug information...$(NC)"
	$(MAKE) html DEBUG=1

html-performance: release
	@echo "$(BLUE)Generating HTML report with performance analysis...$(NC)"
	$(MAKE) html PERFORMANCE=1

html-validate: release
	@echo "$(BLUE)Validating JSON files only (no HTML generation)...$(NC)"
	$(eval INPUT_DIR := $(or $(DIR),MemoryAnalysis/basic_usage))
	$(eval BASE_NAME := $(or $(BASE),snapshot))
	$(eval VERBOSE_FLAG := $(if $(VERBOSE),--verbose,))
	$(eval DEBUG_FLAG := $(if $(DEBUG),--debug,))
	$(eval PERFORMANCE_FLAG := $(if $(PERFORMANCE),--performance,))
	@echo "$(BLUE)Input directory: $(INPUT_DIR)$(NC)"
	@echo "$(BLUE)Base name: $(BASE_NAME)$(NC)"
	@if [ -n "$(VERBOSE)" ]; then echo "$(BLUE)Verbose mode: enabled$(NC)"; fi
	@if [ -n "$(DEBUG)" ]; then echo "$(BLUE)Debug mode: enabled$(NC)"; fi
	@if [ -n "$(PERFORMANCE)" ]; then echo "$(BLUE)Performance mode: enabled$(NC)"; fi
	./target/release/memscope-rs html-from-json --input-dir "$(INPUT_DIR)" --base-name "$(BASE_NAME)" --validate-only $(VERBOSE_FLAG) $(DEBUG_FLAG) $(PERFORMANCE_FLAG)

.PHONY: html-clean
html-clean:
	@echo "$(YELLOW)Cleaning generated HTML files...$(NC)"
	rm -f memory_report.html debug_report.html test_report.html *.html
	@echo "$(GREEN)HTML files cleaned$(NC)"

.PHONY: html-help
html-help:
	@echo "$(BLUE)HTML Report Generation Help (Enhanced)$(NC)"
	@echo "======================================="
	@echo ""
	@echo "$(GREEN)Basic Usage:$(NC)"
	@echo "  make html                    # Use default MemoryAnalysis/basic_usage/ directory"
	@echo "  make html-only               # Generate HTML only, no server"
	@echo ""
	@echo "$(GREEN)Custom Directory:$(NC)"
	@echo "  make html DIR=my_data/       # Use custom directory"
	@echo "  make html DIR=/path/to/json/ # Use absolute path"
	@echo ""
	@echo "$(GREEN)Custom Output:$(NC)"
	@echo "  make html OUTPUT=my_report.html              # Custom output filename"
	@echo "  make html DIR=data/ OUTPUT=custom_report.html # Custom dir and output"
	@echo ""
	@echo "$(GREEN)Custom Base Name:$(NC)"
	@echo "  make html BASE=my_snapshot   # Use custom base name for JSON files"
	@echo "  make html BASE=test_data     # Look for test_data_*.json files"
	@echo ""
	@echo ""
	@echo "$(GREEN)Debug and Performance Options:$(NC)"
	@echo "  make html VERBOSE=1          # Enable verbose output with progress info"
	@echo "  make html DEBUG=1            # Enable debug mode with detailed logging"
	@echo "  make html PERFORMANCE=1      # Enable performance analysis with timing"
	@echo "  make html DEBUG=1 PERFORMANCE=1  # Combine debug and performance modes"
	@echo ""
	@echo "$(GREEN)Combined Examples:$(NC)"
	@echo "  make html DIR=test_data/ OUTPUT=test.html BASE=test VERBOSE=1"
	@echo "  make html-only DIR=../results/ OUTPUT=analysis.html DEBUG=1 PERFORMANCE=1"
	@echo "  make html DIR=MemoryAnalysis/snapshot_memory_analysis BASE=snapshot_memory_analysis VERBOSE=1"
	@echo ""
	@echo "$(GREEN)Requirements:$(NC)"
	@echo "  - Directory must exist and contain .json files"
	@echo "  - JSON files should follow naming pattern: {BASE}_{type}.json"
	@echo ""
	@echo "$(GREEN)Examples of valid directory structures:$(NC)"
	@echo "  my_data/ (with BASE=snapshot)"
	@echo "  ├── snapshot_memory_analysis.json"
	@echo "  ├── snapshot_lifetime.json"
	@echo "  ├── snapshot_unsafe_ffi.json"
	@echo "  ├── snapshot_performance.json"
	@echo "  └── snapshot_complex_types.json"
	@echo ""
	@echo "  custom_data/ (with BASE=my_test)"
	@echo "  ├── my_test_memory_analysis.json"
	@echo "  ├── my_test_lifetime.json"
	@echo "  └── my_test_performance.json"

# Quick demonstration workflow
.PHONY: demo
demo: clean build run-basic html
	@echo "$(GREEN)🎉 Demo completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASS$(NC)"
	@echo "$(GREEN)✅ Basic Example: PASS$(NC)"
	@echo "$(GREEN)✅ HTML Report: PASS$(NC)"
	@echo "$(BLUE)Check memory_report.html to view the analysis results!$(NC)"

# Comprehensive feature demonstration
.PHONY: demo-all
demo-all: clean build run-basic run-ownership run-unsafe-ffi run-improved-tracking html
	@echo "$(GREEN)🎉 Comprehensive demo completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASS$(NC)"
	@echo "$(GREEN)✅ Basic Usage: PASS$(NC)"
	@echo "$(GREEN)✅ Ownership Patterns: PASS$(NC)"
	@echo "$(GREEN)✅ Unsafe/FFI Analysis: PASS$(NC)"
	@echo "$(GREEN)✅ Improved Tracking: PASS$(NC)"
	@echo "$(GREEN)✅ HTML Report: PASS$(NC)"
	@echo "$(BLUE)All features demonstrated! Check the generated HTML reports.$(NC)"

# Performance evaluation workflow
.PHONY: perf-demo
perf-demo: clean build run-benchmark run-simple-benchmark run-core-performance html
	@echo "$(GREEN)🎉 Performance demo completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASS$(NC)"
	@echo "$(GREEN)✅ Comprehensive Benchmark: PASS$(NC)"
	@echo "$(GREEN)✅ Simple Benchmark: PASS$(NC)"
	@echo "$(GREEN)✅ Core Performance: PASS$(NC)"
	@echo "$(GREEN)✅ HTML Report: PASS$(NC)"
	@echo "$(BLUE)Performance analysis completed! Check benchmark_results/ directory.$(NC)"

# Validate all is working
.PHONY: validate
validate: ci run-basic run-lifecycle html
	@echo "$(GREEN)🎉 Full validation completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASS$(NC)"
	@echo "$(GREEN)✅ Tests: PASS$(NC)"
	@echo "$(GREEN)✅ Linting: PASS$(NC)"
	@echo "$(GREEN)✅ Documentation: PASS$(NC)"
	@echo "$(GREEN)✅ Examples: PASS$(NC)"
	@echo "$(GREEN)✅ HTML Report: PASS$(NC)"
	@echo "$(BLUE)memscope-rs is ready for use!$(NC)"


# Testing targets
.PHONY: test-all quick-test test-lib test-integration-coverage test-doc test-examples
test-all:
	@echo "$(BLUE)🧪 Running all tests with coverage...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --lib --tests --examples -- --test-threads=1
	@echo "$(GREEN)✅ All tests with coverage completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

quick-test: test-lib


test-lib:
	@echo "$(BLUE)🧪 Running library tests with coverage...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --lib
	@echo "$(GREEN)✅ Library tests with coverage completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"


test-integration-coverage:
	@echo "$(BLUE)🧪 Running integration tests with coverage...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --tests
	@echo "$(GREEN)✅ Integration tests with coverage completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

test-doc:
	@echo "$(BLUE)🧪 Running documentation tests with coverage...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --doc
	@echo "$(GREEN)✅ Documentation tests with coverage completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

test-examples:
	@echo "$(BLUE)🧪 Running example tests with coverage...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --examples
	@echo "$(GREEN)✅ Example tests with coverage completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

# Enhanced test coverage with tarpaulin
.PHONY: test-coverage test-coverage-enhanced test-coverage-detailed
test-coverage:
	@echo "$(BLUE)📊 Running test coverage analysis with tarpaulin...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report
	@echo "$(GREEN)✅ Test coverage analysis completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

test-coverage-enhanced:
	@echo "$(BLUE)📊 Running enhanced test coverage analysis with tarpaulin...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --verbose --all-features
	@echo "$(GREEN)✅ Enhanced test coverage analysis completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

test-coverage-detailed:
	@echo "$(BLUE)📊 Running detailed test coverage analysis with tarpaulin...$(NC)"
	@mkdir -p coverage-report
	@cargo tarpaulin --out Html --output-dir coverage-report --verbose --all-features --include-tests
	@echo "$(GREEN)✅ Detailed test coverage analysis completed$(NC)"
	@echo "$(BLUE)📊 Coverage report: coverage-report/tarpaulin-report.html$(NC)"

improve-coverage:
	@echo "$(BLUE)🚀 Running coverage improvement analysis...$(NC)"
	@./scripts/improve_test_coverage.sh
	@echo "$(GREEN)✅ Coverage improvement analysis completed$(NC)"

# Benchmarking targets
.PHONY: benchmark benchmark-main benchmark-legacy benchmark-clean benchmark-enhanced
benchmark:
	@echo "$(BLUE)⚡ Running all benchmarks...$(NC)"
	@mkdir -p scripts
	@if [ ! -f scripts/benchmark.sh ]; then \
		echo "$(RED)❌ Benchmark script not found$(NC)"; \
		exit 1; \
	fi
	@./scripts/benchmark.sh
	@echo "$(GREEN)✅ Benchmark analysis completed$(NC)"

benchmark-enhanced:
	@echo "$(BLUE)⚡ Running enhanced benchmarks with HTML dashboard...$(NC)"
	@mkdir -p scripts
	@if [ ! -f scripts/enhanced_benchmark.sh ]; then \
		echo "$(RED)❌ Enhanced benchmark script not found$(NC)"; \
		exit 1; \
	fi
	@./scripts/enhanced_benchmark.sh
	@echo "$(GREEN)✅ Enhanced benchmark analysis completed$(NC)"

benchmark-main:
	@echo "$(BLUE)⚡ Running fast benchmarks...$(NC)"
	@mkdir -p reports/benchmarks
	@echo "$(YELLOW)Running fast realistic tracking benchmark...$(NC)"
	@timeout 120s cargo bench --bench fast_realistic_tracking || echo "$(YELLOW)⚠️  Fast benchmark timeout$(NC)"
	@echo "$(YELLOW)Running minimal performance benchmark...$(NC)"
	@timeout 60s cargo bench --bench minimal_performance || echo "$(YELLOW)⚠️  Minimal benchmark timeout$(NC)"
	@echo "$(GREEN)✅ Fast benchmarks completed$(NC)"

benchmark-slow:
	@echo "$(BLUE)⚡ Running original (slower) benchmarks...$(NC)"
	@mkdir -p reports/benchmarks
	@echo "$(YELLOW)Running realistic memory tracking benchmark...$(NC)"
	@timeout 300s cargo bench --bench realistic_memory_tracking || echo "$(YELLOW)⚠️  Benchmark may have issues$(NC)"
	@echo "$(YELLOW)Running performance comparison benchmark...$(NC)"
	@timeout 300s cargo bench --bench performance_comparison || echo "$(YELLOW)⚠️  Benchmark may have issues$(NC)"
	@echo "$(GREEN)✅ Original benchmarks completed$(NC)"

benchmark-legacy:
	@echo "$(BLUE)⚡ Running legacy benchmarks (compatibility check)...$(NC)"
	@mkdir -p reports/benchmarks
	@cargo bench --bench binary_performance || echo "$(YELLOW)⚠️  Legacy benchmark failed (expected)$(NC)"
	@cargo bench --bench binary_export_performance || echo "$(YELLOW)⚠️  Legacy benchmark failed (expected)$(NC)"
	@cargo bench --bench lock_optimization_benchmark || echo "$(YELLOW)⚠️  Legacy benchmark failed (expected)$(NC)"
	@cargo bench --bench real_optimization_benchmark || echo "$(YELLOW)⚠️  Legacy benchmark failed (expected)$(NC)"
	@echo "$(GREEN)✅ Legacy benchmarks completed$(NC)"

benchmark-clean:
	@echo "$(BLUE)🧹 Cleaning benchmark artifacts...$(NC)"
	@rm -rf target/criterion
	@rm -rf reports/benchmarks
	@echo "$(GREEN)✅ Benchmark artifacts cleaned$(NC)"
