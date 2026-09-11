use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use skillindex::infra::hash::{normalize_line_endings, sha256_buffer};
use skillindex::registry::parse_skill_path;
use skillindex::skills::{COMBO_SKILLS_MAP, FRONTEND_BONUS_SKILLS, SKILLS_MAP};

fn main() -> anyhow::Result<()> {
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let registry_dir: PathBuf = manifest_dir.join("skills-registry");
    let manifest_path: PathBuf = registry_dir.join("index.json");

    if !manifest_path.exists() {
        anyhow::bail!("Registry manifest not found: {}", manifest_path.display());
    }

    let manifest_str: String = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str)?;
    let registry_skills: serde_json::Map<String, serde_json::Value> = manifest
        .get("skills")
        .and_then(|v: &serde_json::Value| v.as_object())
        .cloned()
        .unwrap_or_default();

    // Collect declared skills from Rust skills map
    let mut declared: HashMap<String, (String, HashSet<String>)> = HashMap::new();
    let mut conflicts: Vec<String> = Vec::new();

    let mut add = |skill: &str, source: &str| {
        let parsed: skillindex::registry::ParsedSkillPath = parse_skill_path(skill);
        if parsed.skill_name.is_empty() {
            return;
        }
        let entry: &mut (String, HashSet<String>) = declared
            .entry(parsed.skill_name.clone())
            .or_insert_with(|| (skill.to_string(), HashSet::new()));
        if entry.0 != skill {
            conflicts.push(format!(
                "{}: declared as both {} and {}",
                parsed.skill_name, entry.0, skill
            ));
        }
        entry.1.insert(source.to_string());
    };

    for tech in SKILLS_MAP.iter() {
        for &skill in tech.skills {
            add(skill, tech.id);
        }
    }
    for combo in COMBO_SKILLS_MAP.iter() {
        for &skill in combo.skills {
            add(skill, combo.id);
        }
    }
    for &skill in FRONTEND_BONUS_SKILLS {
        add(skill, "frontend-bonus");
    }

    let mut errors: Vec<String> = Vec::new();
    for c in conflicts {
        errors.push(c);
    }

    for (skill_name, (full, sources)) in &declared {
        if let Some(entry) = registry_skills.get(skill_name) {
            let skill_path = entry
                .get("skillPath")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if skill_path != full {
                errors.push(format!(
                    "{}: registry skillPath is {}, expected {}",
                    skill_name, skill_path, full
                ));
            }
            // Validate files
            let files: Vec<String> = entry
                .get("files")
                .and_then(|v: &serde_json::Value| v.as_array())
                .map(|arr: &Vec<serde_json::Value>| {
                    arr.iter()
                        .filter_map(|v: &serde_json::Value| v.as_str().map(|s: &str| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if files.is_empty() {
                errors.push(format!("{}: manifest entry has no files", skill_name));
                continue;
            }
            let sha_map: HashMap<String, String> = entry
                .get("sha256")
                .and_then(|v| v.as_object())
                .map(|obj: &serde_json::Map<String, serde_json::Value>| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s: &str| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default();
            let bundle_hash: &str = entry
                .get("bundleHash")
                .and_then(|v: &serde_json::Value| v.as_str())
                .unwrap_or("");

            let mut parts: Vec<String> = Vec::new();
            for file in &files {
                let file_path: PathBuf = registry_dir.join(skill_name).join(file);
                if !file_path.exists() {
                    errors.push(format!("{}: missing file {}", skill_name, file));
                    continue;
                }
                if !file_path.is_file() {
                    errors.push(format!("{}: {} is not a file", skill_name, file));
                    continue;
                }
                let data: Vec<u8> = fs::read(&file_path)?;
                let normalized: Vec<u8> = normalize_line_endings(&data);
                let actual_sha: String = sha256_buffer(&normalized);
                if sha_map.get(file) != Some(&actual_sha) {
                    errors.push(format!("{}: hash mismatch for {}", skill_name, file));
                }
                parts.push(format!("{}:{}", file, actual_sha));
            }
            parts.sort();
            let actual_bundle: String = {
                let joined: String = parts.join("\n");
                sha256_buffer(joined.as_bytes())
            };
            if bundle_hash != actual_bundle {
                errors.push(format!("{}: bundleHash mismatch", skill_name));
            }
        } else {
            let sources_str: String = {
                let mut v: Vec<_> = sources.iter().cloned().collect();
                v.sort();
                v.join(", ")
            };
            errors.push(format!(
                "{}: declared in skills map ({}) but missing from registry",
                skill_name, sources_str
            ));
        }
    }

    for skill_name in {
        let mut v: Vec<_> = registry_skills.keys().cloned().collect();
        v.sort();
        v
    } {
        if !declared.contains_key(&skill_name) {
            errors.push(format!(
                "{}: present in registry but not declared in skills map",
                skill_name
            ));
        }
    }

    if !errors.is_empty() {
        eprintln!("\nRegistry validation failed:");
        for e in &errors {
            eprintln!("- {}", e);
        }
        eprintln!(
            "\nChecked manifest: {}",
            manifest_path
                .strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
                .unwrap_or(&manifest_path)
                .display()
        );
        std::process::exit(1);
    }

    println!(
        "Registry validation passed: {} declared skill{} are installable.",
        declared.len(),
        if declared.len() == 1 { "" } else { "s" }
    );
    Ok(())
}
