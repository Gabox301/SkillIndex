//! Integration tests for workspace resolution.
//! Mirrors workspace.test.ts — one test per `it(...)` block.

use skillindex::workspace::resolve_workspaces;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// ── Helpers ────────────────────────────────────────────────────────

fn write_file(base: &Path, rel: &str, content: &str) {
    let p = base.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

fn write_json(base: &Path, rel: &str, data: serde_json::Value) {
    write_file(base, rel, &serde_json::to_string(&data).unwrap());
}

fn add_workspace(root: &Path, ws: &str) {
    let p = root.join(ws);
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("package.json"), "{}").unwrap();
}

// ── resolveWorkspaces ─────────────────────────────────────────────

#[test]
fn returns_empty_for_non_monorepo_project() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", r#"{"name":"single"}"#);
    assert!(resolve_workspaces(dir.path()).is_empty());
}

#[test]
fn returns_empty_when_no_package_json_exists() {
    let dir = tempdir().unwrap();
    assert!(resolve_workspaces(dir.path()).is_empty());
}

#[test]
fn detects_npm_yarn_workspaces_array_format() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["packages/*"]}"#,
    );
    add_workspace(dir.path(), "packages/app-a");
    add_workspace(dir.path(), "packages/app-b");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|d| d.to_string_lossy().contains("app-a")));
    assert!(result.iter().any(|d| d.to_string_lossy().contains("app-b")));
}

#[test]
fn detects_npm_yarn_workspaces_object_format_with_packages_key() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":{"packages":["packages/*"]}}"#,
    );
    add_workspace(dir.path(), "packages/lib");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("lib"));
}

#[test]
fn detects_pnpm_workspace_yaml() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", "{}");
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n  - apps/*\n",
    );
    add_workspace(dir.path(), "packages/ui");
    add_workspace(dir.path(), "apps/web");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|d| d.to_string_lossy().contains("ui")));
    assert!(result.iter().any(|d| d.to_string_lossy().contains("web")));
}

#[test]
fn pnpm_workspace_yaml_takes_precedence_over_package_json_workspaces() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", r#"{"workspaces":["other/*"]}"#);
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );
    add_workspace(dir.path(), "packages/core");
    add_workspace(dir.path(), "other/ignored");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("core"));
}

#[test]
fn skips_directories_without_package_json() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["packages/*"]}"#,
    );
    add_workspace(dir.path(), "packages/has-pkg");
    // Directory without package.json
    fs::create_dir_all(dir.path().join("packages/no-pkg")).unwrap();
    write_file(dir.path(), "packages/no-pkg/.gitkeep", "");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("has-pkg"));
}

#[test]
fn skips_scan_skip_dirs_like_node_modules() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["packages/*"]}"#,
    );
    add_workspace(dir.path(), "packages/node_modules");
    add_workspace(dir.path(), "packages/real-pkg");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("real-pkg"));
}

#[test]
fn handles_multiple_patterns() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["packages/*","apps/*","tools/*"]}"#,
    );
    add_workspace(dir.path(), "packages/ui");
    add_workspace(dir.path(), "apps/web");
    assert_eq!(resolve_workspaces(dir.path()).len(), 2);
}

#[test]
fn handles_exact_directory_references_no_glob() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["tools/special-tool"]}"#,
    );
    add_workspace(dir.path(), "tools/special-tool");
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("special-tool"));
}

#[test]
fn handles_pnpm_workspace_yaml_with_quoted_patterns() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - 'packages/*'\n  - \"apps/*\"\n",
    );
    add_workspace(dir.path(), "packages/a");
    add_workspace(dir.path(), "apps/b");
    assert_eq!(resolve_workspaces(dir.path()).len(), 2);
}

#[test]
fn returns_empty_for_pnpm_workspace_yaml_without_packages_key() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "# empty config\nsome_other_key:\n  - foo\n",
    );
    assert!(resolve_workspaces(dir.path()).is_empty());
}

#[test]
fn returns_empty_for_empty_workspaces_array() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", r#"{"workspaces":[]}"#);
    assert!(resolve_workspaces(dir.path()).is_empty());
}

#[test]
fn detects_deno_workspace_members_from_deno_json() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"workspace":["./api","./shared"]}),
    );
    fs::create_dir_all(dir.path().join("api")).unwrap();
    fs::write(dir.path().join("api/deno.json"), "{}").unwrap();
    fs::create_dir_all(dir.path().join("shared")).unwrap();
    fs::write(dir.path().join("shared/deno.json"), "{}").unwrap();
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|d| d.to_string_lossy().contains("api")));
    assert!(
        result
            .iter()
            .any(|d| d.to_string_lossy().contains("shared"))
    );
}

#[test]
fn deno_workspace_members_with_deno_jsonc_are_detected() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"workspace":["./lib"]}),
    );
    fs::create_dir_all(dir.path().join("lib")).unwrap();
    fs::write(dir.path().join("lib/deno.jsonc"), "{}").unwrap();
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("lib"));
}

#[test]
fn pnpm_workspace_yaml_takes_precedence_over_deno_json_workspace() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", "{}");
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"workspace":["./deno-member"]}),
    );
    add_workspace(dir.path(), "packages/core");
    fs::create_dir_all(dir.path().join("deno-member")).unwrap();
    fs::write(dir.path().join("deno-member/deno.json"), "{}").unwrap();
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("core"));
}

#[test]
fn package_json_workspaces_take_precedence_over_deno_json_workspace() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "package.json",
        r#"{"workspaces":["packages/*"]}"#,
    );
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"workspace":["./deno-member"]}),
    );
    add_workspace(dir.path(), "packages/ui");
    fs::create_dir_all(dir.path().join("deno-member")).unwrap();
    fs::write(dir.path().join("deno-member/deno.json"), "{}").unwrap();
    let result = resolve_workspaces(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result[0].to_string_lossy().contains("ui"));
}
