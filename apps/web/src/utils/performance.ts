/**
 * Performance monitoring utilities for WASM engine
 */

import { logger } from './logger';

export interface PerformanceMetrics {
  wasmLoadTime?: number;
  wasmInitTime?: number;
  workerCreateTime?: number;
  firstSearchTime?: number;
  searchCount: number;
  totalSearchTime: number;
  avgSearchTime: number;
  peakMemoryMB?: number;
}

interface PerformanceTimers {
  wasmLoadStart?: number;
  workerCreateStart?: number;
  searchStart?: number;
}

class PerformanceMonitor {
  private metrics: PerformanceMetrics = {
    searchCount: 0,
    totalSearchTime: 0,
    avgSearchTime: 0,
  };

  private timers: PerformanceTimers = {};

  /**
   * Mark the start of WASM loading
   */
  startWasmLoad(): void {
    this.timers.wasmLoadStart = performance.now();
  }

  /**
   * Mark the end of WASM loading
   */
  endWasmLoad(): void {
    if (this.timers.wasmLoadStart) {
      this.metrics.wasmLoadTime = performance.now() - this.timers.wasmLoadStart;
      this.timers.wasmLoadStart = undefined;
    }
  }

  /**
   * Mark the start of worker creation
   */
  startWorkerCreate(): void {
    this.timers.workerCreateStart = performance.now();
  }

  /**
   * Mark the end of worker creation
   */
  endWorkerCreate(): void {
    if (this.timers.workerCreateStart) {
      this.metrics.workerCreateTime = performance.now() - this.timers.workerCreateStart;
      this.timers.workerCreateStart = undefined;
    }
  }

  /**
   * Mark the start of a search
   */
  startSearch(): void {
    this.timers.searchStart = performance.now();
  }

  /**
   * Mark the end of a search
   */
  endSearch(): void {
    if (this.timers.searchStart) {
      const searchTime = performance.now() - this.timers.searchStart;

      // Update first search time if not set
      if (!this.metrics.firstSearchTime) {
        this.metrics.firstSearchTime = searchTime;
      }

      // Update search statistics
      this.metrics.searchCount++;
      this.metrics.totalSearchTime += searchTime;
      this.metrics.avgSearchTime = this.metrics.totalSearchTime / this.metrics.searchCount;

      this.timers.searchStart = undefined;
    }
  }

  /**
   * Get current memory usage (if available)
   */
  updateMemoryUsage(): void {
    if ('memory' in performance && performance.memory) {
      const memoryInfo = performance.memory as {
        usedJSHeapSize: number;
        totalJSHeapSize: number;
        jsHeapSizeLimit: number;
      };
      const usedMB = memoryInfo.usedJSHeapSize / (1024 * 1024);

      if (!this.metrics.peakMemoryMB || usedMB > this.metrics.peakMemoryMB) {
        this.metrics.peakMemoryMB = usedMB;
      }
    }
  }

  /**
   * Get current metrics
   */
  getMetrics(): PerformanceMetrics {
    this.updateMemoryUsage();
    return { ...this.metrics };
  }

  /**
   * Reset all metrics
   */
  reset(): void {
    this.metrics = {
      searchCount: 0,
      totalSearchTime: 0,
      avgSearchTime: 0,
    };
    this.timers = {};
  }

  /**
   * Get a formatted report of metrics
   */
  getReport(): string {
    this.updateMemoryUsage();
    const m = this.metrics;

    const lines: string[] = ['=== Performance Metrics ==='];

    if (m.wasmLoadTime !== undefined) {
      lines.push(`WASM Load Time: ${m.wasmLoadTime.toFixed(2)}ms`);
    }

    if (m.workerCreateTime !== undefined) {
      lines.push(`Worker Create Time: ${m.workerCreateTime.toFixed(2)}ms`);
    }

    if (m.firstSearchTime !== undefined) {
      lines.push(`First Search Time: ${m.firstSearchTime.toFixed(2)}ms`);
    }

    lines.push(`Total Searches: ${m.searchCount}`);

    if (m.searchCount > 0) {
      lines.push(`Avg Search Time: ${m.avgSearchTime.toFixed(2)}ms`);
      lines.push(`Total Search Time: ${m.totalSearchTime.toFixed(2)}ms`);
    }

    if (m.peakMemoryMB !== undefined) {
      lines.push(`Peak Memory: ${m.peakMemoryMB.toFixed(2)}MB`);
    }

    return lines.join('\n');
  }
}

// Singleton instance
export const performanceMonitor = new PerformanceMonitor();

/**
 * Decorator for measuring function execution time
 */
export function measureTime(label: string) {
  return function (_target: unknown, _propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: unknown[]) {
      const start = performance.now();
      try {
        return await originalMethod.apply(this, args);
      } finally {
        const duration = performance.now() - start;
        logger.debug(`[Performance] ${label}: ${duration.toFixed(2)}ms`);
      }
    };

    return descriptor;
  };
}

/**
 * Measure async function execution time
 */
export async function measure<T>(label: string, fn: () => Promise<T> | T): Promise<T> {
  const start = performance.now();
  try {
    return await fn();
  } finally {
    const duration = performance.now() - start;
    logger.debug(`[Performance] ${label}: ${duration.toFixed(2)}ms`);
  }
}

/**
 * Get browser performance info
 */
export function getBrowserPerformance(): {
  connection?: string;
  hardwareConcurrency?: number;
  deviceMemory?: number;
} {
  const nav = navigator as Navigator & {
    connection?: { effectiveType?: string };
    deviceMemory?: number;
  };

  return {
    connection: nav.connection?.effectiveType,
    hardwareConcurrency: navigator.hardwareConcurrency,
    deviceMemory: nav.deviceMemory,
  };
}
