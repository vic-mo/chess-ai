#!/bin/bash

# Elo Calibration Script for Chess AI Engine
# Tests engine at various depths against Stockfish at known Elo levels

ENGINE_PATH="./target/release/examples/uci_stdio"
STOCKFISH_PATH="/opt/homebrew/bin/stockfish"
FASTCHESS_PATH="/tmp/fast-chess-src/fastchess"
RESULTS_DIR="./elo_calibration_results"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Configuration
GAMES_PER_MATCHUP=50
# Use nodes instead of time for consistent strength measurement
NODES_PER_MOVE=100000  # Fixed nodes per move

# Depths to test (1-12, since max Elo is 1800)
DEPTHS=(1 2 3 4 5 6 7 8 9 10 11 12)

# Stockfish Elo levels to test against
STOCKFISH_ELOS=(800 1000 1200 1400 1600 1800 2000)

echo "=========================================="
echo "Chess AI Engine Elo Calibration"
echo "=========================================="
echo "Engine: $ENGINE_PATH"
echo "Opponent: Stockfish (limited strength)"
echo "Games per matchup: $GAMES_PER_MATCHUP"
echo "Nodes per move (Stockfish): $NODES_PER_MOVE"
echo "Results directory: $RESULTS_DIR"
echo "=========================================="
echo ""

# Verify binaries exist
if [ ! -f "$ENGINE_PATH" ]; then
    echo "Error: Engine not found at $ENGINE_PATH"
    echo "Please run: cargo build --release --example uci_stdio"
    exit 1
fi

if [ ! -f "$STOCKFISH_PATH" ]; then
    echo "Error: Stockfish not found at $STOCKFISH_PATH"
    exit 1
fi

if [ ! -f "$FASTCHESS_PATH" ]; then
    echo "Error: fast-chess not found at $FASTCHESS_PATH"
    exit 1
fi

# Summary file
SUMMARY_FILE="$RESULTS_DIR/calibration_summary.txt"
echo "Elo Calibration Results - $(date)" > "$SUMMARY_FILE"
echo "==========================================" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# Run calibration matches
for depth in "${DEPTHS[@]}"; do
    echo ""
    echo "=========================================="
    echo "Testing Engine at Depth $depth"
    echo "=========================================="
    echo "" >> "$SUMMARY_FILE"
    echo "--- Depth $depth ---" >> "$SUMMARY_FILE"

    for elo in "${STOCKFISH_ELOS[@]}"; do
        OUTPUT_FILE="$RESULTS_DIR/depth${depth}_vs_sf${elo}.pgn"

        echo "Running: Depth $depth vs Stockfish $elo Elo ($GAMES_PER_MATCHUP games)..."

        # Run fast-chess match
        # Using depth-based search for our engine, nodes-based for Stockfish
        "$FASTCHESS_PATH" \
            -engine cmd="$ENGINE_PATH" name="ChessAI-D$depth" \
            -engine cmd="$STOCKFISH_PATH" name="Stockfish-$elo" \
                    option.UCI_LimitStrength=true \
                    option.UCI_Elo="$elo" \
            -each tc=inf proto=uci \
            -engine1 option.Depth="$depth" st=0 nodes=0 \
            -engine2 nodes="$NODES_PER_MOVE" st=0 \
            -games "$GAMES_PER_MATCHUP" \
            -repeat \
            -rounds 1 \
            -concurrency 1 \
            -pgnout "$OUTPUT_FILE" \
            -openings file=./crates/engine/positions/opening_suite.epd format=epd order=random \
            > "$RESULTS_DIR/depth${depth}_vs_sf${elo}.log" 2>&1

        # Extract results from log
        RESULT=$(tail -20 "$RESULTS_DIR/depth${depth}_vs_sf${elo}.log" | grep -E "Score of|Elo difference" | head -2)

        echo "  Results: $RESULT"
        echo "Depth $depth vs SF $elo: $RESULT" >> "$SUMMARY_FILE"

        # Brief pause between matches
        sleep 1
    done
done

echo ""
echo "=========================================="
echo "Calibration Complete!"
echo "=========================================="
echo "Results saved to: $RESULTS_DIR"
echo "Summary: $SUMMARY_FILE"
echo ""
echo "Next steps:"
echo "1. Review $SUMMARY_FILE"
echo "2. Run analysis script to compute Elo ratings"
