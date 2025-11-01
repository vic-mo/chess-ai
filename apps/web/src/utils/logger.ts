/**
 * Conditional logger that only logs in development mode
 * In production, all logs are no-ops
 */

const isDev = import.meta.env.DEV;

export const logger = {
  log: (...args: any[]) => {
    if (isDev) console.log(...args);
  },
  debug: (...args: any[]) => {
    if (isDev) console.debug(...args);
  },
  warn: (...args: any[]) => {
    if (isDev) console.warn(...args);
  },
  error: (...args: any[]) => {
    if (isDev) console.error(...args);
  },
  info: (...args: any[]) => {
    if (isDev) console.info(...args);
  },
};
