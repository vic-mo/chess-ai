import { create } from 'zustand';
import { useGameEngine, useEngine } from '../engine/engineClient';
import type { GameEngineClient } from '../engine/engineClient';

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
  difficulty: number; // Search depth (1-20)

  // Engine state
  isEngineThinking: boolean;

  // Actions
  newGame: (playerColor: 'white' | 'black') => void;
  makeMove: (uciMove: string) => Promise<boolean>;
  makeEngineMove: () => Promise<void>;
  resign: () => void;
  setDifficulty: (depth: number) => void;
  resetGame: () => void;
}

export const useGameStore = create<GameState>((set, get) => {
  const gameEngine = useGameEngine();
  const engine = useEngine();

  return {
    // Initial state
    fen: STARTPOS,
    moveHistory: [],
    lastMove: null,
    isGameOver: false,
    gameResult: null,
    winner: null,
    playerColor: 'white',
    difficulty: 10,
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

      console.log('[GameStore] makeMove called:', { uciMove, fen, isGameOver, isEngineThinking });

      if (isGameOver || isEngineThinking) {
        console.log('[GameStore] Move rejected: game over or engine thinking');
        return false;
      }

      try {
        // Validate move
        console.log('[GameStore] Validating move...');
        const isValid = await gameEngine.validateMove(fen, uciMove);
        console.log('[GameStore] Move valid:', isValid);
        if (!isValid) {
          return false;
        }

        // Apply move
        console.log('[GameStore] Applying move...');
        const newFen = await gameEngine.makeMove(fen, uciMove);
        console.log('[GameStore] New FEN:', newFen);

        // Update state
        set({
          fen: newFen,
          moveHistory: [...get().moveHistory, uciMove],
          lastMove: uciMove,
        });
        console.log('[GameStore] State updated');

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
        console.error('Failed to make move:', e);
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
        // Request engine analysis
        const requestId = Math.random().toString(36).substring(7);
        let bestMove: string | null = null;

        const stop = engine.analyze(
          {
            id: requestId,
            fen,
            limit: { kind: 'depth', depth: difficulty },
          },
          (event) => {
            if (event.type === 'bestMove') {
              bestMove = event.payload.best;
            }
          },
        );

        // Wait for best move (with timeout)
        const timeout = new Promise((_, reject) =>
          setTimeout(() => reject(new Error('Engine timeout')), 30000),
        );

        await Promise.race([
          new Promise((resolve) => {
            const checkInterval = setInterval(() => {
              if (bestMove) {
                clearInterval(checkInterval);
                resolve(bestMove);
              }
            }, 100);
          }),
          timeout,
        ]);

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
        console.error('Engine move failed:', e);
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

    setDifficulty: (depth: number) => {
      set({ difficulty: Math.max(1, Math.min(20, depth)) });
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
