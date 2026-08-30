//! Integration tests for technology detection.
//! Mirrors detect.test.ts — one test per `it(...)` block.

use skillindex::detect::{DetectResult, detect_technologies};
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

fn write_pkg(base: &Path, json: &str) {
    fs::write(base.join("package.json"), json).unwrap();
}

fn write_json(base: &Path, rel: &str, data: serde_json::Value) {
    write_file(base, rel, &serde_json::to_string(&data).unwrap());
}

fn add_workspace(root: &Path, ws: &str) {
    let p = root.join(ws);
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("package.json"), "{}").unwrap();
}

fn ids(r: &DetectResult) -> Vec<&str> {
    r.detected.iter().map(|t| t.id.as_str()).collect()
}

// ── detectTechnologies — basic package detection ──────────────────

#[test]
fn returns_empty_when_no_package_json_or_config_files() {
    let dir = tempdir().unwrap();
    let r = detect_technologies(dir.path());
    assert!(r.detected.is_empty());
}

#[test]
fn detects_react_from_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"react":"^19","react-dom":"^19"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"react"));
}

#[test]
fn detects_nextjs_from_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"next":"^15"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"nextjs"));
}

#[test]
fn detects_nextjs_from_config_file_without_package() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "next.config.mjs", "export default {}");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"nextjs"));
}

#[test]
fn detects_vue_from_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"vue":"^3"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"vue"));
}

#[test]
fn detects_typescript_from_tsconfig() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "tsconfig.json", "{}");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"typescript"));
}

#[test]
fn detects_azure_from_scoped_package() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"@azure/storage-blob":"^12"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"azure"));
}

#[test]
fn detects_aws_from_scoped_package() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"@aws-sdk/client-s3":"^3"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"aws"));
}

#[test]
fn detects_tailwind_from_dev_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"devDependencies":{"tailwindcss":"^4"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"tailwind"));
}

#[test]
fn detects_tailwind_from_vite_plugin() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"@tailwindcss/vite":"^4"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"tailwind"));
}

#[test]
fn detects_zod_from_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"zod":"^4.3.6"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"zod"));
}

// ── Deno ─────────────────────────────────────────────────────────

#[test]
fn detects_react_from_deno_json_npm_import() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"imports":{"react":"npm:react@^19","react-dom":"npm:react-dom@^19"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"react"));
}

#[test]
fn detects_hono_from_deno_json_npm_import() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"imports":{"hono":"npm:hono@^4"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"hono"));
}

#[test]
fn detects_supabase_from_deno_json_scoped_import() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"imports":{"@supabase/supabase-js":"npm:@supabase/supabase-js@^2"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"supabase"));
}

#[test]
fn detects_frontend_from_deno_json_imports() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"imports":{"react":"npm:react@^19"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(r.is_frontend);
}

#[test]
fn merges_package_json_and_deno_json_dependencies() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"next":"^15"}}"#);
    write_json(
        dir.path(),
        "deno.json",
        serde_json::json!({"imports":{"react":"npm:react@^19"}}),
    );
    let r = detect_technologies(dir.path());
    let i = ids(&r);
    assert!(i.contains(&"nextjs"));
    assert!(i.contains(&"react"));
}

// ── Go ───────────────────────────────────────────────────────────

#[test]
fn detects_go_from_go_mod() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "go.mod",
        "module example.com/test\n\ngo 1.24.0\n",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"go"));
}

#[test]
fn detects_go_from_go_work() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "go.work", "go 1.24.0\n");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"go"));
}

#[test]
fn does_not_detect_go_without_go_files() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    let r = detect_technologies(dir.path());
    assert!(!ids(&r).contains(&"go"));
}

// ── Rust ─────────────────────────────────────────────────────────

#[test]
fn detects_rust_from_cargo_toml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "Cargo.toml",
        "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"rust"));
}

#[test]
fn rust_detection_returns_correct_skills() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "Cargo.toml", "[package]\nname = \"my-crate\"");
    let r = detect_technologies(dir.path());
    let rust = r.detected.iter().find(|t| t.id == "rust").unwrap();
    assert!(
        rust.skills
            .iter()
            .any(|s| s.contains("rust-best-practices"))
    );
}

// ── Python ────────────────────────────────────────────────────────

#[test]
fn detects_python_from_pyproject_toml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pyproject.toml",
        "[tool.poetry]\nname = \"test\"",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"python"));
}

#[test]
fn detects_python_from_requirements_txt() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "requirements.txt", "requests==2.31.0");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"python"));
}

#[test]
fn detects_fastapi_and_pydantic_from_requirements() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "requirements.txt",
        "fastapi==0.100.0\npydantic==2.0.0",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"fastapi"));
    assert!(ids(&r).contains(&"pydantic"));
}

#[test]
fn detects_django_from_pyproject_toml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pyproject.toml",
        "[tool.poetry.dependencies]\nDjango = \"^5.0\"",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"django"));
}

#[test]
fn detects_flask_from_requirements_txt() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "requirements.txt", "Flask>=2.0.0");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"flask"));
}

// ── PHP / Laravel ─────────────────────────────────────────────────

#[test]
fn detects_php_from_composer_json() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "composer.json",
        serde_json::json!({"require":{"php":">=8.2"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"php"));
}

#[test]
fn detects_laravel_from_artisan_file() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "artisan",
        "#!/usr/bin/env php\n<?php\ndefine('LARAVEL_START', microtime(true));",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"laravel"));
}

#[test]
fn detects_laravel_from_composer_json_framework() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "composer.json",
        serde_json::json!({"require":{"laravel/framework":"^11.0"}}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"laravel"));
    assert!(ids(&r).contains(&"php"));
}

// ── Kotlin / Android / Java ───────────────────────────────────────

#[test]
fn detects_kotlin_multiplatform_from_root_build_gradle() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "build.gradle.kts",
        r#"plugins { kotlin("multiplatform") version "2.0.0" }"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"kotlin-multiplatform"));
}

#[test]
fn detects_android_from_app_build_gradle() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "app/build.gradle.kts",
        r#"plugins { id("com.android.application") }"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"android"));
}

#[test]
fn detects_java_from_pom_xml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pom.xml",
        "<project><groupId>com.example</groupId></project>",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"java"));
}

#[test]
fn detects_spring_boot_from_application_properties() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "src/main/resources/application.properties",
        "server.port=8080",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"springboot"));
}

#[test]
fn detects_dotnet_from_global_json() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "global.json",
        r#"{"sdk":{"version":"8.0.100"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"dotnet"));
}

#[test]
fn detects_csharp_from_csproj() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "MyProject.csproj",
        r#"<Project Sdk="Microsoft.NET.Sdk">"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"csharp"));
    assert!(ids(&r).contains(&"dotnet"));
}

#[test]
fn skips_bin_and_obj_when_scanning_dotnet() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "bin/Debug/net8.0/ExcludeMe.csproj",
        r#"<Project Sdk="Microsoft.NET.Sdk">"#,
    );
    let r = detect_technologies(dir.path());
    assert!(!ids(&r).contains(&"csharp"));
}

// ── Clerk ────────────────────────────────────────────────────────

#[test]
fn detects_clerk_from_clerk_nextjs() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"@clerk/nextjs":"^6"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"clerk"));
}

#[test]
fn detects_clerk_from_any_scoped_clerk_package() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"@clerk/expo":"^2"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"clerk"));
}

#[test]
fn clerk_detection_returns_correct_skills() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"@clerk/nextjs":"^6"}}"#);
    let r = detect_technologies(dir.path());
    let clerk = r.detected.iter().find(|t| t.id == "clerk").unwrap();
    assert!(
        clerk
            .skills
            .iter()
            .any(|s| s.contains("clerk/skills/clerk"))
    );
    assert!(clerk.skills.iter().any(|s| s.contains("clerk-setup")));
}

// ── Combos ───────────────────────────────────────────────────────

#[test]
fn detects_react_hook_form_zod_combo() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"react-hook-form":"^7.58.0","zod":"^4.3.6"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(r.combos.iter().any(|c| c.id == "react-hook-form-zod"));
}

#[test]
fn no_combo_when_only_one_tech_of_pair() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"expo":"^52"}}"#);
    let r = detect_technologies(dir.path());
    assert!(!r.combos.iter().any(|c| c.id == "expo-tailwind"));
}

#[test]
fn detects_expo_tailwind_combo() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"expo":"^52","tailwindcss":"^4"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(r.combos.iter().any(|c| c.id == "expo-tailwind"));
}

#[test]
fn detects_nextjs_clerk_combo() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"next":"^15","@clerk/nextjs":"^6"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(r.combos.iter().any(|c| c.id == "nextjs-clerk"));
}

// ── isFrontend flag ──────────────────────────────────────────────

#[test]
fn marks_frontend_projects_correctly() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"react":"^19"}}"#);
    let r = detect_technologies(dir.path());
    assert!(r.is_frontend);
}

#[test]
fn marks_non_frontend_projects_correctly() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"express":"^4"}}"#);
    let r = detect_technologies(dir.path());
    assert!(!r.is_frontend);
}

#[test]
fn marks_frontend_from_file_scan() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "src/App.vue", "<template></template>");
    let r = detect_technologies(dir.path());
    assert!(r.is_frontend);
}

// ── Dart / Flutter ────────────────────────────────────────────────

#[test]
fn detects_dart_from_pubspec_yaml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pubspec.yaml",
        "name: dart_app\ndescription: A Dart CLI tool\nenvironment:\n  sdk: '^3.2.0'",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"dart"));
}

#[test]
fn detects_flutter_from_pubspec_yaml_with_flutter_key() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pubspec.yaml",
        "name: flutter_app\nflutter:\n  uses-material-design: true",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"flutter"));
}

#[test]
fn detects_both_dart_and_flutter_for_flutter_projects() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pubspec.yaml",
        "name: flutter_app\nenvironment:\n  sdk: '^3.2.0'\nflutter:\n  uses-material-design: true",
    );
    let r = detect_technologies(dir.path());
    let i = ids(&r);
    assert!(i.contains(&"dart"), "Dart should be detected");
    assert!(i.contains(&"flutter"), "Flutter should be detected");
}

#[test]
fn detects_only_dart_when_no_flutter_key() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pubspec.yaml",
        "name: dart_cli\nenvironment:\n  sdk: '^3.2.0'\ndependencies:\n  args: ^2.4.0",
    );
    let r = detect_technologies(dir.path());
    let i = ids(&r);
    assert!(i.contains(&"dart"));
    assert!(!i.contains(&"flutter"));
}

// ── Terraform ────────────────────────────────────────────────────

#[test]
fn detects_terraform_from_lock_file() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        ".terraform.lock.hcl",
        "# This file is maintained automatically by terraform",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"terraform"));
}

#[test]
fn detects_terraform_from_tf_files() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "main.tf",
        "terraform {\n  required_version = \">= 1.0\"\n}",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"terraform"));
}

// ── Monorepo ─────────────────────────────────────────────────────

#[test]
fn detects_technologies_from_workspace_subpackages() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"workspaces":["packages/*"]}"#);
    add_workspace(dir.path(), "packages/web");
    fs::write(
        dir.path().join("packages/web/package.json"),
        r#"{"dependencies":{"next":"^15","react":"^19"}}"#,
    )
    .unwrap();
    let r = detect_technologies(dir.path());
    let i = ids(&r);
    assert!(i.contains(&"nextjs"));
    assert!(i.contains(&"react"));
}

#[test]
fn deduplicates_technologies_across_workspaces() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"workspaces":["packages/*"]}"#);
    for ws in &["packages/ui", "packages/app"] {
        add_workspace(dir.path(), ws);
        fs::write(
            dir.path().join(ws).join("package.json"),
            r#"{"dependencies":{"react":"^19"}}"#,
        )
        .unwrap();
    }
    let r = detect_technologies(dir.path());
    let count = r.detected.iter().filter(|t| t.id == "react").count();
    assert_eq!(count, 1, "react should appear only once");
}

#[test]
fn detects_combos_across_workspaces() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"next":"^15"},"workspaces":["packages/*"]}"#,
    );
    add_workspace(dir.path(), "packages/db");
    fs::write(
        dir.path().join("packages/db/package.json"),
        r#"{"dependencies":{"@supabase/supabase-js":"^2"}}"#,
    )
    .unwrap();
    let r = detect_technologies(dir.path());
    assert!(
        r.combos.iter().any(|c| c.id == "nextjs-supabase"),
        "cross-workspace combo should be detected"
    );
}

// ── Bash ─────────────────────────────────────────────────────────

#[test]
fn detects_bash_from_sh_files() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "scripts/deploy.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\n",
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"bash"));
}

#[test]
fn ignores_bash_inside_node_modules() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(
        dir.path(),
        "node_modules/package/postinstall.sh",
        "#!/usr/bin/env bash\n",
    );
    let r = detect_technologies(dir.path());
    assert!(!ids(&r).contains(&"bash"));
}

// ── Chrome Extension ─────────────────────────────────────────────

#[test]
fn detects_chrome_extension_from_manifest_with_manifest_version() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "manifest.json",
        serde_json::json!({"manifest_version":3,"name":"My Extension","version":"1.0"}),
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"chrome-extension"));
}

#[test]
fn does_not_detect_chrome_extension_from_manifest_without_manifest_version() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "manifest.json",
        serde_json::json!({"name":"My PWA","short_name":"PWA","start_url":"/"}),
    );
    let r = detect_technologies(dir.path());
    assert!(!ids(&r).contains(&"chrome-extension"));
}

// ── React Router / TanStack ───────────────────────────────────────

#[test]
fn detects_react_router_from_package() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"dependencies":{"react-router":"^7"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"react-router"));
}

#[test]
fn detects_tanstack_start_from_package() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"@tanstack/react-start":"^1"}}"#,
    );
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"tanstack-start"));
}

// ── shadcn / Tauri / Electron ─────────────────────────────────────

#[test]
fn detects_shadcn_from_components_json() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "components.json", "{}");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"shadcn"));
}

#[test]
fn detects_tauri_from_src_tauri_config() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "src-tauri/tauri.conf.json", "{}");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"tauri"));
}

#[test]
fn detects_electron_from_package() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), r#"{"devDependencies":{"electron":"^30"}}"#);
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"electron"));
}

#[test]
fn detects_electron_from_builder_config() {
    let dir = tempdir().unwrap();
    write_pkg(dir.path(), "{}");
    write_file(dir.path(), "electron-builder.yml", "productName: My App");
    let r = detect_technologies(dir.path());
    assert!(ids(&r).contains(&"electron"));
}

// ── Multiple techs at once ────────────────────────────────────────

#[test]
fn detects_multiple_technologies_simultaneously() {
    let dir = tempdir().unwrap();
    write_pkg(
        dir.path(),
        r#"{"dependencies":{"next":"^15","react":"^19","react-dom":"^19"},"devDependencies":{"typescript":"^5","@playwright/test":"^1.40"}}"#,
    );
    write_file(dir.path(), "tsconfig.json", "{}");
    let r = detect_technologies(dir.path());
    let i = ids(&r);
    assert!(i.contains(&"react"));
    assert!(i.contains(&"nextjs"));
    assert!(i.contains(&"typescript"));
    assert!(i.contains(&"playwright"));
}
