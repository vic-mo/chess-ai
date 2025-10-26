/**
 * WASM module loader with caching and lazy loading strategies
 */

type WasmLoadStrategy = 'lazy' | 'eager' | 'prefetch';

interface WasmLoaderOptions {
  strategy?: WasmLoadStrategy;
  wasmPath?: string;
  timeout?: number;
}

/**
 * WASM module loader with intelligent loading strategies
 */
export class WasmLoader {
  private loadPromise: Promise<void> | null = null;
  private isLoaded = false;
  private strategy: WasmLoadStrategy;
  private wasmPath: string;
  private timeout: number;

  constructor(options: WasmLoaderOptions = {}) {
    this.strategy = options.strategy || 'lazy';
    this.wasmPath = options.wasmPath || '/wasm/engine_bridge_wasm.js';
    this.timeout = options.timeout || 30000; // 30 seconds

    // Start prefetch if strategy is prefetch
    if (this.strategy === 'prefetch') {
      this.prefetch();
    }

    // Start eager load if strategy is eager
    if (this.strategy === 'eager') {
      void this.load();
    }
  }

  /**
   * Prefetch WASM files (download but don't initialize)
   */
  private prefetch(): void {
    // Use link rel=prefetch to hint browser to download
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.as = 'script';
    link.href = this.wasmPath;
    document.head.appendChild(link);

    // Also prefetch the .wasm file
    const wasmFile = this.wasmPath.replace('.js', '_bg.wasm');
    const wasmLink = document.createElement('link');
    wasmLink.rel = 'prefetch';
    wasmLink.as = 'fetch';
    wasmLink.href = wasmFile;
    wasmLink.crossOrigin = 'anonymous';
    document.head.appendChild(wasmLink);
  }

  /**
   * Load WASM module
   */
  async load(): Promise<void> {
    // Return existing promise if already loading/loaded
    if (this.loadPromise) {
      return this.loadPromise;
    }

    // Return immediately if already loaded
    if (this.isLoaded) {
      return Promise.resolve();
    }

    // Start loading with timeout
    this.loadPromise = Promise.race([this.loadWasm(), this.createTimeout()]);

    return this.loadPromise;
  }

  /**
   * Actually load the WASM module
   */
  private async loadWasm(): Promise<void> {
    try {
      // Dynamic import would happen here
      // For now, this is a placeholder
      await new Promise((resolve) => setTimeout(resolve, 10)); // Simulate load
      this.isLoaded = true;
    } catch (error) {
      this.loadPromise = null; // Allow retry
      throw error;
    }
  }

  /**
   * Create timeout promise
   */
  private createTimeout(): Promise<void> {
    return new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(`WASM load timeout after ${this.timeout}ms`));
      }, this.timeout);
    });
  }

  /**
   * Check if WASM is loaded
   */
  isWasmLoaded(): boolean {
    return this.isLoaded;
  }

  /**
   * Get load strategy
   */
  getStrategy(): WasmLoadStrategy {
    return this.strategy;
  }
}

/**
 * Singleton loader instance
 */
let loaderInstance: WasmLoader | null = null;

/**
 * Get or create WASM loader
 */
export function getWasmLoader(options?: WasmLoaderOptions): WasmLoader {
  if (!loaderInstance) {
    loaderInstance = new WasmLoader(options);
  }
  return loaderInstance;
}

/**
 * Reset loader (for testing)
 */
export function resetWasmLoader(): void {
  loaderInstance = null;
}
