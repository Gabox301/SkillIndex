import { ok, strictEqual } from 'node:assert/strict';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { cleanupClaudeMd } from '../cli/claude.ts';
import { useTmpDir } from './helpers.ts';

describe('cleanupClaudeMd', () => {
  const tmp = useTmpDir();

  it('returns cleaned=false when CLAUDE.md does not exist', () => {
    const result = cleanupClaudeMd(tmp.path);
    strictEqual(result.cleaned, false);
    strictEqual(result.deleted, false);
  });

  it('returns cleaned=false when CLAUDE.md has no skillindex markers', () => {
    writeFileSync(join(tmp.path, 'CLAUDE.md'), '# CLAUDE.md\n\nMy custom instructions.\n');
    const result = cleanupClaudeMd(tmp.path);
    strictEqual(result.cleaned, false);
    strictEqual(result.deleted, false);
    const output = readFileSync(join(tmp.path, 'CLAUDE.md'), 'utf-8');
    strictEqual(output, '# CLAUDE.md\n\nMy custom instructions.\n');
  });

  it('deletes CLAUDE.md when only the skillindex section remains', () => {
    writeFileSync(
      join(tmp.path, 'CLAUDE.md'),
      '# CLAUDE.md\n\n<!-- skillindex:start -->\n\nGenerated content.\n\n<!-- skillindex:end -->\n',
    );
    const result = cleanupClaudeMd(tmp.path);
    strictEqual(result.cleaned, true);
    strictEqual(result.deleted, true);
    ok(!existsSync(join(tmp.path, 'CLAUDE.md')));
  });

  it('removes skillindex section but preserves user content', () => {
    const content =
      '# CLAUDE.md\n\nMy custom instructions.\n\n<!-- skillindex:start -->\n\nGenerated content.\n\n<!-- skillindex:end -->\n\n## My notes\n\nDo not touch this.\n';
    writeFileSync(join(tmp.path, 'CLAUDE.md'), content);
    const result = cleanupClaudeMd(tmp.path);
    strictEqual(result.cleaned, true);
    strictEqual(result.deleted, false);
    const output = readFileSync(join(tmp.path, 'CLAUDE.md'), 'utf-8');
    ok(output.includes('My custom instructions.'));
    ok(output.includes('Do not touch this.'));
    ok(!output.includes('<!-- skillindex:start -->'));
    ok(!output.includes('Generated content.'));
  });

  it('does not leave triple newlines after removing the section', () => {
    const content = '# CLAUDE.md\n\nBefore.\n\n<!-- skillindex:start -->\nstuff\n<!-- skillindex:end -->\n\nAfter.\n';
    writeFileSync(join(tmp.path, 'CLAUDE.md'), content);
    cleanupClaudeMd(tmp.path);
    const output = readFileSync(join(tmp.path, 'CLAUDE.md'), 'utf-8');
    ok(!output.includes('\n\n\n'));
  });

  it('deletes file when heading is the only remaining content', () => {
    writeFileSync(
      join(tmp.path, 'CLAUDE.md'),
      '# CLAUDE.md\n\n<!-- skillindex:start -->\ngenerated\n<!-- skillindex:end -->\n',
    );
    const result = cleanupClaudeMd(tmp.path);
    strictEqual(result.cleaned, true);
    strictEqual(result.deleted, true);
    ok(!existsSync(join(tmp.path, 'CLAUDE.md')));
  });
});
