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
	$(CARGO) build --release

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

.PHONY: run-comprehensive-demo
run-comprehensive-demo:
	@echo "$(BLUE)Running comprehensive analysis demo...$(NC)"
	$(CARGO) run --example comprehensive_analysis_demo

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

# Validate all is working
.PHONY: validate
validate: ci run-basic run-lifecycle
	@echo "$(GREEN)🎉 Full validation completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASS$(NC)"
	@echo "$(GREEN)✅ Tests: PASS$(NC)"
	@echo "$(GREEN)✅ Linting: PASS$(NC)"
	@echo "$(GREEN)✅ Documentation: PASS$(NC)"
	@echo "$(GREEN)✅ Examples: PASS$(NC)"
	@echo "$(BLUE)trace_tools is ready for production use!$(NC)"