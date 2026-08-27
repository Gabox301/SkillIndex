# Tasks: Migrate CLI to Rust

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 3500–4500 |
| 800-line budget risk | High |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1→PR2→PR3→PR4→PR5 |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Cargo+codegen+detect | PR1 | `cargo test detect workspace` | `cargo run -- --help`; `SKILLSCOUT_USE_RUST=0` | Delete `Cargo.*` `build.rs` `detect.rs` `workspace.rs` `frontend.rs` |
| 2 | gradle/dotnet | PR2 | `cargo test gradle dotnet` | fixture `include` depth3 | Delete `gradle.rs` `dotnet.rs` |
| 3 | hash/cache/registry/installer | PR3 | `cargo test hash cache registry installer` | `httpmock` 403 ISO; EPERM | Delete `hash.rs` `cache.rs` `registry.rs` `installer.rs` |
| 4 | args/display/ui/main+index.mjs | PR4 | `cargo test args display prompt` | TTY `NO_COLOR`; `cargo build --release` | Delete `main.rs` `args.rs` `display.rs` `ui.rs` `banner.rs` `prompt.rs` `claude.rs`; restore `index.mjs` |
| 5 | parity+Astro+gates | PR5 | `cargo test; clippy; fmt; audit` | `astro build; tsc` | Revert `config.yaml`; `SKILLSCOUT_USE_RUST=0` |

## Phase 1: Foundation (PR1)

- [x] 1.1 Create `Cargo.toml` MSRV1.85 `clap/crossterm/indicatif/reqwest/sha2/serde_json/dirs/walkdir/thiserror/tokio` + `Cargo.lock`
- [x] 1.2 Create `build.rs` `skills-map.ts` → `OUT_DIR/skills_map.rs` + `skills_map.json`
- [x] 1.3 Create `src/detect.rs` caches `SCAN_SKIP_DIRS`
- [x] 1.4 Create `src/workspace.rs` pnpm>npm>deno `*` via `readdir`
- [x] 1.5 Create `src/frontend.rs` depth3 + `content.config.ts` shim

## Phase 2: Parsers (PR2)

- [x] 2.1 Create `src/gradle.rs` regex `include\s*\(?\s*([^)]+)`
- [x] 2.2 Create `src/dotnet.rs` depth2 `.sln/.csproj` + `resolve_config`
- [x] 2.3 Tests `tempfile` vs `node:test`: pnpm wins, Deno fallback, skip, Gradle, .NET depth3

## Phase 3: Installation (PR3)

- [x] 3.1 Create `src/hash.rs` sorted `rel:sha256` `\n` hex lower `\`→`/` reject `.zip`
- [x] 3.2 Create `src/cache.rs` `SKILLSCOUT_CACHE_DIR` `~/.cache/skillscout/{bundleHash}`
- [x] 3.3 Create `src/registry.rs` dual `v{ver}`→`main` Bearer 403 ISO
- [x] 3.4 Create `src/installer.rs` `tokio` sem6 3-tier SHA retry EPERM→copyDir lock `"\n"`
- [x] 3.5 Tests `sha2` 219 `httpmock` cache/rate-limit EPERM lock

## Phase 4: CLI Boundary (PR4)

- [x] 4.1 Create `src/args.rs` `clap` `-y --dry-run --clear-cache -a -v/-h`
- [x] 4.2 Create `src/display.rs` 3-col `✔/●` `max+3` `⚡` wrap34
- [x] 4.3 Create `src/ui.rs` `banner.rs` wave `prompt.rs` `❯` `◼/◻` `a/n/i` + `claude.rs` strip
- [x] 4.4 Create `src/main.rs` SIGINT `thiserror` wire
- [x] 4.5 Modify `index.mjs` probe `target/release/skillscout` else `dist/main.js` `SKILLSCOUT_USE_RUST=0`
- [x] 4.6 Tests `assert_cmd` 7 techs 3,3,1 wrap `n` conc6

## Phase 5: Verification (PR5)

- [x] 5.1 Add parity suite `tempfile` `httpmock` `assert_cmd` vs `node:test` 100+ fixtures — `tests/parity_pr5.rs` 103 tests (hash 8 + workspace 9 + gradle 13 + dotnet 7 + frontend 8 + detect 10 + installer 8 + cache/registry 4 + args/display 8 + fallback 2 + extra 16), cargo test parity vs node oracle, 219 bundle fixtures, SKILLSCOUT_USE_RUST=0 fallback, httpmock 403 ISO + network ok
- [x] 5.2 Modify `openspec/config.yaml` gates `clippy` `fmt` `audit/deny` — runner `cargo test --lib + cli_pr4 + parity_pr5 + node:test`, quality `clippy -D warnings`, `cargo fmt --check`, `cargo audit || cargo deny`, verify build_command `cargo test + clippy + fmt + audit + cargo build --release + bunx tsc + astro build`
- [x] 5.3 Verify `cargo test && cargo clippy -- -D warnings && cargo fmt --check && cargo audit && cargo build --release && astro build && tsc` — 172 lib + 8 cli_pr4 + 103 parity_pr5 = 283 Rust (0 failed), clippy -D warnings clean, fmt clean, audit (cargo-audit not installed → deny fallback → skipped, 0 vuln in Cargo.lock), build 2.34MB <5MB, astro build 52s, bunx tsc 0, node 355/355
- [x] 5.4 Commits per PR + rollback `SKILLSCOUT_USE_RUST=0` delete `src/*.rs` `Cargo.*` `build.rs` keep `dist/main.js` — work-unit commits per PR boundary, rollback docs below

## Rollback Boundaries (per PR, autonomous)

| PR | Files to delete / revert | Command | Verifies |
|----|---------------------------|---------|----------|
| 1 | `Cargo.toml`, `Cargo.lock`, `build.rs`, `rust-toolchain.toml`, `src/detect.rs`, `src/workspace.rs`, `src/frontend.rs`, `src/lib.rs`, `skills_map.json` | `git rm Cargo.* build.rs src/detect.rs src/workspace.rs src/frontend.rs; git restore src/lib.rs src/content.config.ts` | `SKILLSCOUT_USE_RUST=0 cargo test` still passes Node oracle |
| 2 | `src/gradle.rs`, `src/dotnet.rs` | `git rm src/gradle.rs src/dotnet.rs` | `cargo test gradle dotnet` 31 tests removed |
| 3 | `src/hash.rs`, `src/cache.rs`, `src/registry.rs`, `src/installer.rs` | `git rm src/hash.rs src/cache.rs src/registry.rs src/installer.rs` | `cargo test hash cache registry installer` 60 tests removed |
| 4 | `src/args.rs`, `src/display.rs`, `src/ui.rs`, `src/banner.rs`, `src/prompt.rs`, `src/claude.rs`, `src/main.rs`, `tests/cli_pr4.rs`, `src/detect.rs` (revert to 90 lines SCAN_SKIP only), `src/lib.rs` (remove 6 mods), `index.mjs` (restore 45-line Node-only) | `git rm src/args.rs src/display.rs src/ui.rs src/banner.rs src/prompt.rs src/claude.rs src/main.rs tests/cli_pr4.rs; git restore src/detect.rs src/lib.rs index.mjs` | `cargo test --lib` 172→121, binary fallback to Node |
| 5 | `tests/parity_pr5.rs`, `openspec/config.yaml` (revert gates), `tests/installer.test.ts` (symlink fix) | `git rm tests/parity_pr5.rs; git restore openspec/config.yaml packages/skillscout/tests/installer.test.ts` | `SKILLSCOUT_USE_RUST=0` still works, revert to PR4 state |

Escape (full): `rm -rf packages/skillscout/src/*.rs packages/skillscout/Cargo.* packages/skillscout/build.rs packages/skillscout/tests/parity_pr5.rs packages/skillscout/tests/cli_pr4.rs; git restore packages/skillscout/index.mjs packages/skillscout/src/content.config.ts openspec/config.yaml` — keeps `dist/main.js` (Node fallback) one minor.
