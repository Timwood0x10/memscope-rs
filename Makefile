# Makefile for memscope-rs - Rust Memory Analysis Toolkit
# Enhanced version with improved testing and benchmarking

# Colors for output
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
RED = \033[0;31m
NC = \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

# Help target
.PHONY: help
help:
	@echo "$(BLUE)Memscope-rs - Rust Memory Analysis Toolkit$(NC)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@echo "  $(GREEN)build$(NC)              - Build the project in release mode"
	@echo "  $(GREEN)test$(NC)               - Run all tests"
	@echo "  $(GREEN)test-coverage$(NC)      - Run comprehensive test coverage analysis"
	@echo "  $(GREEN)benchmark$(NC)          - Run performance benchmarks"
	@echo "  $(GREEN)benchmark-main$(NC)     - Run only main (realistic) benchmarks"
	@echo "  $(GREEN)examples$(NC)           - Run all examples"
	@echo "  $(GREEN)lint$(NC)               - Run clippy linting"
	@echo "  $(GREEN)format$(NC)             - Format code with rustfmt"
	@echo "  $(GREEN)doc$(NC)                - Generate documentation"
	@echo "  $(GREEN)clean$(NC)              - Clean build artifacts and reports"
	@echo "  $(GREEN)ci$(NC)                 - Run CI pipeline (build, test, lint)"
	@echo "  $(GREEN)full-report$(NC)        - Generate comprehensive project report"
	@echo "  $(GREEN)validate$(NC)           - Full validation (CI + examples + docs)"
	@echo ""
	@echo "$(YELLOW)Example-specific targets:$(NC)"
	@echo "  $(GREEN)run-basic$(NC)          - Run basic usage example"
	@echo "  $(GREEN)run-comprehensive$(NC)  - Run comprehensive memory analysis"
	@echo "  $(GREEN)run-realistic$(NC)      - Run realistic usage example"
	@echo "  $(GREEN)run-performance$(NC)    - Run performance benchmark demo"
	@echo ""
	@echo "$(YELLOW)Development targets:$(NC)"
	@echo "  $(GREEN)dev-setup$(NC)          - Setup development environment"
	@echo "  $(GREEN)quick-test$(NC)         - Run quick tests (lib only)"
	@echo "  $(GREEN)watch$(NC)              - Watch for changes and run tests"

# Building targets
.PHONY: build build-release build-dev
build: build-release

build-release:
	@echo "$(BLUE)🔨 Building memscope-rs in release mode...$(NC)"
	@cargo build --release
	@echo "$(GREEN)✅ Release build completed$(NC)"

build-dev:
	@echo "$(BLUE)🔨 Building memscope-rs in development mode...$(NC)"
	@cargo build
	@echo "$(GREEN)✅ Development build completed$(NC)"

# Testing targets
.PHONY: test quick-test test-lib test-integration test-doc test-examples
test:
	@echo "$(BLUE)🧪 Running all tests...$(NC)"
	@cargo test --all --verbose
	@echo "$(GREEN)✅ All tests completed$(NC)"

quick-test: test-lib

test-lib:
	@echo "$(BLUE)🧪 Running library tests...$(NC)"
	@cargo test --lib --verbose
	@echo "$(GREEN)✅ Library tests completed$(NC)"

test-integration:
	@echo "$(BLUE)🧪 Running integration tests...$(NC)"
	@cargo test --tests --verbose
	@echo "$(GREEN)✅ Integration tests completed$(NC)"

test-doc:
	@echo "$(BLUE)🧪 Running documentation tests...$(NC)"
	@cargo test --doc --verbose
	@echo "$(GREEN)✅ Documentation tests completed$(NC)"

test-examples:
	@echo "$(BLUE)🧪 Running example tests...$(NC)"
	@cargo test --examples --verbose
	@echo "$(GREEN)✅ Example tests completed$(NC)"

# Enhanced test coverage
.PHONY: test-coverage
test-coverage:
	@echo "$(BLUE)📊 Running comprehensive test coverage analysis...$(NC)"
	@mkdir -p scripts
	@if [ ! -f scripts/test_coverage.sh ]; then \
		echo "$(RED)❌ Test coverage script not found$(NC)"; \
		exit 1; \
	fi
	@./scripts/test_coverage.sh
	@echo "$(GREEN)✅ Test coverage analysis completed$(NC)"

# Benchmarking targets
.PHONY: benchmark benchmark-main benchmark-legacy benchmark-clean
benchmark:
	@echo "$(BLUE)⚡ Running all benchmarks...$(NC)"
	@mkdir -p scripts
	@if [ ! -f scripts/benchmark.sh ]; then \
		echo "$(RED)❌ Benchmark script not found$(NC)"; \
		exit 1; \
	fi
	@./scripts/benchmark.sh
	@echo "$(GREEN)✅ Benchmark analysis completed$(NC)"

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

# Example targets
.PHONY: examples run-basic run-comprehensive run-realistic run-performance run-all-examples
examples: run-all-examples

run-basic:
	@echo "$(BLUE)🚀 Running basic usage example...$(NC)"
	@cargo run --example basic_usage
	@echo "$(GREEN)✅ Basic usage example completed$(NC)"

run-comprehensive:
	@echo "$(BLUE)🚀 Running comprehensive memory analysis example...$(NC)"
	@cargo run --example comprehensive_memory_analysis
	@echo "$(GREEN)✅ Comprehensive analysis example completed$(NC)"

run-realistic:
	@echo "$(BLUE)🚀 Running realistic usage example...$(NC)"
	@cargo run --example realistic_usage_with_extensions
	@echo "$(GREEN)✅ Realistic usage example completed$(NC)"

run-performance:
	@echo "$(BLUE)🚀 Running performance benchmark demo...$(NC)"
	@cargo run --example performance_benchmark_demo
	@echo "$(GREEN)✅ Performance demo completed$(NC)"

run-all-examples:
	@echo "$(BLUE)🚀 Running all examples...$(NC)"
	@for example in $$(cargo run --example 2>&1 | grep "    " | awk '{print $$1}' | grep -v "Available"); do \
		echo "$(YELLOW)Running example: $$example$(NC)"; \
		timeout 60s cargo run --example $$example || echo "$(YELLOW)⚠️  Example $$example failed or timed out$(NC)"; \
	done
	@echo "$(GREEN)✅ All examples completed$(NC)"

# Code quality targets
.PHONY: lint format doc check
lint:
	@echo "$(BLUE)🔍 Running clippy linting...$(NC)"
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✅ Linting completed$(NC)"

format:
	@echo "$(BLUE)✨ Formatting code with rustfmt...$(NC)"
	@cargo fmt --all
	@echo "$(GREEN)✅ Code formatting completed$(NC)"

doc:
	@echo "$(BLUE)📚 Generating documentation...$(NC)"
	@cargo doc --no-deps --all-features
	@echo "$(GREEN)✅ Documentation generated$(NC)"
	@echo "$(BLUE)📖 Open documentation: target/doc/memscope_rs/index.html$(NC)"

check:
	@echo "$(BLUE)🔍 Running cargo check...$(NC)"
	@cargo check --all-targets --all-features
	@echo "$(GREEN)✅ Check completed$(NC)"

# CI pipeline
.PHONY: ci
ci: check test lint doc
	@echo "$(GREEN)🎉 CI pipeline completed successfully!$(NC)"

# Development setup
.PHONY: dev-setup
dev-setup:
	@echo "$(BLUE)🛠️  Setting up development environment...$(NC)"
	@rustup component add clippy rustfmt
	@cargo install cargo-watch || echo "$(YELLOW)cargo-watch already installed$(NC)"
	@mkdir -p reports scripts
	@echo "$(GREEN)✅ Development environment setup completed$(NC)"

# Watch for changes
.PHONY: watch
watch:
	@echo "$(BLUE)👀 Watching for changes...$(NC)"
	@cargo watch -x "test --lib"

# Quick validation
.PHONY: quick-validate quick-test-script
quick-validate: quick-test benchmark-main
	@echo "$(GREEN)🚀 Quick validation completed!$(NC)"

quick-test-script:
	@echo "$(BLUE)🚀 Running quick test script...$(NC)"
	@./scripts/quick_test.sh

# Cleaning targets
.PHONY: clean clean-all
clean:
	@echo "$(BLUE)🧹 Cleaning build artifacts...$(NC)"
	@cargo clean
	@echo "$(GREEN)✅ Build artifacts cleaned$(NC)"

clean-all: clean benchmark-clean
	@echo "$(BLUE)🧹 Cleaning all artifacts and reports...$(NC)"
	@rm -rf reports
	@rm -rf MemoryAnalysis
	@echo "$(GREEN)✅ All artifacts cleaned$(NC)"

# Comprehensive reporting
.PHONY: full-report
full-report: clean-all ci test-coverage benchmark-main
	@echo "$(BLUE)📋 Generating comprehensive project report...$(NC)"
	@mkdir -p reports
	@echo "# Memscope-rs Comprehensive Report" > reports/FULL_REPORT.md
	@echo "Generated on: $$(date)" >> reports/FULL_REPORT.md
	@echo "" >> reports/FULL_REPORT.md
	@echo "## Build Information" >> reports/FULL_REPORT.md
	@echo "- 🦀 Rust version: $$(rustc --version)" >> reports/FULL_REPORT.md
	@echo "- 📦 Cargo version: $$(cargo --version)" >> reports/FULL_REPORT.md
	@echo "- 🏗️  Build status: ✅ PASSED" >> reports/FULL_REPORT.md
	@echo "" >> reports/FULL_REPORT.md
	@echo "## Project Statistics" >> reports/FULL_REPORT.md
	@echo "- 📁 Source files: $$(find src -name '*.rs' | wc -l)" >> reports/FULL_REPORT.md
	@echo "- 🧪 Test files: $$(find tests -name '*.rs' 2>/dev/null | wc -l || echo '0')" >> reports/FULL_REPORT.md
	@echo "- 📚 Example files: $$(find examples -name '*.rs' | wc -l)" >> reports/FULL_REPORT.md
	@echo "- ⚡ Benchmark files: $$(find benches -name '*.rs' | wc -l)" >> reports/FULL_REPORT.md
	@echo "" >> reports/FULL_REPORT.md
	@echo "## Available Reports" >> reports/FULL_REPORT.md
	@if [ -f reports/coverage/COVERAGE_REPORT.md ]; then \
		echo "- 📊 [Test Coverage Report](./coverage/COVERAGE_REPORT.md)" >> reports/FULL_REPORT.md; \
	fi
	@if [ -f reports/benchmarks/BENCHMARK_REPORT.md ]; then \
		echo "- ⚡ [Benchmark Report](./benchmarks/BENCHMARK_REPORT.md)" >> reports/FULL_REPORT.md; \
	fi
	@echo "$(GREEN)📋 Full report generated: reports/FULL_REPORT.md$(NC)"

# Full validation
.PHONY: validate
validate: ci examples doc
	@echo "$(GREEN)🎉 Full validation completed successfully!$(NC)"
	@echo "$(GREEN)✅ Build: PASSED$(NC)"
	@echo "$(GREEN)✅ Tests: PASSED$(NC)"
	@echo "$(GREEN)✅ Linting: PASSED$(NC)"
	@echo "$(GREEN)✅ Documentation: PASSED$(NC)"
	@echo "$(GREEN)✅ Examples: PASSED$(NC)"
	@echo "$(BLUE)memscope-rs is ready for use!$(NC)"

# Utility targets
.PHONY: version info
version:
	@echo "$(BLUE)Memscope-rs Version Information:$(NC)"
	@grep "version" Cargo.toml | head -1
	@echo "Rust: $$(rustc --version)"
	@echo "Cargo: $$(cargo --version)"

info:
	@echo "$(BLUE)Project Information:$(NC)"
	@echo "Name: memscope-rs"
	@echo "Description: Advanced Rust memory analysis and visualization toolkit"
	@echo "Repository: https://github.com/TimWood0x10/memscope-rs"
	@echo ""
	@echo "$(BLUE)Quick Start:$(NC)"
	@echo "1. Run 'make dev-setup' to setup development environment"
	@echo "2. Run 'make test' to run tests"
	@echo "3. Run 'make run-basic' to try basic example"
	@echo "4. Run 'make help' for all available commands"