/**
 * Engine Client Integration Tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { getEngineMode, setEngineMode, getWasmStatus } from './engineClient';

describe('engineClient', () => {
  beforeEach(() => {
    // Reset to remote mode before each test
    setEngineMode('remote');
  });

  describe('Mode Management', () => {
    it('should default to remote mode', () => {
      expect(getEngineMode()).toBe('remote');
    });

    it('should switch to wasm mode', () => {
      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');
    });

    it('should switch to remote mode', () => {
      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');
    });

    it('should switch back to remote mode', () => {
      setEngineMode('wasm');
      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');
    });
  });

  describe('WASM Status', () => {
    it('should start as uninitialized', () => {
      expect(getWasmStatus()).toBe('uninitialized');
    });

    it('should remain uninitialized in remote mode', () => {
      setEngineMode('remote');
      expect(getWasmStatus()).toBe('uninitialized');
    });
  });

  describe('Mode Switching Cleanup', () => {
    it('should clean up WASM worker when switching away from wasm mode', () => {
      setEngineMode('wasm');
      setEngineMode('remote');
      expect(getWasmStatus()).toBe('uninitialized');
    });

    it('should allow switching between all modes', () => {
      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');

      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');

      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');

      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');
    });
  });
});
