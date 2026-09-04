use std::fs;
use std::path::{Path, PathBuf};

use crate::detect::SCAN_SKIP_DIRS;

/// Parse a minimal `pnpm-workspace.yaml` — only handles `packages:` + `- ` lines, like lib.ts
fn parse_pnpm_workspace_yaml(content: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut in_packages = false;

    for raw in content.lines() {
        let line = raw.trim();
        if line == "packages:" || line == "packages :" {
            in_packages = true;
            continue;
        }
        if in_packages {
            if let Some(stripped) = line.strip_prefix("- ") {
                let pat = stripped.trim().trim_matches(|c| c == '\'' || c == '"');
                patterns.push(pat.to_string());
            } else if line.is_empty() || line.starts_with('#') {
                continue;
            } else if !line.is_empty() {
                break;
            }
        }
    }

    patterns
}

fn expand_workspace_patterns(project_dir: &Path, patterns: &[String]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    for pattern in patterns {
        if pattern.contains('*') {
            // parent = pattern without /* suffix
            let parent_rel = pattern
                .rfind('*')
                .map(|idx| {
                    let prefix = &pattern[..idx];
                    prefix
                        .trim_end_matches('/')
                        .trim_end_matches('*')
                        .trim_end_matches('/')
                })
                .unwrap_or("");
            let parent = if parent_rel.is_empty() {
                project_dir.to_path_buf()
            } else {
                project_dir.join(parent_rel)
            };

            let entries = match fs::read_dir(&parent) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let file_type = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };
                if !file_type.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if SCAN_SKIP_DIRS.contains(name.as_str()) || name.starts_with('.') {
                    continue;
                }
                let ws_dir = parent.join(&name);
                if ws_dir.join("package.json").exists()
                    || ws_dir.join("deno.json").exists()
                    || ws_dir.join("deno.jsonc").exists()
                {
                    dirs.push(ws_dir);
                }
            }
        } else {
            let ws_dir = project_dir.join(pattern);
            if ws_dir.join("package.json").exists()
                || ws_dir.join("deno.json").exists()
                || ws_dir.join("deno.jsonc").exists()
            {
                dirs.push(ws_dir);
            }
        }
    }

    dirs
}

fn read_package_json_workspaces(project_dir: &Path) -> Option<Vec<String>> {
    let data = fs::read_to_string(project_dir.join("package.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    let ws = v.get("workspaces")?;
    if let Some(arr) = ws.as_array() {
        let mut out = Vec::new();
        for item in arr {
            if let Some(s) = item.as_str() {
                out.push(s.to_string());
            }
        }
        if !out.is_empty() {
            return Some(out);
        }
    } else if let Some(obj) = ws.as_object()
        && let Some(arr) = obj.get("packages").and_then(|x| x.as_array())
    {
        let mut out = Vec::new();
        for item in arr {
            if let Some(s) = item.as_str() {
                out.push(s.to_string());
            }
        }
        if !out.is_empty() {
            return Some(out);
        }
    }
    None
}

fn read_deno_json_workspaces(project_dir: &Path) -> Option<Vec<String>> {
    for name in ["deno.json", "deno.jsonc"] {
        if let Ok(data) = fs::read_to_string(project_dir.join(name))
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&data)
            && let Some(arr) = v.get("workspace").and_then(|x| x.as_array())
        {
            let mut out = Vec::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    out.push(s.to_string());
                }
            }
            if !out.is_empty() {
                return Some(out);
            }
        }
    }
    None
}

/// Resolve workspaces with precedence: `pnpm-workspace.yaml` > `package.json` > `deno.json`
/// Mirrors `resolveWorkspaces` in lib.ts
pub fn resolve_workspaces(project_dir: &Path) -> Vec<PathBuf> {
    // 1. pnpm-workspace.yaml
    let pnpm_path = project_dir.join("pnpm-workspace.yaml");
    if pnpm_path.exists()
        && let Ok(content) = fs::read_to_string(&pnpm_path)
    {
        let patterns = parse_pnpm_workspace_yaml(&content);
        if !patterns.is_empty() {
            return expand_workspace_patterns(project_dir, &patterns)
                .into_iter()
                .filter(|p| {
                    // filter resolve(d) !== resolve(root)
                    p.canonicalize().ok() != project_dir.canonicalize().ok()
                })
                .collect();
        }
    }

    // 2. package.json workspaces
    if let Some(patterns) = read_package_json_workspaces(project_dir) {
        return expand_workspace_patterns(project_dir, &patterns)
            .into_iter()
            .filter(|p| p.canonicalize().ok() != project_dir.canonicalize().ok())
            .collect();
    }

    // 3. deno.json workspace
    if let Some(patterns) = read_deno_json_workspaces(project_dir) {
        return expand_workspace_patterns(project_dir, &patterns)
            .into_iter()
            .filter(|p| p.canonicalize().ok() != project_dir.canonicalize().ok())
            .collect();
    }

    Vec::new()
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

    fn add_workspace(root: &Path, rel: &str) {
        let p = root.join(rel);
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("package.json"), "{}").unwrap();
    }

    #[test]
    fn returns_empty_for_non_monorepo() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "package.json", r#"{"name":"single"}"#);
        assert!(resolve_workspaces(dir.path()).is_empty());
    }

    #[test]
    fn returns_empty_when_no_package_json() {
        let dir = tempdir().unwrap();
        assert!(resolve_workspaces(dir.path()).is_empty());
    }

    #[test]
    fn detects_npm_workspaces_array_format() {
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
        assert!(result.iter().any(|p| p.ends_with("app-a")));
        assert!(result.iter().any(|p| p.ends_with("app-b")));
    }

    #[test]
    fn detects_npm_workspaces_object_format() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "package.json",
            r#"{"workspaces":{"packages":["packages/*"]}}"#,
        );
        add_workspace(dir.path(), "packages/lib");
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("lib"));
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
    }

    #[test]
    fn pnpm_takes_precedence_over_package_json() {
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
        assert!(result[0].ends_with("core"));
    }

    #[test]
    fn skips_dirs_without_package_json() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "package.json",
            r#"{"workspaces":["packages/*"]}"#,
        );
        add_workspace(dir.path(), "packages/has-pkg");
        fs::create_dir_all(dir.path().join("packages/no-pkg")).unwrap();
        fs::write(dir.path().join("packages/no-pkg/.gitkeep"), "").unwrap();
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn skips_scan_skip_dirs() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "package.json",
            r#"{"workspaces":["packages/*"]}"#,
        );
        add_workspace(dir.path(), "packages/node_modules");
        add_workspace(dir.path(), "packages/real-pkg");
        // But also need to test that node_modules is skipped even if it has package.json
        // Our expand should skip it because SCAN_SKIP_DIRS contains node_modules
        let result = resolve_workspaces(dir.path());
        // real-pkg should be found, node_modules should be skipped
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("real-pkg"));
    }

    #[test]
    fn handles_pnpm_quoted_patterns() {
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
    fn deno_workspace_detected() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "deno.json",
            r#"{"workspace":["./api","./shared"]}"#,
        );
        fs::create_dir_all(dir.path().join("api")).unwrap();
        fs::write(dir.path().join("api/deno.json"), "{}").unwrap();
        fs::create_dir_all(dir.path().join("shared")).unwrap();
        fs::write(dir.path().join("shared/deno.json"), "{}").unwrap();
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deno_jsonc_detected() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "deno.json", r#"{"workspace":["./lib"]}"#);
        fs::create_dir_all(dir.path().join("lib")).unwrap();
        fs::write(dir.path().join("lib/deno.jsonc"), "{}").unwrap();
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn pnpm_precedence_over_deno() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "package.json", "{}");
        write_file(
            dir.path(),
            "pnpm-workspace.yaml",
            "packages:\n  - packages/*\n",
        );
        write_file(
            dir.path(),
            "deno.json",
            r#"{"workspace":["./deno-member"]}"#,
        );
        add_workspace(dir.path(), "packages/core");
        fs::create_dir_all(dir.path().join("deno-member")).unwrap();
        fs::write(dir.path().join("deno-member/deno.json"), "{}").unwrap();
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("core"));
    }

    #[test]
    fn package_json_precedence_over_deno() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "package.json",
            r#"{"workspaces":["packages/*"]}"#,
        );
        write_file(
            dir.path(),
            "deno.json",
            r#"{"workspace":["./deno-member"]}"#,
        );
        add_workspace(dir.path(), "packages/ui");
        fs::create_dir_all(dir.path().join("deno-member")).unwrap();
        fs::write(dir.path().join("deno-member/deno.json"), "{}").unwrap();
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("ui"));
    }

    #[test]
    fn deno_workspace_with_star_expansion() {
        // deno.json workspace may contain patterns with *
        let dir = tempdir().unwrap();
        write_file(dir.path(), "deno.json", r#"{"workspace":["packages/*"]}"#);
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::write(dir.path().join("packages/a/deno.json"), "{}").unwrap();
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();
        fs::write(dir.path().join("packages/b/deno.json"), "{}").unwrap();
        let result = resolve_workspaces(dir.path());
        // Our implementation treats deno workspace patterns through same expand, so should expand
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn handles_exact_directory_reference() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "package.json",
            r#"{"workspaces":["tools/special-tool"]}"#,
        );
        add_workspace(dir.path(), "tools/special-tool");
        let result = resolve_workspaces(dir.path());
        assert_eq!(result.len(), 1);
    }
}
