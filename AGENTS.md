# AGENTS

<!-- fendo:start -->

## Supply Chain Security

This project is hardened against supply chain attacks with local `fendo`
(`src/bin/fendo.rs`, 100% local, no external dependency; run with
`cargo run --bin fendo`).

### Rules for AI assistants and contributors

- **Pin exact versions in `package.json`** — never use `^` or `~` in npm
  dependency specifiers. (Rust `Cargo.toml` uses semver-compatible ranges by
  convention; the pin there is the committed `Cargo.lock`.)
- **Always commit the lockfiles** (`bun.lock`, `Cargo.lock`). Never delete
  them or add them to `.gitignore`.
- **Install scripts are disabled** (`ignore-scripts=true` in `.npmrc`). If a
  new npm dependency requires a build step, it must be explicitly approved.
- Release-age gating is **not** enforced: if `.npmrc` contains
  `minimum-release-age`, `fendo` reports it, but its absence is not a failure.
  Do not claim otherwise.
- When adding a dependency, verify it on [npmjs.com](https://www.npmjs.com)
  (npm) or [crates.io](https://crates.io) (Rust) before installing.
- Prefer well-maintained packages with verified publishers and provenance.
- Run `bun install --frozen-lockfile` with the lockfile present — never
  bypass it. In CI and scripts, prefer deterministic installs.
- Do not add git-based or tarball URL dependencies unless explicitly approved.
- **Do not run `npm update`**, `npx npm-check-updates`, or any blind upgrade
  command. Review each update individually.
<!-- fendo:end -->

## Testing

- Rust is the source of truth: `cargo test --manifest-path Cargo.toml`
  (run the full suite; use `cargo test --lib -j 1 <module>::` for a focused
  slice — `-j 1` works around parallel `cl.exe` failures on Windows).
- The npm shim is typechecked, linted and built with
  `bun run check` / `bun run lint` / `bun run build`.
- Registry integrity: `cargo run --bin validate-registry` (hashes) and
  `cargo run --bin sync-skills -- --dry-run` (drift with upstreams) both run
  in CI — keep them green.
- New synced skills land as `unreviewed` (pinned by hash, **not** audited),
  never `approved`. `approved` means a real audit ran.

## Output helpers

- **Never print directly in `src/**`** — use the `log` and `write` helpers
  from `src/ui` (`use crate::ui::{log, write};`).

```rust
// ✅ Correct
use crate::ui::{log, write};

log("hello");
write("raw output\n");

// ❌ Wrong
println!("hello");
print!("raw output\n");
```

- The thin npm delegation layer (`index.ts`, `postinstall.mjs`) uses
  `console` directly — that is allowed, it never implements behavior.
