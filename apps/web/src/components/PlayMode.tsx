import { Game } from './Game';
import { GameControls } from './GameControls';
import { GameStatus } from './GameStatus';
import { MoveHistory } from './MoveHistory';

export function PlayMode() {
  return (
    <div className="game-container">
      <div className="game-sidebar">
        <GameStatus />
        <GameControls />
        <MoveHistory />
      </div>
      <Game />
    </div>
  );
}
