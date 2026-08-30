import { existsSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SECTION_START = '<!-- skillindex:start -->';
const SECTION_END = '<!-- skillindex:end -->';

export interface CleanupResult {
  cleaned: boolean;
  deleted: boolean;
}

function findGenericBlock(content: string): { start: number; end: number; endLen: number } | null {
  let searchFrom = 0;
  let startIdx: number | null = null;
  let endIdx: number | null = null;
  let endLen = 0;

  while (true) {
    const open = content.indexOf('<!--', searchFrom);
    if (open === -1) break;
    const close = content.indexOf('-->', open);
    if (close === -1) break;
    const inner = content.slice(open, close + 3);
    if (inner.includes(':start') && startIdx === null) {
      startIdx = open;
      // search for end after this
      let innerSearch = close + 3;
      while (true) {
        const open2 = content.indexOf('<!--', innerSearch);
        if (open2 === -1) break;
        const close2 = content.indexOf('-->', open2);
        if (close2 === -1) break;
        const inner2 = content.slice(open2, close2 + 3);
        if (inner2.includes(':end')) {
          endIdx = open2;
          endLen = close2 + 3 - open2;
          break;
        }
        innerSearch = close2 + 3;
      }
      break;
    } else if (inner.includes(':end')) {
      searchFrom = close + 3;
      continue;
    }
    searchFrom = close + 3;
    if (searchFrom >= content.length) break;
  }

  if (startIdx !== null && endIdx !== null) {
    return { start: startIdx, end: endIdx, endLen };
  }
  return null;
}

export function cleanupClaudeMd(projectDir: string): CleanupResult {
  const outputPath = join(projectDir, 'CLAUDE.md');
  if (!existsSync(outputPath)) {
    return { cleaned: false, deleted: false };
  }
  const existing = readFileSync(outputPath, 'utf-8');
  let startIdx = existing.indexOf(SECTION_START);
  let endIdx = existing.indexOf(SECTION_END);
  let sectionEndLen = SECTION_END.length;

  if (startIdx === -1 || endIdx === -1) {
    const generic = findGenericBlock(existing);
    if (!generic) {
      return { cleaned: false, deleted: false };
    }
    startIdx = generic.start;
    endIdx = generic.end;
    sectionEndLen = generic.endLen;
    if (endIdx < startIdx) {
      return { cleaned: false, deleted: false };
    }
  }

  const before = existing.slice(0, startIdx);
  const after = existing.slice(endIdx + sectionEndLen);
  const remaining = (before + after).replace(/\n{3,}/g, '\n\n').trim();
  if (!remaining || remaining === '# CLAUDE.md') {
    unlinkSync(outputPath);
    return { cleaned: true, deleted: true };
  }
  writeFileSync(outputPath, remaining + '\n');
  return { cleaned: true, deleted: false };
}
