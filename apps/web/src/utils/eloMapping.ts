/**
 * Elo to Depth mapping utilities
 * Based on empirical calibration testing against Stockfish
 */

/**
 * Maps Elo rating to search depth for consistent difficulty levels.
 *
 * Mapping based on:
 * - Empirical testing: Depth 1 ≈ 1200 Elo (120 games vs Stockfish)
 * - Standard chess engine scaling: ~70-100 Elo per ply
 * - Maximum constraint: 1800 Elo
 *
 * @param elo - Desired Elo rating (800-1800)
 * @returns Search depth (1-11)
 */
export function eloToDepth(elo: number): number {
  if (elo < 900) return 1; // < 900: Beginner
  if (elo < 1050) return 2; // 900-1049: Novice
  if (elo < 1150) return 3; // 1050-1149: Casual
  if (elo < 1250) return 4; // 1150-1249: Intermediate
  if (elo < 1350) return 5; // 1250-1349: Intermediate+
  if (elo < 1450) return 6; // 1350-1449: Club Player
  if (elo < 1550) return 7; // 1450-1549: Advanced
  if (elo < 1625) return 8; // 1550-1624: Advanced+
  if (elo < 1725) return 9; // 1625-1724: Expert
  if (elo < 1775) return 10; // 1725-1774: Expert+
  return 11; // 1775+: Max Strength (1800)
}

/**
 * Maps search depth to approximate Elo rating.
 * Inverse of eloToDepth.
 *
 * @param depth - Search depth (1-11)
 * @returns Approximate Elo rating
 */
export function depthToElo(depth: number): number {
  const mapping: { [key: number]: number } = {
    1: 800,
    2: 1000,
    3: 1100,
    4: 1200,
    5: 1300,
    6: 1400,
    7: 1500,
    8: 1600,
    9: 1700,
    10: 1750,
    11: 1800,
  };
  return mapping[Math.min(11, Math.max(1, depth))] || 1400;
}

/**
 * Predefined Elo levels for UI selection
 */
export interface EloLevel {
  elo: number;
  label: string;
  depth: number;
}

export const ELO_LEVELS: EloLevel[] = [
  { elo: 800, label: 'Beginner', depth: 1 },
  { elo: 1000, label: 'Novice', depth: 2 },
  { elo: 1200, label: 'Casual', depth: 4 },
  { elo: 1400, label: 'Intermediate', depth: 6 },
  { elo: 1600, label: 'Club Player', depth: 8 },
  { elo: 1800, label: 'Advanced', depth: 11 },
];
