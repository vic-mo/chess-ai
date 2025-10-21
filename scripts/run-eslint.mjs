#!/usr/bin/env node

import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const binName = process.platform === 'win32' ? 'eslint.cmd' : 'eslint';
const eslintBin = path.resolve(__dirname, '..', 'node_modules', '.bin', binName);

const proc = spawn(eslintBin, ['--ext', '.ts,.tsx', '.', ...args], {
  stdio: 'inherit',
  env: { ...process.env, ESLINT_USE_FLAT_CONFIG: 'false' },
});

proc.on('exit', (code) => process.exit(code ?? 0));
