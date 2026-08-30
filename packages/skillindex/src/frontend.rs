use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use walkdir::WalkDir;

use crate::detect::SCAN_SKIP_DIRS;

/// File extensions that indicate a web frontend — mirrors `WEB_FRONTEND_EXTENSIONS` in skills-map.ts
pub static WEB_FRONTEND_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        ".html", ".htm", ".css", ".scss", ".sass", ".less", ".vue", ".svelte", ".jsx", ".tsx",
        ".twig", ".tpl", ".ejs", ".hbs", ".pug", ".njk",
    ])
});

/// Returns true if `project_dir` contains a web frontend file within `max_depth` (default 3).
/// Scans recursively, skipping `SCAN_SKIP_DIRS` and dot-directories.
pub fn has_web_frontend_files(project_dir: &Path, max_depth: usize) -> bool {
    fn scan(dir: &Path, depth: usize, max_depth: usize, extensions: &HashSet<&str>) -> bool {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return false,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };

            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    if name.ends_with(".blade.php") {
                        return true;
                    }
                    if let Some(dot) = name.rfind('.') {
                        let ext = &name[dot..];
                        if extensions.contains(ext) {
                            return true;
                        }
                    }
                } else if ft.is_dir() && depth < max_depth {
                    if SCAN_SKIP_DIRS.contains(name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    if scan(&path, depth + 1, max_depth, extensions) {
                        return true;
                    }
                }
            }
        }
        false
    }

    scan(project_dir, 0, max_depth, &WEB_FRONTEND_EXTENSIONS)
}

/// Walk-based alternative used for testing depth behaviour explicitly
#[allow(dead_code)]
pub fn has_web_frontend_files_walk(project_dir: &Path, max_depth: usize) -> bool {
    for entry in WalkDir::new(project_dir)
        .max_depth(max_depth + 1)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if e.depth() == 0 {
                return true;
            }
            if e.file_type().is_dir() {
                !SCAN_SKIP_DIRS.contains(name.as_ref()) && !name.starts_with('.')
            } else {
                true
            }
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            let name = entry.file_name().to_string_lossy();
            if name.ends_with(".blade.php") {
                return true;
            }
            if let Some(dot) = name.rfind('.')
                && WEB_FRONTEND_EXTENSIONS.contains(&name[dot..])
            {
                return true;
            }
        }
    }
    false
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

    #[test]
    fn web_frontend_extensions_contains_expected() {
        let expected = [
            ".html", ".htm", ".css", ".scss", ".sass", ".less", ".vue", ".svelte", ".jsx", ".tsx",
            ".twig", ".tpl", ".ejs", ".hbs", ".pug", ".njk",
        ];
        assert_eq!(WEB_FRONTEND_EXTENSIONS.len(), expected.len());
        for e in expected {
            assert!(WEB_FRONTEND_EXTENSIONS.contains(e), "missing {e}");
        }
    }

    #[test]
    fn detects_html_at_depth_1() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "index.html", "<html></html>");
        assert!(has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn detects_vue_at_depth_3() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "src/components/App.vue",
            "<template></template>",
        );
        assert!(has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn detects_blade_php() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "resources/views/home.blade.php", "blade");
        assert!(has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn detects_twig_and_tpl() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "templates/page.twig", "twig");
        assert!(has_web_frontend_files(dir.path(), 3));
        let dir2 = tempdir().unwrap();
        write_file(dir2.path(), "page.tpl", "tpl");
        assert!(has_web_frontend_files(dir2.path(), 3));
    }

    #[test]
    fn does_not_detect_php_alone() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "index.php", "<?php echo 1;");
        assert!(!has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn does_not_descend_into_skip_dirs() {
        let dir = tempdir().unwrap();
        write_file(
            dir.path(),
            "node_modules/pkg/App.vue",
            "<template></template>",
        );
        assert!(!has_web_frontend_files(dir.path(), 3));
        write_file(dir.path(), "dist/App.jsx", "jsx");
        // dist is also skip, but we have no other file, so still false
        assert!(!has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn does_not_descend_beyond_max_depth() {
        let dir = tempdir().unwrap();
        // depth 0 = dir itself, depth 3 should allow src/a/b/file.vue? Let's create depth 4
        write_file(dir.path(), "a/b/c/d/App.vue", "<template></template>");
        // a (1) -> b (2) -> c (3) -> d (4) -> file at depth 4 should be beyond max_depth 3
        assert!(!has_web_frontend_files(dir.path(), 3));
        // But file at depth 3 should be found
        let dir2 = tempdir().unwrap();
        write_file(dir2.path(), "a/b/c/App.vue", "<template></template>");
        assert!(has_web_frontend_files(dir2.path(), 3));
    }

    #[test]
    fn skips_dot_directories() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), ".hidden/App.vue", "<template></template>");
        assert!(!has_web_frontend_files(dir.path(), 3));
    }

    #[test]
    fn detects_multiple_extensions() {
        for ext in [
            ".jsx", ".tsx", ".css", ".scss", ".svelte", ".hbs", ".pug", ".njk", ".ejs",
        ] {
            let dir = tempdir().unwrap();
            write_file(dir.path(), &format!("src/app{ext}"), "content");
            assert!(has_web_frontend_files(dir.path(), 3), "should detect {ext}");
        }
    }

    #[test]
    fn empty_dir_no_frontend() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# hi").unwrap();
        assert!(!has_web_frontend_files(dir.path(), 3));
    }
}
