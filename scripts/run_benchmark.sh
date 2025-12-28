#!/bin/bash
# scripts/run_benchmarks.sh

echo "Running ConwayBets Performance Benchmarks..."
echo "=========================================="

# Set up environment
export RUST_LOG=info
export CARGO_PROFILE_BENCH_DEBUG=true

cd ../tests || { echo "Failed to change directory to tests"; exit 1; }
# Create output directory
mkdir -p benchmark_results/$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="benchmark_results/$(date +%Y%m%d_%H%M%S)"

echo "Output directory: $OUTPUT_DIR"

# Run benchmarks with different configurations
echo ""
echo "1. Running Market Creation Benchmarks..."
cargo bench --bench performance -- --profile-time=5 market_creation

echo ""
echo "2. Running Bet Placement Benchmarks..."
cargo bench --bench performance -- --profile-time=10 bet_placement

echo ""
echo "3. Running Concurrent Operations Benchmarks..."
cargo bench --bench performance -- --profile-time=15 concurrent_operations

echo ""
echo "4. Running Cross-chain Benchmarks..."
cargo bench --bench performance -- --profile-time=10 cross_chain

echo ""
echo "5. Running Microchain Scalability Benchmarks..."
cargo bench --bench performance -- --profile-time=20 microchain_scalability

echo ""
echo "6. Generating HTML Report..."
# Convert results to HTML (requires criterion's html_reports feature)
# This would generate interactive HTML reports

echo ""
echo "7. Analyzing Results..."
# Generate summary report
cargo run --bin benchmark_analyzer -- --input ./target/criterion --output $OUTPUT_DIR/report.md

echo ""
echo "Benchmarks completed!"
echo "Results saved to: $OUTPUT_DIR"
echo "View report: $OUTPUT_DIR/report.md"

# Optional: Generate comparison with previous run
if [ -d "benchmark_results/latest" ]; then
    echo ""
    echo "Comparing with previous run..."
    cargo run --bin benchmark_comparison -- \
        --current $OUTPUT_DIR \
        --previous benchmark_results/latest \
        --output $OUTPUT_DIR/comparison.md
fi

# Update latest symlink
ln -sfn $OUTPUT_DIR benchmark_results/latest