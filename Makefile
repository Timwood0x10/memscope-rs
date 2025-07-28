# Makefile for trace_tools - Rust Memory Analysis Toolkit
# Author: trace_tools development team
# Description: Build, test, and deployment automation

# Variables
CARGO := cargo
PROJECT_NAME := trace_tools
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
	@echo "$(BLUE)trace_tools Makefile - Available targets:$(NC)"
	@echo ""
	@echo "$(GREEN)Building:$(NC)"
	@echo "  build          - Build the project in debug mode"
	@echo "  release        - Build the project in release mode"
	@echo "  check          - Check the project for errors"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-stress    - Run stress tests"
	@echo "  test-safety    - Run safety tests"
	@echo "  test-performance - Run performance tests"
	@echo "  test-edge      - Run edge case tests"
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
	@echo "  run-lifecycle  - Run lifecycle example"
	@echo "  run-main       - Run main application"
	@echo "  run-memory-stress - Run memory stress test example"
	@echo "  run-complex-lifecycle-showcase - Run complex lifecycle showcase example"
	@echo ""
	@echo "$(GREEN)HTML Reports (Enhanced):$(NC)"
	@echo "  html           - Generate HTML report"
	@echo "                   Usage: make html [DIR=path] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]"
	@echo "  html-only      - Generate HTML report only"
	@echo "                   Usage: make html-only [DIR=path] [OUTPUT=report.html] [BASE=snapshot] [VERBOSE=1] [DEBUG=1] [PERFORMANCE=1]"
	@echo "  html-clean     - Clean generated HTML files"
	@echo "  html-help      - Show detailed HTML command usage and examples"
	@echo ""
	@echo "$(GREEN)CI/CD:$(NC)"
	@echo "  ci             - Run full CI pipeline locally"
	@echo "  pre-commit     - Run pre-commit checks"
	@echo "  coverage       - Generate test coverage report"
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@echo "  update         - Update dependencies"
	@echo "  outdated       - Check for outdated dependencies"
	@echo "  tree           - Show dependency tree"
	@echo "  bloat          - Analyze binary size"

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
	$(CARGO) test --all

.PHONY: test-unit
test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	$(CARGO) test --lib

.PHONY: test-integration
test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	$(CARGO) test --test comprehensive_integration_test

.PHONY: test-stress
test-stress:
	@echo "$(BLUE)Running stress tests...$(NC)"
	$(CARGO) test --test stress_test

.PHONY: test-safety
test-safety:
	@echo "$(BLUE)Running safety tests...$(NC)"
	$(CARGO) test --test safety_test

.PHONY: test-performance
test-performance:
	@echo "$(BLUE)Running performance tests...$(NC)"
	$(CARGO) test --test performance_test --release

.PHONY: test-edge
test-edge:
	@echo "$(BLUE)Running edge case tests...$(NC)"
	$(CARGO) test --test edge_cases_test

.PHONY: test-comprehensive
test-comprehensive:
	@echo "$(BLUE)Running comprehensive integration tests...$(NC)"
	$(CARGO) test --test comprehensive_integration_test

.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running all tests with verbose output...$(NC)"
	$(CARGO) test --all --verbose

# Quality assurance targets
.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

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

# CI/CD targets
.PHONY: ci
ci: clean check fmt-check clippy test doc
	@echo "$(GREEN)✅ Full CI pipeline completed successfully!$(NC)"

.PHONY: pre-commit
pre-commit: fmt clippy test-unit
	@echo "$(GREEN)✅ Pre-commit checks completed!$(NC)"

.PHONY: coverage
coverage:
	@echo "$(BLUE)Generating test coverage report...$(NC)"
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		mkdir -p $(COVERAGE_DIR); \
		$(CARGO) tarpaulin --out Html --output-dir $(COVERAGE_DIR); \
		echo "$(GREEN)Coverage report generated in $(COVERAGE_DIR)/tarpaulin-report.html$(NC)"; \
	else \
		echo "$(YELLOW)cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin$(NC)"; \
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
	@echo "$(BLUE)trace_tools is ready for production use!$(NC)"