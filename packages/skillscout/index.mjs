#!/usr/bin/env node

import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const [major, minor] = process.versions.node.split(".").map(Number);

if (major < 22 || (major === 22 && minor < 6)) {
  console.error(
    `\n  ⚠ skillscout requires Node.js >= 22.6.0.` +
      `\n  Current version: ${process.version}` +
      `\n  Please upgrade → https://nodejs.org\n`,
  );
  process.exit(1);
}

const __dirname = dirname(fileURLToPath(import.meta.url));

// ── Rust probe gate (PR4) ───────────────────────────────────────
// Try Rust binary first, unless forced to Node via SKILLSCOUT_USE_RUST=0
// Respects both SKILLSCOUT_USE_RUST and AUTOSKILLS_USE_RUST for backward compat.
const forceNode =
  process.env.SKILLSCOUT_USE_RUST === "0" ||
  process.env.AUTOSKILLS_USE_RUST === "0" ||
  process.env.SKILLSCOUT_USE_NODE === "1" ||
  process.env.SKILLSCOUT_USE_RUST === "false";

function findRustBinary() {
  const binName = process.platform === "win32" ? "skillscout.exe" : "skillscout";
  const candidates = [
    join(__dirname, "target", "release", binName),
    join(__dirname, "target", "debug", binName),
    join(__dirname, "..", "target", "release", binName),
    join(__dirname, "..", "target", "debug", binName),
    join(__dirname, "..", "..", "target", "release", binName),
  ];
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  return null;
}

if (!forceNode) {
  const rustBin = findRustBinary();
  if (rustBin) {
    const result = spawnSync(rustBin, process.argv.slice(2), { stdio: "inherit" });
    // If spawn succeeded, exit with Rust's code; if error (e.g. EACCES), fallback to Node
    if (!result.error) {
      process.exit(result.status ?? 0);
    }
  } else if (existsSync(join(__dirname, "Cargo.toml"))) {
    // Dev fallback: try `cargo run --quiet` if binary not yet built
    const manifest = join(__dirname, "Cargo.toml");
    const result = spawnSync("cargo", ["run", "--quiet", "--manifest-path", manifest, "--", ...process.argv.slice(2)], {
      stdio: "inherit",
    });
    if (!result.error) {
      process.exit(result.status ?? 0);
    }
    // cargo not found or failed to spawn → fall through to Node
  }
}

// ── Node fallback (original, preserved) ──────────────────────────
// Keep `npx/bunx skillscout` working when Rust is absent or forced off.
// Preserves `dist/main.js` probe + `--experimental-strip-types` for `main.ts`.

if (existsSync(join(__dirname, "dist", "main.js"))) {
  await import("./dist/main.js");
} else {
  try {
    await import("./main.ts");
  } catch (err) {
    if (err.code !== "ERR_UNKNOWN_FILE_EXTENSION") throw err;

    const { spawn } = await import("node:child_process");
    const mainPath = join(__dirname, "main.ts");
    const child = spawn(
      process.execPath,
      [
        "--experimental-strip-types",
        "--disable-warning=ExperimentalWarning",
        mainPath,
        ...process.argv.slice(2),
      ],
      { stdio: "inherit" },
    );
    child.on("exit", (code, signal) => {
      if (signal) process.kill(process.pid, signal);
      else process.exit(code ?? 1);
    });
  }
}
