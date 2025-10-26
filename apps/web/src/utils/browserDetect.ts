/**
 * Browser Detection and Compatibility Utilities
 *
 * Detects browser type, version, and checks for required features like
 * WebAssembly, Web Workers, and SharedArrayBuffer support.
 */

export interface BrowserInfo {
  name: string;
  version: string;
  os: string;
  isMobile: boolean;
}

export interface CompatibilityInfo {
  browser: BrowserInfo;
  features: {
    webAssembly: boolean;
    webWorkers: boolean;
    sharedArrayBuffer: boolean;
    performanceMemory: boolean;
  };
  warnings: string[];
  errors: string[];
}

/**
 * Detect browser name and version
 */
export function detectBrowser(): BrowserInfo {
  const ua = navigator.userAgent;
  const mobile = /Mobile|Android|iPhone|iPad|iPod/i.test(ua);

  // Detect OS - check mobile OSes first before desktop
  let os = 'Unknown';
  if (/iPhone|iPad|iPod/i.test(ua)) os = 'iOS';
  else if (/Android/i.test(ua)) os = 'Android';
  else if (/Windows/i.test(ua)) os = 'Windows';
  else if (/Mac OS X/i.test(ua)) os = 'macOS';
  else if (/Linux/i.test(ua)) os = 'Linux';

  // Detect browser - order matters!
  // Check Edge before Chrome (Edge includes "Chrome" in UA)
  if (/Edg\/(\d+)/.test(ua)) {
    const version = ua.match(/Edg\/(\d+)/)?.[1] || 'unknown';
    return { name: 'Edge', version, os, isMobile: mobile };
  }

  // Check Chrome before Safari (Chrome includes "Safari" in UA)
  if (/Chrome\/(\d+)/.test(ua) && !/Edg/.test(ua)) {
    const version = ua.match(/Chrome\/(\d+)/)?.[1] || 'unknown';
    return { name: 'Chrome', version, os, isMobile: mobile };
  }

  // Safari
  if (/Safari\/(\d+)/.test(ua) && !/Chrome/.test(ua)) {
    const version = ua.match(/Version\/(\d+)/)?.[1] || 'unknown';
    return { name: 'Safari', version, os, isMobile: mobile };
  }

  // Firefox
  if (/Firefox\/(\d+)/.test(ua)) {
    const version = ua.match(/Firefox\/(\d+)/)?.[1] || 'unknown';
    return { name: 'Firefox', version, os, isMobile: mobile };
  }

  return { name: 'Unknown', version: 'unknown', os, isMobile: mobile };
}

/**
 * Check if WebAssembly is supported
 */
export function hasWebAssembly(): boolean {
  try {
    return typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function';
  } catch {
    return false;
  }
}

/**
 * Check if Web Workers are supported
 */
export function hasWebWorkers(): boolean {
  try {
    return typeof Worker === 'function';
  } catch {
    return false;
  }
}

/**
 * Check if SharedArrayBuffer is supported
 */
export function hasSharedArrayBuffer(): boolean {
  try {
    return typeof SharedArrayBuffer === 'function';
  } catch {
    return false;
  }
}

/**
 * Check if Performance Memory API is available
 */
export function hasPerformanceMemory(): boolean {
  try {
    return 'memory' in performance;
  } catch {
    return false;
  }
}

/**
 * Get full compatibility information
 */
export function getCompatibilityInfo(): CompatibilityInfo {
  const browser = detectBrowser();
  const warnings: string[] = [];
  const errors: string[] = [];

  // Check required features
  const webAssembly = hasWebAssembly();
  const webWorkers = hasWebWorkers();
  const sharedArrayBuffer = hasSharedArrayBuffer();
  const performanceMemory = hasPerformanceMemory();

  // Critical errors
  if (!webAssembly) {
    errors.push('WebAssembly is not supported. WASM mode will not work.');
  }
  if (!webWorkers) {
    errors.push('Web Workers are not supported. WASM mode will not work.');
  }

  // Warnings
  if (!sharedArrayBuffer) {
    warnings.push(
      'SharedArrayBuffer is not available. Multi-threaded search is not available (future feature).',
    );
  }
  if (!performanceMemory) {
    warnings.push('Performance Memory API is not available. Memory tracking will not work.');
  }

  // Browser-specific warnings
  if (browser.name === 'Safari' && browser.isMobile) {
    warnings.push('iOS Safari may have reduced WASM performance due to JIT limitations.');
  }

  if (browser.name === 'Firefox' && parseInt(browser.version) < 79) {
    warnings.push('Firefox versions before 79 may have slower WASM performance.');
  }

  return {
    browser,
    features: {
      webAssembly,
      webWorkers,
      sharedArrayBuffer,
      performanceMemory,
    },
    warnings,
    errors,
  };
}

/**
 * Format compatibility info as a readable string
 */
export function formatCompatibilityReport(info: CompatibilityInfo): string {
  const lines: string[] = [];

  lines.push('=== Browser Compatibility Report ===');
  lines.push('');
  lines.push(`Browser: ${info.browser.name} ${info.browser.version}`);
  lines.push(`OS: ${info.browser.os}`);
  lines.push(`Mobile: ${info.browser.isMobile ? 'Yes' : 'No'}`);
  lines.push('');
  lines.push('Features:');
  lines.push(`  WebAssembly: ${info.features.webAssembly ? '✓' : '✗'}`);
  lines.push(`  Web Workers: ${info.features.webWorkers ? '✓' : '✗'}`);
  lines.push(`  SharedArrayBuffer: ${info.features.sharedArrayBuffer ? '✓' : '✗'}`);
  lines.push(`  Performance Memory: ${info.features.performanceMemory ? '✓' : '✗'}`);

  if (info.errors.length > 0) {
    lines.push('');
    lines.push('Errors:');
    info.errors.forEach((err) => lines.push(`  ✗ ${err}`));
  }

  if (info.warnings.length > 0) {
    lines.push('');
    lines.push('Warnings:');
    info.warnings.forEach((warn) => lines.push(`  ⚠ ${warn}`));
  }

  return lines.join('\n');
}

/**
 * Check if browser is compatible with WASM mode
 */
export function isWasmCompatible(): boolean {
  return hasWebAssembly() && hasWebWorkers();
}
