#!/bin/bash

# DS3231 Test Coverage Script
# Runs coverage for both blocking and async implementations

set -e

echo "🔍 Running test coverage for DS3231 driver..."

# Clean previous coverage data
cargo llvm-cov clean

# Run coverage for blocking implementation (default features)
echo "📊 Testing blocking implementation..."
cargo llvm-cov --no-report test --features temperature_f32

# Run coverage for async implementation
echo "📊 Testing async implementation..."
cargo llvm-cov --no-report test --features async,temperature_f32

# Generate coverage reports
echo "📋 Generating coverage reports..."

# Generate HTML report
cargo llvm-cov report --html --output-dir coverage/html

# Generate LCOV report (for CI/external tools)
cargo llvm-cov report --lcov --output-path coverage/lcov.info

# Generate summary to console
cargo llvm-cov report

echo "✅ Coverage analysis complete!"
echo "📁 HTML report: coverage/html/index.html"
echo "📁 LCOV report: coverage/lcov.info" 