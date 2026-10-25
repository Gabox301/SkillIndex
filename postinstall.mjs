#!/usr/bin/env node
// Postinstall: ensures the Rust binary is available for npx/pnpm/bunx.
//
// Supply-chain gate: the release archive is downloaded into a private temp
// directory and verified against the SHA-256 published next to it
// (<asset>.sha256, emitted by cargo-dist and guaranteed by the release
// workflow). Extraction, chmod and copy only happen after the hash matches.
// A missing or malformed checksum fails closed: nothing is extracted, marked
// executable or copied.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  copyFileSync,
  createReadStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
} from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { basename, join } from 'node:path';

const require = createRequire(import.meta.url);
let pkgVersion = '3.1.0';
try {
  const pkg = require('./package.json');
  if (pkg.version) pkgVersion = pkg.version;
} catch {}

const __dirname = import.meta.dirname;
const MAX_CHECKSUM_BYTES = 4096;

// 1) Local binary from a previous install or a dev build: no network needed.
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

// 2) Dev fallback: cargo build when cargo and Cargo.toml are available.
// This path is fully offline and keeps local development working.
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

// 3) Download the prebuilt binary from GitHub Releases (cargo-dist).
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
  // -f makes curl exit non-zero on HTTP errors instead of writing an error
  // page to dest, so a 404 can never be mistaken for a downloaded asset.
  let res = spawnSync('curl', ['-f', '-L', '-o', dest, url], { stdio: 'inherit' });
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

async function sha256File(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

// Accept a bare digest or the sha256sum/cargo-dist format
// "<hex>  <filename>". Anything else is rejected.
function parseChecksum(raw) {
  const match = raw.trim().match(/^([a-fA-F0-9]{64})(?:\s|$)/);
  return match ? match[1].toLowerCase() : null;
}

async function verifyArchiveChecksum(archivePath, checksumPath) {
  if (!existsSync(checksumPath)) throw new Error('no se descargó el archivo .sha256');
  if (statSync(checksumPath).size > MAX_CHECKSUM_BYTES) {
    throw new Error('el archivo .sha256 tiene un tamaño inesperado');
  }
  const expected = parseChecksum(readFileSync(checksumPath, 'utf8'));
  if (!expected) throw new Error('el archivo .sha256 no contiene un hash SHA-256 válido');
  const actual = await sha256File(archivePath);
  if (actual !== expected) throw new Error(`hash esperado ${expected}, obtenido ${actual}`);
  return expected;
}

function extractArchive(archivePath, ext, destDir, destBin, workDir) {
  const binName = basename(destBin);
  if (ext === 'zip') {
    const ps = spawnSync(
      'powershell',
      [
        '-Command',
        `Expand-Archive -Path "${archivePath}" -DestinationPath "${destDir}" -Force; Get-ChildItem -Recurse "${destDir}" | Where-Object { $_.Name -eq "${binName}" } | ForEach-Object { Move-Item $_.FullName "${destBin}" -Force }`,
      ],
      { stdio: 'inherit' },
    );
    return !ps.error && ps.status === 0 && existsSync(destBin);
  }

  const tarCheck = spawnSync('tar', ['--version'], { stdio: 'ignore' });
  if (tarCheck.error) {
    console.error('  tar no disponible para extraer .tar.xz');
    return false;
  }
  const extractDir = join(workDir, 'extract');
  mkdirSync(extractDir, { recursive: true });
  const extRes = spawnSync('tar', ['-xf', archivePath, '-C', extractDir], { stdio: 'inherit' });
  if (extRes.error || extRes.status !== 0) return false;

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
  const found = walk(extractDir);
  if (!found) return false;
  copyFileSync(found, destBin);
  return true;
}

async function installFromRelease() {
  const targetInfo = getTarget();
  if (!targetInfo) return 'unavailable';

  const { target, ext } = targetInfo;
  const fileName = `skillindex-${target}.${ext}`;
  const assetUrl = `https://github.com/Gabox301/SkillIndex/releases/download/v${pkgVersion}/${fileName}`;
  const checksumUrl = `${assetUrl}.sha256`;
  const destDir = join(__dirname, 'bin');
  const binName = process.platform === 'win32' ? 'skillindex.exe' : 'skillindex';
  const destBin = join(destDir, binName);
  const releaseBin = join(__dirname, 'target', 'release', binName);
  // The offline fallbacks above already returned when a binary exists, so
  // these files should not exist yet. Remember their state so a failed
  // verification never deletes a binary this script did not create.
  const hadDestBin = existsSync(destBin);
  const hadReleaseBin = existsSync(releaseBin);

  const workDir = mkdtempSync(join(tmpdir(), 'skillindex-dl-'));
  const archivePath = join(workDir, fileName);
  const checksumPath = join(workDir, `${fileName}.sha256`);
  let checksumError = null;
  let installed = false;

  try {
    console.log(`  Descargando binario precompilado desde ${assetUrl} ...`);
    if (!downloadWithCurl(assetUrl, archivePath)) {
      console.error('  Descarga falló: no se pudo descargar el binario');
      return 'unavailable';
    }
    if (!downloadWithCurl(checksumUrl, checksumPath)) {
      checksumError = `no se pudo descargar el checksum (${checksumUrl})`;
      return 'checksum-failed';
    }
    try {
      await verifyArchiveChecksum(archivePath, checksumPath);
    } catch (e) {
      checksumError = e.message;
      return 'checksum-failed';
    }

    // Verified: only now is it safe to extract, chmod and copy the binary.
    mkdirSync(destDir, { recursive: true });
    if (extractArchive(archivePath, ext, destDir, destBin, workDir) && existsSync(destBin)) {
      try {
        if (process.platform !== 'win32') chmodSync(destBin, 0o755);
        mkdirSync(join(__dirname, 'target', 'release'), { recursive: true });
        copyFileSync(destBin, releaseBin);
        if (process.platform !== 'win32') chmodSync(releaseBin, 0o755);
      } catch {}
      console.log('  ✔ Binario descargado y verificado');
      installed = true;
      return 'installed';
    }
    return 'unavailable';
  } catch (e) {
    console.error(`  Descarga falló: ${e.message}`);
    return 'unavailable';
  } finally {
    rmSync(workDir, { recursive: true, force: true });
    if (checksumError) {
      console.error(`  ✘ Verificación SHA-256 falló: ${checksumError}`);
      console.error('  El binario no se extrajo, no se marcó como ejecutable y no se copió.');
    }
    // Never leave partial artifacts behind when the install did not finish.
    if (!installed) {
      if (!hadDestBin) rmSync(destBin, { force: true });
      if (!hadReleaseBin) rmSync(releaseBin, { force: true });
    }
  }
}

const installResult = await installFromRelease();
if (installResult === 'installed') process.exit(0);
if (installResult === 'checksum-failed') process.exit(1);

console.error(`
  ✘ Binario Rust no encontrado.

  En desarrollo:  cargo build --release
  Instalación publicada: el binario se distribuye vía GitHub Releases (cargo-dist).
  Si ves esto tras npx/pnpm dlx/bunx skillindex, espera a que el release v${pkgVersion} publique los binarios
  o instala Rust y ejecuta cargo build --release.

  Reporta el problema en https://github.com/Gabox301/SkillIndex/issues
`);
process.exit(1);
