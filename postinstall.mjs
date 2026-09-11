#!/usr/bin/env node
// Postinstall: asegura binario Rust para npx/pnpm/bunx
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, existsSync, mkdirSync, readdirSync, rmSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const require = createRequire(import.meta.url);
let pkgVersion = '3.1.0';
try {
  const pkg = require('./package.json');
  if (pkg.version) pkgVersion = pkg.version;
} catch {}

const __dirname = import.meta.dirname;

function findRustBinary() {
  const binName = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const candidates = [
    join(__dirname, 'bin', binName),
    join(__dirname, 'target', 'release', binName),
    join(__dirname, 'target', 'debug', binName),
  ];
  for (const p of candidates) if (existsSync(p)) return p;
  return null;
}

if (findRustBinary()) process.exit(0);

// 2) cargo build si hay Cargo.toml y cargo disponible (solo dev)
const cargoCheck = spawnSync('cargo', ['--version'], { stdio: 'ignore' });
if (!cargoCheck.error && existsSync(join(__dirname, 'Cargo.toml'))) {
  console.log('  Building Rust binary via cargo build --release...');
  const result = spawnSync('cargo', ['build', '--release'], { stdio: 'inherit', cwd: __dirname });
  if (!result.error && result.status === 0 && findRustBinary()) {
    console.log('  ✔ Rust binary built');
    process.exit(0);
  }
  console.error('  ✘ cargo build failed, intentando descarga desde GitHub Releases...');
}

// 3) descarga desde GitHub Releases (cargo-dist)
function getTarget() {
  const plat = process.platform;
  const arch = process.arch;
  if (plat === 'win32' && arch === 'x64') return { target: 'x86_64-pc-windows-msvc', ext: 'zip' };
  if (plat === 'win32' && arch === 'arm64') return { target: 'aarch64-pc-windows-msvc', ext: 'zip' };
  if (plat === 'darwin' && arch === 'x64') return { target: 'x86_64-apple-darwin', ext: 'tar.xz' };
  if (plat === 'darwin' && arch === 'arm64') return { target: 'aarch64-apple-darwin', ext: 'tar.xz' };
  if (plat === 'linux' && arch === 'x64') return { target: 'x86_64-unknown-linux-gnu', ext: 'tar.xz' };
  if (plat === 'linux' && arch === 'arm64') return { target: 'aarch64-unknown-linux-gnu', ext: 'tar.xz' };
  return null;
}

function downloadWithCurl(url, dest) {
  let res = spawnSync('curl', ['-L', '-o', dest, url], { stdio: 'inherit' });
  if (!res.error && res.status === 0 && existsSync(dest)) return true;
  res = spawnSync('wget', ['-O', dest, url], { stdio: 'inherit' });
  if (!res.error && res.status === 0 && existsSync(dest)) return true;
  // Windows fallback: powershell Invoke-WebRequest
  if (process.platform === 'win32') {
    res = spawnSync('powershell', ['-Command', `Invoke-WebRequest -Uri "${url}" -OutFile "${dest}"`], {
      stdio: 'inherit',
    });
    if (!res.error && res.status === 0 && existsSync(dest)) return true;
  }
  return false;
}

const targetInfo = getTarget();
if (targetInfo) {
  const { target, ext } = targetInfo;
  const fileName = `skillindex-${target}.${ext}`;
  const url = `https://github.com/Gabox301/SkillIndex/releases/download/v${pkgVersion}/${fileName}`;
  console.log(`  Descargando binario precompilado desde ${url} ...`);
  const tmpFile = join(tmpdir(), fileName);
  const destDir = join(__dirname, 'bin');
  mkdirSync(destDir, { recursive: true });
  const binName = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const destBin = join(destDir, binName);
  const releaseBin = join(__dirname, 'target', 'release', binName);
  try {
    if (downloadWithCurl(url, tmpFile)) {
      let extracted = false;
      if (ext === 'zip') {
        const ps = spawnSync(
          'powershell',
          [
            '-Command',
            `Expand-Archive -Path "${tmpFile}" -DestinationPath "${destDir}" -Force; Get-ChildItem -Recurse "${destDir}" | Where-Object { $_.Name -eq "${binName}" } | ForEach-Object { Move-Item $_.FullName "${destBin}" -Force }`,
          ],
          { stdio: 'inherit' },
        );
        if (!ps.error && ps.status === 0 && existsSync(destBin)) extracted = true;
      } else {
        const tarCheck = spawnSync('tar', ['--version'], { stdio: 'ignore' });
        if (!tarCheck.error) {
          const tmpExtract = join(tmpdir(), `skillindex-extract-${Date.now()}`);
          mkdirSync(tmpExtract, { recursive: true });
          const extRes = spawnSync('tar', ['-xf', tmpFile, '-C', tmpExtract], { stdio: 'inherit' });
          if (!extRes.error && extRes.status === 0) {
            const walk = (dir) => {
              for (const e of readdirSync(dir, { withFileTypes: true })) {
                const p = join(dir, e.name);
                if (e.isDirectory()) {
                  const r = walk(p);
                  if (r) return r;
                } else if (e.name === binName || e.name === 'skillindex') {
                  return p;
                }
              }
              return null;
            };
            const found = walk(tmpExtract);
            if (found) {
              copyFileSync(found, destBin);
              extracted = true;
            }
            rmSync(tmpExtract, { recursive: true, force: true });
          }
        } else {
          console.error('  tar no disponible para extraer .tar.xz');
        }
      }
      try {
        rmSync(tmpFile, { force: true });
      } catch {}
      if (extracted && existsSync(destBin)) {
        try {
          if (process.platform !== 'win32') chmodSync(destBin, 0o755);
          mkdirSync(join(__dirname, 'target', 'release'), { recursive: true });
          copyFileSync(destBin, releaseBin);
          if (process.platform !== 'win32') chmodSync(releaseBin, 0o755);
        } catch {}
        console.log('  ✔ Binario descargado y extraído');
        process.exit(0);
      }
    }
  } catch (e) {
    console.error(`  Descarga falló: ${e.message}`);
  }
}

console.error(`
  ✘ Binario Rust no encontrado.

  En desarrollo:  cargo build --release
  Instalación publicada: el binario se distribuye vía GitHub Releases (cargo-dist).
  Si ves esto tras npx/pnpm dlx/bunx skillindex, espera a que el release v${pkgVersion} publique los binarios
  o instala Rust y ejecuta cargo build --release.

  Reporta el problema en https://github.com/Gabox301/SkillIndex/issues
`);
process.exit(1);
