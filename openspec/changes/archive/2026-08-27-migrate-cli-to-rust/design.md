# Design: Migrate CLI to Rust

## Technical Approach

Port TS CLI (3.8k LOC, Node >=22.6) to Rust crate `skillscout` (1.85, Cargo.lock) with byte parity. `build.rs` gens `skills_map.rs`+`skills_map.json` from `skills-map.ts`. Dual npx/cargo via `index.mjs` probe + `SKILLSCOUT_USE_RUST=0`; local `cargo build --release`. 5 chained PRs cover specs; <20ms, <5MB, `cargo audit/deny`.

## Architecture Decisions

### Decision: CLI stack

| Option | Tradeoff | Decision |
|---|---|---|
| `clap` derive | Macro weight vs help | **Chosen** `-y/--yes --dry-run --clear-cache -a/--agent -v/-h` |
| `crossterm`+`indicatif` | Raw-mode + spinner throttle | **Chosen** |
| `reqwest`+`rustls` | No OpenSSL, Bearer header | **Chosen** |
| `sha2`+`serde_json` | Match Node `crypto` hex lower | **Chosen** |

### Decision: Verbatim parsers

| Option | Tradeoff | Decision |
|---|---|---|
| `serde_yaml` | Dep + diverges from line scan | Rejected |
| Line scan `parsePnpmWorkspaceYaml` + regex `include\s*\(?\s*([^)]+)` + `walkdir` caps | Zero deps, parity | **Chosen** |

Preserves `pnpm>package.json>deno.json`, `*` via `readdir`, `SCAN_SKIP_DIRS`, `.NET` depth-2.

### Decision: Symlink fallback

| Option | Tradeoff | Decision |
|---|---|---|
| Symlink only | EPERM 1314 Win non-dev | Rejected |
| `symlink_dir`→`copyDir` on EPERM | Matches TS | **Chosen** |

### Decision: Integrity

| Option | Tradeoff | Decision |
|---|---|---|
| Unsorted hash | Non-deterministic | Rejected |
| `sha256(sorted "rel:sha256" "\n")` `\`→`/` + reject `.zip` | Matches 219 fixtures | **Chosen** |

### Decision: Codegen SSOT

| Option | Tradeoff | Decision |
|---|---|---|
| Duplicate map | Drift | Rejected |
| `build.rs` → `OUT_DIR/skills_map.rs` + `skills_map.json` | TS SSOT, `include_str!` | **Chosen** |

### Decision: Dual gate

| Option | Tradeoff | Decision |
|---|---|---|
| Replace `index.mjs` | Breaking | Rejected |
| Probe binary, `SKILLSCOUT_USE_RUST=0` → Node | Zero break | **Chosen** |

## Data Flow

```
index.mjs probe → Rust? Rust : Node dist/main.js
main.rs→args.rs→detect.rs→workspace.rs→gradle.rs/dotnet.rs/frontend.rs
          │           │ collectSkills()
          └─install.rs←registry.rs+hash.rs+cache.rs (3-tier)
             canonical→local→cache(~/.cache/skillscout/{bundleHash})→network, conc 6
             →claude.rs→display/prompt/banner
```

Sequence `detect→collect→install`:

```
detectTechnologies→resolveWorkspaces(pnpm|npm|deno, filter !=root)
 →detectTechnologiesInDir(caches)→isFrontend(FRONTEND_PACKAGES||hasWebFrontendFiles depth3)
 →collectSkills→multiSelect(TTY raw ❯◼/◻ grouped, a/n/i)
 →installAll sorted repo 6→installSkill→verify→local→cache→network(Bearer, 403+0 ISO, sha retry)
 →ensureSymlinkTo(symlink→copy)→updateSkillsLock(sorted+"\n")→cleanupClaudeMd
```

Errors: `thiserror`; exit1 red. Rate-limit aborts, hash retry. `tokio` sem 6 + `indicatif` 80ms (TTY); non-TTY sequential. `NO_COLOR`/`!isTTY` disables wave. Windows CI tests EPERM.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Create | Crate 0.3.6 MSRV1.85 |
| `Cargo.lock` | Create | Committed |
| `build.rs` | Create | TS→RS+JSON |
| `src/main.rs` | Create | Entry SIGINT |
| `src/args.rs` | Create | clap flags |
| `src/display.rs` | Create | 3-col/list/table |
| `src/detect.rs` | Create | detect+caches |
| `src/workspace.rs` | Create | resolveWorkspaces |
| `src/gradle.rs` | Create | parseSettingsGradle |
| `src/dotnet.rs` | Create | depth2 scan |
| `src/frontend.rs` | Create | hasWebFrontendFiles |
| `src/installer.rs` | Create | installAll conc6 |
| `src/registry.rs` | Create | loadRegistry dual URL |
| `src/hash.rs` | Create | bundleHash |
| `src/cache.rs` | Create | SKILLSCOUT_CACHE_DIR |
| `src/claude.rs` | Create | cleanupClaudeMd |
| `src/ui.rs` | Create | colors/cursor |
| `src/banner.rs` | Create | wave/static |
| `src/prompt.rs` | Create | multiSelect |
| `src/skills_map.rs` | Create | Generated OUT_DIR |
| `skills_map.json` | Create | Astro shim |
| `index.mjs` | Modify | Probe gate |
| `src/content.config.ts` | Modify | JSON import |
| `openspec/config.yaml` | Modify | cargo gates |
| `tests/` | Modify | cargo parity |

## Interfaces / Contracts

```rust
pub fn detect_technologies(p: &Path) -> DetectResult
pub fn resolve_workspaces(p: &Path) -> Vec<PathBuf>
pub fn parse_settings_gradle_modules(s: &str) -> Vec<String> // include\s*\(?\s*([^)]+)
pub fn bundle_hash(f: &[(String,Vec<u8>)]) -> String // sorted "rel:sha" "\n" hex lower
pub async fn install_skill(s: &str, agents: &[String]) -> InstallResult
pub async fn install_all(v: Vec<SkillEntry>) -> InstallAllResult // sem 6
fn ensure_symlink_to(t: &Path,l: &Path)->io::Result<()> // EPERM→copy
pub fn load_registry()->Option<Registry>
fn build_main(){ let (rs,js)=codegen(read("skills-map.ts")); write(out_dir(),rs); write("skills_map.json",js); }
```

Lock sorted+`\n`; reject `.zip`; Bearer on `githubusercontent.com`.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | gradle/workspace/frontend/hash/wrap/symlink | `cargo test` vs `node:test`, 219 fixtures |
| Integration | detect oracle, 3-tier, gate, lock | `useTmpDir` mock fetch |
| E2E | bin <5MB <20ms TTY/non-TTY flags `clippy -D` `fmt --check` `audit/deny` `astro build` | CI windows/ubuntu/macos `hyperfine` |

Parity CI asserts identical JSON (100+ fixtures) and lock/hash/skills.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. HTTPS `reqwest`+`rustls` only, no `Command`.

## Migration / Rollout

5 chained PRs `ask-on-risk`:
1. PR1 detect+workspace `detect/workspace/frontend/skills_map/build.rs`+`Cargo`
2. PR2 gradle/dotnet verbatim parsers
3. PR3 installer 3-tier Bearer+ISO hash symlink lock
4. PR4 main/args/display/ui/claude `clap/crossterm/indicatif` 6
5. PR5 parity CI `audit/deny` `clippy/fmt` Windows `astro build`

Revertible; `SKILLSCOUT_USE_RUST=0`→Node; keep `dist/main.js` one minor; escape delete `src/*.rs/Cargo.*/build.rs`.

## Open Questions

- [ ] `lto=true opt-level=z` for <5MB?
- [ ] `indicatif` vs manual spinner for `SPINNER` parity?

