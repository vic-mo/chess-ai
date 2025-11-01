#!/usr/bin/env python3
"""
Analyze Elo calibration results and compute estimated Elo ratings for each depth.
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

def parse_log_file(log_path: Path) -> Tuple[str, str, float, int, int, int]:
    """Parse a calibration log file and extract results."""
    content = log_path.read_text()

    # Extract depth and Stockfish Elo from filename
    # Format: d{depth}_sf{elo}.log
    match = re.match(r'd(\d+)_sf(\d+)\.log', log_path.name)
    if not match:
        return None

    depth = match.group(1)
    sf_elo = match.group(2)

    # Extract game results
    wins_match = re.search(r'Wins: (\d+)', content)
    losses_match = re.search(r'Losses: (\d+)', content)
    draws_match = re.search(r'Draws: (\d+)', content)
    points_match = re.search(r'Points: ([\d.]+)', content)

    if not all([wins_match, losses_match, draws_match, points_match]):
        return None

    wins = int(wins_match.group(1))
    losses = int(losses_match.group(1))
    draws = int(draws_match.group(1))
    points = float(points_match.group(1))
    games = wins + losses + draws

    score_percentage = (points / games) * 100 if games > 0 else 0

    return depth, sf_elo, score_percentage, wins, losses, draws

def estimate_elo_from_score(opponent_elo: int, score_percentage: float) -> int:
    """
    Estimate Elo rating based on score percentage against opponent.
    Uses the standard Elo formula: Expected Score = 1 / (1 + 10^((opponent - player) / 400))
    Solving for player rating: player = opponent - 400 * log10(1/score - 1)
    """
    import math

    # Clamp score to avoid division by zero or log of negative
    score = max(0.01, min(0.99, score_percentage / 100))

    # Elo difference formula
    elo_diff = -400 * math.log10((1 / score) - 1)

    return int(opponent_elo + elo_diff)

def main():
    results_dir = Path("./elo_calibration_results")

    if not results_dir.exists():
        print("Error: Results directory not found")
        sys.exit(1)

    # Parse all log files
    results: Dict[str, List[Tuple[int, float, int, int, int]]] = {}

    for log_file in results_dir.glob("d*_sf*.log"):
        parsed = parse_log_file(log_file)
        if parsed:
            depth, sf_elo, score_pct, wins, losses, draws = parsed
            if depth not in results:
                results[depth] = []
            results[depth].append((int(sf_elo), score_pct, wins, losses, draws))

    # Analyze and estimate Elo for each depth
    print("=" * 80)
    print("ELO CALIBRATION ANALYSIS")
    print("=" * 80)
    print()

    depth_elos = {}

    for depth in sorted(results.keys(), key=int):
        print(f"DEPTH {depth}")
        print("-" * 80)

        elo_estimates = []
        total_games = 0
        total_points = 0

        for sf_elo, score_pct, wins, losses, draws in sorted(results[depth]):
            games = wins + losses + draws
            points = wins + draws * 0.5
            total_games += games
            total_points += points

            estimated_elo = estimate_elo_from_score(sf_elo, score_pct)
            elo_estimates.append(estimated_elo)

            result_str = f"W{wins}-L{losses}-D{draws}"
            print(f"  vs SF {sf_elo:4d}: {score_pct:5.1f}% {result_str:12s} → Est. Elo: {estimated_elo:4d}")

        # Compute average estimated Elo
        avg_elo = int(sum(elo_estimates) / len(elo_estimates)) if elo_estimates else 0
        overall_score_pct = (total_points / total_games * 100) if total_games > 0 else 0

        depth_elos[int(depth)] = avg_elo

        print(f"  Overall: {overall_score_pct:.1f}% ({total_games} games)")
        print(f"  → Estimated Elo: {avg_elo}")
        print()

    # Print summary mapping
    print("=" * 80)
    print("DEPTH TO ELO MAPPING")
    print("=" * 80)
    print()
    print(" Depth | Est. Elo | Skill Level")
    print("-------|----------|----------------------------------")

    skill_levels = {
        (0, 800): "Beginner",
        (800, 1000): "Novice",
        (1000, 1200): "Casual Player",
        (1200, 1400): "Intermediate",
        (1400, 1600): "Club Player",
        (1600, 1800): "Advanced",
        (1800, 2000): "Expert",
        (2000, 2200): "Master",
        (2200, float('inf')): "Strong Master",
    }

    for depth in sorted(depth_elos.keys()):
        elo = depth_elos[depth]
        skill = "Unknown"
        for (low, high), level in skill_levels.items():
            if low <= elo < high:
                skill = level
                break
        print(f"   {depth:2d}  |  {elo:4d}    | {skill}")

    print()
    print("=" * 80)

    # Generate code snippet
    print("\nSuggested Elo-to-Depth mapping function:")
    print()
    print("```typescript")
    print("export function eloToDepth(elo: number): number {")

    depths_sorted = sorted(depth_elos.items())
    for i, (depth, depth_elo) in enumerate(depths_sorted):
        if i == 0:
            print(f"  if (elo < {depth_elo}) return {depth};")
        elif i == len(depths_sorted) - 1:
            prev_depth, prev_elo = depths_sorted[i-1]
            mid_elo = (prev_elo + depth_elo) // 2
            print(f"  if (elo < {mid_elo}) return {prev_depth};")
            print(f"  return {depth};")
        else:
            prev_depth, prev_elo = depths_sorted[i-1]
            mid_elo = (prev_elo + depth_elo) // 2
            print(f"  if (elo < {mid_elo}) return {prev_depth};")

    print("}")
    print("```")
    print()

if __name__ == "__main__":
    main()
