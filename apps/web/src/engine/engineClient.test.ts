/**
 * Engine Client Integration Tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { getEngineMode, setEngineMode, getWasmStatus } from './engineClient';

describe('engineClient', () => {
  beforeEach(() => {
    // Reset to fake mode before each test
    setEngineMode('fake');
  });

  describe('Mode Management', () => {
    it('should default to fake mode', () => {
      expect(getEngineMode()).toBe('fake');
    });

    it('should switch to wasm mode', () => {
      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');
    });

    it('should switch to remote mode', () => {
      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');
    });

    it('should switch back to fake mode', () => {
      setEngineMode('wasm');
      setEngineMode('fake');
      expect(getEngineMode()).toBe('fake');
    });
  });

  describe('WASM Status', () => {
    it('should start as uninitialized', () => {
      expect(getWasmStatus()).toBe('uninitialized');
    });

    it('should remain uninitialized after switching to fake mode', () => {
      setEngineMode('fake');
      expect(getWasmStatus()).toBe('uninitialized');
    });
  });

  describe('Mode Switching Cleanup', () => {
    it('should clean up WASM worker when switching away from wasm mode', () => {
      setEngineMode('wasm');
      setEngineMode('fake');
      expect(getWasmStatus()).toBe('uninitialized');
    });

    it('should allow switching between all modes', () => {
      setEngineMode('fake');
      expect(getEngineMode()).toBe('fake');

      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');

      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');

      setEngineMode('fake');
      expect(getEngineMode()).toBe('fake');
    });
  });
});
