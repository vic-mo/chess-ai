import { useGameStore } from '../store/gameStore';

export function GameControls() {
  const { playerColor, difficulty, isGameOver, isEngineThinking, newGame, resign, setDifficulty } =
    useGameStore();

  const handleNewGame = () => {
    newGame(playerColor);
  };

  const handleColorChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const color = e.target.value as 'white' | 'black';
    newGame(color);
  };

  const handleDifficultyChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setDifficulty(parseInt(e.target.value));
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
        <div className="difficulty-label">
          <label htmlFor="difficulty">Difficulty</label>
          <span className="difficulty-value">{difficulty}</span>
        </div>
        <input
          id="difficulty"
          type="range"
          min="1"
          max="20"
          value={difficulty}
          onChange={handleDifficultyChange}
        />
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
