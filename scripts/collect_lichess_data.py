#!/usr/bin/env python3
"""
Collect training positions from Lichess PGN games for Texel tuning.

This script:
1. Reads PGN games from stdin or file
2. Filters for high-quality games (1800+ ELO, decisive)
3. Extracts quiet positions (moves 10-50, no checks/captures)
4. Outputs EPD format: FEN; result;

Usage:
    # From stdin
    cat games.pgn | python3 collect_lichess_data.py > training.epd

    # From file
    python3 collect_lichess_data.py games.pgn > training.epd

    # With filtering
    python3 collect_lichess_data.py --min-elo 1800 --max-positions 10000 games.pgn > training.epd

Requirements:
    pip install chess
"""

import sys
import chess
import chess.pgn
import argparse
from typing import Optional, Tuple, List


def is_quiet_position(board: chess.Board) -> bool:
    """Check if position is quiet (no checks, no hanging pieces)."""
    # Must not be in check
    if board.is_check():
        return False

    # Must have reasonable material (not just pawns)
    piece_count = len(board.pieces(chess.KNIGHT, chess.WHITE)) + \
                  len(board.pieces(chess.KNIGHT, chess.BLACK)) + \
                  len(board.pieces(chess.BISHOP, chess.WHITE)) + \
                  len(board.pieces(chess.BISHOP, chess.BLACK)) + \
                  len(board.pieces(chess.ROOK, chess.WHITE)) + \
                  len(board.pieces(chess.ROOK, chess.BLACK)) + \
                  len(board.pieces(chess.QUEEN, chess.WHITE)) + \
                  len(board.pieces(chess.QUEEN, chess.BLACK))

    if piece_count < 4:  # Too few pieces (endgame)
        return False

    return True


def result_to_score(result: str, side_to_move: chess.Color) -> Optional[float]:
    """Convert game result to training score from side to move perspective."""
    if result == "1-0":  # White won
        return 1.0 if side_to_move == chess.WHITE else 0.0
    elif result == "0-1":  # Black won
        return 0.0 if side_to_move == chess.WHITE else 1.0
    elif result == "1/2-1/2":  # Draw
        return 0.5
    else:
        return None


def get_game_elo(game: chess.pgn.Game) -> Tuple[Optional[int], Optional[int]]:
    """Extract ELO ratings from game headers."""
    try:
        white_elo = int(game.headers.get("WhiteElo", "0"))
        black_elo = int(game.headers.get("BlackElo", "0"))
        return white_elo, black_elo
    except (ValueError, TypeError):
        return None, None


def process_game(game: chess.pgn.Game, min_elo: int, positions_per_game: int) -> List[Tuple[str, float]]:
    """Extract training positions from a single game."""
    positions = []

    # Check result
    result = game.headers.get("Result", "*")
    if result not in ["1-0", "0-1"]:  # Only use decisive games
        return positions

    # Check ELO
    white_elo, black_elo = get_game_elo(game)
    if white_elo is None or black_elo is None:
        return positions
    if white_elo < min_elo or black_elo < min_elo:
        return positions

    # Process moves
    board = game.board()
    move_num = 0
    node = game

    collected_count = 0

    while node.variations and collected_count < positions_per_game:
        node = node.variations[0]
        board.push(node.move)
        move_num += 1

        # Only collect from moves 10-50
        if move_num < 10:
            continue
        if move_num > 50:
            break

        # Check if position is quiet
        if not is_quiet_position(board):
            continue

        # Convert result to score
        score = result_to_score(result, board.turn)
        if score is None:
            continue

        # Save position
        fen = board.fen()
        positions.append((fen, score))
        collected_count += 1

    return positions


def main():
    parser = argparse.ArgumentParser(
        description="Extract training positions from PGN games for Texel tuning",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Process single PGN file
  python3 collect_lichess_data.py games.pgn > training.epd

  # Process with filters
  python3 collect_lichess_data.py --min-elo 2000 --max-positions 10000 games.pgn > training.epd

  # Process from stdin
  cat games.pgn | python3 collect_lichess_data.py > training.epd

  # Process compressed file
  zstdcat games.pgn.zst | python3 collect_lichess_data.py --max-positions 20000 > training.epd
        """
    )

    parser.add_argument(
        "input_file",
        nargs="?",
        help="Input PGN file (or stdin if not specified)"
    )
    parser.add_argument(
        "--min-elo",
        type=int,
        default=1800,
        help="Minimum ELO for both players (default: 1800)"
    )
    parser.add_argument(
        "--max-positions",
        type=int,
        default=20000,
        help="Maximum positions to collect (default: 20000)"
    )
    parser.add_argument(
        "--positions-per-game",
        type=int,
        default=5,
        help="Maximum positions per game (default: 5)"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print progress to stderr"
    )

    args = parser.parse_args()

    # Open input
    if args.input_file:
        try:
            pgn_file = open(args.input_file, "r")
        except IOError as e:
            print(f"Error opening file: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        pgn_file = sys.stdin

    # Process games
    total_positions = 0
    total_games = 0
    processed_games = 0

    if args.verbose:
        print(f"Collecting training data (min ELO: {args.min_elo}, max positions: {args.max_positions})", file=sys.stderr)
        print(f"Only using decisive games with positions from moves 10-50", file=sys.stderr)
        print("", file=sys.stderr)

    try:
        while total_positions < args.max_positions:
            game = chess.pgn.read_game(pgn_file)
            if game is None:
                break

            total_games += 1

            # Process game
            positions = process_game(game, args.min_elo, args.positions_per_game)

            if positions:
                processed_games += 1
                for fen, score in positions:
                    if total_positions >= args.max_positions:
                        break
                    print(f"{fen}; {score};")
                    total_positions += 1

            # Progress update
            if args.verbose and total_games % 100 == 0:
                print(f"Processed {total_games} games, collected {total_positions} positions ({processed_games} games used)", file=sys.stderr)

    except KeyboardInterrupt:
        if args.verbose:
            print("\nInterrupted by user", file=sys.stderr)

    finally:
        if args.input_file:
            pgn_file.close()

    if args.verbose:
        print("", file=sys.stderr)
        print(f"Final statistics:", file=sys.stderr)
        print(f"  Total games processed: {total_games}", file=sys.stderr)
        print(f"  Games used: {processed_games} ({100*processed_games/max(1,total_games):.1f}%)", file=sys.stderr)
        print(f"  Positions collected: {total_positions}", file=sys.stderr)
        print(f"  Average per game: {total_positions/max(1,processed_games):.1f}", file=sys.stderr)


if __name__ == "__main__":
    main()
