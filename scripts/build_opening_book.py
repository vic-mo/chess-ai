#!/usr/bin/env python3
"""
Build an opening book from PGN files.

Extracts positions from high-rated games and builds a frequency-based opening book.
Outputs positions in a format that can be integrated into opening_book.rs.
"""

import chess
import chess.pgn
import sys
from collections import defaultdict
from pathlib import Path


def extract_openings(pgn_path, max_games=None, min_rating=2200, max_moves=12):
    """
    Extract opening positions from PGN file.

    Args:
        pgn_path: Path to PGN file
        max_games: Maximum number of games to process (None = all)
        min_rating: Minimum player rating
        max_moves: Maximum number of moves to extract from each game

    Returns:
        Dictionary mapping (FEN, move) -> count
    """
    book = defaultdict(lambda: defaultdict(int))
    games_processed = 0
    games_used = 0

    print(f"Processing {pgn_path}...")
    print(f"Filters: rating >= {min_rating}, max_moves = {max_moves}")

    with open(pgn_path) as pgn_file:
        while True:
            game = chess.pgn.read_game(pgn_file)
            if game is None:
                break

            games_processed += 1
            if games_processed % 1000 == 0:
                print(f"Processed {games_processed} games, used {games_used}...", end='\r')

            if max_games and games_processed > max_games:
                break

            # Filter by rating
            try:
                white_elo = int(game.headers.get("WhiteElo", 0))
                black_elo = int(game.headers.get("BlackElo", 0))
                avg_elo = (white_elo + black_elo) / 2

                if avg_elo < min_rating:
                    continue
            except (ValueError, TypeError):
                continue

            games_used += 1

            # Extract opening moves
            board = game.board()
            move_count = 0

            for move in game.mainline_moves():
                if move_count >= max_moves:
                    break

                # Store position and move
                fen = board.fen()
                move_uci = move.uci()
                book[fen][move_uci] += 1

                board.push(move)
                move_count += 1

    print(f"\nProcessed {games_processed} games, used {games_used}")
    print(f"Unique positions: {len(book)}")

    return book


def filter_book(book, min_frequency=0.05, min_games=10):
    """
    Filter book to keep only popular moves.

    Args:
        book: Dictionary of (FEN -> move -> count)
        min_frequency: Minimum frequency (e.g., 0.05 = 5% of games)
        min_games: Minimum number of games a position must appear in

    Returns:
        Filtered book
    """
    filtered = {}

    for fen, moves in book.items():
        total_games = sum(moves.values())

        if total_games < min_games:
            continue

        # Keep moves that appear in at least min_frequency of games
        popular_moves = {
            move: count
            for move, count in moves.items()
            if count / total_games >= min_frequency
        }

        if popular_moves:
            filtered[fen] = popular_moves

    print(f"Filtered to {len(filtered)} positions")
    return filtered


def generate_rust_code(book, output_path):
    """
    Generate Rust code for opening_book.rs.

    Args:
        book: Dictionary of (FEN -> moves -> count)
        output_path: Where to write the Rust code
    """
    with open(output_path, 'w') as f:
        f.write("// Auto-generated opening book from PGN analysis\n")
        f.write("// Add these positions to populate_basic_openings() in opening_book.rs\n\n")

        for fen, moves in sorted(book.items(), key=lambda x: sum(x[1].values()), reverse=True):
            total = sum(moves.values())

            # Sort moves by frequency
            sorted_moves = sorted(moves.items(), key=lambda x: x[1], reverse=True)

            # Format as Rust code
            f.write(f"// Position seen in {total} games\n")
            f.write(f'self.add_position(\n')
            f.write(f'    "{fen}",\n')
            f.write(f'    vec![')

            move_strs = [f'"{move}"' for move, _ in sorted_moves[:5]]  # Top 5 moves
            f.write(', '.join(move_strs))
            f.write('],\n')
            f.write(f');\n\n')


def generate_statistics(book, output_path):
    """Generate statistics about the opening book."""
    with open(output_path, 'w') as f:
        f.write("# Opening Book Statistics\n\n")

        total_positions = len(book)
        total_moves = sum(len(moves) for moves in book.values())
        total_games = sum(sum(moves.values()) for moves in book.values())

        f.write(f"Total positions: {total_positions}\n")
        f.write(f"Total move options: {total_moves}\n")
        f.write(f"Total game references: {total_games}\n")
        f.write(f"Avg moves per position: {total_moves / total_positions:.1f}\n\n")

        # Top 20 most common positions
        f.write("## Top 20 Most Common Positions\n\n")
        sorted_positions = sorted(
            book.items(),
            key=lambda x: sum(x[1].values()),
            reverse=True
        )[:20]

        for i, (fen, moves) in enumerate(sorted_positions, 1):
            total = sum(moves.values())
            f.write(f"{i}. Games: {total}\n")
            f.write(f"   FEN: {fen}\n")
            f.write(f"   Moves: {', '.join(f'{m} ({c})' for m, c in sorted(moves.items(), key=lambda x: x[1], reverse=True)[:3])}\n\n")


def main():
    if len(sys.argv) < 2:
        print("Usage: python build_opening_book.py <pgn_file> [max_games]")
        print("\nExample:")
        print("  python build_opening_book.py lichess_elite.pgn 10000")
        sys.exit(1)

    pgn_path = sys.argv[1]
    max_games = int(sys.argv[2]) if len(sys.argv) > 2 else None

    # Extract openings
    book = extract_openings(pgn_path, max_games=max_games)

    # Filter to keep only popular moves
    filtered_book = filter_book(book, min_frequency=0.05, min_games=20)

    # Generate output files
    output_dir = Path("data/opening_book")
    output_dir.mkdir(parents=True, exist_ok=True)

    rust_output = output_dir / "generated_book.rs"
    stats_output = output_dir / "book_statistics.md"

    generate_rust_code(filtered_book, rust_output)
    generate_statistics(filtered_book, stats_output)

    print(f"\nGenerated files:")
    print(f"  Rust code: {rust_output}")
    print(f"  Statistics: {stats_output}")
    print(f"\nNext steps:")
    print(f"  1. Review {stats_output}")
    print(f"  2. Copy relevant positions from {rust_output} to opening_book.rs")
    print(f"  3. Rebuild engine and test")


if __name__ == "__main__":
    main()
