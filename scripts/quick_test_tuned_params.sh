#!/bin/bash
# Quick test of conservative tuned parameters
# This implements Option 1 from TEXEL-ANALYSIS.md

set -e

echo "=== Quick Test: Conservative Tuned Parameters ==="
echo ""
echo "This will test the following divisor changes:"
echo "  PST scale: 4 → 6 (weaker PST)"
echo "  Pawn divisor: 4 → 3 (stronger pawn structure)"
echo "  Mobility divisor: 8 → 12 (weaker mobility)"
echo ""
echo "These are conservative values between defaults and tuned extremes."
echo ""

# Check if we're in the right directory
if [ ! -f "crates/engine/src/eval.rs" ]; then
    echo "Error: Must run from project root"
    exit 1
fi

# Backup original eval.rs
echo "1. Backing up eval.rs..."
cp crates/engine/src/eval.rs crates/engine/src/eval.rs.backup

# Apply changes using sed
echo "2. Applying conservative parameter changes..."

# This is a bit tricky - we need to change the defaults in get_param_or_default calls
# For simplicity, let's just create a temporary version with hardcoded values

cat > /tmp/eval_patch.txt << 'EOF'
        // 2. PST with tunable divisor (default: 4)
        let pst_divisor = 6;  // TESTING: Conservative tuned value
        let white_pst = self.pst.evaluate_position(board, Color::White) / pst_divisor;
        let black_pst = self.pst.evaluate_position(board, Color::Black) / pst_divisor;
        let pst = white_pst - black_pst;

        // 3. Calculate game phase for MG/EG blending
        let phase = phase::calculate_phase(board);

        // 4. Pawn structure with tunable divisor (default: 4)
        let pawn_divisor = 3;  // TESTING: Conservative tuned value
        let (white_pawn_mg, white_pawn_eg, black_pawn_mg, black_pawn_eg) =
            evaluate_pawns_cached(board, &mut self.pawn_hash);
        let white_pawn = (white_pawn_mg * (256 - phase) + white_pawn_eg * phase) / 256;
        let black_pawn = (black_pawn_mg * (256 - phase) + black_pawn_eg * phase) / 256;
        let pawn_structure = (white_pawn - black_pawn) / pawn_divisor;

        // 5. Mobility with tunable divisor (default: 8)
        let mobility_divisor = 12;  // TESTING: Conservative tuned value
EOF

# Actually, let's do it more carefully with a proper patch
echo "   Creating test version with hardcoded values..."
echo "   (This is safer than sed for complex changes)"
echo ""
echo "⚠️  Manual step required:"
echo ""
echo "   Please edit crates/engine/src/eval.rs, function evaluate_minimal()"
echo "   Change these three lines (around line 67-85):"
echo ""
echo "   FROM:"
echo '     let pst_divisor = tune::get_param_or_default(|p| p.pst_scale, 4);'
echo '     let pawn_divisor = tune::get_param_or_default(|p| p.pawn_structure_divisor, 4);'
echo '     let mobility_divisor = tune::get_param_or_default(|p| p.mobility_divisor, 8);'
echo ""
echo "   TO:"
echo '     let pst_divisor = 6;        // TEST: Conservative tuned'
echo '     let pawn_divisor = 3;       // TEST: Conservative tuned'
echo '     let mobility_divisor = 12;  // TEST: Conservative tuned'
echo ""
read -p "Press Enter when you've made the changes (or Ctrl-C to cancel)..."

# Rebuild
echo ""
echo "3. Rebuilding engine..."
cargo build --release --example uci_stdio --quiet

echo ""
echo "4. Running tactical test (must maintain 95%+)..."
cargo run --release --example tactical_test_runner \
    crates/engine/positions/wacnew.epd \
    --depth 8 --limit 50 2>&1 | tee /tmp/tactical_test_results.txt

# Extract score
TACTICAL_SCORE=$(grep -E "Score:|Solved:" /tmp/tactical_test_results.txt | tail -1)
echo ""
echo "Tactical result: $TACTICAL_SCORE"
echo ""

read -p "Does tactical score look good (≥95%)? (y/n): " tactical_ok

if [ "$tactical_ok" != "y" ]; then
    echo ""
    echo "Tactical test failed! Restoring backup..."
    mv crates/engine/src/eval.rs.backup crates/engine/src/eval.rs
    cargo build --release --example uci_stdio --quiet
    echo "Restored to original. Test aborted."
    exit 1
fi

# Quick game test
echo ""
echo "5. Running quick game test (10 games vs SF1800)..."
echo "   This will take ~5-10 minutes..."
echo ""

/tmp/fastchess/fastchess \
    -engine cmd=./target/release/examples/uci_stdio name=ChessAI \
    -engine cmd=stockfish name=SF1800 \
        option.UCI_LimitStrength=true option.UCI_Elo=1800 \
    -each tc=40/10+0.1 -rounds 5 -repeat -concurrency 2 \
    2>&1 | tee /tmp/quick_game_results.txt

echo ""
echo "=== Test Results ==="
echo ""
echo "Tactical: $TACTICAL_SCORE"
echo ""
echo "Games (see full output above):"
grep -E "Score of|Elo difference:" /tmp/quick_game_results.txt || echo "(Check full output)"
echo ""
echo ""
echo "6. Next steps:"
echo ""
read -p "Keep these parameters? (y/n): " keep_params

if [ "$keep_params" = "y" ]; then
    echo ""
    echo "✅ Keeping test parameters."
    echo "   Backup saved at: crates/engine/src/eval.rs.backup"
    echo ""
    echo "   To test more thoroughly, run:"
    echo "   source TEXEL-COMMANDS.sh && test_vs_1800  # 50 games"
    echo ""
else
    echo ""
    echo "Restoring original parameters..."
    mv crates/engine/src/eval.rs.backup crates/engine/src/eval.rs
    cargo build --release --example uci_stdio --quiet
    echo "✅ Restored to original."
fi

echo ""
echo "Test complete!"
