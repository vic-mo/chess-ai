import { useGameStore } from '../store/gameStore';

export function GameStatus() {
  const { fen, isEngineThinking, isGameOver, gameResult, winner } = useGameStore();

  const getCurrentTurn = () => {
    const turn = fen.split(' ')[1];
    return turn === 'w' ? 'White' : 'Black';
  };

  const getStatusMessage = () => {
    if (isGameOver) {
      if (gameResult === 'checkmate') {
        return (
          <div className="status-message status-game-over">
            Checkmate! {winner === 'white' ? 'White' : 'Black'} wins!
          </div>
        );
      } else if (gameResult === 'stalemate') {
        return <div className="status-message status-game-over">Stalemate - Draw</div>;
      } else if (gameResult === 'resignation') {
        return (
          <div className="status-message status-game-over">
            {winner === 'white' ? 'White' : 'Black'} wins by resignation
          </div>
        );
      }
    }

    if (isEngineThinking) {
      return <div className="status-message status-thinking">Engine thinking...</div>;
    }

    return <div className="status-message">{getCurrentTurn()} to move</div>;
  };

  return (
    <div className="game-status">
      <h3>Game Status</h3>
      {getStatusMessage()}
    </div>
  );
}
