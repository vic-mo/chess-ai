import { useEffect } from 'react';
import { setEngineMode } from './engine/engineClient';
import { PlayMode } from './components/PlayMode';
import './styles.css';

export default function App() {
  useEffect(() => {
    // Set engine mode to remote on mount
    setEngineMode('remote');
  }, []);

  return (
    <div className="container">
      <h1>♟️ Chess AI</h1>
      <PlayMode />
    </div>
  );
}
