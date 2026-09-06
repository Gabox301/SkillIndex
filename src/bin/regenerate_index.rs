use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use skillindex::hash::{bundle_hash, sha256_buffer};

fn normalize_line_endings(data: &[u8]) -> Vec<u8> {
    let s = String::from_utf8_lossy(data);
    if !s.contains('\r') {
        return data.to_vec();
    }
    s.replace("\r\n", "\n").replace('\r', "\n").into_bytes()
}

fn list_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(list_files_recursive(&path));
            } else if path.is_file() {
                // relative path check for .zip
                if let Ok(rel) = path.strip_prefix(dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if rel_str.to_lowercase().ends_with(".zip") {
                        continue;
                    }
                }
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn main() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let registry_dir = manifest_dir.join("skills-registry");
    let manifest_path = registry_dir.join("index.json");

    if !manifest_path.exists() {
        anyhow::bail!("index.json not found at {}", manifest_path.display());
    }

    let manifest_str = fs::read_to_string(&manifest_path)?;
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_str)?;

    let skill_names: Vec<String> = {
        let skills_obj = manifest
            .get("skills")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow::anyhow!("manifest skills is not an object"))?;
        skills_obj.keys().cloned().collect()
    };

    let mut fixed_skills = 0;
    let mut fixed_files = 0;
    let mut total_files = 0;

    for skill_name in skill_names {
        let entry = manifest
            .get_mut("skills")
            .and_then(|v| v.as_object_mut())
            .unwrap()
            .get_mut(&skill_name)
            .unwrap();
        let skill_dir = registry_dir.join(&skill_name);
        if !skill_dir.exists() || !skill_dir.is_dir() {
            eprintln!("skip {skill_name}: dir missing");
            continue;
        }

        let files = list_files_recursive(&skill_dir);
        // Build relFiles with normalized content
        let mut rel_files: Vec<(String, Vec<u8>)> = Vec::new();
        for abs_path in &files {
            let rel = abs_path
                .strip_prefix(&skill_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let data = fs::read(abs_path)?;
            let normalized = normalize_line_endings(&data);
            // Write back if changed (normalize on disk to LF)
            if normalized != data {
                fs::write(abs_path, &normalized)?;
                fixed_files += 1;
            }
            rel_files.push((rel, normalized));
        }
        total_files += rel_files.len();

        // Compute shaMap and bundleHash
        let mut sha_map: HashMap<String, String> = HashMap::new();
        let mut entries_for_hash: Vec<(String, String)> = Vec::new();
        for (rel, buf) in &rel_files {
            let hash = sha256_buffer(buf);
            sha_map.insert(rel.clone(), hash.clone());
            entries_for_hash.push((rel.clone(), hash));
        }
        let bundle_hash = bundle_hash(&entries_for_hash);

        // Check if needs update
        let prev_files: Vec<String> = entry
            .get("files")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let new_files: Vec<String> = rel_files.iter().map(|(rel, _)| rel.clone()).collect();

        let mut prev_sorted = prev_files.clone();
        prev_sorted.sort();
        let mut new_sorted = new_files.clone();
        new_sorted.sort();
        let files_equal = prev_sorted == new_sorted;

        let prev_sha_map = entry
            .get("sha256")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        let sha_equal = {
            if sha_map.len() != prev_sha_map.len() {
                false
            } else {
                sha_map.iter().all(|(k, v)| prev_sha_map.get(k) == Some(v))
                    && prev_sha_map.iter().all(|(k, v)| sha_map.get(k) == Some(v))
            }
        };

        let prev_bundle = entry
            .get("bundleHash")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !files_equal || !sha_equal || prev_bundle != bundle_hash {
            // Update entry
            let files_json: Vec<serde_json::Value> = new_files
                .iter()
                .map(|f| serde_json::Value::String(f.clone()))
                .collect();
            let mut sha_json = serde_json::Map::new();
            for (k, v) in &sha_map {
                sha_json.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            entry["files"] = serde_json::Value::Array(files_json);
            entry["sha256"] = serde_json::Value::Object(sha_json);
            entry["bundleHash"] = serde_json::Value::String(bundle_hash);
            fixed_skills += 1;
        }
    }

    manifest["generatedAt"] = serde_json::Value::String(
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    );

    let total_skills = manifest
        .get("skills")
        .and_then(|v| v.as_object())
        .map(|obj| obj.len())
        .unwrap_or(0);

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;

    println!(
        "Regenerated index.json: {} skills updated, {} files normalized CRLF->LF, totalFiles {}",
        fixed_skills, fixed_files, total_files
    );
    println!("Total skills: {}", total_skills);

    Ok(())
}
