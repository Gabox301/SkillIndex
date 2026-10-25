# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Entries start at 3.3.0: earlier releases predate this file.

## [3.3.0] - 2026-10-25

Supply-chain verification, an honest review status, and two new skills.

### Added

- **Postinstall checksum gate.** `postinstall.mjs` now downloads the release
  archive into a private temporary directory and verifies it against the
  `<archive>.sha256` asset published next to it by cargo-dist before extracting,
  marking executable or copying anything. The checksum file must parse as a
  SHA-256 digest and match the archive byte for byte.
- **`.sha256` assets per release archive.** The release workflow ships a
  checksum asset for every platform archive, so the postinstall gate and manual
  installs can verify downloads independently of the npm registry.
- **Registry traversal guard.** Relative paths taken from the registry are
  validated before they are joined onto any base directory. Absolute paths,
  `.` and `..` segments, empty segments, NUL bytes, backslash tricks and Windows
  drive prefixes are rejected, and the offending relative path is named in the
  failure. The guard is applied on install downloads and on registry entry
  verification, so a hostile manifest fails closed instead of writing outside
  the install directory.
- **New skills, both `unreviewed` (pinned by hash, not audited):**
  - `landing-page-design` — landing page design guidance.
  - `scroll-world` — scroll-driven web experiences, with references.

  The registry now describes 1122 skills.

### Changed

- **`sync-skills` writes `unreviewed` instead of a fake `approved`.** Previously
  every synced entry was stamped `approved` even though no content audit ran.
  Synced entries now record `review.status = "unreviewed"` with an explicit
  "pinned by hash, no content audit" summary and a matching security check, in
  both real and `--dry-run` runs. `unreviewed` is a valid registry status;
  content integrity (files, `sha256`, `bundleHash`) is still enforced for it.
- **Documentation matches the code.** `AGENTS.md` was rewritten around the real
  Rust layout (module layout, test commands, registry integrity checks, output
  helpers) instead of a stale description.
- **Housekeeping is coherent again.** Dead `packages/skillindex/*` paths were
  removed from the supply-chain configuration so the local `fendo` audit reads
  the actual workspace.

### Intentional behavior changes

- **Postinstall fails closed without a checksum.** If the `.sha256` asset cannot
  be downloaded, is malformed, or does not match, installation exits non-zero
  and leaves no extracted binary behind. Installations that used to succeed on
  an unverifiable download now fail; that is the point of the gate.
- **New pins surface as a security warning.** Skills synced since this release
  are pinned by hash but un-audited, so they now show up as a warning during
  selection instead of a silent `ok`. Reviewing a pinned skill is what makes the
  warning go away.

### Notes

- Registry entries that were already `approved` before 3.3.0 keep that status;
  they were not re-audited by this release.
- `fendo` only reports a missing `minimum-release-age` setting; it does not
  fail on it. Supply-chain hardening in this release does not change that.
