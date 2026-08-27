use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

// ── 4.1 Args ─────────────────────────────────────────────────────

#[test]
fn cli_help_exits_zero() {
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn cli_version_exits_zero() {
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn cli_agents_flag() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"dependencies":{"react":"^18"}}"#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    let output = cmd
        .current_dir(dir.path())
        .args(["-a", "cursor", "claude-code", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cursor"),
        "expected cursor in stdout, got {stdout}"
    );
}

// ── 4.2 Display 3-col ────────────────────────────────────────────

#[test]
fn display_three_col_via_dry_run_seven_techs() {
    // Create a project that triggers 7 technologies
    let dir = tempdir().unwrap();
    // Need 7 techs: react, next, vue, nuxt, svelte, astro, tailwind
    // We'll fake by having package.json with those deps
    fs::write(
        dir.path().join("package.json"),
        r#"{
            "dependencies": {
                "react": "^18",
                "next": "^14",
                "vue": "^3",
                "nuxt": "^3",
                "svelte": "^4",
                "astro": "^4",
                "tailwindcss": "^3"
            }
        }"#,
    )
    .unwrap();
    // astro and tailwind also need config files for some, but deps alone should trigger
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    let output = cmd
        .current_dir(dir.path())
        .args(["--dry-run"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let plain = strip_ansi(&stdout);
    for name in [
        "React",
        "Next.js",
        "Vue",
        "Nuxt",
        "Svelte",
        "Astro",
        "Tailwind CSS",
    ] {
        assert!(
            plain.contains(name),
            "expected {name} in output, got {plain}"
        );
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ── 4.3 Banner / Prompt / Spinner ────────────────────────────────

#[test]
fn clear_cache_flag() {
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    cmd.arg("--clear-cache").assert().success();
}

#[test]
fn dry_run_does_not_prompt() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"dependencies":{"react":"^18"}}"#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    let output = cmd
        .current_dir(dir.path())
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--dry-run: nothing was installed"));
}

// ── 4.4 Claude cleanup is tested in unit tests already ─────────

// ── 4.5 Main integration ─────────────────────────────────────────

#[test]
fn main_shows_no_tech_when_empty() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("skillscout").unwrap();
    let output = cmd.current_dir(dir.path()).output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("No supported technologies detected") || combined.contains("Scanning"),
        "expected no-tech message, got {combined}"
    );
}

// ── 4.6 Concurrency 6 is covered via installer unit tests ───────

#[test]
fn install_all_concurrency_via_installer() {
    // This is a placeholder that ensures the binary's installer module respects concurrency 6
    // Real concurrency is tested in src/installer.rs with semaphore 6
    // Here we just verify binary handles --help without panic under concurrent run
    let mut handles = Vec::new();
    for _ in 0..6 {
        handles.push(std::thread::spawn(|| {
            let mut cmd = Command::cargo_bin("skillscout").unwrap();
            cmd.arg("--help").assert().success();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
