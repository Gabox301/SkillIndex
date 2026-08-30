#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const [major, minor]: number[] = process.versions.node.split('.').map(Number) as number[];

if (major < 22 || (major === 22 && minor < 6)) {
  console.error(
    `\n  ⚠ skillindex requiere Node.js >= 22.6.0.` +
      `\n  Versión actual: ${process.version}` +
      `\n  Por favor, actualiza → https://nodejs.org\n`,
  );
  process.exit(1);
}

const __dirname: string = dirname(fileURLToPath(import.meta.url));

// ── Rust probe gate ───────────────────────────────────────
// Try Rust binary first, unless forced to Node via SKILLINDEX_USE_RUST=0
const forceNode: boolean =
  process.env.SKILLINDEX_USE_RUST === '0' ||
  process.env.SKILLINDEX_USE_NODE === '1' ||
  process.env.SKILLINDEX_USE_RUST === 'false';

function findRustBinary(): string | null {
  const binName: string = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const candidates: string[] = [
    join(__dirname, 'target', 'release', binName),
    join(__dirname, 'target', 'debug', binName),
    join(__dirname, '..', 'target', 'release', binName),
    join(__dirname, '..', 'target', 'debug', binName),
    join(__dirname, '..', '..', 'target', 'release', binName),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  return null;
}

if (!forceNode) {
  const rustBin: string | null = findRustBinary();
  if (rustBin) {
    const result = spawnSync(rustBin, process.argv.slice(2), { stdio: 'inherit' });
    // If spawn succeeded, exit with Rust's code; if error (e.g. EACCES), fallback to Node
    if (!result.error) {
      process.exit(result.status ?? 0);
    }
  } else if (existsSync(join(__dirname, 'Cargo.toml'))) {
    // Dev fallback: try `cargo run --quiet` if binary not yet built
    const manifest: string = join(__dirname, 'Cargo.toml');
    const result = spawnSync('cargo', ['run', '--quiet', '--manifest-path', manifest, '--', ...process.argv.slice(2)], {
      stdio: 'inherit',
    });
    if (!result.error) {
      process.exit(result.status ?? 0);
    }
    // cargo not found or failed to spawn → fall through to Node
  }
}

// ── Node/Bun fallback (original, preserved) ──────────────────────────
// Mantiene `npx/bunx skillindex` funcionando cuando Rust no está presente o se fuerza Node.
// Preserva el probe `dist/main.js` + `bun` para `main.ts`.

if (existsSync(join(__dirname, 'dist', 'main.js'))) {
  // @ts-ignore — dist artifact has no types, runtime import only
  await import('./dist/main.js');
} else {
  try {
    await import('./main.ts');
  } catch (err: unknown) {
    const code = (err as { code?: string })?.code;
    if (code !== 'ERR_UNKNOWN_FILE_EXTENSION') throw err;
    const { spawn } = await import('node:child_process');
    const mainPath: string = join(__dirname, 'main.ts');
    const child = spawn('bun', [mainPath, ...process.argv.slice(2)], { stdio: 'inherit' });
    child.on('exit', (code: number | null, signal: string | null) => {
      if (signal) process.kill(process.pid, signal as NodeJS.Signals);
      else process.exit(code ?? 1);
    });
  }
}
