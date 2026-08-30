use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use crate::detect::SCAN_SKIP_DIRS;

/// Files scanned at project root for Gradle detection — mirrors GRADLE_SCAN_ROOT_FILES in lib.ts
pub static GRADLE_SCAN_ROOT_FILES: &[&str] = &[
    "build.gradle.kts",
    "build.gradle",
    "settings.gradle.kts",
    "settings.gradle",
    "gradle/libs.versions.toml",
];

static GRADLE_CACHE: LazyLock<Mutex<HashMap<String, Vec<PathBuf>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Verbatim port of `parseSettingsGradleModules` in lib.ts
/// Regex `/include\s*\(?\s*([^)]+)/g` + inner quoted strings, then `: -> /` handling
pub fn parse_settings_gradle_modules(content: &str) -> Vec<String> {
    let mut modules = Vec::new();
    let mut i = 0;
    let bytes_len = content.len();

    while i < bytes_len {
        let remaining = &content[i..];
        let Some(rel_pos) = remaining.find("include") else {
            break;
        };
        let start = i + rel_pos;
        let mut cursor = start + "include".len();

        // \s* — skip whitespace (space, tab, newline, etc.)
        while cursor < bytes_len {
            let ch = content[cursor..].chars().next().unwrap();
            if ch.is_whitespace() {
                cursor += ch.len_utf8();
            } else {
                break;
            }
        }

        // \(? — optional '('
        if cursor < bytes_len && content[cursor..].starts_with('(') {
            cursor += 1;
            while cursor < bytes_len {
                let ch = content[cursor..].chars().next().unwrap();
                if ch.is_whitespace() {
                    cursor += ch.len_utf8();
                } else {
                    break;
                }
            }
        }

        // ([^)]+) — capture until next ')' or end
        let capture_start = cursor;
        if capture_start >= bytes_len || content[capture_start..].starts_with(')') {
            // No capture (empty or immediate ')'), advance and continue
            i = cursor + 1;
            continue;
        }

        let capture_end = if let Some(close) = content[capture_start..].find(')') {
            capture_start + close
        } else {
            bytes_len
        };

        let args = &content[capture_start..capture_end];

        // inner quotedRe /['"]([^'"]+)['"]/g
        let mut q = 0;
        while q < args.len() {
            let single = args[q..].find('\'');
            let double = args[q..].find('"');
            let next = match (single, double) {
                (Some(s), Some(d)) => {
                    if s < d {
                        Some((s, '\''))
                    } else {
                        Some((d, '"'))
                    }
                }
                (Some(s), None) => Some((s, '\'')),
                (None, Some(d)) => Some((d, '"')),
                (None, None) => None,
            };
            let Some((rel_idx, quote_char)) = next else {
                break;
            };
            let open = q + rel_idx;
            let after_open = open + 1;
            if after_open >= args.len() {
                break;
            }
            let Some(close_rel) = args[after_open..].find(quote_char) else {
                break;
            };
            let close = after_open + close_rel;
            let inner = &args[after_open..close];
            let mut module = inner.to_string();
            if module.starts_with(':') {
                module = module[1..].to_string();
            }
            module = module.replace(':', "/");
            modules.push(module);
            q = close + 1;
        }

        if capture_end < bytes_len {
            // had a ')', continue after it
            i = capture_end + 1;
        } else {
            // captured to end, no more includes
            break;
        }
    }

    modules
}

/// Helper to join a project dir with a slash-separated relative path in a platform-safe way
fn join_relative(base: &Path, relative: &str) -> PathBuf {
    let mut p = base.to_path_buf();
    for part in relative.split('/') {
        if !part.is_empty() {
            p = p.join(part);
        }
    }
    p
}

/// Cached candidate paths for Gradle layout — mirrors `gradleLayoutCandidatePaths` in lib.ts
pub fn gradle_layout_candidate_paths(project_dir: &Path) -> Vec<PathBuf> {
    let key = project_dir.to_string_lossy().to_string();
    {
        let cache = GRADLE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    let mut add = |p: PathBuf| {
        if seen.insert(p.clone()) {
            candidates.push(p);
        }
    };

    for f in GRADLE_SCAN_ROOT_FILES {
        add(join_relative(project_dir, f));
    }

    // Scan immediate subdirectories for build.gradle.kts / build.gradle
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || SCAN_SKIP_DIRS.contains(name.as_str()) {
                continue;
            }
            for g in ["build.gradle.kts", "build.gradle"] {
                add(project_dir.join(&name).join(g));
            }
        }
    }

    // Parse settings.gradle.kts / settings.gradle for module includes
    for settings_file in ["settings.gradle.kts", "settings.gradle"] {
        let settings_path = project_dir.join(settings_file);
        let content = match fs::read_to_string(&settings_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for module_path in parse_settings_gradle_modules(&content) {
            for g in ["build.gradle.kts", "build.gradle"] {
                add(join_relative(project_dir, &module_path).join(g));
            }
        }
        break;
    }

    let mut cache = GRADLE_CACHE.lock().unwrap();
    cache.insert(key, candidates.clone());
    candidates
}

/// Test helper to clear cache between isolated tests
#[cfg(test)]
pub fn clear_gradle_cache() {
    GRADLE_CACHE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(base: &Path, rel: &str, content: &str) {
        let p = base.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, content).unwrap();
    }

    // ── parse_settings_gradle_modules parity with detect.test.ts ──

    #[test]
    fn extracts_module_from_kotlin_dsl_include() {
        let modules = parse_settings_gradle_modules(r#"include("app")"#);
        assert_eq!(modules, vec!["app"]);
    }

    #[test]
    fn extracts_module_from_groovy_include() {
        let modules = parse_settings_gradle_modules("include 'app'");
        assert_eq!(modules, vec!["app"]);
    }

    #[test]
    fn strips_leading_colon() {
        let modules = parse_settings_gradle_modules(r#"include(":app")"#);
        assert_eq!(modules, vec!["app"]);
    }

    #[test]
    fn converts_colon_separated_to_fs_path() {
        let modules = parse_settings_gradle_modules(r#"include(":feature:login")"#);
        assert_eq!(modules, vec!["feature/login"]);
    }

    #[test]
    fn handles_multiple_modules_groovy() {
        let modules = parse_settings_gradle_modules("include 'app', 'core', 'data'");
        assert_eq!(modules, vec!["app", "core", "data"]);
    }

    #[test]
    fn handles_multiple_modules_kotlin_dsl() {
        let modules = parse_settings_gradle_modules(r#"include(":app", ":core", ":data")"#);
        assert_eq!(modules, vec!["app", "core", "data"]);
    }

    #[test]
    fn handles_multiline_include_block() {
        let content = "include(\n  \":app\",\n  \":core\",\n  \":shared:data\"\n)";
        assert_eq!(
            parse_settings_gradle_modules(content),
            vec!["app", "core", "shared/data"]
        );
    }

    #[test]
    fn handles_multiple_separate_includes() {
        let content = "include(\":app\")\ninclude(\":core\")";
        assert_eq!(parse_settings_gradle_modules(content), vec!["app", "core"]);
    }

    #[test]
    fn returns_empty_when_no_includes() {
        let content = "rootProject.name = \"my-app\"\npluginManagement { }";
        assert!(parse_settings_gradle_modules(content).is_empty());
    }

    #[test]
    fn returns_empty_for_empty_content() {
        assert!(parse_settings_gradle_modules("").is_empty());
    }

    #[test]
    fn ignores_non_include_content_around_includes() {
        let content = "rootProject.name = \"my-app\"\npluginManagement {\n    repositories { google() }\n}\ninclude(\":app\")";
        assert_eq!(parse_settings_gradle_modules(content), vec!["app"]);
    }

    #[test]
    fn spec_gradle_include_app_lib_core() {
        // Direct from spec: include(":app",":lib:core") -> ["app","lib/core"]
        let modules = parse_settings_gradle_modules(r#"include(":app",":lib:core")"#);
        assert_eq!(modules, vec!["app", "lib/core"]);
    }

    // ── gradle_layout_candidate_paths ──

    #[test]
    fn gradle_layout_includes_root_files() {
        let dir = tempdir().unwrap();
        let paths = gradle_layout_candidate_paths(dir.path());
        // Must contain 5 root entries even if files don't exist
        assert_eq!(paths.len(), 5);
        assert!(paths.iter().any(|p| p.ends_with("build.gradle.kts")));
        assert!(
            paths
                .iter()
                .any(|p| p.ends_with("gradle/libs.versions.toml"))
        );
    }

    #[test]
    fn gradle_layout_includes_subdir_build_files() {
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
    fn gradle_layout_skips_dot_and_skip_dirs() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        fs::write(dir.path().join(".hidden/build.gradle.kts"), "").unwrap();
        fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        fs::write(dir.path().join("node_modules/build.gradle.kts"), "").unwrap();
        let paths = gradle_layout_candidate_paths(dir.path());
        // Should not contain hidden or node_modules entries
        assert!(
            !paths
                .iter()
                .any(|p| p.to_string_lossy().contains(".hidden"))
        );
        assert!(
            !paths
                .iter()
                .any(|p| p.to_string_lossy().contains("node_modules"))
        );
    }

    #[test]
    fn gradle_layout_parses_settings_kts() {
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
        assert!(
            paths
                .iter()
                .any(|p| p.ends_with("feature/login/build.gradle"))
        );
    }

    #[test]
    fn gradle_layout_parses_settings_gradle_fallback() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "settings.gradle", "include 'shared'");
        let paths = gradle_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("shared/build.gradle.kts")));
    }

    #[test]
    fn gradle_layout_cache_returns_same_instance() {
        let dir = tempdir().unwrap();
        let first = gradle_layout_candidate_paths(dir.path());
        // Create a new file after first call — cached result should NOT include it
        fs::create_dir_all(dir.path().join("newMod")).unwrap();
        fs::write(dir.path().join("newMod/build.gradle.kts"), "").unwrap();
        let second = gradle_layout_candidate_paths(dir.path());
        assert_eq!(first, second);
        clear_gradle_cache();
        // After clear, new file should appear
        let third = gradle_layout_candidate_paths(dir.path());
        assert!(third.iter().any(|p| p.ends_with("newMod/build.gradle.kts")));
        clear_gradle_cache();
    }

    #[test]
    fn gradle_layout_deduplicates() {
        let dir = tempdir().unwrap();
        // Create settings that includes a module that is also a direct subdir
        fs::create_dir_all(dir.path().join("app")).unwrap();
        fs::write(dir.path().join("app/build.gradle.kts"), "").unwrap();
        write_file(dir.path(), "settings.gradle.kts", r#"include(":app")"#);
        let paths = gradle_layout_candidate_paths(dir.path());
        // app/build.gradle.kts should appear only once despite being added via subdir scan and via settings
        let count = paths
            .iter()
            .filter(|p| p.ends_with("app/build.gradle.kts"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn gradle_layout_settings_kts_takes_precedence_over_gradle() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "settings.gradle.kts", r#"include(":a")"#);
        write_file(dir.path(), "settings.gradle", r#"include(":b")"#);
        let paths = gradle_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("a/build.gradle.kts")));
        assert!(!paths.iter().any(|p| p.ends_with("b/build.gradle.kts")));
    }
}
