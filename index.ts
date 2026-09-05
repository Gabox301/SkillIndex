#!/usr/bin/env node
// Shim mínimo — single source of truth en Rust. Node solo delega.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

const [major, minor]: number[] = process.versions.node.split('.').map(Number) as number[];

if (major < 22 || (major === 22 && minor < 6)) {
  console.error(
    `\n  ⚠ skillindex requiere Node.js >= 22.6.0.` +
      `\n  Versión actual: ${process.version}` +
      `\n  Por favor, actualiza → https://nodejs.org\n`,
  );
  process.exit(1);
}

const __dirname: string = import.meta.dirname;

function findRustBinary(): string | null {
  const binName: string = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const candidates: string[] = [
    join(__dirname, 'target', 'release', binName),
    join(__dirname, 'target', 'debug', binName),
    join(__dirname, '..', 'target', 'release', binName),
    join(__dirname, '..', 'target', 'debug', binName),
    join(__dirname, '..', '..', 'target', 'release', binName),
    join(__dirname, '..', '..', 'target', 'debug', binName),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  return null;
}

const rustBin: string | null = findRustBinary();
if (rustBin) {
  const result = spawnSync(rustBin, process.argv.slice(2), { stdio: 'inherit' });
  if (!result.error) process.exit(result.status ?? 0);
  console.error(`\n  ✘ No se pudo ejecutar el binario Rust: ${result.error.message}\n`);
  process.exit(1);
}

if (existsSync(join(__dirname, 'Cargo.toml'))) {
  const manifest: string = join(__dirname, 'Cargo.toml');
  const result = spawnSync('cargo', ['run', '--quiet', '--manifest-path', manifest, '--', ...process.argv.slice(2)], {
    stdio: 'inherit',
  });
  if (!result.error) process.exit(result.status ?? 0);
}

console.error(`
  ✘ Binario Rust no encontrado.

  En dev:  cargo build --release
  Publicado: el binario se distribuye con el paquete npm (cargo-dist).
  Si ves esto tras npx skillindex, reportá en https://github.com/Gabox301/SkillIndex/issues
`);
process.exit(1);
