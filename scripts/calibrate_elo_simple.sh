#!/bin/bash

# Simple Elo Calibration Script
# Tests engine at various depths against Stockfish at known Elo levels

ENGINE_PATH="./target/release/examples/uci_stdio"
STOCKFISH_PATH="/opt/homebrew/bin/stockfish"
FASTCHESS_PATH="/tmp/fast-chess-src/fastchess"
RESULTS_DIR="./elo_calibration_results"

mkdir -p "$RESULTS_DIR"

# Configuration - start with quick test
GAMES_PER_MATCHUP=20
TIME_CONTROL="5+0.05"  # 5 seconds + 0.05 increment

# Test fewer depths initially to verify setup
DEPTHS=(1 3 5 7 9 11)
# Stockfish minimum Elo is 1320, maximum is 3190
STOCKFISH_ELOS=(1320 1400 1500 1600 1700 1800 1900 2000)

echo "=========================================="
echo "Chess AI Engine Elo Calibration (Simple)"
echo "=========================================="
echo ""

# Verify binaries
if [ ! -f "$ENGINE_PATH" ]; then
    echo "Error: Engine not found"
    exit 1
fi

SUMMARY_FILE="$RESULTS_DIR/summary.txt"
echo "Calibration Results - $(date)" > "$SUMMARY_FILE"
echo "==========================================" >> "$SUMMARY_FILE"

for depth in "${DEPTHS[@]}"; do
    echo ""
    echo "Testing Depth $depth"
    echo "-------------------"

    for elo in "${STOCKFISH_ELOS[@]}"; do
        OUTPUT_FILE="$RESULTS_DIR/d${depth}_sf${elo}.pgn"
        LOG_FILE="$RESULTS_DIR/d${depth}_sf${elo}.log"

        echo "  vs Stockfish $elo..."

        "$FASTCHESS_PATH" \
            -engine cmd="$ENGINE_PATH" name="ChessAI-D$depth" option.FixedDepth="$depth" \
            -engine cmd="$STOCKFISH_PATH" name="SF-$elo" option.UCI_LimitStrength=true option.UCI_Elo="$elo" \
            -each tc="$TIME_CONTROL" proto=uci \
            -rounds $(( GAMES_PER_MATCHUP / 2 )) \
            -games 2 \
            -repeat \
            -concurrency 1 \
            -pgnout file="$OUTPUT_FILE" \
            -openings file=./crates/engine/positions/opening_suite.epd format=epd order=random \
            > "$LOG_FILE" 2>&1

        # Extract score
        SCORE=$(grep -E "Score of" "$LOG_FILE" | tail -1)
        echo "    $SCORE"
        echo "Depth $depth vs SF$elo: $SCORE" >> "$SUMMARY_FILE"

        sleep 1
    done
done

echo ""
echo "Done! Results in: $RESULTS_DIR/summary.txt"
