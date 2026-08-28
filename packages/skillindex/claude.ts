import { existsSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SECTION_START = '<!-- skillindex:start -->';
const SECTION_END = '<!-- skillindex:end -->';
const LEGACY_SKILLSCOUT_START = '<!-- skillscout:start -->';
const LEGACY_SKILLSCOUT_END = '<!-- skillscout:end -->';
const LEGACY_SECTION_START = '<!-- autoskills:start -->';
const LEGACY_SECTION_END = '<!-- autoskills:end -->';

export interface CleanupResult {
  cleaned: boolean;
  deleted: boolean;
}

export function cleanupClaudeMd(projectDir: string): CleanupResult {
  const outputPath = join(projectDir, 'CLAUDE.md');
  if (!existsSync(outputPath)) {
    return { cleaned: false, deleted: false };
  }
  const existing = readFileSync(outputPath, 'utf-8');
  let startIdx = existing.indexOf(SECTION_START);
  let endIdx = existing.indexOf(SECTION_END);
  let sectionEnd = SECTION_END;
  if (startIdx === -1 || endIdx === -1) {
    startIdx = existing.indexOf(LEGACY_SKILLSCOUT_START);
    endIdx = existing.indexOf(LEGACY_SKILLSCOUT_END);
    sectionEnd = LEGACY_SKILLSCOUT_END;
    if (startIdx === -1 || endIdx === -1) {
      startIdx = existing.indexOf(LEGACY_SECTION_START);
      endIdx = existing.indexOf(LEGACY_SECTION_END);
      sectionEnd = LEGACY_SECTION_END;
      if (startIdx === -1 || endIdx === -1) {
        return { cleaned: false, deleted: false };
      }
    }
  }
  const before = existing.slice(0, startIdx);
  const after = existing.slice(endIdx + sectionEnd.length);
  const remaining = (before + after).replace(/\n{3,}/g, '\n\n').trim();
  if (!remaining || remaining === '# CLAUDE.md') {
    unlinkSync(outputPath);
    return { cleaned: true, deleted: true };
  }
  writeFileSync(outputPath, remaining + '\n');
  return { cleaned: true, deleted: false };
}
