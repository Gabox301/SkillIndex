#!/usr/bin/env node
// Shim — entry bin para npm/bun. Importa el JS compilado, no TS, para evitar
// ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING en node_modules.
// La lógica canónica vive en index.ts -> dist/index.js (generado por tsc).

import { existsSync } from 'node:fs';
import { join } from 'node:path';

const __dirname = import.meta.dirname;

// En el paquete publicado, dist/index.js existe y es el entry compilado.
// En dev (sin dist), fallback a index.ts via bun (no requiere strip-types en node_modules).
if (existsSync(join(__dirname, 'dist', 'index.js'))) {
  await import('./dist/index.js');
} else {
  // Dev fallback: intenta index.ts con bun (evita ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING)
  const { spawn } = await import('node:child_process');
  const tsEntry = join(__dirname, 'index.ts');
  const bunChild = spawn('bun', [tsEntry, ...process.argv.slice(2)], { stdio: 'inherit' });
  bunChild.on('error', () => {
    // Si bun no está, intenta node con strip-types solo en dev (fuera de node_modules)
    const nodeChild = spawn(
      process.execPath,
      ['--experimental-strip-types', '--disable-warning=ExperimentalWarning', tsEntry, ...process.argv.slice(2)],
      { stdio: 'inherit' },
    );
    nodeChild.on('exit', (c, s) => {
      if (s) process.kill(process.pid, s);
      else process.exit(c ?? 1);
    });
  });
  bunChild.on('exit', (c, s) => {
    if (s) process.kill(process.pid, s);
    else process.exit(c ?? 1);
  });
}
