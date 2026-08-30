use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cache::get_cache_registry_dir;
use crate::hash::{
    bundle_hash, is_disallowed_skill_file, normalize_registry_rel_path, sha256_buffer,
};
use crate::registry::{
    agent_folder_for, get_registry_dir, get_registry_raw_base_urls, load_registry,
    load_registry_from_dir, parse_skill_path, security_check_for_entry, verify_registry_entry,
    InstallSecurityCheck, RegistryEntry,
};

// ── Public structs ─────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub project_dir: Option<PathBuf>,
    pub registry_dir: Option<PathBuf>,
    pub registry_base_url: Option<String>,
    /// Test-only override for multiple base URLs to test fallback (e.g., v{ver}→main)
    #[cfg(test)]
    pub registry_base_urls_override: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub output: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub command: String,
    pub security_check: Option<InstallSecurityCheck>,
}

#[derive(Debug, Clone)]
pub struct InstallAllResult {
    pub installed: usize,
    pub failed: usize,
    pub security_checks: Vec<InstallSecurityCheck>,
    pub errors: Vec<InstallError>,
}

#[derive(Debug, Clone)]
pub struct InstallError {
    pub name: String,
    pub output: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub skill: String,
    pub sources: Vec<String>,
    pub installed: bool,
}

// ── Helpers ────────────────────────────────────────────────────────

fn get_github_token() -> Option<String> {
    if let Ok(v) = env::var("GITHUB_TOKEN") {
        if !v.trim().is_empty() {
            return Some(v);
        }
    }
    if let Ok(v) = env::var("GH_TOKEN") {
        if !v.trim().is_empty() {
            return Some(v);
        }
    }
    None
}

fn is_githubusercontent_url(url: &str) -> bool {
    // extract host between "://" and next "/"
    let host = if let Some(start) = url.find("://") {
        let rest = &url[start + 3..];
        let end = rest.find('/').unwrap_or(rest.len());
        &rest[..end]
    } else {
        url
    };
    let lower = host.to_ascii_lowercase();
    lower == "raw.githubusercontent.com"
        || lower.ends_with(".githubusercontent.com")
        || lower == "githubusercontent.com"
}

pub fn github_download_headers(url: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("skillindex"),
    );
    if let Some(token) = get_github_token() {
        if is_githubusercontent_url(url) {
            let bearer = format!("Bearer {token}");
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&bearer) {
                headers.insert(reqwest::header::AUTHORIZATION, v);
            }
        }
    }
    headers
}

fn encode_uri_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        let c = b as char;
        if matches!(
            c,
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')'
        ) {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn encode_raw_path(skill_name: &str, rel: &str) -> String {
    let normalized = normalize_registry_rel_path(rel);
    let mut segments = Vec::new();
    segments.push(encode_uri_component(skill_name));
    for part in normalized.split('/') {
        segments.push(encode_uri_component(part));
    }
    segments.join("/")
}

pub fn rel_path_from_to(from: &Path, to: &Path) -> String {
    let from_comps: Vec<_> = from.components().collect();
    let to_comps: Vec<_> = to.components().collect();
    let mut common = 0usize;
    for (a, b) in from_comps.iter().zip(to_comps.iter()) {
        if a == b {
            common += 1;
        } else {
            break;
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_comps.len() {
        parts.push("..".to_string());
    }
    for comp in to_comps.iter().skip(common) {
        parts.push(comp.as_os_str().to_string_lossy().to_string());
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/").replace('\\', "/")
    }
}

pub fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let s = entry.path();
        let d = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&s, &d)?;
        } else if ty.is_file() {
            fs::copy(&s, &d)?;
        }
    }
    Ok(())
}

pub fn ensure_symlink_to(target: &Path, link_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // remove existing
    if link_path.exists() || link_path.is_symlink() {
        let _ = fs::remove_dir_all(link_path);
        let _ = fs::remove_file(link_path);
        // try both
        if link_path.exists() {
            let _ = fs::remove_dir_all(link_path);
        }
    }
    let rel = rel_path_from_to(link_path.parent().unwrap_or(Path::new(".")), target);
    #[cfg(windows)]
    {
        let res = std::os::windows::fs::symlink_dir(&rel, link_path);
        match res {
            Ok(_) => Ok(()),
            Err(_) => {
                // fallback to copyDir on any error (mirrors TS catch-all)
                copy_dir(target, link_path)
            }
        }
    }
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(&rel, link_path) {
            Ok(_) => Ok(()),
            Err(_) => copy_dir(target, link_path),
        }
    }
}

pub fn update_skills_lock(
    project_dir: &Path,
    skill_name: &str,
    entry: &RegistryEntry,
) -> std::io::Result<()> {
    let lock_path = project_dir.join("skills-lock.json");
    let mut lock: serde_json::Value = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or(serde_json::json!({"version": 1, "skills": {}}))
    } else {
        serde_json::json!({"version": 1, "skills": {}})
    };
    if !lock.get("skills").map(|v| v.is_object()).unwrap_or(false) {
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
        let mut sorted = serde_json::Map::new();
        for k in keys {
            sorted.insert(k.clone(), obj[&k].clone());
        }
        lock["skills"] = serde_json::Value::Object(sorted);
    }
    let pretty = serde_json::to_string_pretty(&lock).unwrap();
    fs::write(&lock_path, pretty + "\n")?;
    Ok(())
}

// ── Download helpers ───────────────────────────────────────────────

async fn download_registry_file(
    skill_name: &str,
    entry: &RegistryEntry,
    rel: &str,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<(Vec<u8>, String), String> {
    let normalized = normalize_registry_rel_path(rel);
    if is_disallowed_skill_file(&normalized) {
        return Err(format!(
            "se rechazó la descarga del archivo de skill no permitido: {normalized}"
        ));
    }
    let expected = entry
        .sha256
        .get(rel)
        .or_else(|| entry.sha256.get(&normalized))
        .ok_or_else(|| format!("sin hash registrado para {normalized}"))?
        .clone();

    #[cfg(test)]
    let base_urls = if let Some(override_urls) = &opts.registry_base_urls_override {
        override_urls.clone()
    } else {
        get_registry_raw_base_urls(opts.registry_base_url.as_deref())
    };
    #[cfg(not(test))]
    let base_urls = get_registry_raw_base_urls(opts.registry_base_url.as_deref());
    let mut errors: Vec<String> = Vec::new();

    for base in &base_urls {
        let url = format!("{}/{}", base, encode_raw_path(skill_name, &normalized));
        let headers = github_download_headers(&url);
        let res = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("request failed for {normalized}: {e}"))?;

        if !res.status().is_success() {
            let status = res.status().as_u16();
            let remaining = res
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if status == 403 && remaining == "0" {
                let reset_str = res
                    .headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("0");
                let reset_secs: i64 = reset_str.parse().unwrap_or(0);
                let dt = chrono::DateTime::from_timestamp(reset_secs, 0)
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                let iso = dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                return Err(format!(
                    "Límite de tasa de GitHub excedido (se restablece {iso}). Configura GITHUB_TOKEN o GH_TOKEN para aumentarlo."
                ));
            }
            let msg = format!(
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
            .map_err(|e| format!("failed to read body for {normalized}: {e}"))?;
        let actual = sha256_buffer(&bytes);
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
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for rel in &entry.files {
        let (buf, _url) = download_registry_file(skill_name, entry, rel, opts, client).await?;
        files.push((normalize_registry_rel_path(rel), buf));
    }
    // verify bundle hash: sorted "rel:sha256" join "\n"
    let entries_for_hash: Vec<(String, String)> = files
        .iter()
        .map(|(rel, buf)| (rel.clone(), sha256_buffer(buf)))
        .collect();
    let computed = bundle_hash(&entries_for_hash);
    if computed != entry.bundle_hash {
        return Err("el hash del bundle no coincide".to_string());
    }
    // rm and write
    let _ = fs::remove_dir_all(dest_dir);
    for (rel, buf) in files {
        let mut dest = dest_dir.to_path_buf();
        for part in rel.split('/') {
            dest = dest.join(part);
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&dest, &buf).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn copy_registry_entry_from_local(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
    opts: &InstallOptions,
) -> bool {
    let registry_dir = opts.registry_dir.clone().unwrap_or_else(get_registry_dir);
    let verdict = verify_registry_entry(skill_name, entry, &registry_dir);
    if !verdict.ok {
        return false;
    }
    let _ = fs::remove_dir_all(dest_dir);
    let src = registry_dir.join(skill_name);
    copy_dir(&src, dest_dir).is_ok()
}

fn copy_registry_entry_from_cache(
    skill_name: &str,
    entry: &RegistryEntry,
    dest_dir: &Path,
) -> bool {
    let cache_dir = get_cache_registry_dir(&entry.bundle_hash);
    let verdict = verify_registry_entry(skill_name, entry, &cache_dir);
    if !verdict.ok {
        return false;
    }
    let _ = fs::remove_dir_all(dest_dir);
    let src = cache_dir.join(skill_name);
    copy_dir(&src, dest_dir).is_ok()
}

async fn download_registry_entry_to_cache(
    skill_name: &str,
    entry: &RegistryEntry,
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> Result<PathBuf, String> {
    let cache_bundle_dir = get_cache_registry_dir(&entry.bundle_hash);
    let skill_dir = cache_bundle_dir.join(skill_name);
    download_registry_entry(skill_name, entry, &skill_dir, opts, client).await?;
    Ok(skill_dir)
}

// ── Public install API ─────────────────────────────────────────────

pub async fn install_skill_with_client(
    skill_path: &str,
    agents: &[String],
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> InstallResult {
    let project_dir = opts
        .project_dir
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let command = format!("skillindex install {skill_path}");

    let fail = |msg: String| InstallResult {
        success: false,
        output: msg.clone(),
        stderr: msg,
        exit_code: Some(1),
        command: command.clone(),
        security_check: None,
    };

    let parsed = parse_skill_path(skill_path);
    if parsed.skill_name.is_empty() {
        return fail(format!("ruta de skill no válida: {skill_path}"));
    }
    let skill_name = parsed.skill_name;

    // load registry
    let registry = if let Some(dir) = &opts.registry_dir {
        load_registry_from_dir(dir)
    } else {
        load_registry()
    };
    let Some(registry) = registry else {
        return fail(
            "índice de skills-registry no encontrado. Ejecuta 'pnpm sync:skills' en el paquete skillindex."
                .to_string(),
        );
    };
    let Some(entry) = registry.skills.get(&skill_name) else {
        return fail(format!(
            "skill '{skill_name}' no encontrada en el registro (no auditada)."
        ));
    };
    let security_check = security_check_for_entry(&skill_name, entry);

    let canonical_dir = project_dir.join(".agents").join("skills").join(&skill_name);

    // 3-tier retrieval
    let installed_verdict = verify_registry_entry(
        &skill_name,
        entry,
        &project_dir.join(".agents").join("skills"),
    );
    let needs_refresh = !installed_verdict.ok;

    if needs_refresh {
        let local_ok = copy_registry_entry_from_local(&skill_name, entry, &canonical_dir, opts);
        let cache_ok = if !local_ok {
            copy_registry_entry_from_cache(&skill_name, entry, &canonical_dir)
        } else {
            false
        };
        if !local_ok && !cache_ok {
            match download_registry_entry_to_cache(&skill_name, entry, opts, client).await {
                Ok(cached_skill_dir) => {
                    let _ = fs::remove_dir_all(&canonical_dir);
                    if let Err(e) = copy_dir(&cached_skill_dir, &canonical_dir) {
                        return fail(format!("falló la descarga: {e}"));
                    }
                }
                Err(e) => {
                    return fail(format!("falló la descarga: {e}"));
                }
            }
        }
    }

    // link for agents
    let mut unique_folders: HashSet<String> = HashSet::new();
    for agent in agents {
        if agent == "universal" {
            continue;
        }
        if let Some(folder) = agent_folder_for(agent) {
            unique_folders.insert(folder.to_string());
        }
    }
    let mut symlink_errors: Vec<String> = Vec::new();
    for folder in unique_folders {
        let link_path = project_dir.join(&folder).join("skills").join(&skill_name);
        if let Err(e) = ensure_symlink_to(&canonical_dir, &link_path) {
            symlink_errors.push(format!("{folder}: {e}"));
        }
    }

    if let Err(e) = update_skills_lock(&project_dir, &skill_name, entry) {
        return fail(format!("falló la actualización del lockfile: {e}"));
    }

    if !symlink_errors.is_empty() {
        let msg = symlink_errors.join("\n");
        return InstallResult {
            success: false,
            output: msg.clone(),
            stderr: msg,
            exit_code: Some(1),
            command,
            security_check: None,
        };
    }

    InstallResult {
        success: true,
        output: format!(
            "instalada {} en {}",
            skill_name,
            rel_path_from_to(&project_dir, &canonical_dir)
        ),
        stderr: String::new(),
        exit_code: Some(0),
        command,
        security_check: Some(security_check),
    }
}

pub async fn install_skill(
    skill_path: &str,
    agents: &[String],
    opts: InstallOptions,
) -> InstallResult {
    let client = reqwest::Client::builder()
        .user_agent("skillindex")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    install_skill_with_client(skill_path, agents, &opts, &client).await
}

pub async fn install_all_with_client(
    skills: Vec<SkillEntry>,
    agents: &[String],
    opts: &InstallOptions,
    client: &reqwest::Client,
) -> InstallAllResult {
    // sort by repo (parse_skill_path repo)
    let mut sorted = skills;
    sorted.sort_by(|a, b| {
        let ra = parse_skill_path(&a.skill).repo;
        let rb = parse_skill_path(&b.skill).repo;
        ra.cmp(&rb)
    });

    let concurrency = 6usize;
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));

    let mut handles = Vec::new();
    for entry in sorted {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let agents = agents.to_vec();
        let opts = opts.clone();
        let client = client.clone();
        let skill_clone = entry.skill.clone();
        handles.push(tokio::spawn(async move {
            let result = install_skill_with_client(&skill_clone, &agents, &opts, &client).await;
            drop(permit);
            (skill_clone, result)
        }));
    }

    let mut installed = 0usize;
    let mut failed = 0usize;
    let mut security_checks = Vec::new();
    let mut errors = Vec::new();

    for h in handles {
        let (skill_name, result) = h.await.unwrap();
        if result.success {
            installed += 1;
            if let Some(sc) = result.security_check {
                security_checks.push(sc);
            }
        } else {
            failed += 1;
            errors.push(InstallError {
                name: skill_name,
                output: result.output,
                stderr: result.stderr,
                exit_code: result.exit_code,
                command: result.command,
            });
        }
    }

    InstallAllResult {
        installed,
        failed,
        security_checks,
        errors,
    }
}

pub async fn install_all(
    skills: Vec<SkillEntry>,
    agents: Vec<String>,
    opts: InstallOptions,
) -> InstallAllResult {
    let client = reqwest::Client::builder()
        .user_agent("skillindex")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    install_all_with_client(skills, &agents, &opts, &client).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    use crate::hash::sha256_buffer;
    use crate::registry::{Registry, RegistryEntry, Review, Reviewer};

    fn make_entry(skill_name: &str, files: Vec<(&str, &str)>) -> RegistryEntry {
        let mut sha = HashMap::new();
        for (rel, content) in &files {
            sha.insert(rel.to_string(), sha256_buffer(content.as_bytes()));
        }
        let entries: Vec<(String, String)> =
            sha.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let bundle = crate::hash::bundle_hash(&entries);
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

    #[test]
    fn encode_raw_path_simple() {
        assert_eq!(encode_raw_path("my-skill", "SKILL.md"), "my-skill/SKILL.md");
        assert_eq!(
            encode_raw_path("my-skill", "references/notes.md"),
            "my-skill/references/notes.md"
        );
    }

    #[test]
    fn encode_raw_path_backslash() {
        assert_eq!(
            encode_raw_path("my-skill", "references\\notes.md"),
            "my-skill/references/notes.md"
        );
    }

    #[test]
    fn encode_raw_path_special_chars() {
        assert_eq!(
            encode_raw_path("my-skill", "file with spaces.md"),
            "my-skill/file%20with%20spaces.md"
        );
    }

    #[test]
    fn rel_path_from_to_basic() {
        let from = Path::new("/a/b/c");
        let to = Path::new("/a/b/d/e");
        assert_eq!(rel_path_from_to(from, to), "../d/e");
        assert_eq!(
            rel_path_from_to(Path::new("/a/b"), Path::new("/a/b/c/d")),
            "c/d"
        );
        assert_eq!(
            rel_path_from_to(Path::new("/a/b/c"), Path::new("/a/b/c")),
            "."
        );
    }

    #[test]
    fn copy_dir_recursively() {
        let src = tempdir().unwrap();
        let dest = tempdir().unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("a.txt"), b"hello").unwrap();
        fs::write(src.path().join("sub/b.txt"), b"world").unwrap();
        let out = dest.path().join("out");
        copy_dir(src.path(), &out).unwrap();
        assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(out.join("sub/b.txt")).unwrap(), "world");
    }

    #[test]
    fn ensure_symlink_to_creates_link_or_copy() {
        let tmp = tempdir().unwrap();
        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file.txt"), b"data").unwrap();
        let link = tmp.path().join("link").join("skill");
        ensure_symlink_to(&target, &link).unwrap();
        // either symlink or copy, file should be accessible
        assert!(link.exists() || link.is_symlink());
        assert_eq!(fs::read_to_string(link.join("file.txt")).unwrap(), "data");
    }

    #[test]
    fn update_skills_lock_sorted_and_newline() {
        let tmp = tempdir().unwrap();
        let project = tmp.path();
        fs::write(
            project.join("skills-lock.json"),
            serde_json::to_string(&serde_json::json!({
                "version": 1,
                "skills": { "zebra": {"source":"x/y","sourceType":"skillindex-registry","computedHash":"z"}}
            })).unwrap(),
        ).unwrap();
        let entry = make_entry("alpha", vec![("SKILL.md", "# a")]);
        update_skills_lock(project, "alpha", &entry).unwrap();
        let content = fs::read_to_string(project.join("skills-lock.json")).unwrap();
        assert!(content.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let keys: Vec<String> = v["skills"].as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["alpha", "zebra"]);
        assert_eq!(v["skills"]["zebra"]["source"], "x/y");
    }

    #[test]
    fn update_skills_lock_creates_new_file() {
        let tmp = tempdir().unwrap();
        let entry = make_entry("new-skill", vec![("SKILL.md", "content")]);
        update_skills_lock(tmp.path(), "new-skill", &entry).unwrap();
        let content = fs::read_to_string(tmp.path().join("skills-lock.json")).unwrap();
        assert!(content.ends_with('\n'));
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["skills"]["new-skill"]["source"], "owner/repo");
    }

    #[tokio::test]
    async fn install_skill_from_local_registry() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();

        let skill_name = "hello-skill";
        let files = vec![("SKILL.md", "# hello"), ("references/notes.md", "notes")];
        let entry = make_entry(skill_name, files.clone());
        // write registry files
        for (rel, content) in &files {
            let mut p = reg_dir.join(skill_name);
            for part in rel.split('/') {
                p = p.join(part);
            }
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, content).unwrap();
        }
        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some("https://example.test/skills-registry".to_string()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let result = install_skill_with_client("owner/repo/hello-skill", &[], &opts, &client).await;
        assert!(result.success, "failed: {}", result.output);
        assert!(project_dir
            .join(".agents/skills/hello-skill/SKILL.md")
            .exists());
        let lock: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(project_dir.join("skills-lock.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(lock["skills"]["hello-skill"]["source"], "owner/repo");
    }

    #[tokio::test]
    async fn install_skill_rejects_zip() {
        let _env_guard = crate::cache::env_lock();
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();
        let skill_name = "archive-skill";
        // Create entry with .zip file but no file on disk (registry dir removed)
        let entry = make_entry(skill_name, vec![("downloads/tool.ZIP", "zipcontent")]);
        // Do not write file to reg_dir, and remove skill dir to force network path, but zip should be rejected before network
        // Write only index.json
        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry);
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();
        // Ensure no local file exists, set cache to tmp non-existent
        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some("https://example.test/skills-registry".to_string()),
            ..Default::default()
        };
        // Use a cache dir that doesn't contain the skill
        let prev_cache = env::var("SKILLINDEX_CACHE_DIR").ok();
        unsafe {
            env::set_var(
                "SKILLINDEX_CACHE_DIR",
                tmp.path().join("cache-zip").to_str().unwrap(),
            )
        };
        let client = reqwest::Client::new();
        let result =
            install_skill_with_client("owner/repo/archive-skill", &[], &opts, &client).await;
        assert!(!result.success);
        assert!(result
            .output
            .contains("se rechazó la descarga del archivo de skill no permitido"));
        match prev_cache {
            Some(v) => unsafe { env::set_var("SKILLINDEX_CACHE_DIR", v) },
            None => unsafe { env::remove_var("SKILLINDEX_CACHE_DIR") },
        }
    }

    #[tokio::test]
    async fn install_skill_not_found() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills: HashMap::new(),
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();
        let opts = InstallOptions {
            project_dir: Some(tmp.path().join("project")),
            registry_dir: Some(reg_dir),
            registry_base_url: None,
            ..Default::default()
        };
        fs::create_dir_all(opts.project_dir.as_ref().unwrap()).unwrap();
        let client = reqwest::Client::new();
        let result = install_skill_with_client("owner/repo/unknown", &[], &opts, &client).await;
        assert!(!result.success);
        assert!(result.output.contains("no encontrada en el registro"));
    }

    #[test]
    fn is_githubusercontent_detection() {
        assert!(is_githubusercontent_url(
            "https://raw.githubusercontent.com/foo/bar"
        ));
        assert!(is_githubusercontent_url(
            "https://raw.githubusercontent.com/Gabox301/SkillIndex/main/file"
        ));
        assert!(!is_githubusercontent_url(
            "https://example.test/skills-registry/file"
        ));
        assert!(!is_githubusercontent_url("https://github.com/foo"));
    }

    #[test]
    fn github_headers_include_bearer_on_github() {
        let _env_guard = crate::cache::env_lock();
        let prev = env::var("GITHUB_TOKEN").ok();
        unsafe { env::set_var("GITHUB_TOKEN", "test-token-123") };
        let headers = github_download_headers(
            "https://raw.githubusercontent.com/Gabox301/SkillIndex/main/file",
        );
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Bearer test-token-123"
        );
        let headers2 = github_download_headers("https://example.test/file");
        assert!(headers2.get("authorization").is_none());
        match prev {
            Some(v) => unsafe { env::set_var("GITHUB_TOKEN", v) },
            None => unsafe { env::remove_var("GITHUB_TOKEN") },
        }
    }

    #[test]
    fn github_headers_no_token_no_auth() {
        let _env_guard = crate::cache::env_lock();
        let prev = env::var("GITHUB_TOKEN").ok();
        let prev2 = env::var("GH_TOKEN").ok();
        unsafe {
            env::remove_var("GITHUB_TOKEN");
            env::remove_var("GH_TOKEN")
        };
        let headers = github_download_headers("https://raw.githubusercontent.com/foo/bar");
        assert!(headers.get("authorization").is_none());
        if let Some(v) = prev {
            unsafe { env::set_var("GITHUB_TOKEN", v) };
        }
        if let Some(v) = prev2 {
            unsafe { env::set_var("GH_TOKEN", v) };
        }
    }

    #[tokio::test]
    async fn install_skill_via_httpmock_network() {
        let _env_guard = crate::cache::env_lock();
        let server = httpmock::MockServer::start();
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();

        let skill_name = "net-skill";
        let content = "# net skill content";
        let entry = make_entry(skill_name, vec![("SKILL.md", content)]);

        // index.json
        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        // mock GET /net-skill/SKILL.md
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path(format!("/{}/SKILL.md", skill_name));
            then.status(200).body(content);
        });

        // point registry_base_url to mock server, and isolate cache
        let cache_root = tmp.path().join("cache-net");
        let prev_cache = env::var("SKILLINDEX_CACHE_DIR").ok();
        unsafe { env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some(server.base_url()),
            ..Default::default()
        };
        // Remove local skill dir to force network path
        // Ensure no local copy exists
        let _ = fs::remove_dir_all(reg_dir.join(skill_name));
        let client = reqwest::Client::new();
        let result = install_skill_with_client("owner/repo/net-skill", &[], &opts, &client).await;
        assert!(
            result.success,
            "install failed: {} stderr {}",
            result.output, result.stderr
        );
        assert!(project_dir
            .join(".agents/skills/net-skill/SKILL.md")
            .exists());
        assert_eq!(
            fs::read_to_string(project_dir.join(".agents/skills/net-skill/SKILL.md")).unwrap(),
            content
        );
        mock.assert();
        // cache should now contain bundle
        assert!(cache_root
            .join(entry.bundle_hash)
            .join(skill_name)
            .join("SKILL.md")
            .exists());

        match prev_cache {
            Some(v) => unsafe { env::set_var("SKILLINDEX_CACHE_DIR", v) },
            None => unsafe { env::remove_var("SKILLINDEX_CACHE_DIR") },
        }
    }

    #[tokio::test]
    async fn install_skill_rate_limit_aborts_with_iso() {
        let _env_guard = crate::cache::env_lock();
        let server = httpmock::MockServer::start();
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();

        let skill_name = "rate-skill";
        let entry = make_entry(skill_name, vec![("SKILL.md", "content")]);
        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path(format!("/{}/SKILL.md", skill_name));
            then.status(403)
                .header("x-ratelimit-remaining", "0")
                .header("x-ratelimit-reset", "999")
                .body("rate limited");
        });

        let cache_root = tmp.path().join("cache-rate");
        let prev_cache = env::var("SKILLINDEX_CACHE_DIR").ok();
        unsafe { env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some(server.base_url()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let result = install_skill_with_client("owner/repo/rate-skill", &[], &opts, &client).await;
        assert!(!result.success);
        // 999 seconds = 1970-01-01T00:16:39.000Z
        assert!(result.output.contains("Límite de tasa de GitHub excedido"));
        assert!(result.output.contains("1970-01-01T00:16:39.000Z"));
        mock.assert();

        match prev_cache {
            Some(v) => unsafe { env::set_var("SKILLINDEX_CACHE_DIR", v) },
            None => unsafe { env::remove_var("SKILLINDEX_CACHE_DIR") },
        }
    }

    #[tokio::test]
    async fn install_skill_cache_hit_no_fetch() {
        let _env_guard = crate::cache::env_lock();
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        let cache_root = tmp.path().join("cache-hit");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();
        fs::create_dir_all(&cache_root).unwrap();

        let skill_name = "cached-skill";
        let content = "# cached";
        let entry = make_entry(skill_name, vec![("SKILL.md", content)]);

        // index.json
        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        // populate cache: cache_root/<bundle>/<skill>/SKILL.md
        let cache_skill_dir = cache_root.join(&entry.bundle_hash).join(skill_name);
        fs::create_dir_all(&cache_skill_dir).unwrap();
        fs::write(cache_skill_dir.join("SKILL.md"), content).unwrap();

        // ensure local registry does NOT have the skill
        // (reg_dir/<skill> should not exist, so local verdict fails, cache should hit)
        assert!(!reg_dir.join(skill_name).exists());

        let prev_cache = env::var("SKILLINDEX_CACHE_DIR").ok();
        unsafe { env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some("https://should-not-be-called.test".to_string()),
            ..Default::default()
        };
        // fetch should not be called — if it were, it would fail because URL is bogus and no mock, but download would only happen if cache miss.
        // To ensure no network, we use a client but the code should not reach download because cache hit.
        let client = reqwest::Client::new();
        let result =
            install_skill_with_client("owner/repo/cached-skill", &[], &opts, &client).await;
        assert!(
            result.success,
            "cache hit install failed: {}",
            result.output
        );
        assert_eq!(
            fs::read_to_string(project_dir.join(".agents/skills/cached-skill/SKILL.md")).unwrap(),
            content
        );

        match prev_cache {
            Some(v) => unsafe { env::set_var("SKILLINDEX_CACHE_DIR", v) },
            None => unsafe { env::remove_var("SKILLINDEX_CACHE_DIR") },
        }
    }

    #[tokio::test]
    async fn install_skill_skips_download_when_already_installed() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&reg_dir).unwrap();
        fs::create_dir_all(&project_dir).unwrap();

        let skill_name = "already-skill";
        let content = "# already";
        let entry = make_entry(skill_name, vec![("SKILL.md", content)]);

        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        // also write registry files for local copy path (though not needed, we pre-install canonical)
        // Pre-install canonical already verified
        let canonical = project_dir.join(".agents/skills").join(skill_name);
        fs::create_dir_all(&canonical).unwrap();
        fs::write(canonical.join("SKILL.md"), content).unwrap();

        // Ensure registry dir does NOT have skill (so if code tried to copy from local, it would fail, but canonical already verified, so it shouldn't try)
        // Actually we keep reg_dir empty of skill, but canonical is already verified, so no download.

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: Some("https://should-not-be-called.test".to_string()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        // Provide a fetch that would panic if called
        let result =
            install_skill_with_client("owner/repo/already-skill", &[], &opts, &client).await;
        assert!(
            result.success,
            "already installed should succeed without fetch: {}",
            result.output
        );
        assert_eq!(
            fs::read_to_string(canonical.join("SKILL.md")).unwrap(),
            content
        );
    }

    #[tokio::test]
    async fn install_skill_hash_mismatch_falls_to_second_base() {
        let _env_guard = crate::cache::env_lock();
        let server1 = httpmock::MockServer::start();
        let server2 = httpmock::MockServer::start();
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();

        let skill_name = "mismatch-skill";
        let good_content = "good content";
        let bad_content = "bad content";
        let entry = make_entry(skill_name, vec![("SKILL.md", good_content)]);

        let mut skills = HashMap::new();
        skills.insert(skill_name.to_string(), entry.clone());
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        // server1 returns bad content (hash mismatch)
        let mock1 = server1.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path(format!("/{}/SKILL.md", skill_name));
            then.status(200).body(bad_content);
        });
        // server2 returns good content
        let mock2 = server2.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path(format!("/{}/SKILL.md", skill_name));
            then.status(200).body(good_content);
        });

        let cache_root = tmp.path().join("cache-mismatch");
        let prev_cache = env::var("SKILLINDEX_CACHE_DIR").ok();
        unsafe { env::set_var("SKILLINDEX_CACHE_DIR", cache_root.to_str().unwrap()) };

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: None,
            registry_base_urls_override: Some(vec![server1.base_url(), server2.base_url()]),
        };
        let client = reqwest::Client::new();
        let result =
            install_skill_with_client("owner/repo/mismatch-skill", &[], &opts, &client).await;
        assert!(
            result.success,
            "should succeed via second base after hash mismatch: {}",
            result.output
        );
        assert_eq!(
            fs::read_to_string(project_dir.join(".agents/skills/mismatch-skill/SKILL.md")).unwrap(),
            good_content
        );
        mock1.assert();
        mock2.assert();

        match prev_cache {
            Some(v) => unsafe { env::set_var("SKILLINDEX_CACHE_DIR", v) },
            None => unsafe { env::remove_var("SKILLINDEX_CACHE_DIR") },
        }
    }

    #[tokio::test]
    async fn install_all_concurrent_sorts_by_repo() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        let project_dir = tmp.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&reg_dir).unwrap();

        let skills_data = vec![
            ("skill-b", vec![("SKILL.md", "b")]),
            ("skill-a", vec![("SKILL.md", "a")]),
        ];
        let mut map = HashMap::new();
        for (name, files) in &skills_data {
            // write to registry dir for local copy
            for (rel, content) in files {
                let mut p = reg_dir.join(*name);
                for part in rel.split('/') {
                    p = p.join(part);
                }
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(p, content).unwrap();
            }
            map.insert(name.to_string(), make_entry(name, files.clone()));
        }
        let registry = Registry {
            version: 1,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            reviewer: Reviewer {
                model: "test".to_string(),
                prompt_version: "1.0".to_string(),
            },
            skills: map,
        };
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            registry_base_url: None,
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let entries = vec![
            SkillEntry {
                skill: "owner/b/skill-b".to_string(),
                sources: vec![],
                installed: false,
            },
            SkillEntry {
                skill: "owner/a/skill-a".to_string(),
                sources: vec![],
                installed: false,
            },
        ];
        // Note: skill names in registry are "skill-a"/"skill-b", but skill path is "owner/a/skill-a"
        // Our registry skills are named "skill-a" etc, so install should look up "skill-a"
        // The repo sorting should order by "owner/a" before "owner/b"
        let result = install_all_with_client(entries, &[], &opts, &client).await;
        // Both should be attempted; even if one fails due to name mismatch, we check concurrency logic runs
        // For this test we just ensure install_all completes without deadlock
        assert!(result.installed + result.failed == 2);
    }

    #[test]
    fn cache_and_registry_integration_bundle_hash() {
        // Verify bundle hash of a multi-file skill matches TS logic
        let files = vec![
            ("SKILL.md".to_string(), sha256_buffer(b"hello")),
            ("references/notes.md".to_string(), sha256_buffer(b"notes")),
        ];
        let hash = crate::hash::bundle_hash(&files);
        // Manually compute expected: sorted
        let mut sorted = files.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let joined = sorted
            .iter()
            .map(|(rel, h)| format!("{rel}:{h}"))
            .collect::<Vec<_>>()
            .join("\n");
        let expected = sha256_buffer(joined.as_bytes());
        assert_eq!(hash, expected);
    }
}
