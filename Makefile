# Zendvo Smart Contracts Makefile

.PHONY: all build test clean fmt lint fix

# Default target
all: build test

# Build all contracts in the workspace
build:
	@echo "Building all contracts..."
	cargo build --target wasm32-unknown-unknown --release

# Run tests for all contracts
test:
	@echo "Running all tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Lint code using clippy
lint:
	@echo "Linting code..."
	cargo clippy --all-targets --all-features -- -D warnings

# Automatically fix logic/lint issues where possible
fix:
	@echo "Fixing code..."
	cargo fix --allow-dirty --allow-staged
	cargo fmt --all

# Build specific contract (e.g., make build-time-lock)
build-%:
	@echo "Building contract: $*"
	cargo build --package $* --target wasm32-unknown-unknown --release

# Test specific contract (e.g., make test-time-lock)
test-%:
	@echo "Testing contract: $*"
	cargo test --package $*
