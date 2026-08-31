#!/usr/bin/env node

import { existsSync, readFileSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { sha256Hex } from '../cli/helper/hash.ts';
import { normalizeLineEndings } from '../cli/helper/line-endings.ts';
import { parseSkillPath } from '../cli/lib.ts';
import { COMBO_SKILLS_MAP, FRONTEND_BONUS_SKILLS, SKILLS_MAP } from '../skills-map.ts';

const __dirname: string = import.meta.dirname;
const PKG_ROOT: string = resolve(__dirname, '..');
const REGISTRY_DIR: string = join(PKG_ROOT, 'skills-registry');
const MANIFEST_PATH: string = join(REGISTRY_DIR, 'index.json');

interface DeclaredSkillInfo {
  full: string;
  sources: Set<string>;
}

interface RegistryEntry {
  files?: string[];
  sha256?: Record<string, string>;
  bundleHash?: string;
  skillPath?: string;
  source?: string;
  commitSha?: string;
  [key: string]: unknown;
}

interface Manifest {
  skills: Record<string, RegistryEntry>;
  [key: string]: unknown;
}

function collectDeclaredSkills(): { declared: Map<string, DeclaredSkillInfo>; conflicts: string[] } {
  const declared = new Map<string, DeclaredSkillInfo>();
  const conflicts: string[] = [];
  const add = (skill: string, source: string): void => {
    const { skillName } = parseSkillPath(skill);
    if (!skillName) return;
    if (!declared.has(skillName)) {
      declared.set(skillName, { full: skill, sources: new Set<string>() });
    } else if (declared.get(skillName)!.full !== skill) {
      conflicts.push(`${skillName}: declared as both ${declared.get(skillName)!.full} and ${skill}`);
    }
    declared.get(skillName)!.sources.add(source);
  };
  for (const tech of SKILLS_MAP) {
    for (const skill of tech.skills) add(skill, tech.id);
  }
  for (const combo of COMBO_SKILLS_MAP) {
    for (const skill of combo.skills) add(skill, combo.id);
  }
  for (const skill of FRONTEND_BONUS_SKILLS) add(skill, 'frontend-bonus');
  return { declared, conflicts };
}

function formatSources(sources: Set<string> | Iterable<string>): string {
  return [...sources].sort().join(', ');
}

function validateEntryFiles(skillName: string, entry: RegistryEntry, errors: string[]): void {
  if (!Array.isArray(entry.files) || entry.files.length === 0) {
    errors.push(`${skillName}: manifest entry has no files`);
    return;
  }
  const shaMap: Record<string, string> = (entry.sha256 ?? {}) as Record<string, string>;
  const parts: string[] = [];
  for (const file of entry.files) {
    const filePath = join(REGISTRY_DIR, skillName, file);
    if (!existsSync(filePath)) {
      errors.push(`${skillName}: missing file ${file}`);
      continue;
    }
    if (!statSync(filePath).isFile()) {
      errors.push(`${skillName}: ${file} is not a file`);
      continue;
    }
    const actualSha = sha256Hex(normalizeLineEndings(readFileSync(filePath)));
    if (shaMap[file] !== actualSha) {
      errors.push(`${skillName}: hash mismatch for ${file}`);
    }
    parts.push(`${file}:${actualSha}`);
  }
  const actualBundleHash = sha256Hex(parts.sort().join('\n'));
  if (entry.bundleHash !== actualBundleHash) {
    errors.push(`${skillName}: bundleHash mismatch`);
  }
}

function main(): void {
  if (!existsSync(MANIFEST_PATH)) {
    throw new Error(`Registry manifest not found: ${MANIFEST_PATH}`);
  }
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, 'utf-8')) as Manifest;
  const registrySkills: Record<string, RegistryEntry> = (manifest.skills ?? {}) as Record<string, RegistryEntry>;
  const { declared, conflicts } = collectDeclaredSkills();
  const errors: string[] = [];
  for (const conflict of conflicts) errors.push(conflict);
  for (const [skillName, declaredSkill] of declared) {
    const entry = registrySkills[skillName];
    if (!entry) {
      errors.push(
        `${skillName}: declared in skills map (${formatSources(declaredSkill.sources)}) but missing from registry`,
      );
      continue;
    }
    if (entry.skillPath !== declaredSkill.full) {
      errors.push(`${skillName}: registry skillPath is ${entry.skillPath}, expected ${declaredSkill.full}`);
    }
    validateEntryFiles(skillName, entry, errors);
  }
  for (const skillName of Object.keys(registrySkills).sort()) {
    if (!declared.has(skillName)) {
      errors.push(`${skillName}: present in registry but not declared in skills map`);
    }
  }
  if (errors.length > 0) {
    console.error('\nRegistry validation failed:');
    for (const error of errors) console.error(`- ${error}`);
    console.error(`\nChecked manifest: ${relative(process.cwd(), MANIFEST_PATH)}`);
    process.exit(1);
  }
  console.log(
    `Registry validation passed: ${declared.size} declared skill${declared.size === 1 ? '' : 's'} are installable.`,
  );
}

main();
