import { useGameStore } from '../store/gameStore';
import { ELO_LEVELS } from '../utils/eloMapping';

export function GameControls() {
  const { playerColor, eloRating, isGameOver, isEngineThinking, newGame, resign, setEloRating } =
    useGameStore();

  const handleNewGame = () => {
    newGame(playerColor);
  };

  const handleColorChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const color = e.target.value as 'white' | 'black';
    newGame(color);
  };

  const handleEloChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setEloRating(parseInt(e.target.value));
  };

  return (
    <div className="game-controls">
      <h3>Game Controls</h3>

      <div className="control-group">
        <label htmlFor="player-color">Play as</label>
        <select id="player-color" value={playerColor} onChange={handleColorChange}>
          <option value="white">White</option>
          <option value="black">Black</option>
        </select>
      </div>

      <div className="control-group">
        <label htmlFor="elo-rating">Difficulty</label>
        <select id="elo-rating" value={eloRating} onChange={handleEloChange}>
          {ELO_LEVELS.map((level) => (
            <option key={level.elo} value={level.elo}>
              {level.label} ({level.elo} Elo)
            </option>
          ))}
        </select>
      </div>

      <div className="row">
        <button className="btn" onClick={handleNewGame} disabled={isEngineThinking}>
          New Game
        </button>
        <button className="btn" onClick={resign} disabled={isGameOver || isEngineThinking}>
          Resign
        </button>
      </div>
    </div>
  );
}
