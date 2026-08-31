// Shared SHA-256 hashing helpers used across scripts and the installer.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

/** SHA-256 hex digest of a buffer or string. */
export function sha256Hex(buf: Buffer | string): string {
  return createHash('sha256').update(buf).digest('hex');
}

/** SHA-256 hex digest of a file's contents. */
export function sha256File(path: string): string {
  return sha256Hex(readFileSync(path));
}
