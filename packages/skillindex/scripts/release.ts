#!/usr/bin/env node

import type { ExecSyncOptions } from 'node:child_process';
import { execSync } from 'node:child_process';
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname: string = dirname(fileURLToPath(import.meta.url));
const ROOT: string = resolve(__dirname, '..');
const REPO_ROOT: string = resolve(ROOT, '..', '..');
const PKG_PATH: string = resolve(ROOT, 'package.json');
const CHANGELOG_PATH: string = resolve(ROOT, 'CHANGELOG.md');
const VALID_BUMPS = ['patch', 'minor', 'major'] as const;

type BumpType = (typeof VALID_BUMPS)[number];

/**
 * Normalizes repository URLs to a browser-safe HTTPS GitHub URL.
 * Supports git+https, SSH (git@github.com:owner/repo), and ssh://git@ URLs.
 * Falls back to the project default when input is missing or invalid.
 */
function normalizeRepoUrl(rawUrl?: string): string {
  const fallback = 'https://github.com/Gabox301/SkillIndex';
  if (!rawUrl || typeof rawUrl !== 'string') return fallback;
  let url: string = rawUrl.trim();
  // Convert git+https://... -> https://...
  if (url.startsWith('git+https://')) {
    url = url.replace(/^git\+/, '');
  }
  // Convert git@github.com:owner/repo(.git)? -> https://github.com/owner/repo
  if (url.startsWith('git@github.com:')) {
    url = url.replace(/^git@github\.com:/, 'https://github.com/');
  }
  // Convert ssh://git@github.com/owner/repo(.git)? -> https://github.com/owner/repo
  if (url.startsWith('ssh://git@github.com/')) {
    url = url.replace(/^ssh:\/\/git@github\.com\//, 'https://github.com/');
  }
  // Remove trailing .git and trailing slash
  url = url.replace(/\.git$/, '').replace(/\/+$/, '');
  // Allow only http/https browser-safe URLs
  if (!/^https?:\/\//.test(url)) {
    return fallback;
  }
  return url;
}

/**
 * Executes a shell command synchronously and returns its trimmed stdout.
 */
function run(cmd: string, opts: ExecSyncOptions = {}): string {
  return (execSync(cmd, { encoding: 'utf-8', cwd: ROOT, stdio: 'pipe', ...opts } as ExecSyncOptions) as string).trim();
}

/**
 * Executes a shell command with inherited stdio so output is visible to the user.
 */
function runVisible(cmd: string, opts: ExecSyncOptions = {}): void {
  execSync(cmd, { cwd: ROOT, stdio: 'inherit', ...opts });
}

/**
 * Prints an error message and exits the process with code 1.
 */
function fail(msg: string): never {
  console.error(`\n❌ ${msg}`);
  process.exit(1);
}

/**
 * Increments a semver version string by the given bump type.
 */
function bumpVersion(version: string, type: BumpType): string {
  const [major, minor, patch] = version.split('.').map(Number) as [number, number, number];
  if (type === 'major') return `${major + 1}.0.0`;
  if (type === 'minor') return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

/**
 * Returns the most recent git tag reachable from HEAD, or `null` if none exists.
 */
function getLastTag(): string | null {
  try {
    return run('git describe --tags --abbrev=0', { cwd: REPO_ROOT });
  } catch {
    return null;
  }
}

interface CommitInfo {
  message: string;
  hash: string;
}

/**
 * Collects commits affecting `packages/skillindex/` since the given tag.
 */
function getCommitsSinceTag(tag: string | null): CommitInfo[] {
  const range = tag ? `${tag}..HEAD` : 'HEAD';
  const log = run(`git log ${range} --pretty=format:"%s|%h" -- packages/skillindex/`, {
    cwd: REPO_ROOT,
  });
  if (!log) return [];
  return log
    .split('\n')
    .filter(Boolean)
    .map((line) => {
      const [message, hash] = line.split('|');
      return { message: message ?? '', hash: hash ?? '' };
    });
}

interface CategorizedCommits {
  breaking: CommitInfo[];
  feat: CommitInfo[];
  fix: CommitInfo[];
  other: CommitInfo[];
}

/**
 * Sorts commits into changelog categories: breaking, feat, fix, and other.
 * Uses prefix/keyword heuristics on the commit message.
 */
function categorizeCommits(commits: CommitInfo[]): CategorizedCommits {
  const categories: CategorizedCommits = {
    breaking: [],
    feat: [],
    fix: [],
    other: [],
  };
  for (const { message, hash } of commits) {
    const lower = message.toLowerCase();
    if (lower.startsWith('feat') || lower.includes('add ') || lower.includes('add:')) {
      categories.feat.push({ message, hash });
    } else if (lower.startsWith('fix') || lower.includes('fix ') || lower.includes('fix:')) {
      categories.fix.push({ message, hash });
    } else if (lower.includes('breaking') || lower.includes('!:')) {
      categories.breaking.push({ message, hash });
    } else {
      categories.other.push({ message, hash });
    }
  }
  return categories;
}

/**
 * Generates a Markdown changelog section for a release.
 */
function buildChangelog(version: string, categories: CategorizedCommits, repoUrl: string): string {
  const date: string = new Date().toISOString().split('T')[0] as string;
  let md = `## [${version}](${repoUrl}/releases/tag/v${version}) (${date})\n\n`;
  if (categories.breaking.length) {
    md += `### ⚠️ Breaking Changes\n\n`;
    for (const { message, hash } of categories.breaking) {
      md += `- ${message} [\`${hash}\`](${repoUrl}/commit/${hash})\n`;
    }
    md += '\n';
  }
  if (categories.feat.length) {
    md += `### ✨ Features\n\n`;
    for (const { message, hash } of categories.feat) {
      md += `- ${message} [\`${hash}\`](${repoUrl}/commit/${hash})\n`;
    }
    md += '\n';
  }
  if (categories.fix.length) {
    md += `### 🐛 Bug Fixes\n\n`;
    for (const { message, hash } of categories.fix) {
      md += `- ${message} [\`${hash}\`](${repoUrl}/commit/${hash})\n`;
    }
    md += '\n';
  }
  if (categories.other.length) {
    md += `### 📦 Other Changes\n\n`;
    for (const { message, hash } of categories.other) {
      md += `- ${message} [\`${hash}\`](${repoUrl}/commit/${hash})\n`;
    }
    md += '\n';
  }
  return md;
}

/**
 * Prepends a new release entry to CHANGELOG.md.
 * Creates the file with a default header if it doesn't exist.
 */
function updateChangelog(newEntry: string): void {
  if (existsSync(CHANGELOG_PATH)) {
    const existing: string = readFileSync(CHANGELOG_PATH, 'utf-8');
    const headerEnd: number = existing.indexOf('\n## ');
    if (headerEnd !== -1) {
      const header: string = existing.slice(0, headerEnd + 1);
      const rest: string = existing.slice(headerEnd + 1);
      writeFileSync(CHANGELOG_PATH, `${header}${newEntry}${rest}`);
    } else {
      const lines: string[] = existing.split('\n');
      const headerLines: string = lines.slice(0, 2).join('\n') + '\n\n';
      writeFileSync(CHANGELOG_PATH, `${headerLines}${newEntry}`);
    }
  } else {
    writeFileSync(CHANGELOG_PATH, `# Changelog\n\n${newEntry}`);
  }
}

// --- Main ---

const bump = process.argv[2] as BumpType | undefined;

if (!bump || !(VALID_BUMPS as readonly string[]).includes(bump)) {
  fail(`Uso: bun scripts/release.ts <${VALID_BUMPS.join('|')}>`);
}

interface PackageJson {
  name: string;
  version: string;
  repository?: string | { url?: string };
  [key: string]: unknown;
}

const pkg = JSON.parse(readFileSync(PKG_PATH, 'utf-8')) as PackageJson;
const repoUrl: string = normalizeRepoUrl(typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url);
const currentVersion: string = pkg.version;
const newVersion: string = bumpVersion(currentVersion, bump as BumpType);
const originalPkgContent: string = readFileSync(PKG_PATH, 'utf-8');
const originalChangelogContent: string | null = existsSync(CHANGELOG_PATH)
  ? readFileSync(CHANGELOG_PATH, 'utf-8')
  : null;
const releaseStartHead: string = run('git rev-parse HEAD', { cwd: REPO_ROOT });

let commitCreated = false;
let tagCreated = false;

/**
 * Restores local repository/files to the state before the release attempt.
 * This keeps failed releases from leaving bumped versions, tags or commits behind.
 */
function rollbackRelease(): void {
  console.log('\n↩️  Revirtiendo cambios locales de la release fallida...');
  if (tagCreated) {
    try {
      run(`git tag -d v${newVersion}`, { cwd: REPO_ROOT });
      console.log(`✅ Tag v${newVersion} eliminado`);
    } catch {
      console.warn(`⚠️  No se pudo eliminar el tag v${newVersion}`);
    }
  }
  if (commitCreated) {
    try {
      run(`git reset --hard ${releaseStartHead}`, { cwd: REPO_ROOT });
      console.log('✅ Commit de release revertido');
      return;
    } catch {
      console.warn('⚠️  No se pudo revertir el commit de release automáticamente');
    }
  }
  // If no release commit was created, restore touched files directly.
  try {
    writeFileSync(PKG_PATH, originalPkgContent);
    if (originalChangelogContent === null) {
      if (existsSync(CHANGELOG_PATH)) {
        rmSync(CHANGELOG_PATH);
      }
    } else {
      writeFileSync(CHANGELOG_PATH, originalChangelogContent);
    }
    run('git restore --staged package.json CHANGELOG.md', { cwd: ROOT });
    console.log('✅ package.json y CHANGELOG.md restaurados');
  } catch {
    console.warn('⚠️  No se pudieron restaurar todos los archivos automáticamente');
  }
}

console.log(`\n📦 ${pkg.name} ${currentVersion} → ${newVersion} (${bump})\n`);

// 1. Ensure release is run only on main and from a fully clean working tree.
const currentBranch: string = run('git branch --show-current', { cwd: REPO_ROOT });
if (currentBranch !== 'main') {
  fail(`La release solo se puede ejecutar en main. Rama actual: ${currentBranch}`);
}

const status: string = run('git status --porcelain -- .', { cwd: REPO_ROOT });
const dirtyFiles: string[] = status.split('\n').filter((f) => f.trim());
if (dirtyFiles.length) {
  fail(`Hay cambios sin commitear:\n${dirtyFiles.join('\n')}`);
}

// 2. Ensure every advertised skill is present and installable from the registry.
console.log('🧭 Validando skills registry...');
try {
  runVisible('bun scripts/validate-registry.ts');
} catch {
  fail('El registry no coincide con skills-map. Ejecuta sync:skills y corrige el registry antes de publicar.');
}

// 3. Run tests
console.log('🧪 Ejecutando tests...');
try {
  runVisible('bun --test tests/*.test.ts');
} catch {
  fail('Los tests han fallado. Arregla los errores antes de publicar.');
}

// 4. Generate changelog
console.log('\n📝 Generando changelog...');
const lastTag: string | null = getLastTag();
const commits: CommitInfo[] = getCommitsSinceTag(lastTag);

if (commits.length === 0) {
  fail('No hay commits nuevos desde el último tag.');
}

const categories: CategorizedCommits = categorizeCommits(commits);
const changelogEntry: string = buildChangelog(newVersion, categories, repoUrl);

console.log(changelogEntry);

try {
  // 5. Update version in package.json
  pkg.version = newVersion;
  writeFileSync(PKG_PATH, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`✅ package.json actualizado a ${newVersion}`);
  // 6. Update CHANGELOG.md
  updateChangelog(changelogEntry);
  console.log('✅ CHANGELOG.md actualizado');
  // 7. Git commit + tag
  console.log('\n🔖 Creando commit y tag...');
  run(`git add package.json CHANGELOG.md`, { cwd: ROOT });
  run(`git commit -m "release: v${newVersion}"`, { cwd: REPO_ROOT });
  commitCreated = true;
  run(`git tag -a v${newVersion} -m "v${newVersion}"`, { cwd: REPO_ROOT });
  tagCreated = true;
  // 8. Build TypeScript
  console.log('\n🔨 Compilando TypeScript...');
  run('rm -rf dist');
  runVisible('npx tsc');
  // 9. Publish to npm
  console.log('\n🚀 Publicando en npm...');
  const cleanEnv: Record<string, string | undefined> = Object.fromEntries(
    Object.entries(process.env).filter(
      ([k]) => !k.startsWith('npm_config_') || k === 'npm_config_registry' || k.startsWith('npm_config_//'),
    ),
  ) as Record<string, string | undefined>;
  execSync('npm publish --access public', {
    cwd: ROOT,
    stdio: 'inherit',
    env: cleanEnv,
  });
  // 10. Push to GitHub
  console.log('\n📤 Pusheando a GitHub...');
  run('git push', { cwd: REPO_ROOT });
  run('git push --tags', { cwd: REPO_ROOT });
} catch (error) {
  rollbackRelease();
  const message = error instanceof Error ? error.message : String(error);
  fail(`La release falló y se revirtieron los cambios locales.\n${message}`);
}

// 11. Create GitHub release
console.log('\n🏷️  Creando GitHub Release...');
const releaseNotes: string = changelogEntry.replace(/^## .*\n\n/, '');
const tempFile: string = resolve(ROOT, '.release-notes-tmp.md');
writeFileSync(tempFile, releaseNotes);

try {
  run(`gh release create v${newVersion} --title "v${newVersion}" --notes-file .release-notes-tmp.md`, { cwd: ROOT });
  console.log(`✅ Release v${newVersion} creada en GitHub`);
} catch {
  console.warn(`⚠️  No se pudo crear la release en GitHub (¿tienes gh instalado y autenticado?)`);
  console.warn(`   Puedes crearla manualmente: ${repoUrl}/releases/new?tag=v${newVersion}`);
} finally {
  try {
    run(`rm .release-notes-tmp.md`);
  } catch {}
}

console.log(`\n🎉 ¡Release v${newVersion} completada!\n`);
