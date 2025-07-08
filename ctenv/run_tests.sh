#!/bin/bash

# Test runner script for ctenv

set -e

echo "=== Running ctenv Tests ==="

# Change to the ctenv directory
cd "$(dirname "$0")"

echo ""
echo "1. Running unit tests..."
cargo test --lib

echo ""
echo "2. Running integration tests..."
cargo test --test integration_tests

echo ""
echo "3. Running UI tests..."
cargo test --test ui_tests

echo ""
echo "4. Running dotenv integration tests..."
cargo test --test dotenv_integration

echo ""
echo "5. Running benchmark tests..."
cargo test --test benchmarks

echo ""
echo "6. Running examples..."
echo "6a. Comprehensive demo:"
cargo run --example comprehensive_demo

echo ""
echo "6b. Basic example from parent directory:"
(cd .. && RUST_LOG=info MY_CUSTOM_VAR=test_value cargo run --example ctenv_test)

echo ""
echo "7. Testing with different environment setups..."

# Test with specific environment variables set
echo "7a. Testing with compile-time environment variables:"
COMPILE_TIME_VAR="set_at_compile_time" cargo test test_compile_time_env_var

# Test with .env file
echo "7b. Testing with .env file:"
cp .env.test .env
cargo test test_runtime_fallback
rm -f .env

echo ""
echo "8. Building documentation..."
cargo doc --no-deps

echo ""
echo "=== All Tests Passed! ==="