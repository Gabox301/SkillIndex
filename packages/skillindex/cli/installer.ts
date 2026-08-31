import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { AGENT_FOLDER_MAP } from '../skills-map.ts';
import { HIDE_CURSOR, SHOW_CURSOR, SPINNER, cyan, dim, green, log, red, write } from './colors.ts';
import { sha256File, sha256Hex } from './helper/hash.ts';
import { relativePosixPath, toPosixPath } from './helper/paths.ts';
import type { SkillEntry } from './lib.ts';
import { parseSkillPath } from './lib.ts';

// ── Registry ─────────────────────────────────────────────────

const DEFAULT_REGISTRY_RAW_BASE_URL_PREFIX = 'https://raw.githubusercontent.com/Gabox301/SkillIndex';
const GITHUB_TOKEN = process.env.GITHUB_TOKEN || process.env.GH_TOKEN || '';

export interface RegistryEntry {
  source: string;
  skillPath: string;
  commitSha: string;
  files: string[];
  sha256: Record<string, string>;
  bundleHash: string;
  review: {
    status: 'approved' | 'flagged';
    flags: string[];
    summary: string;
    model: string;
    promptVersion: string;
    reviewedAt: string;
  };
  securityCheck?: {
    status: 'ok' | 'warning';
    findings: string[];
    summary: string;
    checkedAt: string;
  };
}

export interface Registry {
  version: number;
  generatedAt: string;
  reviewer: { model: string; promptVersion: string };
  skills: Record<string, RegistryEntry>;
}

const __dirname = import.meta.dirname;

let _cachedRegistry: Registry | null | undefined;
let _cachedRegistryDir: string | null = null;
let _cachedPackageVersion: string | null | undefined;

function getPackageVersion(): string | null {
  if (_cachedPackageVersion !== undefined) return _cachedPackageVersion;
  const candidates = [
    join(__dirname, 'package.json'),
    join(__dirname, '..', 'package.json'),
    join(__dirname, '..', '..', 'package.json'),
  ];
  for (const c of candidates) {
    try {
      const pkg = JSON.parse(readFileSync(c, 'utf-8')) as { version?: unknown };
      if (typeof pkg.version === 'string' && pkg.version.length > 0) {
        _cachedPackageVersion = pkg.version;
        return _cachedPackageVersion;
      }
    } catch {}
  }
  _cachedPackageVersion = null;
  return _cachedPackageVersion;
}

export function getRegistryDir(): string {
  if (_cachedRegistryDir) return _cachedRegistryDir;
  const candidates = [
    join(__dirname, 'skills-registry'),
    join(__dirname, '..', 'skills-registry'),
    join(__dirname, '..', '..', 'skills-registry'),
  ];
  for (const c of candidates) {
    if (existsSync(join(c, 'index.json'))) {
      _cachedRegistryDir = c;
      return c;
    }
  }
  _cachedRegistryDir = candidates[0];
  return _cachedRegistryDir;
}

export function loadRegistry(): Registry | null {
  if (_cachedRegistry !== undefined) return _cachedRegistry;
  const manifestPath = join(getRegistryDir(), 'index.json');
  try {
    const body = JSON.parse(readFileSync(manifestPath, 'utf-8')) as Registry;
    _cachedRegistry = body;
    return body;
  } catch {
    _cachedRegistry = null;
    return null;
  }
}

/** @internal — exported for testing only */
export function _setRegistryDir(dir: string | null): void {
  _cachedRegistryDir = dir;
  _cachedRegistry = undefined;
}

// ── Integrity ────────────────────────────────────────────────

export function verifyRegistryEntry(
  skillName: string,
  entry: RegistryEntry,
  registryDir: string = getRegistryDir(),
): { ok: boolean; reason?: string } {
  const skillDir = join(registryDir, skillName);
  if (!existsSync(skillDir)) {
    return { ok: false, reason: `directorio faltante ${skillDir}` };
  }
  for (const rel of entry.files) {
    const normalizedRel = toPosixPath(rel);
    const abs = join(skillDir, ...normalizedRel.split('/'));
    if (!existsSync(abs)) {
      return { ok: false, reason: `archivo faltante ${normalizedRel}` };
    }
    const expected = entry.sha256[rel] || entry.sha256[normalizedRel];
    if (!expected) {
      return { ok: false, reason: `sin hash registrado para ${normalizedRel}` };
    }
    const actual = sha256File(abs);
    if (actual !== expected) {
      return { ok: false, reason: `hash no coincide para ${normalizedRel}` };
    }
  }
  return { ok: true };
}

// ── Install ──────────────────────────────────────────────────

export interface InstallResult {
  success: boolean;
  output: string;
  stderr: string;
  exitCode: number | null;
  command: string;
  securityCheck?: InstallSecurityCheck;
}

export interface InstallSecurityCheck {
  name: string;
  status: 'ok' | 'warning';
  summary: string;
  findings: string[];
}

interface InstallOptions {
  projectDir?: string;
  registryDir?: string;
  registryBaseUrl?: string;
  fetchImpl?: typeof fetch;
  verbose?: boolean;
  onTrace?: (message: string) => void;
}

function getRegistryRawBaseUrls(opts: InstallOptions): string[] {
  const configured = opts.registryBaseUrl || process.env.SKILLINDEX_REGISTRY_BASE_URL;
  if (configured) return [configured.replace(/\/+$/, '')];
  const version = getPackageVersion();
  if (!version) {
    throw new Error('no se pudo resolver la versión del paquete skillindex para la descarga del registro');
  }
  const base = DEFAULT_REGISTRY_RAW_BASE_URL_PREFIX;
  const suffix = 'packages/skillindex/skills-registry';
  // Prefer the exact release tag; fall back to the repository's default
  // branch, which is `master`.
  return [`${base}/v${version}/${suffix}`, `${base}/master/${suffix}`];
}

function getInstallRegistryDir(opts: InstallOptions): string {
  return opts.registryDir || getRegistryDir();
}

export function getSkillIndexCacheDir(): string {
  return process.env.SKILLINDEX_CACHE_DIR || join(homedir(), '.cache', 'skillindex', 'skills-registry');
}

export function clearSkillIndexCache(): { cacheDir: string; removed: boolean } {
  const cacheDir = getSkillIndexCacheDir();
  const removed = existsSync(cacheDir);
  rmSync(cacheDir, { recursive: true, force: true });
  return { cacheDir, removed };
}

function getCacheRegistryDir(entry: RegistryEntry): string {
  return join(getSkillIndexCacheDir(), entry.bundleHash);
}

function securityCheckForEntry(skillName: string, entry: RegistryEntry): InstallSecurityCheck {
  if (entry.securityCheck) {
    return {
      name: skillName,
      status: entry.securityCheck.status,
      summary: entry.securityCheck.summary,
      findings: entry.securityCheck.findings,
    };
  }
  return {
    name: skillName,
    status: entry.review?.status === 'flagged' ? 'warning' : 'ok',
    summary:
      entry.review?.summary ||
      (entry.review?.status === 'flagged'
        ? 'La revisión de sincronización encontró observaciones que deberías revisar.'
        : 'La revisión de sincronización no encontró problemas de seguridad.'),
    findings: entry.review?.flags || [],
  };
}

export function securityCheckForSkillPath(skillPath: string): InstallSecurityCheck | null {
  const { skillName } = parseSkillPath(skillPath);
  if (!skillName) return null;
  const registry = loadRegistry();
  const entry = registry?.skills[skillName];
  if (!entry) return null;
  return securityCheckForEntry(skillName, entry);
}

function encodeRawPath(skillName: string, rel: string): string {
  return [skillName, ...toPosixPath(rel).split('/')].map(encodeURIComponent).join('/');
}

function githubDownloadHeaders(url: string): HeadersInit {
  const headers: Record<string, string> = { 'User-Agent': 'skillindex' };
  const host = new URL(url).hostname;
  if (GITHUB_TOKEN && /(^|\.)githubusercontent\.com$/i.test(host)) {
    headers.Authorization = `Bearer ${GITHUB_TOKEN}`;
  }
  return headers;
}

function isDisallowedSkillFile(rel: string): boolean {
  return rel.toLowerCase().endsWith('.zip');
}

async function downloadRegistryFile(
  skillName: string,
  entry: RegistryEntry,
  rel: string,
  opts: InstallOptions,
): Promise<{ buf: Buffer; url: string }> {
  const normalizedRel = toPosixPath(rel);
  if (isDisallowedSkillFile(normalizedRel)) {
    throw new Error(`se rechazó la descarga del archivo de skill no permitido: ${normalizedRel}`);
  }
  const expected = entry.sha256[rel] || entry.sha256[normalizedRel];
  if (!expected) {
    throw new Error(`sin hash registrado para ${normalizedRel}`);
  }
  const fetchFile = opts.fetchImpl || fetch;
  const errors = [];
  for (const baseUrl of getRegistryRawBaseUrls(opts)) {
    const url = `${baseUrl}/${encodeRawPath(skillName, normalizedRel)}`;
    opts.onTrace?.(`GET ${url}`);
    const res = await fetchFile(url, {
      headers: githubDownloadHeaders(url),
    });
    if (!res.ok) {
      const resetAt = Number(res.headers.get('x-ratelimit-reset') || 0) * 1000;
      const resetSuffix = resetAt ? ` (se restablece ${new Date(resetAt).toISOString()})` : '';
      if (res.status === 403 && res.headers.get('x-ratelimit-remaining') === '0') {
        throw new Error(
          `Límite de tasa de GitHub excedido${resetSuffix}. Configura GITHUB_TOKEN o GH_TOKEN para aumentarlo.`,
        );
      }
      errors.push(`${res.status} ${res.statusText} desde ${baseUrl}`);
      opts.onTrace?.(`no encontrado ${normalizedRel}: ${res.status} ${res.statusText} desde ${baseUrl}`);
      continue;
    }
    const buf = Buffer.from(await res.arrayBuffer());
    const actual = sha256Hex(buf);
    if (actual !== expected) {
      errors.push(`hash no coincide desde ${baseUrl}`);
      opts.onTrace?.(`hash no coincide para ${normalizedRel} desde ${baseUrl}`);
      continue;
    }
    opts.onTrace?.(`descargado ${normalizedRel} desde ${url}`);
    return { buf, url };
  }
  throw new Error(`falló la descarga para ${normalizedRel}: ${errors.join('; ')}`);
}

async function downloadRegistryEntry(
  skillName: string,
  entry: RegistryEntry,
  destDir: string,
  opts: InstallOptions,
): Promise<void> {
  const files = [];
  for (const rel of entry.files) {
    files.push({
      rel: toPosixPath(rel),
      ...(await downloadRegistryFile(skillName, entry, rel, opts)),
    });
  }
  const bundleHash = sha256Hex(
    files
      .map(({ rel, buf }) => `${rel}:${sha256Hex(buf)}`)
      .sort()
      .join('\n'),
  );
  if (bundleHash !== entry.bundleHash) {
    throw new Error('el hash del bundle no coincide');
  }
  rmSync(destDir, { recursive: true, force: true });
  for (const { rel, buf } of files) {
    const dest = join(destDir, ...rel.split('/'));
    mkdirSync(dirname(dest), { recursive: true });
    writeFileSync(dest, buf);
  }
  opts.onTrace?.(`bundle descargado escrito en ${destDir}`);
}

function copyRegistryEntryFromLocal(
  skillName: string,
  entry: RegistryEntry,
  destDir: string,
  opts: InstallOptions,
): boolean {
  const registryDir = getInstallRegistryDir(opts);
  opts.onTrace?.(`verificando registro local: ${join(registryDir, skillName)}`);
  const verdict = verifyRegistryEntry(skillName, entry, registryDir);
  if (!verdict.ok) {
    opts.onTrace?.(`no encontrada en registro local: ${verdict.reason}`);
    return false;
  }
  rmSync(destDir, { recursive: true, force: true });
  copyDir(join(registryDir, skillName), destDir);
  opts.onTrace?.(`copiada desde registro local: ${join(registryDir, skillName)}`);
  return true;
}

function copyRegistryEntryFromCache(
  skillName: string,
  entry: RegistryEntry,
  destDir: string,
  opts: InstallOptions,
): boolean {
  const registryDir = getCacheRegistryDir(entry);
  opts.onTrace?.(`verificando caché de descarga: ${join(registryDir, skillName)}`);
  const verdict = verifyRegistryEntry(skillName, entry, registryDir);
  if (!verdict.ok) {
    opts.onTrace?.(`no encontrada en caché: ${verdict.reason}`);
    return false;
  }
  rmSync(destDir, { recursive: true, force: true });
  copyDir(join(registryDir, skillName), destDir);
  opts.onTrace?.(`copiada desde caché de descarga: ${join(registryDir, skillName)}`);
  return true;
}

async function downloadRegistryEntryToCache(
  skillName: string,
  entry: RegistryEntry,
  opts: InstallOptions,
): Promise<string> {
  const registryDir = getCacheRegistryDir(entry);
  const skillDir = join(registryDir, skillName);
  opts.onTrace?.(`descargando a caché: ${skillDir}`);
  await downloadRegistryEntry(skillName, entry, skillDir, opts);
  return skillDir;
}

function copyDir(src: string, dest: string): void {
  mkdirSync(dest, { recursive: true });
  for (const e of readdirSync(src, { withFileTypes: true })) {
    const s = join(src, e.name);
    const d = join(dest, e.name);
    if (e.isDirectory()) {
      copyDir(s, d);
    } else if (e.isFile()) {
      copyFileSync(s, d);
    }
  }
}

export function agentFolderFor(agent: string): string | null {
  for (const [folder, name] of Object.entries(AGENT_FOLDER_MAP)) {
    if (name === agent) return folder;
  }
  return null;
}

/** Folder used when no mapped agent applies (explicit `universal` or fallback). */
export const UNIVERSAL_SKILLS_FOLDER = '.agents';

export interface InstallTarget {
  /** Agent folder relative to the project root, e.g. `.kiro` or `.agents`. */
  folder: string;
  /** Absolute path to the folder that holds installed skill directories. */
  skillsDir: string;
}

/**
 * Resolve the concrete destination folders where a skill must be installed.
 *
 * Each mapped agent (kiro, claude-code, …) resolves to its own folder, so a
 * skill is copied directly into every agent's `skills` directory. There is no
 * canonical `.agents` copy plus per-agent symlinks anymore — every target is an
 * independent copy, which removes the universal + agent double-path duplication.
 *
 * `.agents` is only used when the destination is `universal`: either chosen
 * explicitly by the user (`-a universal`) or as the fallback when no mapped
 * agent folder resolves.
 */
export function resolveInstallTargets(projectDir: string, agents: string[]): InstallTarget[] {
  const folders = new Set<string>();
  for (const agent of agents) {
    if (agent === 'universal') continue;
    const folder = agentFolderFor(agent);
    if (folder) folders.add(folder);
  }
  // `.agents` is only used when `universal` is the sole destination: either the
  // user chose it explicitly (`-a universal`) or nothing else resolved. When a
  // mapped agent is present, the auto-detector's `universal` entry is ignored so
  // we never recreate the universal + agent double path.
  if (folders.size === 0) {
    folders.add(UNIVERSAL_SKILLS_FOLDER);
  }
  return [...folders].map((folder) => ({
    folder,
    skillsDir: join(projectDir, folder, 'skills'),
  }));
}

/**
 * Materialize a verified skill bundle into `destDir`, trying local registry,
 * download cache, and finally a fresh download to the cache. The bundle is
 * hash-verified end to end, so every destination gets byte-identical content.
 */
async function materializeSkillInto(
  skillName: string,
  entry: RegistryEntry,
  destDir: string,
  opts: InstallOptions,
): Promise<void> {
  if (
    !copyRegistryEntryFromLocal(skillName, entry, destDir, opts) &&
    !copyRegistryEntryFromCache(skillName, entry, destDir, opts)
  ) {
    const cachedSkillDir = await downloadRegistryEntryToCache(skillName, entry, opts);
    rmSync(destDir, { recursive: true, force: true });
    copyDir(cachedSkillDir, destDir);
    opts.onTrace?.(`bundle descargado copiado en ${destDir}`);
  }
}

function updateSkillsLock(projectDir: string, skillName: string, entry: RegistryEntry): void {
  const lockPath = join(projectDir, 'skills-lock.json');
  let lock: { version: number; skills: Record<string, unknown> };
  try {
    lock = JSON.parse(readFileSync(lockPath, 'utf-8'));
    if (!lock || typeof lock !== 'object' || !lock.skills) {
      lock = { version: 1, skills: {} };
    }
  } catch {
    lock = { version: 1, skills: {} };
  }
  lock.skills[skillName] = {
    source: entry.source,
    sourceType: 'skillindex-registry',
    computedHash: entry.bundleHash,
  };
  const sortedSkills: Record<string, unknown> = {};
  for (const k of Object.keys(lock.skills).sort()) {
    sortedSkills[k] = lock.skills[k];
  }
  lock.skills = sortedSkills;
  writeFileSync(lockPath, JSON.stringify(lock, null, 2) + '\n');
}

export async function installSkill(
  skillPath: string,
  agents: string[] = [],
  opts: InstallOptions = {},
): Promise<InstallResult> {
  const projectDir = opts.projectDir || process.cwd();
  const command = `skillindex install ${skillPath}`;
  const fail = (msg: string): InstallResult => ({
    success: false,
    output: msg,
    stderr: msg,
    exitCode: 1,
    command,
  });
  const { skillName } = parseSkillPath(skillPath);
  if (!skillName) return fail(`ruta de skill no válida: ${skillPath}`);
  opts.onTrace?.(`resolviendo ${skillPath}`);
  const registry = loadRegistry();
  if (!registry) {
    return fail(`índice de skills-registry no encontrado. Ejecuta 'pnpm sync:skills' en el paquete skillindex.`);
  }
  const entry = registry.skills[skillName];
  if (!entry) {
    return fail(`skill '${skillName}' no encontrada en el registro (no auditada).`);
  }
  const securityCheck = securityCheckForEntry(skillName, entry);
  opts.onTrace?.(`origen del registro: ${entry.source} @ ${entry.commitSha}`);
  const targets = resolveInstallTargets(projectDir, agents);
  const installErrors: string[] = [];
  for (const target of targets) {
    const destDir = join(target.skillsDir, skillName);
    try {
      // The skill is "already installed" for this target only when it exists
      // and matches the registry hash. If it is missing or drifted, we (re)install
      // just this target — the bundle is hash-verified so all copies stay identical.
      const verdict = verifyRegistryEntry(skillName, entry, target.skillsDir);
      if (verdict.ok) {
        opts.onTrace?.(`ya instalada y verificada: ${destDir}`);
        continue;
      }
      opts.onTrace?.(`la copia en ${target.folder} necesita actualizarse: ${verdict.reason}`);
      await materializeSkillInto(skillName, entry, destDir, opts);
      opts.onTrace?.(`instalada en ${destDir}`);
    } catch (err) {
      installErrors.push(`${target.folder}: ${(err as Error).message}`);
    }
  }
  if (installErrors.length > 0) {
    const msg = `falló la instalación: ${installErrors.join('; ')}`;
    return { success: false, output: msg, stderr: msg, exitCode: 1, command };
  }
  try {
    updateSkillsLock(projectDir, skillName, entry);
    opts.onTrace?.(`lockfile actualizado: ${join(projectDir, 'skills-lock.json')}`);
  } catch (err) {
    return fail(`falló la actualización del lockfile: ${(err as Error).message}`);
  }
  const destinations = targets.map((t) => relativePosixPath(projectDir, join(t.skillsDir, skillName)));
  return {
    success: true,
    output: `instalada ${skillName} en ${destinations.join(', ')}`,
    stderr: '',
    exitCode: 0,
    command,
    securityCheck,
  };
}

// ── Batch install (concurrent + spinner) ─────────────────────

function sortByRepo(skills: SkillEntry[]): SkillEntry[] {
  return [...skills].sort((a, b) => {
    const repoA = parseSkillPath(a.skill).repo;
    const repoB = parseSkillPath(b.skill).repo;
    return repoA.localeCompare(repoB);
  });
}

interface InstallAllResult {
  installed: number;
  failed: number;
  securityChecks: InstallSecurityCheck[];
  errors: {
    name: string;
    output: string;
    stderr: string;
    exitCode: number | null;
    command: string;
  }[];
}

export async function installAll(
  skills: SkillEntry[],
  agents: string[] = [],
  opts: InstallOptions = {},
): Promise<InstallAllResult> {
  if (opts.verbose) return installAllVerbose(skills, agents, opts);
  if (!process.stdout.isTTY) return installAllSimple(skills, agents, opts);
  const CONCURRENCY = 6;
  const sorted = sortByRepo(skills);
  const total = sorted.length;
  const states = sorted.map(({ skill }) => ({
    name: skill,
    skill,
    status: 'pending' as 'pending' | 'installing' | 'success' | 'failed',
    output: '',
  }));
  let frame = 0;
  let rendered = false;
  let activeCount = 0;
  function render(): void {
    if (rendered) {
      write(`\x1b[${total}A\r`);
    }
    rendered = true;
    write('\x1b[J');
    for (const state of states) {
      switch (state.status) {
        case 'pending':
          write(dim(`   ◌ ${state.name}`) + '\n');
          break;
        case 'installing':
          write(cyan(`   ${SPINNER[frame]}`) + ` ${state.name}...\n`);
          break;
        case 'success':
          write(green(`   ✔ ${state.name}`) + '\n');
          break;
        case 'failed':
          write(red(`   ✘ ${state.name}`) + dim(' — failed') + '\n');
          break;
      }
    }
  }
  write(HIDE_CURSOR);
  const timer = setInterval(() => {
    frame = (frame + 1) % SPINNER.length;
    if (activeCount > 0) render();
  }, 80);
  let installed = 0;
  let failed = 0;
  const errors: InstallAllResult['errors'] = [];
  const securityChecks: InstallSecurityCheck[] = [];
  let nextIdx = 0;
  async function worker(): Promise<void> {
    while (nextIdx < total) {
      const idx = nextIdx++;
      const state = states[idx];
      state.status = 'installing';
      activeCount++;
      render();
      const result = await installSkill(state.skill, agents, opts);
      activeCount--;
      if (result.success) {
        state.status = 'success';
        installed++;
        if (result.securityCheck) securityChecks.push(result.securityCheck);
      } else {
        state.status = 'failed';
        state.output = result.output;
        errors.push({
          name: state.name,
          output: result.output,
          stderr: result.stderr,
          exitCode: result.exitCode,
          command: result.command,
        });
        failed++;
      }
      render();
    }
  }
  const workers = Array.from({ length: Math.min(CONCURRENCY, total) }, () => worker());
  await Promise.all(workers);
  clearInterval(timer);
  render();
  write(SHOW_CURSOR);
  return { installed, failed, errors, securityChecks };
}

async function installAllVerbose(
  skills: SkillEntry[],
  agents: string[] = [],
  opts: InstallOptions = {},
): Promise<InstallAllResult> {
  const sorted = sortByRepo(skills);
  let installed = 0;
  let failed = 0;
  const errors: InstallAllResult['errors'] = [];
  const securityChecks: InstallSecurityCheck[] = [];
  for (const { skill } of sorted) {
    log(cyan(`   ◆ ${skill}`));
    const result = await installSkill(skill, agents, {
      ...opts,
      onTrace: (message) => log(dim(`     ${message}`)),
    });
    if (result.success) {
      log(green(`     ✔ installed`));
      installed++;
      if (result.securityCheck) securityChecks.push(result.securityCheck);
    } else {
      log(red(`     ✘ failed`) + dim(` — ${result.output}`));
      errors.push({
        name: skill,
        output: result.output,
        stderr: result.stderr,
        exitCode: result.exitCode,
        command: result.command,
      });
      failed++;
    }
    log();
  }
  return { installed, failed, errors, securityChecks };
}

async function installAllSimple(
  skills: SkillEntry[],
  agents: string[] = [],
  opts: InstallOptions = {},
): Promise<InstallAllResult> {
  const CONCURRENCY = 6;
  const sorted = sortByRepo(skills);
  let installed = 0;
  let failed = 0;
  const errors: InstallAllResult['errors'] = [];
  const securityChecks: InstallSecurityCheck[] = [];
  let nextIdx = 0;
  async function worker(): Promise<void> {
    while (nextIdx < sorted.length) {
      const idx = nextIdx++;
      const { skill } = sorted[idx];
      const result = await installSkill(skill, agents, opts);
      if (result.success) {
        log(green(`   ✔ ${skill}`));
        installed++;
        if (result.securityCheck) securityChecks.push(result.securityCheck);
      } else {
        log(red(`   ✘ ${skill}`) + dim(' — failed'));
        errors.push({
          name: skill,
          output: result.output,
          stderr: result.stderr,
          exitCode: result.exitCode,
          command: result.command,
        });
        failed++;
      }
    }
  }
  const workers = Array.from({ length: Math.min(CONCURRENCY, sorted.length) }, () => worker());
  await Promise.all(workers);
  return { installed, failed, errors, securityChecks };
}

// ── Deprecated shim ──────────────────────────────────────────

/** @deprecated retained so that UI code keeps compiling; no longer used. */
export function resolveSkillsBin(): string | null {
  return null;
}
