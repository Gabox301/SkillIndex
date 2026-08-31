#!/usr/bin/env node
// Regenerate index.json from local FS with correct LF-normalized hashing.
// Mirrors hash.rs: sha256_buffer, bundle_hash (sorted rel:hash join "\n"), normalize backslashes.
// Also normalizes CRLF -> LF before hashing and rewrites files to LF on disk (to match GitHub raw).
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const __dirname = import.meta.dirname;
const PKG_ROOT = resolve(__dirname, '..');
const REGISTRY_DIR = join(PKG_ROOT, 'skills-registry');
const MANIFEST_PATH = join(REGISTRY_DIR, 'index.json');

function sha256Hex(buf) {
  return createHash('sha256').update(buf).digest('hex');
}

function normalizeLineEndings(buf) {
  // Convert CRLF -> LF, and standalone CR -> LF for safety
  const str = buf.toString('utf8');
  if (!str.includes('\r')) return buf;
  const normalized = str.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  return Buffer.from(normalized, 'utf8');
}

function listFilesRecursive(dir) {
  const out = [];
  (function walk(current) {
    for (const e of readdirSync(current, { withFileTypes: true })) {
      const p = join(current, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.isFile()) {
        const rel = relative(dir, p).split('\\').join('/');
        if (rel.toLowerCase().endsWith('.zip')) continue;
        out.push(p);
      }
    }
  })(dir);
  return out.sort();
}

function main() {
  if (!existsSync(MANIFEST_PATH)) throw new Error('index.json not found');
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, 'utf8'));
  let fixedSkills = 0;
  let fixedFiles = 0;
  let totalFiles = 0;
  for (const [skillName, entry] of Object.entries(manifest.skills)) {
    const skillDir = join(REGISTRY_DIR, skillName);
    if (!existsSync(skillDir) || !statSync(skillDir).isDirectory()) {
      console.warn(`skip ${skillName}: dir missing`);
      continue;
    }
    const files = listFilesRecursive(skillDir);
    const relFiles = files.map((f) => ({
      rel: relative(skillDir, f).split('\\').join('/'),
      abs: f,
      buf: readFileSync(f),
    }));
    totalFiles += relFiles.length;
    // Normalize files on disk to LF and compute hashes from normalized content
    const shaMap = {};
    let changed = false;
    for (const rf of relFiles) {
      const normalized = normalizeLineEndings(rf.buf);
      if (!normalized.equals(rf.buf)) {
        writeFileSync(rf.abs, normalized);
        fixedFiles++;
        changed = true;
      }
      shaMap[rf.rel] = sha256Hex(normalized);
    }
    const bundleHash = sha256Hex(
      Object.entries(shaMap)
        .map(([rel, h]) => `${rel}:${h}`)
        .sort()
        .join('\n'),
    );
    // Check if entry needs update
    const prevFilesSorted = [...(entry.files || [])].sort();
    const newFilesSorted = relFiles.map((r) => r.rel).sort();
    const filesEqual =
      prevFilesSorted.length === newFilesSorted.length && prevFilesSorted.every((v, i) => v === newFilesSorted[i]);
    const shaEqual =
      JSON.stringify(entry.sha256) === JSON.stringify(shaMap) ||
      (() => {
        for (const k of Object.keys(shaMap)) if (entry.sha256[k] !== shaMap[k]) return false;
        for (const k of Object.keys(entry.sha256)) if (shaMap[k] !== entry.sha256[k]) return false;
        return true;
      })();
    if (!filesEqual || !shaEqual || entry.bundleHash !== bundleHash) {
      entry.files = relFiles.map((r) => r.rel);
      entry.sha256 = shaMap;
      entry.bundleHash = bundleHash;
      fixedSkills++;
      if (changed) {
        // debug
      }
    }
  }
  manifest.generatedAt = new Date().toISOString();
  writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n');
  console.log(
    `Regenerated index.json: ${fixedSkills} skills updated, ${fixedFiles} files normalized CRLF->LF, totalFiles ${totalFiles}`,
  );
  console.log(`Total skills: ${Object.keys(manifest.skills).length}`);
}

main();
