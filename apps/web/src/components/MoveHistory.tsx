import { useGameStore } from '../store/gameStore';

export function MoveHistory() {
  const { moveHistory } = useGameStore();

  const formatMoveHistory = () => {
    if (moveHistory.length === 0) {
      return 'No moves yet';
    }

    const moves: string[] = [];
    for (let i = 0; i < moveHistory.length; i += 2) {
      const moveNumber = Math.floor(i / 2) + 1;
      const whiteMove = moveHistory[i];
      const blackMove = moveHistory[i + 1];

      if (blackMove) {
        moves.push(`${moveNumber}. ${whiteMove} ${blackMove}`);
      } else {
        moves.push(`${moveNumber}. ${whiteMove}`);
      }
    }

    return moves.join('\n');
  };

  return (
    <div className="move-history">
      <h3>Move History</h3>
      <div className="move-history-list">
        <pre>{formatMoveHistory()}</pre>
      </div>
    </div>
  );
}
