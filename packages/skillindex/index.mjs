#!/usr/bin/env node
// Shim — delega a index.ts (entry tipado). Mantiene compatibilidad bin `index.mjs` para npm/bun.
// La lógica canónica vive en index.ts; este shim solo asegura ejecución con bun si es necesario.

import { spawn } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const tsEntry = join(__dirname, 'index.ts');

try {
  await import('./index.ts');
} catch (err) {
  const code = err && typeof err === 'object' && 'code' in err ? String(err.code) : '';
  const msg = err instanceof Error ? err.message : String(err);
  const isUnknownExt =
    code === 'ERR_UNKNOWN_FILE_EXTENSION' ||
    msg.includes('ERR_UNKNOWN_FILE_EXTENSION') ||
    msg.includes('Unknown file extension');
  if (!isUnknownExt) throw err;
  const bunChild = spawn('bun', [tsEntry, ...process.argv.slice(2)], { stdio: 'inherit' });
  bunChild.on('error', (bunErr) => {
    if (bunErr.code === 'ENOENT') {
      const nodeChild = spawn(
        process.execPath,
        ['--experimental-strip-types', '--disable-warning=ExperimentalWarning', tsEntry, ...process.argv.slice(2)],
        { stdio: 'inherit' },
      );
      nodeChild.on('exit', (c, s) => {
        if (s) process.kill(process.pid, s);
        else process.exit(c ?? 1);
      });
      nodeChild.on('error', (e) => {
        throw e;
      });
    } else throw bunErr;
  });
  bunChild.on('exit', (c, s) => {
    if (s) process.kill(process.pid, s);
    else process.exit(c ?? 1);
  });
}
