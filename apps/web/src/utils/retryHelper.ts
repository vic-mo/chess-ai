/**
 * Retry Helper
 *
 * Provides retry logic with exponential backoff for async operations
 */

import { logger } from './logger';

export interface RetryOptions {
  maxAttempts?: number;
  initialDelay?: number;
  maxDelay?: number;
  backoffMultiplier?: number;
  shouldRetry?: (error: any) => boolean;
}

const DEFAULT_OPTIONS: Required<RetryOptions> = {
  maxAttempts: 3,
  initialDelay: 1000, // 1s
  maxDelay: 5000, // 5s
  backoffMultiplier: 2,
  shouldRetry: () => true,
};

/**
 * Retry an async function with exponential backoff
 */
export async function retry<T>(fn: () => Promise<T>, options: RetryOptions = {}): Promise<T> {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  let lastError: any;
  let delay = opts.initialDelay;

  for (let attempt = 1; attempt <= opts.maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      // Check if we should retry
      if (!opts.shouldRetry(error)) {
        logger.log(`[Retry] Not retrying - shouldRetry returned false`);
        throw error;
      }

      // Check if we've exhausted attempts
      if (attempt >= opts.maxAttempts) {
        logger.log(`[Retry] Max attempts (${opts.maxAttempts}) reached`);
        throw error;
      }

      // Log retry attempt
      logger.log(`[Retry] Attempt ${attempt} failed, retrying in ${delay}ms...`, error);

      // Wait before retrying
      await new Promise((resolve) => setTimeout(resolve, delay));

      // Increase delay with exponential backoff
      delay = Math.min(delay * opts.backoffMultiplier, opts.maxDelay);
    }
  }

  // Should never reach here, but TypeScript needs it
  throw lastError;
}

/**
 * Wrapper for retry with timeout
 */
export async function retryWithTimeout<T>(
  fn: () => Promise<T>,
  timeoutMs: number,
  retryOptions: RetryOptions = {},
): Promise<T> {
  return retry(async () => {
    return Promise.race([
      fn(),
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error('Operation timeout')), timeoutMs),
      ),
    ]);
  }, retryOptions);
}
