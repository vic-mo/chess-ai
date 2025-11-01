#!/bin/bash
# Test tactical strength at multiple depths
# This script runs the tactical test runner at depths 4, 6, 8, 10, and 12
# and saves the results to JSON files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Create results directory
RESULTS_DIR="$PROJECT_ROOT/test_results"
mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
EPD_FILE="crates/engine/positions/wacnew.epd"

echo "========================================="
echo "Tactical Strength Testing - Multiple Depths"
echo "========================================="
echo "EPD File: $EPD_FILE"
echo "Timestamp: $TIMESTAMP"
echo "Results Directory: $RESULTS_DIR"
echo ""

# Build in release mode first
echo "Building in release mode..."
cargo build --release --example tactical_test_runner
echo ""

# Test at different depths
for DEPTH in 4 6 8 10 12; do
    echo "========================================="
    echo "Testing at depth $DEPTH"
    echo "========================================="

    OUTPUT_FILE="$RESULTS_DIR/wac_depth_${DEPTH}_${TIMESTAMP}.json"

    cargo run --release --example tactical_test_runner -- \
        "$EPD_FILE" \
        --depth "$DEPTH" \
        --json "$OUTPUT_FILE"

    echo ""
    echo "Results saved to: $OUTPUT_FILE"
    echo ""
done

echo "========================================="
echo "All tests complete!"
echo "========================================="
echo ""
echo "Results summary:"
for DEPTH in 4 6 8 10 12; do
    RESULT_FILE="$RESULTS_DIR/wac_depth_${DEPTH}_${TIMESTAMP}.json"
    if [ -f "$RESULT_FILE" ]; then
        ACCURACY=$(grep '"accuracy"' "$RESULT_FILE" | head -1 | sed 's/.*: \([0-9.]*\).*/\1/')
        PASSED=$(grep '"passed"' "$RESULT_FILE" | head -1 | sed 's/.*: \([0-9]*\).*/\1/')
        TOTAL=$(grep '"total"' "$RESULT_FILE" | head -1 | sed 's/.*: \([0-9]*\).*/\1/')
        echo "Depth $DEPTH: $PASSED/$TOTAL ($ACCURACY%)"
    fi
done

echo ""
echo "Results directory: $RESULTS_DIR"
