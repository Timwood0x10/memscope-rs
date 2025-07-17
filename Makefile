# Makefile for memscope-rs - Rust Memory Analysis Toolkit
# Author: memscope-rs development team
# Description: Build, test, and deployment automation

# Variables
CARGO := cargo
PROJECT_NAME := memscope-rs
VERSION := $(shell grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
TARGET_DIR := target
DOCS_DIR := target/doc

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
	@echo "  $(GREEN)Build targets:$(NC)"
	@echo "    build         - Build the project"
	@echo "    release       - Build optimized release version"
	@echo "    clean         - Clean build artifacts"
	@echo ""
	@echo "  $(GREEN)Test targets:$(NC)"
	@echo "    test          - Run all tests (main entry point)"
	@echo "    test-core     - Run core existing tests"
	@echo "    test-new      - Run new comprehensive coverage tests"
	@echo "    test-export   - Test export functionality"
	@echo "    test-unsafe   - Test unsafe/FFI functionality"
	@echo "    test-integration - Test integration scenarios"
	@echo "    test-coverage - Test code coverage validation"
	@echo "    test-performance - Run performance benchmarks"
	@echo "    test-safety   - Run safety tests"
	@echo "    test-stress   - Run stress tests"
	@echo "    test-quick    - Quick development tests"
	@echo "    test-verbose  - All tests with verbose output"
	@echo ""
	@echo "  $(GREEN)Quality targets:$(NC)"
	@echo "    check         - Run cargo check"
	@echo "    fmt           - Format code"
	@echo "    clippy        - Run clippy lints"
	@echo "    doc           - Generate documentation"
	@echo ""
	@echo "  $(GREEN)Examples:$(NC)"
	@echo "    examples      - Run all examples"
	@echo "    example-basic - Run basic usage example"
	@echo "    example-complex - Run complex showcase example"

# Build targets
.PHONY: build
build:
	@echo "$(BLUE)Building $(PROJECT_NAME)...$(NC)"
	$(CARGO) build

.PHONY: release
release:
	@echo "$(BLUE)Building $(PROJECT_NAME) in release mode...$(NC)"
	$(CARGO) build --release

.PHONY: clean
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	$(CARGO) clean

# Quality assurance targets
.PHONY: check
check:
	@echo "$(BLUE)Running cargo check...$(NC)"
	$(CARGO) check --all

.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: doc
doc:
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --no-deps --open

# Main test entry point
.PHONY: test
test: test-core test-new
	@echo "$(GREEN)All tests completed successfully!$(NC)"

# Core existing tests
.PHONY: test-core
test-core:
	@echo "$(BLUE)Running core test suite...$(NC)"
	$(CARGO) test --lib
	$(CARGO) test --test advanced_memory_patterns_test
	$(CARGO) test --test advanced_memory_scenarios_test
	$(CARGO) test --test async_memory_test
	$(CARGO) test --test concurrent_memory_test
	$(CARGO) test --test comprehensive_integration_test
	$(CARGO) test --test edge_cases_test
	$(CARGO) test --test heavy_load_test
	$(CARGO) test --test lifecycle_validation_test
	$(CARGO) test --test performance_test
	$(CARGO) test --test safety_test
	$(CARGO) test --test stress_test
	@echo "$(GREEN)Core tests completed$(NC)"

# New comprehensive test coverage
.PHONY: test-new
test-new:
	@echo "$(BLUE)Running new comprehensive test coverage...$(NC)"
	$(CARGO) test --test export_functionality_test
	$(CARGO) test --test unsafe_ffi_comprehensive_test
	$(CARGO) test --test integration_boundary_test
	$(CARGO) test --test coverage_validation_test
	@echo "$(GREEN)New coverage tests completed$(NC)"

# Individual test suites for targeted testing
.PHONY: test-export
test-export:
	@echo "$(BLUE)Testing export functionality...$(NC)"
	$(CARGO) test --test export_functionality_test

.PHONY: test-unsafe
test-unsafe:
	@echo "$(BLUE)Testing unsafe/FFI functionality...$(NC)"
	$(CARGO) test --test unsafe_ffi_comprehensive_test

.PHONY: test-integration
test-integration:
	@echo "$(BLUE)Testing integration scenarios...$(NC)"
	$(CARGO) test --test integration_boundary_test

.PHONY: test-coverage
test-coverage:
	@echo "$(BLUE)Testing code coverage validation...$(NC)"
	$(CARGO) test --test coverage_validation_test

.PHONY: test-performance
test-performance:
	@echo "$(BLUE)Running performance benchmarks...$(NC)"
	$(CARGO) test --test performance_test --release

.PHONY: test-safety
test-safety:
	@echo "$(BLUE)Running safety tests...$(NC)"
	$(CARGO) test --test safety_test

.PHONY: test-stress
test-stress:
	@echo "$(BLUE)Running stress tests...$(NC)"
	$(CARGO) test --test stress_test

# Quick test for development
.PHONY: test-quick
test-quick:
	@echo "$(BLUE)Running quick development tests...$(NC)"
	$(CARGO) test --test safety_test
	$(CARGO) test --test export_functionality_test
	$(CARGO) test --lib

# Full test with verbose output
.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running all tests with verbose output...$(NC)"
	$(CARGO) test --all -- --nocapture

# Example targets
.PHONY: examples
examples: example-basic example-complex
	@echo "$(GREEN)All examples completed!$(NC)"

.PHONY: example-basic
example-basic:
	@echo "$(BLUE)Running basic usage example...$(NC)"
	$(CARGO) run --example basic_usage

.PHONY: example-complex
example-complex:
	@echo "$(BLUE)Running complex unsafe/FFI showcase...$(NC)"
	$(CARGO) run --example complex_unsafe_ffi_showcase

.PHONY: example-lifecycle
example-lifecycle:
	@echo "$(BLUE)Running lifecycle example...$(NC)"
	$(CARGO) run --example lifecycles_simple

.PHONY: example-stress
example-stress:
	@echo "$(BLUE)Running memory stress test example...$(NC)"
	$(CARGO) run --example memory_stress_test

# Development workflow
.PHONY: dev
dev: fmt clippy test-quick
	@echo "$(GREEN)Development workflow completed!$(NC)"

# CI workflow
.PHONY: ci
ci: check fmt clippy test
	@echo "$(GREEN)CI workflow completed!$(NC)"

# Install target
.PHONY: install
install:
	@echo "$(BLUE)Installing $(PROJECT_NAME)...$(NC)"
	$(CARGO) install --path .

# Benchmark target
.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) test --release -- --ignored

# Coverage target (requires cargo-tarpaulin)
.PHONY: coverage
coverage:
	@echo "$(BLUE)Generating test coverage report...$(NC)"
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		$(CARGO) tarpaulin --out Html --output-dir target/coverage; \
		@echo "$(GREEN)Coverage report generated in target/coverage/$(NC)"; \
	else \
		echo "$(YELLOW)cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin$(NC)"; \
	fi

# Show project info
.PHONY: info
info:
	@echo "$(BLUE)Project Information:$(NC)"
	@echo "  Name: $(PROJECT_NAME)"
	@echo "  Version: $(VERSION)"
	@echo "  Target Directory: $(TARGET_DIR)"
	@echo "  Documentation: $(DOCS_DIR)"
	@echo ""
	@echo "$(BLUE)Test Statistics:$(NC)"
	@echo "  Total test files: $(shell find tests -name '*.rs' | wc -l)"
	@echo "  Total test functions: $(shell find tests -name '*.rs' -exec grep -h 'fn test_' {} \; | wc -l)"
	@echo ""
	@echo "$(BLUE)Available examples:$(NC)"
	@ls examples/*.rs 2>/dev/null | sed 's/examples\///g' | sed 's/\.rs//g' | sed 's/^/  /' || echo "  No examples found"