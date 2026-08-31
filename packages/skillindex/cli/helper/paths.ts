// Shared path helpers for producing stable, POSIX-style relative paths
// regardless of the host platform's separator.

import { relative } from 'node:path';

/** Convert a path to POSIX form by normalizing backslashes to forward slashes. */
export function toPosixPath(path: string): string {
  return path.split('\\').join('/');
}

/** Relative path from `from` to `to`, normalized to POSIX form. */
export function relativePosixPath(from: string, to: string): string {
  return toPosixPath(relative(from, to));
}
