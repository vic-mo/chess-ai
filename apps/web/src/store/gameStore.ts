import { create } from 'zustand';
import { useGameEngine, useEngine } from '../engine/engineClient';
import type { GameEngineClient } from '../engine/engineClient';
import { logger } from '../utils/logger';
import { eloToDepth, depthToElo } from '../utils/eloMapping';

const STARTPOS = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

export interface GameState {
  // Board state
  fen: string;
  moveHistory: string[];
  lastMove: string | null;

  // Game status
  isGameOver: boolean;
  gameResult: 'checkmate' | 'stalemate' | 'resignation' | null;
  winner: 'white' | 'black' | null;

  // Player settings
  playerColor: 'white' | 'black';
  eloRating: number; // Elo rating (800-1800)
  difficulty: number; // Search depth (computed from Elo)

  // Engine state
  isEngineThinking: boolean;

  // Actions
  newGame: (playerColor: 'white' | 'black') => void;
  makeMove: (uciMove: string) => Promise<boolean>;
  makeEngineMove: () => Promise<void>;
  resign: () => void;
  setEloRating: (elo: number) => void;
  setDifficulty: (depth: number) => void;
  resetGame: () => void;
}

export const useGameStore = create<GameState>((set, get) => {
  return {
    // Initial state
    fen: STARTPOS,
    moveHistory: [],
    lastMove: null,
    isGameOver: false,
    gameResult: null,
    winner: null,
    playerColor: 'white',
    eloRating: 1400, // Default: Intermediate
    difficulty: eloToDepth(1400), // Computed from Elo
    isEngineThinking: false,

    newGame: async (playerColor: 'white' | 'black') => {
      set({
        fen: STARTPOS,
        moveHistory: [],
        lastMove: null,
        isGameOver: false,
        gameResult: null,
        winner: null,
        playerColor,
        isEngineThinking: false,
      });

      // If player is black, engine makes first move
      if (playerColor === 'black') {
        setTimeout(() => {
          get().makeEngineMove();
        }, 300);
      }
    },

    makeMove: async (uciMove: string): Promise<boolean> => {
      const { fen, isGameOver, isEngineThinking } = get();

      logger.log('[GameStore] makeMove called:', { uciMove, fen, isGameOver, isEngineThinking });

      if (isGameOver || isEngineThinking) {
        logger.log('[GameStore] Move rejected: game over or engine thinking');
        return false;
      }

      try {
        // Get engine client dynamically
        const gameEngine = useGameEngine();

        // Validate move
        logger.log('[GameStore] Validating move...');
        const isValid = await gameEngine.validateMove(fen, uciMove);
        logger.log('[GameStore] Move valid:', isValid);
        if (!isValid) {
          return false;
        }

        // Apply move
        logger.log('[GameStore] Applying move...');
        const newFen = await gameEngine.makeMove(fen, uciMove);
        logger.log('[GameStore] New FEN:', newFen);

        // Update state
        set({
          fen: newFen,
          moveHistory: [...get().moveHistory, uciMove],
          lastMove: uciMove,
        });
        logger.log('[GameStore] State updated');

        // Check game over
        const gameStatus = await gameEngine.checkGameOver(newFen);
        if (gameStatus.isOver) {
          const turn = newFen.split(' ')[1];
          set({
            isGameOver: true,
            gameResult: gameStatus.status as 'checkmate' | 'stalemate',
            winner: gameStatus.status === 'checkmate' ? (turn === 'w' ? 'black' : 'white') : null,
          });
          return true;
        }

        // Trigger engine move after short delay
        setTimeout(() => {
          get().makeEngineMove();
        }, 300);

        return true;
      } catch (e) {
        logger.error('Failed to make move:', e);
        return false;
      }
    },

    makeEngineMove: async () => {
      const { fen, difficulty, isGameOver, isEngineThinking } = get();

      if (isGameOver || isEngineThinking) {
        return;
      }

      set({ isEngineThinking: true });

      try {
        // Get engine clients dynamically
        const gameEngine = useGameEngine();
        const engine = useEngine();

        // Request engine analysis
        const requestId = Math.random().toString(36).substring(7);
        let bestMove: string | null = null;
        let engineError: Error | null = null;

        const stop = engine.analyze(
          {
            id: requestId,
            fen,
            limit: { kind: 'depth', depth: difficulty },
          },
          (event) => {
            if (event.type === 'bestMove') {
              bestMove = event.payload.best;
            } else if (event.type === 'error') {
              engineError = new Error(event.payload.message || 'Engine error');
            }
          },
        );

        // Wait for best move (with timeout)
        let timeoutId: NodeJS.Timeout;
        let checkInterval: NodeJS.Timeout;

        try {
          await Promise.race([
            // Polling promise
            new Promise<string>((resolve, reject) => {
              checkInterval = setInterval(() => {
                if (bestMove) {
                  clearInterval(checkInterval);
                  resolve(bestMove);
                } else if (engineError) {
                  clearInterval(checkInterval);
                  reject(engineError);
                }
              }, 100);
            }),
            // Timeout promise
            new Promise<string>((_, reject) => {
              timeoutId = setTimeout(() => reject(new Error('Engine timeout')), 30000);
            }),
          ]);
        } finally {
          // Always clean up interval and timeout
          clearInterval(checkInterval!);
          clearTimeout(timeoutId!);
          stop();
        }

        if (!bestMove || bestMove === '0000') {
          throw new Error('No valid move from engine');
        }

        // Apply engine's move
        const newFen = await gameEngine.makeMove(fen, bestMove);

        set({
          fen: newFen,
          moveHistory: [...get().moveHistory, bestMove],
          lastMove: bestMove,
          isEngineThinking: false,
        });

        // Check game over
        const gameStatus = await gameEngine.checkGameOver(newFen);
        if (gameStatus.isOver) {
          const turn = newFen.split(' ')[1];
          set({
            isGameOver: true,
            gameResult: gameStatus.status as 'checkmate' | 'stalemate',
            winner: gameStatus.status === 'checkmate' ? (turn === 'w' ? 'black' : 'white') : null,
          });
        }
      } catch (e) {
        logger.error('Engine move failed:', e);
        set({ isEngineThinking: false });
      }
    },

    resign: () => {
      const { playerColor } = get();
      set({
        isGameOver: true,
        gameResult: 'resignation',
        winner: playerColor === 'white' ? 'black' : 'white',
      });
    },

    setEloRating: (elo: number) => {
      const clampedElo = Math.max(800, Math.min(1800, elo));
      const depth = eloToDepth(clampedElo);
      set({ eloRating: clampedElo, difficulty: depth });
    },

    setDifficulty: (depth: number) => {
      const clampedDepth = Math.max(1, Math.min(11, depth));
      const elo = depthToElo(clampedDepth);
      set({ difficulty: clampedDepth, eloRating: elo });
    },

    resetGame: () => {
      set({
        fen: STARTPOS,
        moveHistory: [],
        lastMove: null,
        isGameOver: false,
        gameResult: null,
        winner: null,
        isEngineThinking: false,
      });
    },
  };
});
