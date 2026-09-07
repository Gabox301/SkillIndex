use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use crate::detect::SCAN_SKIP_DIRS;

/// Files scanned at project root for .NET detection — mirrors DOTNET_SCAN_ROOT_FILES in lib.ts
pub static DOTNET_SCAN_ROOT_FILES: &[&str] = &[
    "global.json",
    "NuGet.Config",
    "Directory.Build.props",
    "Directory.Packages.props",
];

static DOTNET_CACHE: LazyLock<Mutex<HashMap<String, Vec<PathBuf>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn dotnet_scan(
    dir: &Path,
    depth: usize,
    candidates: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) {
    if depth > 2 {
        return;
    }
    let entries: fs::ReadDir = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        let name: String = entry.file_name().to_string_lossy().to_string();
        if ft.is_file() {
            let lower: String = name.to_lowercase();
            if lower.ends_with(".sln") || lower.ends_with(".csproj") || lower.ends_with(".fsproj") {
                let p: PathBuf = dir.join(&name);
                if seen.insert(p.clone()) {
                    candidates.push(p);
                }
            }
        } else if ft.is_dir() {
            if name.starts_with('.') || SCAN_SKIP_DIRS.contains(name.as_str()) {
                continue;
            }
            dotnet_scan(&dir.join(&name), depth + 1, candidates, seen);
        }
    }
}

/// Cached candidate paths for .NET layout — mirrors `dotNetLayoutCandidatePaths` in lib.ts
/// Scans depth 2, skips SCAN_SKIP_DIRS and dot-dirs, collects .sln/.csproj/.fsproj case-insensitive
pub fn dotnet_layout_candidate_paths(project_dir: &Path) -> Vec<PathBuf> {
    let key: String = project_dir.to_string_lossy().to_string();
    {
        let cache: std::sync::MutexGuard<'_, HashMap<String, Vec<PathBuf>>> =
            DOTNET_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    for f in DOTNET_SCAN_ROOT_FILES {
        let p: PathBuf = project_dir.join(f);
        if seen.insert(p.clone()) {
            candidates.push(p);
        }
    }

    dotnet_scan(project_dir, 0, &mut candidates, &mut seen);

    let mut cache: std::sync::MutexGuard<'_, HashMap<String, Vec<PathBuf>>> =
        DOTNET_CACHE.lock().unwrap();
    cache.insert(key, candidates.clone());
    candidates
}

#[cfg(test)]
pub fn clear_dotnet_cache() {
    DOTNET_CACHE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(base: &Path, rel: &str, content: &str) {
        let p: PathBuf = base.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, content).unwrap();
    }

    #[test]
    fn dotnet_layout_includes_root_files() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert_eq!(paths.len(), 4);
        assert!(paths.iter().any(|p| p.ends_with("global.json")));
    }

    #[test]
    fn dotnet_layout_finds_csproj_at_depth_0() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(
            dir.path(),
            "MyApp.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">"#,
        );
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("MyApp.csproj")));
    }

    #[test]
    fn dotnet_layout_finds_nested_csproj_depth_2() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(
            dir.path(),
            "src/Library/Library.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">"#,
        );
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("Library.csproj")));
    }

    #[test]
    fn dotnet_layout_excludes_depth_3() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(
            dir.path(),
            "src/A/B/C/App.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">"#,
        );
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(!paths.iter().any(|p| p.ends_with("App.csproj")));
        // also verify a depth 2 file would be found (triangulation: not trivially passing)
        let dir2: tempfile::TempDir = tempdir().unwrap();
        write_file(
            dir2.path(),
            "src/A/App.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">"#,
        );
        let paths2: Vec<PathBuf> = dotnet_layout_candidate_paths(dir2.path());
        assert!(paths2.iter().any(|p| p.ends_with("App.csproj")));
    }

    #[test]
    fn dotnet_layout_case_insensitive() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(dir.path(), "App.CSPROJ", r#"<Project>"#);
        write_file(dir.path(), "Solution.SLN", "");
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(
            paths
                .iter()
                .any(|p| p.file_name().unwrap().to_string_lossy() == "App.CSPROJ")
        );
        assert!(
            paths
                .iter()
                .any(|p| p.file_name().unwrap().to_string_lossy() == "Solution.SLN")
        );
    }

    #[test]
    fn dotnet_layout_skips_bin_and_obj() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(
            dir.path(),
            "bin/Debug/net8.0/Exclude.csproj",
            r#"<Project>"#,
        );
        write_file(dir.path(), "obj/Debug/Exclude2.csproj", r#"<Project>"#);
        write_file(dir.path(), "src/Keep.csproj", r#"<Project>"#);
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(!paths.iter().any(|p| p.to_string_lossy().contains("bin")));
        assert!(!paths.iter().any(|p| p.to_string_lossy().contains("obj")));
        assert!(paths.iter().any(|p| p.ends_with("Keep.csproj")));
    }

    #[test]
    fn dotnet_layout_skips_dot_dirs() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(dir.path(), ".hidden/App.csproj", r#"<Project>"#);
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(
            !paths
                .iter()
                .any(|p| p.to_string_lossy().contains(".hidden"))
        );
    }

    #[test]
    fn dotnet_layout_finds_fsproj() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(dir.path(), "Lib/Lib.fsproj", r#"<Project>"#);
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("Lib.fsproj")));
    }

    #[test]
    fn dotnet_layout_cache_returns_same() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let first: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        write_file(dir.path(), "New/New.csproj", r#"<Project>"#);
        let second: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert_eq!(first, second);
        clear_dotnet_cache();
        let third: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(third.iter().any(|p| p.ends_with("New.csproj")));
        clear_dotnet_cache();
    }

    #[test]
    fn dotnet_layout_finds_sln_at_depth_1() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(dir.path(), "src/MySolution.sln", "");
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        assert!(paths.iter().any(|p| p.ends_with("MySolution.sln")));
    }

    #[test]
    fn dotnet_layout_deduplicates() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        write_file(dir.path(), "App.csproj", "<Project>");
        let paths: Vec<PathBuf> = dotnet_layout_candidate_paths(dir.path());
        let count: usize = paths
            .iter()
            .filter(|p: &&PathBuf| p.ends_with("App.csproj"))
            .count();
        assert_eq!(count, 1);
    }
}
