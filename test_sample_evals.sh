#!/bin/bash

# Test evaluation on sample training positions
echo "Testing evaluation on 5 sample training positions..."
echo

FEN1="r1bqkbnr/ppp3pp/2p5/4N3/4Pp2/3P4/PPP2PPP/RNBQK2R b KQkq - 0 6"
FEN2="r1b1kbnr/ppp3pp/2p5/4N1q1/4Pp2/3P4/PPP2PPP/RNBQK2R w KQkq - 1 7"
FEN3="rnbqk2r/ppp2ppp/3b1n2/3p4/3P1P2/5N2/PPP3PP/RNBQKB1R w KQkq - 1 6"
FEN4="rnbq1rk1/ppp2ppp/3b1n2/3p4/3P1P2/3B1N2/PPP3PP/RNBQK2R w KQ - 3 7"
FEN5="r2qk1nr/ppp2ppp/2nb4/8/6b1/2N2N2/PPP1PPPP/R1BQKB1R w KQkq - 2 6"

for fen in "$FEN1" "$FEN2" "$FEN3" "$FEN4" "$FEN5"; do
    echo "FEN: $fen"
    echo "position fen $fen" | ./target/release/examples/uci_stdio 2>/dev/null | head -1 &
    sleep 0.2
    echo "go depth 1" | ./target/release/examples/uci_stdio 2>/dev/null | grep "score cp" | head -1
    echo
done
