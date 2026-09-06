use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::detect::constants::SCAN_SKIP_DIRS;

pub fn read_package_json(dir: &Path) -> Option<Value> {
    let data = fs::read_to_string(dir.join("package.json")).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn read_deno_json(dir: &Path) -> Option<Value> {
    for name in ["deno.json", "deno.jsonc"] {
        if let Ok(data) = fs::read_to_string(dir.join(name))
            && let Ok(v) = serde_json::from_str::<Value>(&data)
        {
            return Some(v);
        }
    }
    None
}

pub fn get_all_package_names(pkg: Option<&Value>) -> Vec<String> {
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

pub fn get_deno_import_names(deno: Option<&Value>) -> Vec<String> {
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

pub fn has_file_with_extension(dir: &Path, extensions: &[String], max_depth: usize) -> bool {
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

pub fn read_gemfile(dir: &Path) -> Vec<String> {
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

pub fn get_skills_slice() -> &'static [crate::skills::types::Technology] {
    crate::skills::SKILLS.as_slice()
}

pub fn get_combos_slice() -> &'static [crate::skills::types::ComboSkill] {
    crate::skills::COMBO_SKILLS_MAP.as_slice()
}

pub fn get_agent_folder_map_slice() -> &'static [(&'static str, &'static str)] {
    crate::skills::AGENT_FOLDER_MAP
}
