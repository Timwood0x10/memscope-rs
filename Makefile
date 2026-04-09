# Makefile for memscope-rs
# Build, test, and development automation

CARGO := cargo
PROJECT_NAME := memscope-rs
VERSION := $(shell grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m

.PHONY: all
all: build test

.PHONY: help
help:
	@echo "$(BLUE)memscope-rs Makefile$(NC)"
	@echo ""
	@echo "$(GREEN)Building:$(NC)"
	@echo "  build          - Build debug mode"
	@echo "  release        - Build release mode"
	@echo "  check          - Check for errors"
	@echo "  clean          - Clean artifacts"
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests (library)"
	@echo "  test-integration - Run integration tests"
	@echo "  test-examples  - Run example programs"
	@echo "  test-verbose   - Run with verbose output"
	@echo ""
	@echo "$(GREEN)Benchmarking:$(NC)"
	@echo "  bench          - Run all benchmarks"
	@echo "  bench-tracker  - Run tracker benchmarks"
	@echo "  bench-concurrent - Run concurrent benchmarks"
	@echo "  bench-io       - Run IO benchmarks"
	@echo "  bench-stress   - Run stress tests"
	@echo ""
	@echo "$(GREEN)Quality:$(NC)"
	@echo "  fmt            - Format code"
	@echo "  clippy         - Run clippy"
	@echo "  ci             - Run CI pipeline"
	@echo ""
	@echo "$(GREEN)Examples:$(NC)"
	@echo "  run-basic      - Run basic usage example"
	@echo "  run-showcase   - Run global tracker showcase"
	@echo "  run-unsafe-ffi - Run unsafe/FFI demo"
	@echo "  run-dashboard  - Run dashboard export"
	@echo ""
	@echo "$(GREEN)Documentation:$(NC)"
	@echo "  doc            - Generate docs"
	@echo "  doc-open       - Generate and open docs"

# Building
.PHONY: build
build:
	@echo "$(BLUE)Building...$(NC)"
	$(CARGO) build

.PHONY: release
release:
	@echo "$(BLUE)Building release...$(NC)"
	$(CARGO) build --release

.PHONY: check
check:
	@echo "$(BLUE)Checking $(PROJECT_NAME) for errors...$(NC)"
	$(CARGO) check --workspace --all-targets --all-features
	@echo "$(BLUE)Checking code formatting...$(NC)"
	$(CARGO) fmt --all -- --check
	@echo "$(BLUE)Running clippy linter...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✅ All checks completed!$(NC)"

.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning...$(NC)"
	$(CARGO) clean
	rm -rf target/coverage MemoryAnalysis/
	@echo "$(GREEN)Done$(NC)"

# Testing
.PHONY: test
test:
	@echo "$(BLUE)Running all tests...$(NC)"
	$(CARGO) test --workspace -- --test-threads=1

.PHONY: test-unit
test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	$(CARGO) test --lib --workspace -- --test-threads=1

.PHONY: test-integration
test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	$(CARGO) test --test '*' --workspace -- --test-threads=1

.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	$(CARGO) test --tests -- --test-threads=1 --nocapture

# Benchmarking
.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) bench --bench comprehensive_benchmarks

.PHONY: bench-tracker
bench-tracker:
	@echo "$(BLUE)Running tracker benchmarks...$(NC)"
	$(CARGO) bench -- tracker_benches

.PHONY: bench-concurrent
bench-concurrent:
	@echo "$(BLUE)Running concurrent benchmarks...$(NC)"
	$(CARGO) bench -- concurrent_benches

.PHONY: bench-io
bench-io:
	@echo "$(BLUE)Running IO benchmarks...$(NC)"
	$(CARGO) bench -- io_benches

.PHONY: bench-stress
bench-stress:
	@echo "$(BLUE)Running stress tests...$(NC)"
	$(CARGO) bench -- stress_benches

# Quality
.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting...$(NC)"
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	@echo "$(BLUE)Checking format...$(NC)"
	$(CARGO) fmt --all -- --check

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy (strict mode for local dev)...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all -W clippy::perf -W clippy::style -W clippy::complexity -W clippy::suspicious -W clippy::correctness -A clippy::too_many_arguments -A clippy::type_complexity

.PHONY: clippy-ci
clippy-ci:
	@echo "$(BLUE)Running clippy (CI mode - only compiler warnings)...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: ci
ci: fmt-check clippy-ci check test-unit test-integration test-examples
	@echo "$(GREEN)CI pipeline completed$(NC)"

.PHONY: test-examples
test-examples:
	@echo "$(BLUE)Testing examples...$(NC)"
	@echo "$(BLUE)  Running basic_usage...$(NC)"
	$(CARGO) run --example basic_usage
	@echo "$(BLUE)  Running unsafe_ffi_demo...$(NC)"
	$(CARGO) run --example unsafe_ffi_demo

# Examples
.PHONY: run-basic
run-basic:
	@echo "$(BLUE)Running basic example...$(NC)"
	$(CARGO) run --example basic_usage

.PHONY: run-showcase
run-showcase:
	@echo "$(BLUE)Running showcase...$(NC)"
	$(CARGO) run --example global_tracker_showcase

.PHONY: run-unsafe-ffi
run-unsafe-ffi:
	@echo "$(BLUE)Running unsafe FFI demo...$(NC)"
	$(CARGO) run --example unsafe_ffi_demo

.PHONY: run-dashboard
run-dashboard:
	@echo "$(BLUE)Running dashboard export...$(NC)"
	$(CARGO) run --example dashboard_export

.PHONY: run-detector
run-detector:
	@echo "$(BLUE)Running detector workflow...$(NC)"
	$(CARGO) run --example detector_complete_workflow

.PHONY: run-type-inference
run-type-inference:
	@echo "$(BLUE)Running type inference demo...$(NC)"
	$(CARGO) run --example type_inference_demo

# Documentation
.PHONY: doc
doc:
	@echo "$(BLUE)Generating docs...$(NC)"
	$(CARGO) doc --no-deps

.PHONY: doc-open
doc-open:
	@echo "$(BLUE)Generating and opening docs...$(NC)"
	$(CARGO) doc --no-deps --open

# Development
.PHONY: dev
dev: fmt clippy test-unit
	@echo "$(GREEN)Dev check completed$(NC)"

.PHONY: pre-commit
pre-commit: fmt clippy test
	@echo "$(GREEN)Pre-commit checks passed$(NC)"

# Demo
.PHONY: demo
demo: build run-basic run-showcase
	@echo "$(GREEN)Demo completed$(NC)"

# Info
.PHONY: info
info:
	@echo "$(BLUE)Project: $(PROJECT_NAME) v$(VERSION)$(NC)"
	@echo "Rust: $(shell rustc --version)"
	@echo "Cargo: $(shell cargo --version)"
