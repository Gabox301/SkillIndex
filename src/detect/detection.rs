use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::detect::constants::{FRONTEND_PACKAGES_SET, is_optional_security_combo};
use crate::detect::helpers::{
    get_all_package_names, get_combos_slice, get_deno_import_names, get_skills_slice,
    has_file_with_extension, read_deno_json, read_gemfile, read_package_json,
};
use crate::display::{DisplayCombo, DisplayTechnology};

struct DetectInDirResult {
    detected: Vec<DisplayTechnology>,
    is_frontend_by_packages: bool,
    is_frontend_by_files: bool,
}

fn detect_technologies_in_dir(
    dir: &Path,
    preloaded_pkg: Option<Value>,
    preloaded_deno: Option<Value>,
    skip_frontend_files: bool,
) -> DetectInDirResult {
    let pkg = preloaded_pkg.or_else(|| read_package_json(dir));
    let deno = preloaded_deno.or_else(|| read_deno_json(dir));
    let all_packages = get_all_package_names(pkg.as_ref());
    let deno_imports = get_deno_import_names(deno.as_ref());
    let mut all_deps_set: HashSet<String> = all_packages.iter().cloned().collect();
    for d in &deno_imports {
        all_deps_set.insert(d.clone());
    }
    let all_deps_array: Vec<String> = if deno_imports.is_empty() {
        all_packages
    } else {
        all_deps_set.iter().cloned().collect()
    };

    let mut gems_cache: Option<Vec<String>> = None;

    let mut detected: Vec<DisplayTechnology> = Vec::new();

    for tech in get_skills_slice() {
        let id = tech.id;
        let name = tech.name;
        let tech_skills: Vec<String> = tech.skills.iter().map(|s| s.to_string()).collect();
        let detect = &tech.detect;

        let mut found = false;

        // packages
        if !found && !detect.packages.is_empty() {
            for p in detect.packages {
                if all_deps_set.contains(*p) {
                    found = true;
                    break;
                }
            }
        }

        // packagePatterns - simple literal contains check (covers ^@clerk/ etc)
        if !found && !detect.package_patterns.is_empty() {
            'outer: for pat in detect.package_patterns {
                let source = *pat;
                if source.is_empty() {
                    continue;
                }
                let literal = source
                    .trim_start_matches('^')
                    .replace("\\/", "/")
                    .replace(".*", "")
                    .replace("(", "")
                    .replace(")", "")
                    .replace("|", "")
                    .replace("\\", "");
                for dep in &all_deps_array {
                    if dep.contains(&literal) || dep.starts_with(&literal) {
                        found = true;
                        break 'outer;
                    }
                }
            }
        }

        // configFiles
        if !found && !detect.config_files.is_empty() {
            for f in detect.config_files {
                if dir.join(f).exists() {
                    found = true;
                    break;
                }
            }
        }

        // fileExtensions
        if !found && !detect.file_extensions.is_empty() {
            let ext_strs: Vec<String> = detect
                .file_extensions
                .iter()
                .map(|s| s.to_string())
                .collect();
            if has_file_with_extension(dir, &ext_strs, 4) {
                found = true;
            }
        }

        // gems
        if !found && !detect.gems.is_empty() {
            if gems_cache.is_none() {
                gems_cache = Some(read_gemfile(dir));
            }
            let gem_names = gems_cache.as_ref().unwrap();
            for g in detect.gems {
                if gem_names.contains(&g.to_string()) {
                    found = true;
                    break;
                }
            }
        }

        // configFileContent
        if !found && !detect.config_file_content.is_empty() {
            for block in detect.config_file_content {
                let patterns: Vec<String> = block.patterns.iter().map(|s| s.to_string()).collect();
                if patterns.is_empty() {
                    continue;
                }
                let paths: Vec<PathBuf> = if block.scan_gradle_layout {
                    crate::gradle::gradle_layout_candidate_paths(dir)
                } else if block.scan_dotnet_layout {
                    crate::dotnet::dotnet_layout_candidate_paths(dir)
                } else {
                    block.files.iter().map(|s| dir.join(s)).collect()
                };

                for path in &paths {
                    if let Ok(content) = fs::read_to_string(path)
                        && patterns.iter().any(|p| content.contains(p))
                    {
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }

        if found {
            detected.push(DisplayTechnology {
                id: id.to_string(),
                name: name.to_string(),
                skills: tech_skills,
            });
        }
    }

    let is_frontend_by_packages = all_deps_array
        .iter()
        .any(|p| FRONTEND_PACKAGES_SET.contains(p.as_str()));
    let is_frontend_by_files = if is_frontend_by_packages || skip_frontend_files {
        false
    } else {
        crate::frontend::has_web_frontend_files(dir, 3)
    };

    DetectInDirResult {
        detected,
        is_frontend_by_packages,
        is_frontend_by_files,
    }
}

#[derive(Debug, Clone)]
pub struct DetectResult {
    pub detected: Vec<DisplayTechnology>,
    pub is_frontend: bool,
    pub combos: Vec<DisplayCombo>,
}

pub fn detect_technologies(project_dir: &Path) -> DetectResult {
    let pkg = read_package_json(project_dir);
    let deno = read_deno_json(project_dir);
    let root = detect_technologies_in_dir(project_dir, pkg.clone(), deno.clone(), false);
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut detected: Vec<DisplayTechnology> = Vec::new();
    for t in root.detected {
        if seen_ids.insert(t.id.clone()) {
            detected.push(t);
        }
    }
    let mut is_frontend = root.is_frontend_by_packages || root.is_frontend_by_files;

    let workspace_dirs = crate::workspace::resolve_workspaces(project_dir);
    for ws_dir in workspace_dirs {
        let ws = detect_technologies_in_dir(&ws_dir, None, None, is_frontend);
        for tech in ws.detected {
            if seen_ids.insert(tech.id.clone()) {
                detected.push(tech);
            }
        }
        if ws.is_frontend_by_packages || ws.is_frontend_by_files {
            is_frontend = true;
        }
    }

    // detected already in insertion order
    let detected_ids: Vec<String> = detected.iter().map(|t| t.id.clone()).collect();
    let combos = detect_combos(&detected_ids);

    DetectResult {
        detected,
        is_frontend,
        combos,
    }
}

pub fn detect_combos(detected_ids: &[String]) -> Vec<DisplayCombo> {
    let set: HashSet<&String> = detected_ids.iter().collect();
    let mut out = Vec::new();
    for c in get_combos_slice() {
        if c.requires.iter().all(|id| set.contains(&id.to_string())) {
            out.push(DisplayCombo {
                name: c.name.to_string(),
                id: c.id.to_string(),
            });
        }
    }
    out
}

pub fn partition_combos(combos: Vec<DisplayCombo>) -> (Vec<DisplayCombo>, Vec<DisplayCombo>) {
    let mut regular = Vec::new();
    let mut security = Vec::new();
    for c in combos {
        if is_optional_security_combo(&c.id) {
            security.push(c);
        } else {
            regular.push(c);
        }
    }
    (regular, security)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detect_package_hit() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"react":"^18.0.0"}}"#,
        )
        .unwrap();
        let res = detect_technologies(dir.path());
        assert!(
            res.detected.iter().any(|t| t.id == "react"),
            "should detect react, got {:?}",
            res.detected.iter().map(|t| &t.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn frontend_detection_via_package() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"react":"^18.0.0"}}"#,
        )
        .unwrap();
        let res = detect_technologies(dir.path());
        assert!(res.is_frontend, "react should be frontend");
    }
}
