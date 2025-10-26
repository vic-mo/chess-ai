import { describe, it, expect, beforeEach } from 'vitest';
import {
  detectBrowser,
  hasWebAssembly,
  hasWebWorkers,
  hasSharedArrayBuffer,
  hasPerformanceMemory,
  getCompatibilityInfo,
  isWasmCompatible,
} from './browserDetect';

describe('Browser Detection', () => {
  describe('detectBrowser', () => {
    const originalUserAgent = navigator.userAgent;

    beforeEach(() => {
      // Reset user agent
      Object.defineProperty(navigator, 'userAgent', {
        value: originalUserAgent,
        configurable: true,
      });
    });

    it('should detect Chrome', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value:
          'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Chrome');
      expect(browser.version).toBe('120');
      expect(browser.os).toBe('Windows');
      expect(browser.isMobile).toBe(false);
    });

    it('should detect Firefox', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Firefox');
      expect(browser.version).toBe('121');
      expect(browser.os).toBe('Windows');
    });

    it('should detect Safari', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value:
          'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Safari');
      expect(browser.version).toBe('17');
      expect(browser.os).toBe('macOS');
    });

    it('should detect Edge', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value:
          'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Edge');
      expect(browser.version).toBe('120');
    });

    it('should detect mobile Safari', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value:
          'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Safari');
      expect(browser.os).toBe('iOS');
      expect(browser.isMobile).toBe(true);
    });

    it('should detect Chrome Android', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value:
          'Mozilla/5.0 (Linux; Android 10; SM-G973F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36',
        configurable: true,
      });
      const browser = detectBrowser();
      expect(browser.name).toBe('Chrome');
      expect(browser.os).toBe('Android');
      expect(browser.isMobile).toBe(true);
    });
  });

  describe('Feature Detection', () => {
    it('should detect WebAssembly support', () => {
      // WebAssembly is available in test environment
      expect(hasWebAssembly()).toBe(true);
    });

    it('should detect Web Workers support', () => {
      // Worker may not be available in test environment (jsdom)
      const result = hasWebWorkers();
      expect(typeof result).toBe('boolean');
    });

    it('should detect SharedArrayBuffer support', () => {
      // May or may not be available depending on headers
      const result = hasSharedArrayBuffer();
      expect(typeof result).toBe('boolean');
    });

    it('should detect Performance Memory API', () => {
      // Chrome/Edge specific feature
      const result = hasPerformanceMemory();
      expect(typeof result).toBe('boolean');
    });
  });

  describe('getCompatibilityInfo', () => {
    it('should return complete compatibility info', () => {
      const info = getCompatibilityInfo();

      expect(info.browser).toBeDefined();
      expect(info.browser.name).toBeTruthy();
      expect(info.browser.version).toBeTruthy();
      expect(info.browser.os).toBeTruthy();

      expect(info.features).toBeDefined();
      expect(typeof info.features.webAssembly).toBe('boolean');
      expect(typeof info.features.webWorkers).toBe('boolean');
      expect(typeof info.features.sharedArrayBuffer).toBe('boolean');
      expect(typeof info.features.performanceMemory).toBe('boolean');

      expect(Array.isArray(info.warnings)).toBe(true);
      expect(Array.isArray(info.errors)).toBe(true);
    });

    it('should not have errors if WASM and Workers are supported', () => {
      const info = getCompatibilityInfo();
      // In a modern test environment, these should be supported
      if (info.features.webAssembly && info.features.webWorkers) {
        expect(info.errors.length).toBe(0);
      }
    });
  });

  describe('isWasmCompatible', () => {
    it('should return boolean based on WASM and Workers support', () => {
      const result = isWasmCompatible();
      expect(typeof result).toBe('boolean');
      // Should be true only if both are available
      expect(result).toBe(hasWebAssembly() && hasWebWorkers());
    });
  });
});
