/**
 * Performance monitoring tests
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { performanceMonitor } from './performance';

describe('PerformanceMonitor', () => {
  beforeEach(() => {
    performanceMonitor.reset();
  });

  it('should track WASM load time', () => {
    performanceMonitor.startWasmLoad();
    performanceMonitor.endWasmLoad();

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.wasmLoadTime).toBeGreaterThan(0);
  });

  it('should track worker creation time', () => {
    performanceMonitor.startWorkerCreate();
    performanceMonitor.endWorkerCreate();

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.workerCreateTime).toBeGreaterThan(0);
  });

  it('should track search metrics', () => {
    performanceMonitor.startSearch();
    performanceMonitor.endSearch();

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.searchCount).toBe(1);
    expect(metrics.firstSearchTime).toBeGreaterThan(0);
    expect(metrics.avgSearchTime).toBeGreaterThan(0);
  });

  it('should calculate average search time correctly', () => {
    // Simulate 3 searches
    for (let i = 0; i < 3; i++) {
      performanceMonitor.startSearch();
      performanceMonitor.endSearch();
    }

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.searchCount).toBe(3);
    expect(metrics.avgSearchTime).toBe(metrics.totalSearchTime / 3);
  });

  it('should reset metrics', () => {
    performanceMonitor.startSearch();
    performanceMonitor.endSearch();

    performanceMonitor.reset();

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.searchCount).toBe(0);
    expect(metrics.totalSearchTime).toBe(0);
    expect(metrics.avgSearchTime).toBe(0);
    expect(metrics.wasmLoadTime).toBeUndefined();
  });

  it('should generate report', () => {
    performanceMonitor.startWasmLoad();
    performanceMonitor.endWasmLoad();
    performanceMonitor.startSearch();
    performanceMonitor.endSearch();

    const report = performanceMonitor.getReport();
    expect(report).toContain('Performance Metrics');
    expect(report).toContain('WASM Load Time');
    expect(report).toContain('Total Searches: 1');
  });

  it('should handle incomplete timers gracefully', () => {
    performanceMonitor.startSearch();
    // Don't end search

    const metrics = performanceMonitor.getMetrics();
    expect(metrics.searchCount).toBe(0); // Search not completed
  });
});
