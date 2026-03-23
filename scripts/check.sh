#!/bin/bash
# Pre-release verification script
# Run this before publishing to catch common issues

set -e

echo "=== Checking formatting ==="
cargo fmt --all -- --check

echo "=== Running clippy ==="
cargo clippy --all-targets --all-features -- -D warnings

echo "=== Running tests ==="
cargo test --all-features

echo "=== Building docs ==="
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

echo "=== Checking publish dry-run ==="
cargo publish --dry-run

echo ""
echo "All checks passed!"
