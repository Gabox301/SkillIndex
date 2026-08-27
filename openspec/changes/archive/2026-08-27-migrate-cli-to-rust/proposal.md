# Proposal: Migrate CLI to Rust

## Intent
Port TS CLI (Node >=22.6, 3.8k LOC) to Rust for <20ms startup, memory safety, `cargo audit/deny`. Keep `npx/bunx skillscout` — dual `cargo install`+npm fallback, no break.

## Scope

### In Scope
- Full CLI slice-gated: `main/args/display`+`detect/workspace/gradle/dotnet/frontend`+`installer/registry/hash/cache`+`claude`+`ui/banner/prompt`
- `build.rs`: `skills-map.ts` SSOT -> `skills_map.rs`+`skills_map.json`
- Byte parity: `bundleHash=sha256(sorted "rel:hash" "\n")` hex lower, symlink->copy, sorted `skills-lock.json`, identical I/O
- Local `cargo build --release` -> `target/release/skillscout`; `SKILLSCOUT_USE_RUST=0`; keep `dist/main.js`

### Out of Scope
- Astro site (`src/`, `astro.config.mjs`), `scripts/generate-og.mjs`
- Registry curation / `sync-skills.mjs` logic (TS stays SSOT)
- Publish (crates.io, npm wrappers, Releases matrix) — deferred
- Remove TS/Bun/fendo — dual toolchain temporary

## Capabilities

### New Capabilities
- `cli-detection`: caches, `resolveWorkspaces` (minimal pnpm YAML verbatim), Gradle regex, .NET depth-2, `SCAN_SKIP_DIRS`, combo/frontend/agent
- `cli-installation`: 3-tier cache, dual URL+Bearer+rate-limit, SHA-256+bundleHash, `ensureSymlinkTo`, lock+`cleanupClaudeMd`
- `cli-interaction`: clap, 3-col display, `multiSelect` grouping/shortcuts, banner wave, spinner x6, `NO_COLOR`/`isTTY`

### Modified Capabilities
- None — greenfield crate; TS oracle unchanged.

## Approach
Crate `skillscout` (1.85, `Cargo.lock` committed) `clap/crossterm/indicatif/reqwest/rustls/sha2/serde_json/dirs/walkdir`; `build.rs` emits RS+JSON. Ask-on-risk, 5 chained PRs <800: PR1 detect+workspace, PR2 gradle/dotnet/frontend, PR3 installer/registry/hash/cache, PR4 main/args/display/ui/claude, PR5 integration+parity CI. `index.mjs` probes Rust then fallback. `cargo audit/deny/vet` mirrors fendo.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `packages/skillscout/src/*.rs`+`Cargo.*`+`build.rs` | New | Crate + codegen |
| `packages/skillscout/*.ts`+`index.mjs` | Modified | SSOT/oracle + fallback gate |
| `src/content.config.ts` | Modified | Shim to `skills_map.json` |
| `tests/` | Modified | Keep `node:test` + `cargo test` parity |
| `openspec/config.yaml` | Modified | Add cargo gates |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Win symlink EPERM 1314 | High | try `symlink_dir`->`copyDir`; TS fallback PR1; Win CI no dev mode |
| bundleHash drift | High | Fixtures 219 entries; `sha2` vs `crypto` cross-assert |
| Parser divergence | Med | Verbatim port; no `serde_yaml`; keep caches |
| Astro break | Med | Emit RS+JSON; shim |
| Budget >800 | High | 5 chained PRs |
| TUI parity | Med | Defer animation; `isTTY` tests |

## Rollback Plan
`SKILLSCOUT_USE_RUST=0` forces Node; `index.mjs` auto-fallback if missing. Each PR revertible. Keep `dist/main.js` one minor. Escape: delete `src/*.rs`/`Cargo.*`/`build.rs`, restore `src/content.config.ts` import.

## Dependencies
- Rust 1.85, `cargo audit/deny` CI; Node >=22.6+Bun+fendo retained
- `skills-registry/index.json` fixtures

## Success Criteria
- [ ] Binary <5MB stripped, <20ms
- [ ] Parity JSON identical (100+ tech fixtures)
- [ ] `clippy -D warnings`+`fmt --check`+`audit` clean
- [ ] Same `skills-lock.json`/`.agents/skills`/`bundleHash` (219)
- [ ] `--dry-run/--yes/--clear-cache`/`.zip` parity; TTY/non-TTY OK
- [ ] `SKILLSCOUT_USE_RUST=0` fallback; `astro build` passes
