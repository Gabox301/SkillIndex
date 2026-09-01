use clap::Parser;
use skillindex::args::Args;
use skillindex::cache::get_cache_registry_dir;
use skillindex::display::{
    DisplayCombo, DisplayTechnology, format_detected, format_security_checks, format_skill_label,
    truncate_visible, visible_pad, wrap_text,
};
use skillindex::dotnet::dotnet_layout_candidate_paths;
use skillindex::frontend::has_web_frontend_files;
use skillindex::gradle::{gradle_layout_candidate_paths, parse_settings_gradle_modules};
use skillindex::hash::{
    bundle_hash, is_disallowed_skill_file, normalize_registry_rel_path, sha256_buffer,
};
use skillindex::installer::encode_raw_path;
use skillindex::registry::{Registry, get_registry_raw_base_urls, parse_skill_path};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tempfile::tempdir;
use tokio::sync::{Mutex, MutexGuard};

// ── Helpers ────────────────────────────────────────────────────────

/// Serializes tests that mutate the process-global `SKILLINDEX_CACHE_DIR`.
/// Integration tests run in parallel and cannot reach the crate-internal
/// `env_lock`, so without this guard one test can swap the cache dir out from
/// under another mid-download, causing spurious `os error 3` copy failures.
/// Uses tokio's async mutex so the guard can be safely held across `.await`.
async fn cache_env_guard() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().await
}

fn write_file(base: &Path, rel: &str, content: &str) {
    let p = base.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

// ── 5.1 parity — hash (219 fixtures, bundle_hash, normalization) ──

#[test]
fn parity_hash_empty_buffer() {
    assert_eq!(
        sha256_buffer(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn parity_hash_hello_buffer() {
    assert_eq!(
        sha256_buffer(b"hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn parity_hash_bundle_single_entry() {
    // Mirrors installer.test.ts bundle parity: single entry
    let hash = sha256_buffer(b"single content");
    let bundle = bundle_hash(&[("SKILL.md".to_string(), hash.clone())]);
    let expected = sha256_buffer(format!("SKILL.md:{hash}").as_bytes());
    assert_eq!(bundle, expected);
}

#[test]
fn parity_hash_bundle_sorted() {
    let h1 = sha256_buffer(b"a");
    let h2 = sha256_buffer(b"b");
    let bundle1 = bundle_hash(&[
        ("b.md".to_string(), h2.clone()),
        ("a.md".to_string(), h1.clone()),
    ]);
    let bundle2 = bundle_hash(&[("a.md".to_string(), h1), ("b.md".to_string(), h2.clone())]);
    assert_eq!(bundle1, bundle2);
    // sorted join check
    let sorted = {
        let mut v = [
            ("a.md".to_string(), sha256_buffer(b"a")),
            ("b.md".to_string(), sha256_buffer(b"b")),
        ];
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v.iter()
            .map(|(k, h)| format!("{k}:{h}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(bundle1, sha256_buffer(sorted.as_bytes()));
}

#[test]
fn parity_hash_bundle_219_fixtures() {
    // Parity vs Node: verify every registry entry's bundleHash matches recomputed bundle_hash
    let reg_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills-registry/index.json");
    let data = fs::read_to_string(&reg_path).expect("registry index exists");
    let reg: Registry = serde_json::from_str(&data).unwrap();
    let mut ok = 0usize;
    let mut skipped = 0usize;
    for (name, entry) in &reg.skills {
        // skip placeholder non-hex bundleHash (elysiajs)
        if entry.bundle_hash.len() != 64
            || entry.bundle_hash.chars().any(|c| !c.is_ascii_hexdigit())
        {
            skipped += 1;
            continue;
        }
        let mut entries: Vec<(String, String)> = entry
            .sha256
            .iter()
            .map(|(k, v)| (normalize_registry_rel_path(k), v.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let computed = bundle_hash(&entries);
        assert_eq!(
            computed, entry.bundle_hash,
            "bundle mismatch for skill {name}: expected {}, got {}",
            entry.bundle_hash, computed
        );
        ok += 1;
    }
    assert!(
        ok >= 200,
        "expected >=200 valid fixtures, got {ok} (skipped {skipped})"
    );
}

#[test]
fn parity_hash_normalize_backslash() {
    assert_eq!(normalize_registry_rel_path("a\\b\\c"), "a/b/c");
    assert_eq!(
        normalize_registry_rel_path("references\\notes.md"),
        "references/notes.md"
    );
}

#[test]
fn parity_hash_is_disallowed_zip() {
    assert!(is_disallowed_skill_file("archive.zip"));
    assert!(is_disallowed_skill_file("tool.ZIP"));
    assert!(is_disallowed_skill_file("downloads/tool.zip"));
    assert!(!is_disallowed_skill_file("SKILL.md"));
    assert!(!is_disallowed_skill_file("references/notes.md"));
}

#[test]
fn parity_hash_bundle_spec_example() {
    // spec: a.md:h1,b.md:h2 => sha256("a.md:h1\nb.md:h2") sorted matches 219
    let h1 = "h1".to_string();
    let h2 = "h2".to_string();
    let bundle = bundle_hash(&[
        ("b.md".to_string(), h2.clone()),
        ("a.md".to_string(), h1.clone()),
    ]);
    let manual = sha256_buffer("a.md:h1\nb.md:h2".as_bytes());
    assert_eq!(bundle, manual);
}

// ── workspace parity (14 fixtures vs node:test) ───────────────────

// workspace resolution parity lives in tests/workspace.rs (per-language mirror);
// the duplicate cross-checks that used to be here were removed.

// ── gradle parity (12 parse + layout) ─────────────────────────────

#[test]
fn parity_gradle_parse_kotlin_dsl() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include("app")"#),
        vec!["app"]
    );
}

#[test]
fn parity_gradle_parse_groovy() {
    assert_eq!(parse_settings_gradle_modules("include 'app'"), vec!["app"]);
}

#[test]
fn parity_gradle_strip_colon() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include(":app")"#),
        vec!["app"]
    );
}

#[test]
fn parity_gradle_colon_to_slash() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include(":feature:login")"#),
        vec!["feature/login"]
    );
}

#[test]
fn parity_gradle_multiple_groovy() {
    assert_eq!(
        parse_settings_gradle_modules("include 'app', 'core', 'data'"),
        vec!["app", "core", "data"]
    );
}

#[test]
fn parity_gradle_multiple_kotlin() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include(":app", ":core", ":data")"#),
        vec!["app", "core", "data"]
    );
}

#[test]
fn parity_gradle_multiline() {
    let content = "include(\n  \":app\",\n  \":core\",\n  \":shared:data\"\n)";
    assert_eq!(
        parse_settings_gradle_modules(content),
        vec!["app", "core", "shared/data"]
    );
}

#[test]
fn parity_gradle_multiple_statements() {
    assert_eq!(
        parse_settings_gradle_modules("include(\":app\")\ninclude(\":core\")"),
        vec!["app", "core"]
    );
}

#[test]
fn parity_gradle_empty() {
    assert!(parse_settings_gradle_modules("").is_empty());
    assert!(parse_settings_gradle_modules("rootProject.name = \"my-app\"").is_empty());
}

#[test]
fn parity_gradle_spec_example() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include(":app",":lib:core")"#),
        vec!["app", "lib/core"]
    );
}

#[test]
fn parity_gradle_layout_root_files() {
    let dir = tempdir().unwrap();
    let paths = gradle_layout_candidate_paths(dir.path());
    assert_eq!(paths.len(), 5);
    assert!(paths.iter().any(|p| p.ends_with("build.gradle.kts")));
}

#[test]
fn parity_gradle_layout_includes_subdir() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("composeApp")).unwrap();
    fs::write(dir.path().join("composeApp/build.gradle.kts"), "").unwrap();
    let paths = gradle_layout_candidate_paths(dir.path());
    assert!(
        paths
            .iter()
            .any(|p| p.ends_with("composeApp/build.gradle.kts"))
    );
}

#[test]
fn parity_gradle_layout_settings_kts() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "settings.gradle.kts",
        r#"include(":feature:login")"#,
    );
    let paths = gradle_layout_candidate_paths(dir.path());
    assert!(
        paths
            .iter()
            .any(|p| p.ends_with("feature/login/build.gradle.kts"))
    );
}

// ── dotnet parity (depth 2, SCAN_SKIP, csproj) ────────────────────

#[test]
fn parity_dotnet_root_files() {
    let dir = tempdir().unwrap();
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert_eq!(paths.len(), 4);
    assert!(paths.iter().any(|p| p.ends_with("global.json")));
}

#[test]
fn parity_dotnet_csproj_depth0() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "MyApp.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\">",
    );
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(paths.iter().any(|p| p.ends_with("MyApp.csproj")));
}

#[test]
fn parity_dotnet_nested_depth2() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "src/Library/Library.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\">",
    );
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(paths.iter().any(|p| p.ends_with("Library.csproj")));
}

#[test]
fn parity_dotnet_excludes_depth3() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "src/A/B/C/App.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\">",
    );
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(!paths.iter().any(|p| p.ends_with("App.csproj")));
}

#[test]
fn parity_dotnet_case_insensitive() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "App.CSPROJ", "<Project>");
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(
        paths
            .iter()
            .any(|p| p.file_name().unwrap().to_string_lossy() == "App.CSPROJ")
    );
}

#[test]
fn parity_dotnet_skips_bin_obj() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "bin/Debug/Exclude.csproj", "<Project>");
    write_file(dir.path(), "src/Keep.csproj", "<Project>");
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(!paths.iter().any(|p| p.to_string_lossy().contains("bin")));
    assert!(paths.iter().any(|p| p.ends_with("Keep.csproj")));
}

#[test]
fn parity_dotnet_skips_dot_dirs() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), ".hidden/App.csproj", "<Project>");
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(
        !paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".hidden"))
    );
}

// ── frontend parity (depth 3, extensions, skip) ──────────────────

#[test]
fn parity_frontend_html_depth1() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "index.html", "<html></html>");
    assert!(has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_vue_depth3() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "src/components/App.vue",
        "<template></template>",
    );
    assert!(has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_vue_depth4_excluded() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "src/a/b/c/d/App.vue", "<template></template>");
    assert!(!has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_blade() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "resources/views/home.blade.php", "blade");
    assert!(has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_twig() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "templates/page.twig", "twig");
    assert!(has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_php_alone_false() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "index.php", "<?php echo 'hi';");
    assert!(!has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_skip_node_modules() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "node_modules/pkg/App.vue",
        "<template></template>",
    );
    assert!(!has_web_frontend_files(dir.path(), 3));
}

#[test]
fn parity_frontend_dot_skip() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), ".hidden/App.vue", "<template></template>");
    assert!(!has_web_frontend_files(dir.path(), 3));
}

// technology/combo/frontend-flag detection parity lives in tests/detect.rs
// (per-language mirror); the duplicate cross-checks that used to be here were removed.

// ── installer / cache / registry parity ───────────────────────────

#[test]
fn parity_installer_encode_simple() {
    assert_eq!(encode_raw_path("my-skill", "SKILL.md"), "my-skill/SKILL.md");
    assert_eq!(
        encode_raw_path("my-skill", "references/notes.md"),
        "my-skill/references/notes.md"
    );
}

#[test]
fn parity_installer_encode_backslash() {
    assert_eq!(
        encode_raw_path("my-skill", "references\\notes.md"),
        "my-skill/references/notes.md"
    );
}

#[test]
fn parity_installer_encode_spaces() {
    assert_eq!(
        encode_raw_path("my-skill", "file with spaces.md"),
        "my-skill/file%20with%20spaces.md"
    );
}

// rel_path_from_to / copy_dir / ensure_symlink_to parity live in tests/installer.rs
// (per-language mirror); the duplicate cross-checks that used to be here were removed.

// update_skills_lock parity lives in tests/installer.rs (per-language mirror);
// the duplicate cross-check that used to be here was removed.

#[test]
fn parity_registry_base_urls_default() {
    let urls = get_registry_raw_base_urls(None);
    // When no env, should return 2 URLs with version and main
    // If env is set from prior tests, we tolerate either, but at least contains main
    assert!(
        urls.iter()
            .any(|u| u.contains("main/packages/skillindex/skills-registry"))
    );
}

#[test]
fn parity_registry_parse_skill_path() {
    let p = parse_skill_path("owner/repo/hello-skill");
    assert_eq!(p.repo, "owner/repo");
    assert_eq!(p.skill_name, "hello-skill");
    let http = parse_skill_path("https://example.com/skill");
    assert_eq!(http.skill_name, "");
}

// agent_folder_for parity lives in tests/installer.rs (per-language mirror);
// the duplicate cross-check that used to be here was removed.

#[test]
fn parity_cache_dir_appends_hash() {
    let dir = get_cache_registry_dir("abc123");
    assert!(dir.ends_with("abc123"));
}

// ── args / display parity (clap, 3-col, wrap, truncate) ──────────

// --help / --version exit-code parity lives in tests/cli.rs (per-language mirror);
// the duplicate cross-checks that used to be here were removed.

#[test]
fn parity_args_dry_run_flag() {
    // cargo test -- --nocapture should not trigger dry-run logic via binary, but we test via clap directly
    let args = Args::try_parse_from(["skillindex", "--dry-run"]).unwrap();
    assert!(args.dry_run);
}

#[test]
fn parity_args_yes_and_agent() {
    let args =
        Args::try_parse_from(["skillindex", "-y", "-a", "cursor", "-a", "claude-code"]).unwrap();
    assert!(args.yes);
    assert_eq!(args.agent, vec!["cursor", "claude-code"]);
}

#[test]
fn parity_display_skill_label_plain() {
    assert_eq!(format_skill_label("a/b/c", false), "a › c");
    assert_eq!(
        format_skill_label("https://example.com/skill", false),
        "https://example.com/skill"
    );
}

#[test]
fn parity_display_wrap_truncate() {
    assert_eq!(wrap_text("hello world", 5), vec!["hello", "world"]);
    assert_eq!(truncate_visible("hello", 10), "hello");
    assert_eq!(visible_pad("hi", 5).len(), 5);
}

#[test]
fn parity_display_three_col() {
    let techs: Vec<DisplayTechnology> = (0..7)
        .map(|i| DisplayTechnology {
            id: format!("t{i}"),
            name: format!("Tech{i}"),
            skills: vec![],
        })
        .collect();
    let combos: Vec<DisplayCombo> = vec![DisplayCombo {
        id: "combo".to_string(),
        name: "Combo".to_string(),
    }];
    let out = format_detected(&techs, &combos, true);
    let plain = skillindex::ui::strip_ansi(&out);
    for i in 0..7 {
        assert!(plain.contains(&format!("Tech{i}")));
    }
    // should contain frontend checkmark
    assert!(plain.contains('✔') || plain.contains('●'));
}

#[test]
fn parity_display_security_sorted() {
    let checks = vec![
        skillindex::registry::InstallSecurityCheck {
            name: "zebra".to_string(),
            status: "ok".to_string(),
            summary: "ok".to_string(),
            findings: vec!["find z".to_string()],
        },
        skillindex::registry::InstallSecurityCheck {
            name: "alpha".to_string(),
            status: "warning".to_string(),
            summary: "warn".to_string(),
            findings: vec!["flag".to_string()],
        },
    ];
    let out = format_security_checks(&checks);
    let plain = skillindex::ui::strip_ansi(&out);
    let alpha_pos = plain.find("alpha").unwrap();
    let zebra_pos = plain.find("zebra").unwrap();
    assert!(alpha_pos < zebra_pos);
}

// ── installer httpmock + fallback parity (network, rate-limit, EPERM) ─

// --clear-cache / --dry-run / 7-tech dry-run / --help binary parity live in
// tests/cli.rs (per-language mirror); the duplicate cross-checks that used to be
// here were removed. The Node-fallback cross-check below stays: it is the only
// test that verifies the TS implementation still runs.

// SKILLINDEX_USE_RUST=0 fallback — ensure Node index.ts (or shim index.mjs) still works via node
#[test]
fn parity_fallback_node_via_env() {
    // Spawn node index.ts --help with SKILLINDEX_USE_RUST=0, should succeed via Node fallback (fallback to index.mjs shim if needed)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let index_ts = manifest_dir.join("index.ts");
    let index_mjs = manifest_dir.join("index.mjs");
    let index = if index_ts.exists() {
        index_ts
    } else {
        index_mjs
    };
    if !index.exists() {
        return;
    }
    let is_ts = index.extension().and_then(|e| e.to_str()) == Some("ts");
    let mut cmd = if is_ts {
        let mut c = std::process::Command::new("bun");
        c.arg(index.clone());
        c
    } else {
        let mut c = std::process::Command::new("node");
        c.arg(index.clone());
        c
    };
    let output = cmd
        .arg("--help")
        .env("SKILLINDEX_USE_RUST", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "node fallback failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // Node output contains help text (from dist/main.js or main.ts)
    assert!(
        combined.contains("--dry-run") || combined.contains("skillindex"),
        "fallback help missing markers: {combined}"
    );
}

#[tokio::test]
async fn parity_installer_rate_limit_iso() {
    let _cache_guard = cache_env_guard().await;
    use httpmock::MockServer;
    use skillindex::hash::sha256_buffer;
    use skillindex::installer::{InstallOptions, install_skill_with_client};
    use skillindex::registry::{Registry, RegistryEntry, Review, Reviewer};
    use std::collections::HashMap;

    let server = MockServer::start();
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::create_dir_all(&reg_dir).unwrap();

    // build entry
    let skill_name = "rate-skill";
    let content = "content";
    let hash = sha256_buffer(content.as_bytes());
    let bundle = bundle_hash(&[("SKILL.md".to_string(), hash.clone())]);
    let entry = RegistryEntry {
        source: "owner/repo".to_string(),
        skill_path: format!("owner/repo/{skill_name}"),
        commit_sha: "deadbeef".to_string(),
        files: vec!["SKILL.md".to_string()],
        sha256: {
            let mut m = HashMap::new();
            m.insert("SKILL.md".to_string(), hash);
            m
        },
        bundle_hash: bundle,
        review: Review {
            status: "approved".to_string(),
            flags: vec![],
            summary: "t".to_string(),
            model: "m".to_string(),
            prompt_version: "1".to_string(),
            reviewed_at: "2026-01-01T00:00:00Z".to_string(),
        },
        security_check: None,
    };
    let mut skills = HashMap::new();
    skills.insert(skill_name.to_string(), entry);
    let registry = Registry {
        version: 1,
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        reviewer: Reviewer {
            model: "t".to_string(),
            prompt_version: "1".to_string(),
        },
        skills,
    };
    fs::write(
        reg_dir.join("index.json"),
        serde_json::to_string_pretty(&registry).unwrap(),
    )
    .unwrap();
    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path(format!("/{skill_name}/SKILL.md"));
        then.status(403)
            .header("x-ratelimit-remaining", "0")
            .header("x-ratelimit-reset", "999")
            .body("rate limited");
    });
    let cache_root = tmp.path().join("cache-rate-parity");
    let prev = std::env::var("SKILLINDEX_CACHE_DIR").ok();
    unsafe { std::env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };
    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        registry_base_url: Some(server.base_url()),
        ..Default::default()
    };
    let client = reqwest::Client::new();
    let result = install_skill_with_client("owner/repo/rate-skill", &[], &opts, &client).await;
    assert!(!result.success);
    assert!(
        result.output.contains("Límite de tasa de GitHub excedido")
            || result.output.contains("GitHub rate limit exceeded")
    );
    assert!(result.output.contains("1970-01-01T00:16:39.000Z"));
    mock.assert();
    match prev {
        Some(v) => unsafe { std::env::set_var("SKILLINDEX_CACHE_DIR", v) },
        None => unsafe { std::env::remove_var("SKILLINDEX_CACHE_DIR") },
    }
}

#[tokio::test]
async fn parity_installer_httpmock_network_ok() {
    let _cache_guard = cache_env_guard().await;
    use httpmock::MockServer;
    use skillindex::hash::sha256_buffer;
    use skillindex::installer::{InstallOptions, install_skill_with_client};
    use skillindex::registry::{Registry, RegistryEntry, Review, Reviewer};
    use std::collections::HashMap;

    let server = MockServer::start();
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::create_dir_all(&reg_dir).unwrap();

    let skill_name = "net-skill-parity";
    let content = "# net skill parity";
    let hash = sha256_buffer(content.as_bytes());
    let bundle = bundle_hash(&[("SKILL.md".to_string(), hash.clone())]);
    let entry = RegistryEntry {
        source: "owner/repo".to_string(),
        skill_path: format!("owner/repo/{skill_name}"),
        commit_sha: "deadbeef".to_string(),
        files: vec!["SKILL.md".to_string()],
        sha256: {
            let mut m = HashMap::new();
            m.insert("SKILL.md".to_string(), hash);
            m
        },
        bundle_hash: bundle,
        review: Review {
            status: "approved".to_string(),
            flags: vec![],
            summary: "t".to_string(),
            model: "m".to_string(),
            prompt_version: "1".to_string(),
            reviewed_at: "2026-01-01T00:00:00Z".to_string(),
        },
        security_check: None,
    };
    let mut skills = HashMap::new();
    skills.insert(skill_name.to_string(), entry.clone());
    let registry = Registry {
        version: 1,
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        reviewer: Reviewer {
            model: "t".to_string(),
            prompt_version: "1".to_string(),
        },
        skills,
    };
    fs::write(
        reg_dir.join("index.json"),
        serde_json::to_string_pretty(&registry).unwrap(),
    )
    .unwrap();
    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path(format!("/{skill_name}/SKILL.md"));
        then.status(200).body(content);
    });
    let cache_root = tmp.path().join("cache-net-parity");
    let prev = std::env::var("SKILLINDEX_CACHE_DIR").ok();
    unsafe { std::env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };
    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        registry_base_url: Some(server.base_url()),
        ..Default::default()
    };
    let client = reqwest::Client::new();
    let result =
        install_skill_with_client("owner/repo/net-skill-parity", &[], &opts, &client).await;
    assert!(
        result.success,
        "install failed: {} {}",
        result.output, result.stderr
    );
    assert!(
        project_dir
            .join(".agents/skills/net-skill-parity/SKILL.md")
            .exists()
    );
    mock.assert();
    match prev {
        Some(v) => unsafe { std::env::set_var("SKILLINDEX_CACHE_DIR", v) },
        None => unsafe { std::env::remove_var("SKILLINDEX_CACHE_DIR") },
    }
}

// ── additional cross-language checks (dominios sin espejo) ──────
// Detection/workspace duplicates that used to live here were removed; the
// remaining checks cover units with no per-language mirror (frontend, gradle,
// dotnet layout, display, hash).

#[test]
fn parity_extra_frontend_extensions_loop() {
    let exts = vec![
        "page.html",
        "style.css",
        "app.vue",
        "comp.svelte",
        "jsx.jsx",
        "tsx.tsx",
    ];
    for rel in exts {
        let dir = tempdir().unwrap();
        write_file(dir.path(), rel, "x");
        assert!(has_web_frontend_files(dir.path(), 3), "failed for {rel}");
    }
}

#[test]
fn parity_extra_gradle_parse_variants() {
    let variants = vec![
        (r#"include("app")"#, vec!["app"]),
        (r#"include(":app",":lib:core")"#, vec!["app", "lib/core"]),
        ("include 'a', 'b'", vec!["a", "b"]),
    ];
    for (input, expected) in variants {
        assert_eq!(parse_settings_gradle_modules(input), expected);
    }
}

#[test]
fn parity_extra_dotnet_filters() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "App.csproj", "<Project>");
    write_file(dir.path(), "bin/Ignore.csproj", "<Project>");
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(paths.iter().any(|p| p.ends_with("App.csproj")));
    assert!(
        !paths
            .iter()
            .any(|p| p.to_string_lossy().contains("bin/Ignore"))
    );
}

#[test]
fn parity_extra_display_variants() {
    assert_eq!(
        format_skill_label("owner/repo/skill", false),
        "owner › skill"
    );
    assert!(wrap_text("a b c d e f", 2).len() >= 3);
    assert!(truncate_visible("longword", 4).contains('…'));
}

#[test]
fn parity_extra_installer_hash_reject_zip_case_insensitive() {
    assert!(is_disallowed_skill_file("X.ZIP"));
    assert!(is_disallowed_skill_file("a/B.ZiP"));
}

#[test]
fn parity_hash_bundle_two_files_references() {
    let h1 = sha256_buffer(b"skill content");
    let h2 = sha256_buffer(b"references notes");
    let bundle = bundle_hash(&[
        ("SKILL.md".to_string(), h1.clone()),
        ("references/notes.md".to_string(), h2.clone()),
    ]);
    let mut sorted = [
        ("SKILL.md".to_string(), h1),
        ("references/notes.md".to_string(), h2),
    ];
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let expected = sha256_buffer(
        sorted
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    );
    assert_eq!(bundle, expected);
}

#[test]
fn parity_gradle_parse_complex_colon_path() {
    assert_eq!(
        parse_settings_gradle_modules(r#"include(":a:b:c:d")"#),
        vec!["a/b/c/d"]
    );
}

#[test]
fn parity_dotnet_fsproj_and_sln() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "Lib.fsproj", "<Project>");
    write_file(dir.path(), "Solution.sln", "");
    let paths = dotnet_layout_candidate_paths(dir.path());
    assert!(paths.iter().any(|p| p.ends_with("Lib.fsproj")));
    assert!(paths.iter().any(|p| p.ends_with("Solution.sln")));
}

#[test]
fn parity_frontend_scss_and_jsx() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "styles/app.scss", ".a{}");
    assert!(has_web_frontend_files(dir.path(), 3));
    let dir2 = tempdir().unwrap();
    write_file(dir2.path(), "src/app.jsx", "jsx");
    assert!(has_web_frontend_files(dir2.path(), 3));
}

#[test]
fn parity_display_format_time_various() {
    use skillindex::display::format_time;
    assert_eq!(format_time(0), "0ms");
    assert_eq!(format_time(500), "500ms");
    assert_eq!(format_time(1500), "1.5s");
    assert_eq!(format_time(61_000), "1m 1s");
}

#[test]
fn parity_registry_is_disallowed_via_registry() {
    assert!(is_disallowed_skill_file("downloads/file.zip"));
    assert!(!is_disallowed_skill_file("SKILL.md"));
}

#[test]
fn parity_cache_bundle_hash_dir() {
    let dir = get_cache_registry_dir(
        "deadbeef1234567890deadbeef1234567890deadbeef1234567890deadbeef1234",
    );
    assert!(dir.to_string_lossy().contains("deadbeef"));
}

#[test]
fn parity_installer_normalize_and_bundle() {
    assert_eq!(normalize_registry_rel_path("a\\b/c"), "a/b/c");
    let bundle = bundle_hash(&[("a.md".to_string(), "h1".to_string())]);
    assert_eq!(bundle, sha256_buffer("a.md:h1".as_bytes()));
}

#[test]
fn parity_frontend_depth_boundary_exact() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "a/b/c/App.vue", "<template></template>"); // depth 3 from root (a=1,b=2,c=3)
    assert!(has_web_frontend_files(dir.path(), 3));
    let dir2 = tempdir().unwrap();
    write_file(dir2.path(), "a/b/c/d/App.vue", "<template></template>"); // depth 4
    assert!(!has_web_frontend_files(dir2.path(), 3));
}
