# cli-interaction Specification

## Purpose
CLI boundary — args, rendering, selection, concurrency — preserving `main.ts`/`ui.ts`.

## Requirements

### Requirement: Args and Distribution
MUST parse `clap` `-y/--yes`, `--dry-run`, `--clear-cache`, `-a/--agent`, `-v`, `-h`; MUST ship `cargo build --release` and keep `npx` dual with `SKILLSCOUT_USE_RUST=0` forcing Node probe.

#### Scenario: Flags
- GIVEN `skillscout -y --dry-run -a cursor claude-code -v`
- WHEN parsing
- THEN `autoYes/dryRun/agents/verbose` true

#### Scenario: Fallback
- GIVEN `SKILLSCOUT_USE_RUST=0` or missing binary
- WHEN invoking `npx skillscout`
- THEN runs `dist/main.js`

### Requirement: Rendering
MUST render `printDetected` 3-col (`✔`/`●`, `colWidth=max+3`, combos `⚡`), `printSkillsList` numbered padded `author › skill` + tags + `← sources`, `printSecurityChecks` sorted table `| Skill | Check | Findings |` with `wrapText`/`truncate 34`; non-TTY/`NO_COLOR` plain, TTY wave.

#### Scenario: Three-col
- GIVEN 7 techs
- WHEN printing
- THEN rows 3,3,1

#### Scenario: Table wrap
- GIVEN long findings
- WHEN printing checks
- THEN sorted, skill 34, wrapped continuation empty cells

### Requirement: Selection and Concurrency
MUST provide `multiSelect` raw-mode `❯`+`◼/◻` grouped `sources[0]`, shortcuts `a`/`n`/`i`; `installAll` sorted `repo`, concurrency 6, spinner TTY; `cleanupClaudeMd` strips `<!-- skillscout:start -->`, deletes if only `# CLAUDE.md`.

#### Scenario: Shortcut n
- GIVEN 2 installed +2 new
- WHEN pressing `n`
- THEN selects new-only

#### Scenario: Concurrency 6
- GIVEN 10 skills TTY
- WHEN `installAll`
- THEN ≤6 concurrent, spinner animates
