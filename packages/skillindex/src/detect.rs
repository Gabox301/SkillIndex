use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde_json::Value;

use crate::display::{DisplayCombo, DisplayTechnology};
use crate::installer::SkillEntry;

/// Directories that are never descended during scans — mirrors `SCAN_SKIP_DIRS` in lib.ts
pub static SCAN_SKIP_DIRS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "node_modules",
        ".git",
        "vendor",
        ".next",
        "dist",
        "build",
        ".output",
        ".nuxt",
        ".svelte-kit",
        "__pycache__",
        ".cache",
        "coverage",
        ".turbo",
        ".terraform",
        "var",
        "bin",
        "obj",
        ".vs",
    ])
});

/// Returns true if the directory name should be skipped
pub fn is_skip_dir(name: &str) -> bool {
    SCAN_SKIP_DIRS.contains(name)
}

// ── Frontend constants ───────────────────────────────────────────

static FRONTEND_PACKAGES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "react",
        "vue",
        "svelte",
        "astro",
        "next",
        "@angular/core",
        "solid-js",
        "lit",
        "preact",
        "nuxt",
        "@sveltejs/kit",
    ])
});

static FRONTEND_BONUS_SKILLS: &[&str] = &[
    "anthropics/skills/frontend-design",
    "addyosmani/web-quality-skills/accessibility",
    "addyosmani/web-quality-skills/seo",
];

// ── Skills map loading ───────────────────────────────────────────

fn load_skills_map_value() -> Value {
    serde_json::from_str(crate::skills_map::SKILLS_MAP_JSON).unwrap_or(Value::Null)
}

fn get_skills_array() -> Vec<Value> {
    let v = load_skills_map_value();
    v.get("skills")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
}

fn get_combos_array() -> Vec<Value> {
    let v = load_skills_map_value();
    v.get("combos")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
}

// ── Helpers ──────────────────────────────────────────────────────

fn read_package_json(dir: &Path) -> Option<Value> {
    let data = fs::read_to_string(dir.join("package.json")).ok()?;
    serde_json::from_str(&data).ok()
}

fn read_deno_json(dir: &Path) -> Option<Value> {
    for name in ["deno.json", "deno.jsonc"] {
        if let Ok(data) = fs::read_to_string(dir.join(name))
            && let Ok(v) = serde_json::from_str::<Value>(&data)
        {
            return Some(v);
        }
    }
    None
}

fn get_all_package_names(pkg: Option<&Value>) -> Vec<String> {
    let Some(v) = pkg else { return vec![] };
    let mut out = Vec::new();
    if let Some(deps) = v.get("dependencies").and_then(|x| x.as_object()) {
        out.extend(deps.keys().cloned());
    }
    if let Some(dev) = v.get("devDependencies").and_then(|x| x.as_object()) {
        out.extend(dev.keys().cloned());
    }
    out
}

fn get_deno_import_names(deno: Option<&Value>) -> Vec<String> {
    let Some(v) = deno else { return vec![] };
    let Some(imports) = v.get("imports").and_then(|x| x.as_object()) else {
        return vec![];
    };
    let mut out = Vec::new();
    for val in imports.values() {
        if let Some(s) = val.as_str()
            && (s.starts_with("npm:") || s.starts_with("jsr:"))
        {
            let bare = s.replacen("npm:", "", 1).replacen("jsr:", "", 1);
            let name = if bare.starts_with('@') {
                let parts: Vec<&str> = bare.split('/').collect();
                if parts.len() >= 2 {
                    let scope = parts[0];
                    let name_part = parts[1].split('@').next().unwrap_or(parts[1]);
                    format!("{scope}/{name_part}")
                } else {
                    bare.split('@').next().unwrap_or(&bare).to_string()
                }
            } else {
                bare.split('@').next().unwrap_or(&bare).to_string()
            };
            out.push(name);
        }
    }
    out
}

fn has_file_with_extension(dir: &Path, extensions: &[String], max_depth: usize) -> bool {
    let normalized: HashSet<String> = extensions
        .iter()
        .map(|e| {
            let lower = e.to_lowercase();
            if lower.starts_with('.') {
                lower
            } else {
                format!(".{lower}")
            }
        })
        .collect();
    let normalized_vec: Vec<String> = normalized.into_iter().collect();

    fn scan(dir: &Path, depth: usize, max_depth: usize, exts: &[String]) -> bool {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return false,
        };
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if exts.iter().any(|ext| name.ends_with(ext)) {
                        return true;
                    }
                } else if ft.is_dir() && depth < max_depth {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if SCAN_SKIP_DIRS.contains(name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    if scan(&entry.path(), depth + 1, max_depth, exts) {
                        return true;
                    }
                }
            }
        }
        false
    }
    scan(dir, 0, max_depth, &normalized_vec)
}

fn read_gemfile(dir: &Path) -> Vec<String> {
    let path = dir.join("Gemfile");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut gems = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("gem ") {
            let rest = rest.trim();
            if let Some(start) = rest.find(['"', '\'']) {
                let quote = rest.chars().nth(start).unwrap();
                let after = &rest[start + 1..];
                if let Some(end) = after.find(quote) {
                    gems.push(after[..end].to_string());
                }
            }
        }
    }
    gems
}

// ── Detection ────────────────────────────────────────────────────

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

    let skills = get_skills_array();
    let mut detected: Vec<DisplayTechnology> = Vec::new();

    for tech_val in &skills {
        let id = tech_val
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let name = tech_val
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let tech_skills: Vec<String> = tech_val
            .get("skills")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let detect = tech_val.get("detect");

        let mut found = false;

        // packages
        if !found
            && let Some(pkgs) = detect
                .and_then(|d| d.get("packages"))
                .and_then(|x| x.as_array())
        {
            for p in pkgs {
                if let Some(s) = p.as_str()
                    && all_deps_set.contains(s)
                {
                    found = true;
                    break;
                }
            }
        }

        // packagePatterns - simple literal contains check (covers ^@clerk/ etc)
        if !found
            && let Some(patterns) = detect
                .and_then(|d| d.get("packagePatterns"))
                .and_then(|x| x.as_array())
        {
            'outer: for pat_val in patterns {
                let source = pat_val
                    .get("__regexp")
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
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
        if !found
            && let Some(files) = detect
                .and_then(|d| d.get("configFiles"))
                .and_then(|x| x.as_array())
        {
            for f in files {
                if let Some(s) = f.as_str()
                    && dir.join(s).exists()
                {
                    found = true;
                    break;
                }
            }
        }

        // fileExtensions
        if !found
            && let Some(exts) = detect
                .and_then(|d| d.get("fileExtensions"))
                .and_then(|x| x.as_array())
        {
            let ext_strs: Vec<String> = exts
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect();
            if has_file_with_extension(dir, &ext_strs, 4) {
                found = true;
            }
        }

        // gems
        if !found
            && let Some(gems) = detect
                .and_then(|d| d.get("gems"))
                .and_then(|x| x.as_array())
        {
            if gems_cache.is_none() {
                gems_cache = Some(read_gemfile(dir));
            }
            let gem_names = gems_cache.as_ref().unwrap();
            for g in gems {
                if let Some(s) = g.as_str()
                    && gem_names.contains(&s.to_string())
                {
                    found = true;
                    break;
                }
            }
        }

        // configFileContent
        if !found && let Some(cfg) = detect.and_then(|d| d.get("configFileContent")) {
            let blocks: Vec<&Value> = if cfg.is_array() {
                cfg.as_array().unwrap().iter().collect()
            } else {
                vec![cfg]
            };
            for block in blocks {
                let patterns: Vec<String> = block
                    .get("patterns")
                    .and_then(|x| x.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                if patterns.is_empty() {
                    continue;
                }
                let paths: Vec<PathBuf> = if block
                    .get("scanGradleLayout")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false)
                {
                    crate::gradle::gradle_layout_candidate_paths(dir)
                } else if block
                    .get("scanDotNetLayout")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false)
                {
                    crate::dotnet::dotnet_layout_candidate_paths(dir)
                } else {
                    block
                        .get("files")
                        .and_then(|x| x.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str())
                                .map(|s| dir.join(s))
                                .collect()
                        })
                        .unwrap_or_default()
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
                id: id.clone(),
                name,
                skills: tech_skills,
            });
        }
    }

    let is_frontend_by_packages = all_deps_array
        .iter()
        .any(|p| FRONTEND_PACKAGES.contains(p.as_str()));
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
    let combos = get_combos_array();
    let mut out = Vec::new();
    for c in combos {
        let requires: Vec<String> = c
            .get("requires")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        if requires.iter().all(|id| set.contains(id)) {
            let name = c
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let id = c
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            out.push(DisplayCombo { name, id });
        }
    }
    out
}

// ── Skill collection ─────────────────────────────────────────────

pub fn collect_skills(
    detected: &[DisplayTechnology],
    is_frontend: bool,
    combos: &[DisplayCombo],
    installed_names: Option<&HashSet<String>>,
) -> Vec<SkillEntry> {
    let mut skill_map: HashMap<String, SkillEntry> = HashMap::new();
    let mut skills: Vec<SkillEntry> = Vec::new();

    let mut add_skill = |skill: String, source: String| {
        if let Some(existing) = skill_map.get_mut(&skill) {
            if !existing.sources.contains(&source) {
                existing.sources.push(source.clone());
                if let Some(entry) = skills.iter_mut().find(|e| e.skill == skill)
                    && !entry.sources.contains(&source)
                {
                    entry.sources.push(source.clone());
                }
            }
        } else {
            let installed = if let Some(set) = installed_names {
                let parsed = crate::registry::parse_skill_path(&skill);
                set.contains(&parsed.skill_name)
            } else {
                false
            };
            let entry = SkillEntry {
                skill: skill.clone(),
                sources: vec![source.clone()],
                installed,
            };
            skill_map.insert(skill.clone(), entry.clone());
            skills.push(entry);
        }
    };

    for tech in detected {
        for skill in &tech.skills {
            add_skill(skill.clone(), tech.name.clone());
        }
    }

    for combo in combos {
        let combo_val = get_combos_array()
            .into_iter()
            .find(|c| c.get("name").and_then(|x| x.as_str()) == Some(&combo.name));
        if let Some(c) = combo_val
            && let Some(arr) = c.get("skills").and_then(|x| x.as_array())
        {
            for s in arr {
                if let Some(skill_str) = s.as_str() {
                    add_skill(skill_str.to_string(), combo.name.clone());
                }
            }
        }
    }

    if is_frontend {
        for skill in FRONTEND_BONUS_SKILLS {
            add_skill(skill.to_string(), "Frontend".to_string());
        }
    }

    skills
}

// ── Agent detection ──────────────────────────────────────────────

pub fn detect_agents() -> Vec<String> {
    detect_agents_in_home(None)
}

pub fn detect_agents_in_home(home: Option<&Path>) -> Vec<String> {
    let home_path = if let Some(p) = home {
        p.to_path_buf()
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    let mut agents = vec!["universal".to_string()];
    let map = get_agent_folder_map();
    for (folder, agent) in map {
        let folder_path = home_path.join(&folder);
        let skills_path = folder_path.join("skills");
        // Detecta si el agente está instalado: basta que exista la carpeta base
        // (ej. ~/.cursor) o su subcarpeta skills. Antes solo miraba skills/,
        // por lo que cursor/opencode recién instalados sin skills no se detectaban.
        if folder_path.exists() || skills_path.exists() {
            agents.push(agent);
        }
    }
    agents
}

fn get_agent_folder_map() -> Vec<(String, String)> {
    let v = load_skills_map_value();
    let mut out = Vec::new();
    if let Some(map) = v.get("agent_folder_map").and_then(|x| x.as_object()) {
        for (k, v) in map {
            if let Some(agent) = v.as_str() {
                out.push((k.clone(), agent.to_string()));
            }
        }
    }
    if out.is_empty() {
        out.push((".claude".into(), "claude-code".into()));
        out.push((".cline".into(), "cline".into()));
        out.push((".junie".into(), "junie".into()));
        out.push((".codebuddy".into(), "codebuddy".into()));
        out.push((".continue".into(), "continue".into()));
        out.push((".kiro".into(), "kiro-cli".into()));
        out.push((".opencode".into(), "opencode".into()));
        out.push((".cursor".into(), "cursor".into()));
    }
    out
}

// ── Installed skills ─────────────────────────────────────────────

pub fn get_installed_skill_names(project_dir: &Path) -> HashSet<String> {
    let lock_path = project_dir.join("skills-lock.json");
    if let Ok(data) = fs::read_to_string(&lock_path)
        && let Ok(v) = serde_json::from_str::<Value>(&data)
        && let Some(obj) = v.get("skills").and_then(|x| x.as_object())
    {
        return obj.keys().cloned().collect();
    }
    // Fallback when there is no lockfile: scan every known skills folder. Skills
    // now live under each mapped agent folder (e.g. `.kiro/skills`), and `.agents`
    // is only used for the universal destination.
    let mut folders: Vec<String> = get_agent_folder_map()
        .into_iter()
        .map(|(folder, _agent)| folder)
        .collect();
    folders.push(".agents".to_string());
    let mut set = HashSet::new();
    for folder in folders {
        let skills_dir = project_dir.join(&folder).join("skills");
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type()
                    && ft.is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    set.insert(name.to_string());
                }
            }
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scan_skip_dirs_contains_expected_entries() {
        let expected = [
            "node_modules",
            ".git",
            "vendor",
            ".next",
            "dist",
            "build",
            ".output",
            ".nuxt",
            ".svelte-kit",
            "__pycache__",
            ".cache",
            "coverage",
            ".turbo",
            ".terraform",
            "var",
            "bin",
            "obj",
            ".vs",
        ];
        assert_eq!(SCAN_SKIP_DIRS.len(), expected.len());
        for e in expected {
            assert!(SCAN_SKIP_DIRS.contains(e), "missing {e}");
        }
    }

    #[test]
    fn is_skip_dir_true_for_known() {
        assert!(is_skip_dir("node_modules"));
        assert!(is_skip_dir(".git"));
        assert!(is_skip_dir("dist"));
        assert!(is_skip_dir("bin"));
        assert!(is_skip_dir(".vs"));
    }

    #[test]
    fn is_skip_dir_false_for_regular() {
        assert!(!is_skip_dir("src"));
        assert!(!is_skip_dir("packages"));
        assert!(!is_skip_dir("my-app"));
        assert!(!is_skip_dir(""));
    }

    #[test]
    fn cache_identity_is_stable() {
        let a = SCAN_SKIP_DIRS.len();
        let b = SCAN_SKIP_DIRS.len();
        assert_eq!(a, b);
        assert_eq!(a, 18);
    }

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

    #[test]
    fn collect_skills_includes_tech_skills() {
        let tech = DisplayTechnology {
            id: "react".into(),
            name: "React".into(),
            skills: vec!["vercel-labs/agent-skills/react-best-practices".into()],
        };
        let skills = collect_skills(&[tech], false, &[], None);
        assert_eq!(skills.len(), 1);
        assert_eq!(
            skills[0].skill,
            "vercel-labs/agent-skills/react-best-practices"
        );
    }

    #[test]
    fn collect_skills_frontend_bonus() {
        let skills = collect_skills(&[], true, &[], None);
        assert!(skills.iter().any(|s| s.skill.contains("frontend-design")));
    }

    #[test]
    fn get_installed_from_lock() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("skills-lock.json"),
            r#"{"version":1,"skills":{"my-skill":{"source":"x/y","sourceType":"skillindex-registry","computedHash":"abc"}}}"#,
        )
        .unwrap();
        let set = get_installed_skill_names(dir.path());
        assert!(set.contains("my-skill"));
    }

    #[test]
    fn detect_agents_returns_universal() {
        let agents = detect_agents();
        assert!(agents.contains(&"universal".to_string()));
    }
}
