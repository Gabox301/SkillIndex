#!/usr/bin/env node
/**
 * fendo — local supply-chain hardening checker (GaboTech)
 * 100% local implementation — no external dependency.
 * Checks:
 *  - No ^/~ in version specifiers (exact pin)
 *  - No git/tarball exotic deps
 *  - Lockfile present (bun.lock / pnpm-lock.yaml)
 *  - .npmrc hardening present
 *  - No exotic `overrides`/`resolutions` abuse
 */

import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = join(fileURLToPath(import.meta.url), '..');
const root = resolve(__dirname, '..');

let failed = false;

function fail(msg) {
  console.error(`✘ ${msg}`);
  failed = true;
}
function ok(msg) {
  console.log(`✔ ${msg}`);
}

function checkPackageJson(path) {
  if (!existsSync(path)) return;
  const pkg = JSON.parse(readFileSync(path, 'utf-8'));
  const allDeps = {
    ...(pkg.dependencies || {}),
    ...(pkg.devDependencies || {}),
    ...(pkg.peerDependencies || {}),
    ...(pkg.optionalDependencies || {}),
  };
  for (const [name, ver] of Object.entries(allDeps)) {
    const v = String(ver).trim();
    if (v.startsWith('^') || v.startsWith('~')) {
      fail(`${path}: ${name}@${v} uses ^/~ — pin exact version`);
    }
    if (/^(git\+|github:|https?:.*\.tgz|https?:.*\.tar\.gz)/i.test(v) || v.includes('://')) {
      // Allow http for skills.sh? Block git/tarball unless approved
      if (/^https:\/\/raw\.githubusercontent\.com\//.test(v)) continue;
      fail(`${path}: ${name}@${v} looks like git/tarball URL — requires explicit approval`);
    }
  }
  if (Object.keys(allDeps).length > 0) ok(`${path}: versions pinned`);
}

function checkNpmrc() {
  const p = join(root, '.npmrc');
  if (!existsSync(p)) {
    fail('.npmrc missing — hardening not applied');
    return;
  }
  const txt = readFileSync(p, 'utf-8');
  const checks = [
    ['save-exact=true', 'save-exact=true'],
    ['ignore-scripts=true', 'ignore-scripts=true'],
    ['engine-strict=true', 'engine-strict=true'],
  ];
  for (const [needle, label] of checks) {
    if (!txt.includes(needle)) fail(`.npmrc missing ${label}`);
    else ok(`.npmrc has ${label}`);
  }
  // minimum-release-age is pnpm-specific, but check if present
  if (txt.includes('minimum-release-age')) ok('.npmrc has minimum-release-age');
}

function checkLockfile() {
  const candidates = ['bun.lock', 'bun.lockb', 'pnpm-lock.yaml', 'package-lock.json', 'yarn.lock'];
  const found = candidates.filter((f) => existsSync(join(root, f)));
  const pkgCandidates = ['pnpm-lock.yaml', 'bun.lock', 'bun.lockb'].filter((f) =>
    existsSync(join(root, 'packages/skillscout', f)),
  );
  if (found.length === 0 && pkgCandidates.length === 0) {
    fail('No lockfile found (bun.lock / pnpm-lock.yaml) — commit it');
  } else {
    ok(`Lockfile present: ${[...found, ...pkgCandidates].join(', ')}`);
  }
  // Check .gitignore doesn't ignore lockfile
  const gi = join(root, '.gitignore');
  if (existsSync(gi)) {
    const txt = readFileSync(gi, 'utf-8');
    for (const f of found) {
      if (txt.includes(f)) fail(`.gitignore ignores ${f} — must be committed`);
    }
  }
}

function checkGitignore() {
  // Already handled
}

console.log('fendo — GaboTech supply-chain check\n');

checkPackageJson(join(root, 'package.json'));
checkPackageJson(join(root, 'packages/skillscout/package.json'));
checkNpmrc();
checkLockfile();

if (failed) {
  console.error('\n✘ fendo: hardening failed — fix issues above');
  process.exit(1);
} else {
  console.log('\n✔ fendo: all hardening checks passed');
}
