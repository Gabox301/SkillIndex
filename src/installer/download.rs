use std::fs;
use std::path::{Path, PathBuf};

use crate::infra::cache::get_cache_registry_dir;
use crate::infra::hash::{
    InvalidRegistryRelPath, bundle_hash, is_disallowed_skill_file, join_registry_rel,
    normalize_registry_rel_path, sha256_buffer, validate_registry_rel_path,
};
use crate::registry::{
    RegistryEntry, get_registry_dir, get_registry_raw_base_urls, verify_registry_entry,
};

use super::helpers::{copy_dir, encode_raw_path, github_download_headers};
use super::types::InstallOptions;

/// Materialize a verified skill bundle into `dest_dir`, trying local registry,
/// download cache, and finally a fresh download to the cache. The bundle is
/// hash-verified end to end, so every destination gets byte-identical content.
pub(crate) async fn materialize_skill_into(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<(), String> {
    let local_ok: bool = copy_registry_entry_from_local(skill_name, entry, dest_dir, opts)?;
    let cache_ok: bool = if !local_ok {
        copy_registry_entry_from_cache(skill_name, entry, dest_dir)?
    } else {
        false
    };
    if !local_ok && !cache_ok {
        let cached_skill_dir: PathBuf =
            download_registry_entry_to_cache(skill_name, entry, opts, client).await?;
        let _ = fs::remove_dir_all(dest_dir);
        copy_dir(&cached_skill_dir, dest_dir).map_err(|e: std::io::Error| e.to_string())?;
    }
    Ok(())
}

pub fn update_skills_lock(
    project_dir: &Path,
    skill_name: &str,
    entry: &RegistryEntry,
) -> std::io::Result<()> {
    let lock_path: PathBuf = project_dir.join("skills-lock.json");
    let mut lock: serde_json::Value = if lock_path.exists() {
        let content: String = fs::read_to_string(&lock_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or(serde_json::json!({"version": 1, "skills": {}}))
    } else {
        serde_json::json!({"version": 1, "skills": {}})
    };
    if !lock
        .get("skills")
        .map(|v: &serde_json::Value| v.is_object())
        .unwrap_or(false)
    {
        lock["skills"] = serde_json::json!({});
    }
    if lock.get("version").is_none() {
        lock["version"] = serde_json::json!(1);
    }
    lock["skills"][skill_name] = serde_json::json!({
        "source": entry.source,
        "sourceType": "skillindex-registry",
        "computedHash": entry.bundle_hash
    });
    // sort keys
    if let Some(obj) = lock["skills"].as_object().cloned() {
        let mut keys: Vec<String> = obj.keys().cloned().collect();
        keys.sort();
        let mut sorted: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        for k in keys {
            sorted.insert(k.clone(), obj[&k].clone());
        }
        lock["skills"] = serde_json::Value::Object(sorted);
    }
    let pretty: String = serde_json::to_string_pretty(&lock).unwrap();
    fs::write(&lock_path, pretty + "\n")?;
    Ok(())
}

async fn download_registry_file(
    skill_name: &str,
    entry: &RegistryEntry,
    rel: &str,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<(Vec<u8>, String), String> {
    let normalized: String = normalize_registry_rel_path(rel);
    if is_disallowed_skill_file(&normalized) {
        return Err(format!(
            "se rechazó la descarga del archivo de skill no permitido: {normalized}"
        ));
    }
    let expected: String = entry
        .sha256
        .get(rel)
        .or_else(|| entry.sha256.get(&normalized))
        .ok_or_else(|| format!("sin hash registrado para {normalized}"))?
        .clone();

    let base_urls: Vec<String> = if let Some(override_urls) = &opts.registry_base_urls_override {
        override_urls.clone()
    } else {
        get_registry_raw_base_urls(opts.registry_base_url.as_deref())
    };
    let mut errors: Vec<String> = Vec::new();

    for base in &base_urls {
        let url: String = format!("{}/{}", base, encode_raw_path(skill_name, &normalized));
        let headers: reqwest::header::HeaderMap = github_download_headers(&url);
        let res: reqwest::Response = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e: reqwest::Error| format!("request failed for {normalized}: {e}"))?;

        if !res.status().is_success() {
            let status: u16 = res.status().as_u16();
            let remaining: &str = res
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v: &reqwest::header::HeaderValue| v.to_str().ok())
                .unwrap_or("");
            if status == 403 && remaining == "0" {
                let reset_str: &str = res
                    .headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v: &reqwest::header::HeaderValue| v.to_str().ok())
                    .unwrap_or("0");
                let reset_secs: i64 = reset_str.parse().unwrap_or(0);
                let dt: chrono::prelude::DateTime<chrono::prelude::Utc> =
                    chrono::DateTime::from_timestamp(reset_secs, 0)
                        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                let iso: String = dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                return Err(format!(
                    "Límite de tasa de GitHub excedido (se restablece {iso}). Configura GITHUB_TOKEN o GH_TOKEN para aumentarlo."
                ));
            }
            let msg: String = format!(
                "{} {} from {}",
                status,
                res.status().canonical_reason().unwrap_or(""),
                base
            );
            errors.push(msg);
            continue;
        }

        let bytes = res
            .bytes()
            .await
            .map_err(|e: reqwest::Error| format!("failed to read body for {normalized}: {e}"))?;
        let actual: String = sha256_buffer(&bytes);
        if actual != expected {
            errors.push(format!("hash no coincide desde {base}"));
            continue;
        }
        return Ok((bytes.to_vec(), url));
    }

    Err(format!(
        "falló la descarga para {normalized}: {}",
        errors.join("; ")
    ))
}

async fn download_registry_entry(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<(), String> {
    // Destination paths are validated for the whole entry before any download
    // or filesystem write, so a single bad rel aborts the install untouched.
    let mut planned: Vec<(String, PathBuf)> = Vec::with_capacity(entry.files.len());
    for rel in &entry.files {
        let dest: PathBuf = join_registry_rel(dest_dir, rel).map_err(|e| e.to_string())?;
        planned.push((normalize_registry_rel_path(rel), dest));
    }
    let mut files: Vec<(String, PathBuf, Vec<u8>)> = Vec::with_capacity(planned.len());
    for (rel, dest) in planned {
        let (buf, _url) = download_registry_file(skill_name, entry, &rel, opts, client).await?;
        files.push((rel, dest, buf));
    }
    // verify bundle hash: sorted "rel:sha256" join "\n"
    let entries_for_hash: Vec<(String, String)> = files
        .iter()
        .map(|(rel, _dest, buf)| (rel.clone(), sha256_buffer(buf)))
        .collect();
    let computed: String = bundle_hash(&entries_for_hash);
    if computed != entry.bundle_hash {
        return Err("el hash del bundle no coincide".to_string());
    }
    // rm and write
    let _ = fs::remove_dir_all(dest_dir);
    for (_rel, dest, buf) in files {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e: std::io::Error| e.to_string())?;
        }
        fs::write(&dest, &buf).map_err(|e: std::io::Error| e.to_string())?;
    }
    Ok(())
}

/// Fail closed unless every rel in `entry` is a safe relative path.
///
/// Verification, download writes, cache copies and local copies all go through
/// this gate, so a registry entry can never make the installer read or write
/// outside the skill directory.
fn validate_entry_rel_paths(entry: &RegistryEntry) -> Result<(), String> {
    for rel in &entry.files {
        validate_registry_rel_path(rel).map_err(|e: InvalidRegistryRelPath| e.to_string())?;
    }
    Ok(())
}

fn copy_registry_entry_from_local(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
    opts: &InstallOptions,
) -> Result<bool, String> {
    // Local copy path: reject bad rels before reading the registry checkout.
    validate_entry_rel_paths(entry)?;
    let registry_dir: PathBuf = opts.registry_dir.clone().unwrap_or_else(get_registry_dir);
    let verdict: crate::registry::VerifyResult =
        verify_registry_entry(skill_name, entry, &registry_dir);
    if !verdict.ok {
        return Ok(false);
    }
    let _ = fs::remove_dir_all(dest_dir);
    let src: PathBuf = registry_dir.join(skill_name);
    // A failed copy is not a security failure: keep falling back to cache/download.
    Ok(copy_dir(&src, dest_dir).is_ok())
}

fn copy_registry_entry_from_cache(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
) -> Result<bool, String> {
    // Cache copy path: reject bad rels before reading the download cache.
    validate_entry_rel_paths(entry)?;
    let cache_dir: PathBuf = get_cache_registry_dir(&entry.bundle_hash);
    let verdict: crate::registry::VerifyResult =
        verify_registry_entry(skill_name, entry, &cache_dir);
    if !verdict.ok {
        return Ok(false);
    }
    let _ = fs::remove_dir_all(dest_dir);
    let src: PathBuf = cache_dir.join(skill_name);
    // A failed copy is not a security failure: keep falling back to download.
    Ok(copy_dir(&src, dest_dir).is_ok())
}

async fn download_registry_entry_to_cache(
    skill_name: &str,
    entry: &RegistryEntry,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<PathBuf, String> {
    let cache_bundle_dir: PathBuf = get_cache_registry_dir(&entry.bundle_hash);
    let skill_dir: PathBuf = cache_bundle_dir.join(skill_name);
    download_registry_entry(skill_name, entry, &skill_dir, opts, client).await?;
    Ok(skill_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    use crate::infra::hash::sha256_buffer;
    use crate::registry::{RegistryEntry, Review};

    fn make_entry(skill_name: &str, files: Vec<(&str, &str)>) -> RegistryEntry {
        let mut sha: HashMap<String, String> = HashMap::new();
        for (rel, content) in &files {
            sha.insert(rel.to_string(), sha256_buffer(content.as_bytes()));
        }
        let entries: Vec<(String, String)> =
            sha.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let bundle: String = crate::infra::hash::bundle_hash(&entries);
        RegistryEntry {
            source: "owner/repo".to_string(),
            skill_path: format!("owner/repo/{skill_name}"),
            commit_sha: "deadbeef".to_string(),
            files: files.iter().map(|(rel, _)| rel.to_string()).collect(),
            sha256: sha,
            bundle_hash: bundle,
            review: Review {
                status: "approved".to_string(),
                flags: vec![],
                summary: "test".to_string(),
                model: "test-model".to_string(),
                prompt_version: "1.0.0".to_string(),
                reviewed_at: "2026-01-01T00:00:00Z".to_string(),
            },
            security_check: None,
        }
    }

    #[tokio::test]
    async fn download_registry_entry_rejects_traversal_rel_before_writing() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let dest: PathBuf = tmp.path().join("skill");
        let mut entry: RegistryEntry = make_entry("evil", vec![("SKILL.md", "# a")]);
        entry.files = vec!["../escape.md".to_string()];
        entry
            .sha256
            .insert("../escape.md".to_string(), "0".repeat(64));
        let opts: InstallOptions = InstallOptions::default();
        let client: reqwest::Client = reqwest::Client::new();
        let err: String = download_registry_entry("evil", &entry, &dest, &opts, &client)
            .await
            .unwrap_err();
        assert!(
            err.contains("escape.md"),
            "error must name the offending rel: {err}"
        );
        assert!(err.contains("segmento '..'"), "unexpected error: {err}");
        // nothing was created, inside or outside the destination
        assert!(!dest.exists());
        assert!(!tmp.path().join("escape.md").exists());
    }

    #[tokio::test]
    async fn download_registry_entry_rejects_absolute_rel_before_writing() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let dest: PathBuf = tmp.path().join("skill");
        let mut entry: RegistryEntry = make_entry("evil", vec![("SKILL.md", "# a")]);
        entry.files = vec!["/etc/passwd".to_string()];
        entry
            .sha256
            .insert("/etc/passwd".to_string(), "0".repeat(64));
        let opts: InstallOptions = InstallOptions::default();
        let client: reqwest::Client = reqwest::Client::new();
        let err: String = download_registry_entry("evil", &entry, &dest, &opts, &client)
            .await
            .unwrap_err();
        assert!(
            err.contains("/etc/passwd"),
            "error must name the offending rel: {err}"
        );
        assert!(!dest.exists());
    }

    #[test]
    fn copy_paths_reject_traversal_rel_before_copying() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let dest: PathBuf = tmp.path().join("dest");
        let mut entry: RegistryEntry = make_entry("evil", vec![("SKILL.md", "# a")]);
        entry.files = vec!["..\\..\\escape.md".to_string()];
        entry
            .sha256
            .insert("..\\..\\escape.md".to_string(), "0".repeat(64));
        let opts: InstallOptions = InstallOptions::default();
        let local_err: String =
            copy_registry_entry_from_local("evil", &entry, &dest, &opts).unwrap_err();
        assert!(
            local_err.contains("escape.md"),
            "local copy error must name the rel: {local_err}"
        );
        let cache_err: String = copy_registry_entry_from_cache("evil", &entry, &dest).unwrap_err();
        assert!(
            cache_err.contains("escape.md"),
            "cache copy error must name the rel: {cache_err}"
        );
        assert!(!dest.exists());
        assert!(!tmp.path().join("escape.md").exists());
    }

    #[test]
    fn update_skills_lock_sorted_and_newline() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let project: &Path = tmp.path();
        fs::write(
            project.join("skills-lock.json"),
            serde_json::to_string(&serde_json::json!({
                "version": 1,
                "skills": { "zebra": {"source":"x/y","sourceType":"skillindex-registry","computedHash":"z"}}
            }))
            .unwrap(),
        )
        .unwrap();
        let entry: RegistryEntry = make_entry("alpha", vec![("SKILL.md", "# a")]);
        update_skills_lock(project, "alpha", &entry).unwrap();
        let content: String = fs::read_to_string(project.join("skills-lock.json")).unwrap();
        assert!(content.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let keys: Vec<String> = v["skills"].as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["alpha", "zebra"]);
        assert_eq!(v["skills"]["zebra"]["source"], "x/y");
    }

    #[test]
    fn update_skills_lock_creates_new_file() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let entry: RegistryEntry = make_entry("new-skill", vec![("SKILL.md", "content")]);
        update_skills_lock(tmp.path(), "new-skill", &entry).unwrap();
        let content: String = fs::read_to_string(tmp.path().join("skills-lock.json")).unwrap();
        assert!(content.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["skills"]["new-skill"]["source"], "owner/repo");
    }

    #[test]
    fn cache_and_registry_integration_bundle_hash() {
        // Verify bundle hash of a multi-file skill matches TS logic
        let files: Vec<(String, String)> = vec![
            ("SKILL.md".to_string(), sha256_buffer(b"hello")),
            ("references/notes.md".to_string(), sha256_buffer(b"notes")),
        ];
        let hash: String = crate::infra::hash::bundle_hash(&files);
        // Manually compute expected: sorted
        let mut sorted: Vec<(String, String)> = files.clone();
        sorted.sort_by(|a: &(String, String), b: &(String, String)| a.0.cmp(&b.0));
        let joined: String = sorted
            .iter()
            .map(|(rel, h)| format!("{rel}:{h}"))
            .collect::<Vec<_>>()
            .join("\n");
        let expected: String = sha256_buffer(joined.as_bytes());
        assert_eq!(hash, expected);
    }
}
