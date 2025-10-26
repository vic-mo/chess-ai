import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  return {
    plugins: [react()],
    define: {
      __ENGINE_MODE__: JSON.stringify(env.VITE_ENGINE_MODE || 'fake'),
    },
    optimizeDeps: {
      exclude: ['@chess-ai/engine-wasm'],
    },
    test: {
      globals: true,
      environment: 'jsdom',
      exclude: ['node_modules', 'dist', 'e2e/**'],
    },
  };
});
