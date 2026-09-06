#!/usr/bin/env node
// Postinstall: ensure Rust binary exists for npx/bunx
// Tries: 1) check if binary already exists (from cargo-dist or previous build)
//        2) try cargo build --release if cargo is available
//        3) try to download from GitHub releases (if configured)

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

const __dirname = import.meta.dirname;

function findRustBinary() {
  const binName = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const candidates = [join(__dirname, 'target', 'release', binName), join(__dirname, 'target', 'debug', binName)];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  return null;
}

if (findRustBinary()) {
  // Already have binary (from cargo-dist or previous cargo build)
  process.exit(0);
}

// Try cargo build if cargo is available
const cargoCheck = spawnSync('cargo', ['--version'], { stdio: 'ignore' });
if (!cargoCheck.error) {
  console.log('  Building Rust binary via cargo build --release (this may take a minute)...');
  const result = spawnSync('cargo', ['build', '--release'], { stdio: 'inherit', cwd: __dirname });
  if (!result.error && result.status === 0 && findRustBinary()) {
    console.log('  ✔ Rust binary built');
    process.exit(0);
  }
  console.error('  ✘ cargo build failed');
}

// Fallback: try to download from GitHub releases (if GITHUB_TOKEN or public)
// For now, just warn and exit with error to let user know to install Rust
console.error(`
  ✘ Rust binary not found and cargo not available.

  Install Rust: https://rustup.rs/
  Then run: cargo build --release

  Or download the binary from https://github.com/Gabox301/SkillIndex/releases
`);
process.exit(1);
