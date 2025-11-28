#!/bin/bash

echo "Running pre-release checks..."

# Check code format
echo "1. Checking code format..."
cargo fmt -- --check

# Check code quality
echo "2. Running clippy..."
cargo clippy -- -D warnings

# Run tests
echo "3. Running tests..."
cargo test

# Generate documentation
echo "4. Generating documentation..."
cargo doc --no-deps

# Dry run publish
echo "5. Dry run publish..."
cargo publish --dry-run

echo "All checks completed!"