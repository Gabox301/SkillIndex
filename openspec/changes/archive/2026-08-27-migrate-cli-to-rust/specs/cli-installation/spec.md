# cli-installation Specification

## Purpose
Acquisition — integrity, 3-tier cache, linking, lockfile — parity with `installer.ts`.

## Requirements

### Requirement: Integrity
MUST verify SHA-256 hex lowercase vs `entry.sha256` (normalize `\`→`/`) and `bundleHash=sha256(sorted "rel:sha256" join "\n")`; reject `.zip` pre-download.

#### Scenario: Hash mismatch
- GIVEN recorded `abc` vs actual `def`
- WHEN verifying
- THEN `{ok:false}`

#### Scenario: Bundle parity
- GIVEN `a.md:h1,b.md:h2`
- WHEN computing
- THEN `sha256("a.md:h1\nb.md:h2")` sorted matches 219

### Requirement: Three-Tier Retrieval
MUST try `canonical .agents/skills → local → cache (~/.cache/skillscout/{bundleHash}) → network`; iterate `SKILLSCOUT_REGISTRY_BASE_URL` or `v{version}` then `main`, send `Bearer GITHUB_TOKEN` on `githubusercontent.com`, abort on `403`+`x-ratelimit-remaining:0` with ISO, continue on SHA mismatch.

#### Scenario: Cache hit
- GIVEN cache verifies, local stale
- WHEN installing
- THEN copies from cache, no fetch

#### Scenario: Rate limit
- GIVEN `403 remaining:0 reset:999000`
- WHEN downloading
- THEN throws `GitHub rate limit exceeded (resets <ISO>)`

### Requirement: Materialize and Lockfile
MUST write canonical `rmSync`+`copyDir`, link `AGENT_FOLDER_MAP` targets via relative symlink fallback `copyDir` on `EPERM 1314`, and update `skills-lock.json` sorted + trailing `\n`; `clearSkillScoutCache` respects `SKILLSCOUT_CACHE_DIR`.

#### Scenario: Symlink fallback
- GIVEN `symlinkSync` throws `EPERM`
- WHEN linking
- THEN copies and succeeds

#### Scenario: Sorted lock
- GIVEN lock has `z-skill`
- WHEN adding `a-skill`
- THEN writes `a-skill,z-skill` sorted + `"\n"`
