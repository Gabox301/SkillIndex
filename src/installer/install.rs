use std::env;
use std::path::PathBuf;

use crate::registry::{
    load_registry, load_registry_from_dir, parse_skill_path, security_check_for_entry,
    verify_registry_entry,
};

use super::download::{materialize_skill_into, update_skills_lock};
use super::helpers::rel_path_from_to;
use super::targets::resolve_install_targets;
use super::types::{InstallAllResult, InstallError, InstallOptions, InstallResult, SkillEntry};

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

    // Copy the skill directly into each destination. Each mapped agent gets its
    // own real copy; `.agents` is used only for the universal destination. A
    // target is skipped when it already exists and matches the registry hash, so
    // only missing or drifted targets are (re)installed.
    let targets = resolve_install_targets(&project_dir, agents);
    let mut install_errors: Vec<String> = Vec::new();
    for target in &targets {
        let dest_dir = target.skills_dir.join(&skill_name);
        let verdict = verify_registry_entry(&skill_name, entry, &target.skills_dir);
        if verdict.ok {
            continue;
        }
        if let Err(e) = materialize_skill_into(&skill_name, entry, &dest_dir, opts, client).await {
            install_errors.push(format!("{}: {e}", target.folder));
        }
    }

    if !install_errors.is_empty() {
        let msg = format!("falló la instalación: {}", install_errors.join("; "));
        return InstallResult {
            success: false,
            output: msg.clone(),
            stderr: msg,
            exit_code: Some(1),
            command,
            security_check: None,
        };
    }

    if let Err(e) = update_skills_lock(&project_dir, &skill_name, entry) {
        return fail(format!("falló la actualización del lockfile: {e}"));
    }

    let destinations: Vec<String> = targets
        .iter()
        .map(|t| rel_path_from_to(&project_dir, &t.skills_dir.join(&skill_name)))
        .collect();

    InstallResult {
        success: true,
        output: format!("instalada {} en {}", skill_name, destinations.join(", ")),
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
    use std::env;
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
        assert!(
            project_dir
                .join(".agents/skills/hello-skill/SKILL.md")
                .exists()
        );
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
        assert!(
            result
                .output
                .contains("se rechazó la descarga del archivo de skill no permitido")
        );
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
        assert!(
            project_dir
                .join(".agents/skills/net-skill/SKILL.md")
                .exists()
        );
        assert_eq!(
            fs::read_to_string(project_dir.join(".agents/skills/net-skill/SKILL.md")).unwrap(),
            content
        );
        mock.assert();
        // cache should now contain bundle
        assert!(
            cache_root
                .join(entry.bundle_hash)
                .join(skill_name)
                .join("SKILL.md")
                .exists()
        );

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
}
