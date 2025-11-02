import { Chessboard } from 'react-chessboard';
import { useGameStore } from '../store/gameStore';
import { useGameEngine } from '../engine/engineClient';
import { useEffect, useState } from 'react';
import { logger } from '../utils/logger';
import { Chess } from 'chess.js';

// Type assertion for Chessboard to work around type definition issues
const ChessboardComponent = Chessboard as any;

export function Game() {
  const { fen, lastMove, playerColor, isEngineThinking, isGameOver, makeMove } = useGameStore();
  const gameEngine = useGameEngine();
  const [legalMoves, setLegalMoves] = useState<string[]>([]);
  const [selectedSquare, setSelectedSquare] = useState<string | null>(null);
  const [boardWidth, setBoardWidth] = useState(600);
  const [displayFen, setDisplayFen] = useState(fen);

  // Determine if it's the player's turn
  const isPlayerTurn = () => {
    const turn = fen.split(' ')[1]; // 'w' or 'b'
    const result =
      (playerColor === 'white' && turn === 'w') || (playerColor === 'black' && turn === 'b');
    logger.log('[Game] 🔍 isPlayerTurn check:', { playerColor, turn, result, fen });
    return result;
  };

  // Fetch legal moves when position changes
  useEffect(() => {
    const fetchLegalMoves = async () => {
      const playerTurn = isPlayerTurn();
      if (playerTurn && !isEngineThinking && !isGameOver) {
        try {
          const moves = await gameEngine.legalMoves(fen);
          setLegalMoves(moves);

          // Log castling moves if available
          const castlingMoves = moves.filter(
            (m) => m === 'e1g1' || m === 'e1c1' || m === 'e8g8' || m === 'e8c8',
          );
          if (castlingMoves.length > 0) {
            logger.log('[Game] 🏰 Castling moves available:', castlingMoves);
          }
        } catch (e) {
          logger.error('[Game] Failed to fetch legal moves:', e);
        }
      }
    };
    fetchLegalMoves();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fen, isEngineThinking, isGameOver]);

  // Detect if move is castling attempt
  const isCastlingMove = (from: string, to: string): boolean => {
    const fromFile = from.charCodeAt(0);
    const toFile = to.charCodeAt(0);
    const isKingStartPos = from === 'e1' || from === 'e8';
    const moveDistance = Math.abs(toFile - fromFile);
    return isKingStartPos && moveDistance >= 2;
  };

  // Convert king-to-rook castling moves to proper UCI notation
  const convertCastlingMove = (from: string, to: string): string => {
    // Check if this is a king-to-rook castling attempt
    const isKingStartPos = from === 'e1' || from === 'e8';
    const isRookSquare = to === 'a1' || to === 'h1' || to === 'a8' || to === 'h8';

    if (!isKingStartPos || !isRookSquare) {
      return from + to; // Not a king-to-rook move, return as-is
    }

    // Convert king-to-rook to proper UCI castling notation
    const conversions: { [key: string]: string } = {
      e1h1: 'e1g1', // White kingside
      e1a1: 'e1c1', // White queenside
      e8h8: 'e8g8', // Black kingside
      e8a8: 'e8c8', // Black queenside
    };

    const moveKey = from + to;
    const convertedMove = conversions[moveKey];

    if (convertedMove) {
      logger.log(`[Game] 🏰 Converted king-to-rook castling: ${moveKey} → ${convertedMove}`);
      return convertedMove;
    }

    return from + to; // Not a valid castling conversion, return as-is
  };

  // Handle piece drop
  const onDrop = async (sourceSquare: string, targetSquare: string): Promise<boolean> => {
    logger.log('[Game] 📱 onDrop called:', { sourceSquare, targetSquare });

    if (!isPlayerTurn() || isEngineThinking || isGameOver) {
      logger.log('[Game] ❌ Drop rejected:', {
        isPlayerTurn: isPlayerTurn(),
        isEngineThinking,
        isGameOver,
      });
      return false;
    }

    try {
      // Convert king-to-rook moves to proper UCI castling notation
      const uciMove = convertCastlingMove(sourceSquare, targetSquare);

      // Log castling detection
      if (isCastlingMove(sourceSquare, targetSquare)) {
        logger.log('[Game] 🏰 Castling move detected!', uciMove);
      }

      logger.log('[Game] 🎯 Attempting move:', uciMove);

      // Validate and compute new position locally using chess.js
      const chess = new Chess(fen);
      const move = chess.move({
        from: uciMove.substring(0, 2),
        to: uciMove.substring(2, 4),
        promotion: uciMove.length > 4 ? uciMove[4] : undefined,
      });

      if (!move) {
        logger.log('[Game] ⚠️ Invalid move');
        return false;
      }

      // Immediately update display position to prevent flicker
      const newFen = chess.fen();
      setDisplayFen(newFen);
      logger.log('[Game] 🎨 Optimistically updated displayFen:', newFen);

      // Now make the actual move in the background (updates store)
      makeMove(uciMove).then((success) => {
        logger.log('[Game] ✅ Move result:', success);
        if (!success) {
          // Revert on failure
          logger.log('[Game] ⚠️ Move failed - reverting display');
          setDisplayFen(fen);
        }
      });

      // Return true to accept the move visually
      return true;
    } catch (error) {
      logger.error('[Game] 💥 Exception in onDrop:', error);
      return false;
    }
  };

  // Handle square clicks for castling and move selection
  const onSquareClick = async (square: string) => {
    logger.log('[Game] Square clicked:', square, 'Selected:', selectedSquare);

    if (!isPlayerTurn() || isEngineThinking || isGameOver) {
      return;
    }

    // If a square is already selected, try to make a move
    if (selectedSquare) {
      // Convert king-to-rook moves to proper UCI castling notation
      const uciMove = convertCastlingMove(selectedSquare, square);
      logger.log('[Game] Attempting move via click:', uciMove);

      if (isCastlingMove(selectedSquare, square)) {
        logger.log('[Game] 🏰 Castling via click!');
      }

      // Validate and compute new position locally using chess.js
      const chess = new Chess(fen);
      const move = chess.move({
        from: uciMove.substring(0, 2),
        to: uciMove.substring(2, 4),
        promotion: uciMove.length > 4 ? uciMove[4] : undefined,
      });

      if (move) {
        // Immediately update display position to prevent flicker
        const newFen = chess.fen();
        setDisplayFen(newFen);
        logger.log('[Game] 🎨 Optimistically updated displayFen from click:', newFen);

        // Now make the actual move in the background
        makeMove(uciMove).then((success) => {
          logger.log('[Game] Click move result:', success);
          if (!success) {
            // Revert on failure
            logger.log('[Game] ⚠️ Click move failed - reverting display');
            setDisplayFen(fen);
          }
        });
      }

      setSelectedSquare(null); // Clear selection after move attempt
    } else {
      // Select this square
      setSelectedSquare(square);
      logger.log('[Game] Square selected:', square);
    }
  };

  // Highlight last move and selected square
  const customSquareStyles: { [square: string]: React.CSSProperties } = {};
  if (lastMove && lastMove.length >= 4) {
    const from = lastMove.substring(0, 2);
    const to = lastMove.substring(2, 4);
    customSquareStyles[from] = { backgroundColor: 'rgba(255, 255, 0, 0.4)' };
    customSquareStyles[to] = { backgroundColor: 'rgba(255, 255, 0, 0.4)' };
  }
  if (selectedSquare) {
    customSquareStyles[selectedSquare] = {
      backgroundColor: 'rgba(34, 197, 94, 0.7)',
      boxShadow: 'inset 0 0 0 3px rgb(34, 197, 94)',
    };
  }

  // Highlight possible moves from selected square
  if (selectedSquare && isPlayerTurn() && !isEngineThinking && !isGameOver) {
    const possibleMoves = legalMoves.filter((move) => move.startsWith(selectedSquare));
    possibleMoves.forEach((move) => {
      const targetSquare = move.substring(2, 4);
      if (!customSquareStyles[targetSquare]) {
        customSquareStyles[targetSquare] = {
          backgroundColor: 'rgba(34, 197, 94, 0.3)',
          cursor: 'pointer',
        };
      }
    });
  }

  // Sync display FEN with actual FEN from store
  useEffect(() => {
    logger.log('[Game] 🔄 Board position updated:', fen);
    setDisplayFen(fen);
  }, [fen]);

  // Calculate responsive board width
  useEffect(() => {
    const updateBoardWidth = () => {
      const padding = 32; // 2rem padding on each side
      const maxWidth = 600;
      const availableWidth = window.innerWidth - padding * 2;
      const newWidth = Math.min(maxWidth, availableWidth);
      setBoardWidth(newWidth);
    };

    updateBoardWidth();
    window.addEventListener('resize', updateBoardWidth);
    return () => window.removeEventListener('resize', updateBoardWidth);
  }, []);

  return (
    <div className="game">
      <ChessboardComponent
        position={displayFen}
        onPieceDrop={onDrop}
        onSquareClick={onSquareClick}
        boardOrientation={playerColor}
        arePiecesDraggable={isPlayerTurn() && !isEngineThinking && !isGameOver}
        customSquareStyles={customSquareStyles}
        boardWidth={boardWidth}
        animationDuration={200}
        isDraggablePiece={({ piece }) => {
          // Only allow dragging pieces that match the current turn
          const turn = displayFen.split(' ')[1];
          const isWhitePiece = piece[0] === 'w';
          return (turn === 'w' && isWhitePiece) || (turn === 'b' && !isWhitePiece);
        }}
      />
    </div>
  );
}
