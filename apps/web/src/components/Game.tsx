import { Chessboard } from 'react-chessboard';
import { useGameStore } from '../store/gameStore';
import { useGameEngine } from '../engine/engineClient';
import { useEffect, useState } from 'react';

// Type assertion for Chessboard to work around type definition issues
const ChessboardComponent = Chessboard as any;

export function Game() {
  const { fen, lastMove, playerColor, isEngineThinking, isGameOver, makeMove } = useGameStore();
  const gameEngine = useGameEngine();
  const [legalMoves, setLegalMoves] = useState<string[]>([]);
  const [selectedSquare, setSelectedSquare] = useState<string | null>(null);

  // Determine if it's the player's turn
  const isPlayerTurn = () => {
    const turn = fen.split(' ')[1]; // 'w' or 'b'
    return (playerColor === 'white' && turn === 'w') || (playerColor === 'black' && turn === 'b');
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
            console.log('[Game] 🏰 Castling moves available:', castlingMoves);
          }
        } catch (e) {
          console.error('[Game] Failed to fetch legal moves:', e);
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

  // Handle piece drop
  const onDrop = async (sourceSquare: string, targetSquare: string): Promise<boolean> => {
    console.log('[Game] onDrop called:', { sourceSquare, targetSquare });

    if (!isPlayerTurn() || isEngineThinking || isGameOver) {
      console.log('[Game] Drop rejected:', {
        isPlayerTurn: isPlayerTurn(),
        isEngineThinking,
        isGameOver,
      });
      return false;
    }

    const uciMove = sourceSquare + targetSquare;

    // Log castling detection
    if (isCastlingMove(sourceSquare, targetSquare)) {
      console.log('[Game] 🏰 Castling move detected!', uciMove);
    }

    console.log('[Game] Attempting move:', uciMove);
    const success = await makeMove(uciMove);
    console.log('[Game] Move result:', success);
    return success;
  };

  // Handle square clicks for castling and move selection
  const onSquareClick = async (square: string) => {
    console.log('[Game] Square clicked:', square, 'Selected:', selectedSquare);

    if (!isPlayerTurn() || isEngineThinking || isGameOver) {
      return;
    }

    // If a square is already selected, try to make a move
    if (selectedSquare) {
      const uciMove = selectedSquare + square;
      console.log('[Game] Attempting move via click:', uciMove);

      if (isCastlingMove(selectedSquare, square)) {
        console.log('[Game] 🏰 Castling via click!');
      }

      const success = await makeMove(uciMove);
      console.log('[Game] Click move result:', success);
      setSelectedSquare(null); // Clear selection after move attempt
    } else {
      // Select this square
      setSelectedSquare(square);
      console.log('[Game] Square selected:', square);
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
    customSquareStyles[selectedSquare] = { backgroundColor: 'rgba(0, 255, 0, 0.6)' };
  }

  // Debug: log FEN changes
  useEffect(() => {
    console.log('[Game] 🔄 Board position updated:', fen);
  }, [fen]);

  return (
    <div className="game">
      <ChessboardComponent
        position={fen}
        onPieceDrop={onDrop}
        onSquareClick={onSquareClick}
        boardOrientation={playerColor}
        arePiecesDraggable={isPlayerTurn() && !isEngineThinking && !isGameOver}
        customSquareStyles={customSquareStyles}
        boardWidth={600}
        animationDuration={200}
      />
    </div>
  );
}
