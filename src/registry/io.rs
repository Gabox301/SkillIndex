use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::helpers::get_package_version;
use super::types::{InstallSecurityCheck, Registry, RegistryEntry, VerifyResult};
use crate::hash::{normalize_registry_rel_path, sha256_file};

pub fn get_registry_raw_base_urls(registry_base_url: Option<&str>) -> Vec<String> {
    if let Some(url) = registry_base_url {
        let trimmed = url.trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return vec![trimmed];
        }
    }
    if let Ok(v) = env::var("SKILLINDEX_REGISTRY_BASE_URL")
        && !v.trim().is_empty()
    {
        return vec![v.trim_end_matches('/').to_string()];
    }
    let version = get_package_version();
    let base = "https://raw.githubusercontent.com/Gabox301/SkillIndex";
    vec![
        format!("{base}/v{version}/skills-registry"),
        format!("{base}/main/skills-registry"),
    ]
}

pub fn get_registry_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let c1 = manifest_dir.join("skills-registry");
    if c1.join("index.json").exists() {
        return c1;
    }
    if let Some(parent) = manifest_dir.parent() {
        let c2 = parent.join("skills-registry");
        if c2.join("index.json").exists() {
            return c2;
        }
        let c3 = parent
            .join("packages")
            .join("skillindex")
            .join("skills-registry");
        if c3.join("index.json").exists() {
            return c3;
        }
    }
    if let Ok(cwd) = env::current_dir() {
        let c = cwd.join("skills-registry");
        if c.join("index.json").exists() {
            return c;
        }
        let c2 = cwd
            .join("packages")
            .join("skillindex")
            .join("skills-registry");
        if c2.join("index.json").exists() {
            return c2;
        }
    }
    if let Ok(exe) = env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let c = dir.join("skills-registry");
        if c.join("index.json").exists() {
            return c;
        }
        if let Some(parent) = dir.parent() {
            let c2 = parent.join("skills-registry");
            if c2.join("index.json").exists() {
                return c2;
            }
        }
    }
    c1
}

pub fn load_registry() -> Option<Registry> {
    load_registry_from_dir(&get_registry_dir())
}

pub fn load_registry_from_dir(dir: &Path) -> Option<Registry> {
    let p = dir.join("index.json");
    let data = fs::read_to_string(&p).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn verify_registry_entry(
    skill_name: &str,
    entry: &RegistryEntry,
    registry_dir: &Path,
) -> VerifyResult {
    let skill_dir = registry_dir.join(skill_name);
    if !skill_dir.exists() {
        return VerifyResult {
            ok: false,
            reason: Some(format!("directorio faltante {}", skill_dir.display())),
        };
    }
    for rel in &entry.files {
        let normalized = normalize_registry_rel_path(rel);
        let abs = {
            let mut p = skill_dir.clone();
            for part in normalized.split('/') {
                p = p.join(part);
            }
            p
        };
        if !abs.exists() {
            return VerifyResult {
                ok: false,
                reason: Some(format!("archivo faltante {normalized}")),
            };
        }
        let expected = entry
            .sha256
            .get(rel)
            .or_else(|| entry.sha256.get(&normalized));
        let Some(expected) = expected else {
            return VerifyResult {
                ok: false,
                reason: Some(format!("sin hash registrado para {normalized}")),
            };
        };
        let actual = match sha256_file(&abs) {
            Ok(h) => h,
            Err(e) => {
                return VerifyResult {
                    ok: false,
                    reason: Some(format!("fallo al calcular hash de {normalized}: {e}")),
                };
            }
        };
        if &actual != expected {
            return VerifyResult {
                ok: false,
                reason: Some(format!("hash no coincide para {normalized}")),
            };
        }
    }
    VerifyResult {
        ok: true,
        reason: None,
    }
}

pub fn security_check_for_entry(skill_name: &str, entry: &RegistryEntry) -> InstallSecurityCheck {
    if let Some(sc) = &entry.security_check {
        return InstallSecurityCheck {
            name: skill_name.to_string(),
            status: sc.status.clone(),
            summary: sc.summary.clone(),
            findings: sc.findings.clone(),
        };
    }
    let status = if entry.review.status == "flagged" {
        "warning"
    } else {
        "ok"
    };
    let summary = if !entry.review.summary.is_empty() {
        entry.review.summary.clone()
    } else if entry.review.status == "flagged" {
        "La revisión de sincronización encontró observaciones que deberías revisar.".to_string()
    } else {
        "La revisión de sincronización no encontró problemas de seguridad.".to_string()
    };
    InstallSecurityCheck {
        name: skill_name.to_string(),
        status: status.to_string(),
        summary,
        findings: entry.review.flags.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    fn sample_entry(files: Vec<(&str, &str)>) -> RegistryEntry {
        let mut sha = HashMap::new();
        for (rel, content) in &files {
            let hash = crate::hash::sha256_buffer(content.as_bytes());
            sha.insert(rel.to_string(), hash);
        }
        let entries: Vec<(String, String)> =
            sha.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let bundle = crate::hash::bundle_hash(&entries);
        RegistryEntry {
            source: "owner/repo".to_string(),
            skill_path: "owner/repo/test-skill".to_string(),
            commit_sha: "deadbeef".to_string(),
            files: files.iter().map(|(rel, _)| rel.to_string()).collect(),
            sha256: sha,
            bundle_hash: bundle,
            review: crate::registry::types::Review {
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

    fn write_skill_files(dir: &Path, skill_name: &str, files: &[(&str, &str)]) {
        for (rel, content) in files {
            let normalized = normalize_registry_rel_path(rel);
            let mut p = dir.join(skill_name);
            for part in normalized.split('/') {
                p = p.join(part);
            }
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(p, content).unwrap();
        }
    }

    #[test]
    fn load_registry_from_dir_success() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let manifest = serde_json::json!({
            "version": 1,
            "generatedAt": "2026-01-01T00:00:00Z",
            "reviewer": {"model": "test", "promptVersion": "1.0"},
            "skills": {}
        });
        fs::write(
            reg_dir.join("index.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        let reg = load_registry_from_dir(&reg_dir).unwrap();
        assert_eq!(reg.version, 1);
    }

    #[test]
    fn load_registry_missing_returns_none() {
        let tmp = tempdir().unwrap();
        assert!(load_registry_from_dir(tmp.path()).is_none());
    }

    #[test]
    fn verify_registry_entry_ok() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let entry = sample_entry(vec![("SKILL.md", "# hi")]);
        write_skill_files(&reg_dir, "my-skill", &[("SKILL.md", "# hi")]);
        let result = verify_registry_entry("my-skill", &entry, &reg_dir);
        assert!(result.ok, "expected ok, got {:?}", result.reason);
    }

    #[test]
    fn verify_registry_entry_tampered_hash_mismatch() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let entry = sample_entry(vec![("SKILL.md", "# hi")]);
        write_skill_files(&reg_dir, "my-skill", &[("SKILL.md", "# tampered")]);
        let result = verify_registry_entry("my-skill", &entry, &reg_dir);
        assert!(!result.ok);
        assert!(result.reason.unwrap().contains("no coincide"));
    }

    #[test]
    fn verify_registry_entry_missing_file() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let mut entry = sample_entry(vec![("SKILL.md", "# hi")]);
        entry.files.push("references/MISSING.md".to_string());
        entry
            .sha256
            .insert("references/MISSING.md".to_string(), "0".repeat(64));
        write_skill_files(&reg_dir, "my-skill", &[("SKILL.md", "# hi")]);
        let result = verify_registry_entry("my-skill", &entry, &reg_dir);
        assert!(!result.ok);
        assert!(result.reason.unwrap().contains("faltante"));
    }

    #[test]
    fn verify_registry_entry_missing_dir() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let entry = sample_entry(vec![("SKILL.md", "# hi")]);
        let result = verify_registry_entry("my-skill", &entry, &reg_dir);
        assert!(!result.ok);
        assert!(result.reason.unwrap().contains("directorio faltante"));
    }

    #[test]
    fn verify_registry_entry_backslash_normalized() {
        let tmp = tempdir().unwrap();
        let reg_dir = tmp.path().join("registry");
        fs::create_dir_all(&reg_dir).unwrap();
        let mut entry = sample_entry(vec![("references\\notes.md", "notes")]);
        entry.files = vec!["references\\notes.md".to_string()];
        write_skill_files(&reg_dir, "win-skill", &[("references/notes.md", "notes")]);
        let result = verify_registry_entry("win-skill", &entry, &reg_dir);
        assert!(result.ok, "should normalize backslash: {:?}", result.reason);
    }

    #[test]
    fn security_check_for_entry_uses_security_check() {
        let mut entry = sample_entry(vec![("SKILL.md", "hi")]);
        entry.security_check = Some(crate::registry::types::SecurityCheck {
            status: "warning".to_string(),
            findings: vec!["external URL".to_string()],
            summary: "Needs review".to_string(),
            checked_at: "2026-01-01T00:00:00Z".to_string(),
        });
        let check = security_check_for_entry("my-skill", &entry);
        assert_eq!(check.status, "warning");
        assert_eq!(check.summary, "Needs review");
        assert_eq!(check.findings, vec!["external URL"]);
    }

    #[test]
    fn security_check_falls_back_to_review_flagged() {
        let mut entry = sample_entry(vec![("SKILL.md", "hi")]);
        entry.review.status = "flagged".to_string();
        entry.review.flags = vec!["broad shell".to_string()];
        entry.review.summary = "Contains broad shell".to_string();
        entry.security_check = None;
        let check = security_check_for_entry("my-skill", &entry);
        assert_eq!(check.status, "warning");
        assert_eq!(check.summary, "Contains broad shell");
        assert_eq!(check.findings, vec!["broad shell"]);
    }

    #[test]
    fn security_check_falls_back_to_review_approved() {
        let mut entry = sample_entry(vec![("SKILL.md", "hi")]);
        entry.review.status = "approved".to_string();
        entry.review.summary = "".to_string();
        entry.review.flags = vec![];
        entry.security_check = None;
        let check = security_check_for_entry("my-skill", &entry);
        assert_eq!(check.status, "ok");
        assert!(check.summary.contains("no encontró"));
    }

    #[test]
    fn get_registry_raw_base_urls_default() {
        let _guard = crate::cache::env_lock();
        unsafe {
            env::remove_var("SKILLINDEX_REGISTRY_BASE_URL");
        }
        let urls = get_registry_raw_base_urls(None);
        assert_eq!(urls.len(), 2);
        assert!(urls[0].contains(&format!("/v{}/", env!("CARGO_PKG_VERSION"))));
        assert!(urls[1].ends_with("/main/skills-registry"));
    }

    #[test]
    fn get_registry_raw_base_urls_with_custom() {
        let urls = get_registry_raw_base_urls(Some("https://example.test/skills-registry/"));
        assert_eq!(urls, vec!["https://example.test/skills-registry"]);
    }

    #[test]
    fn get_registry_raw_base_urls_env_precedence() {
        let _guard = crate::cache::env_lock();
        let prev = env::var("SKILLINDEX_REGISTRY_BASE_URL").ok();
        unsafe { env::set_var("SKILLINDEX_REGISTRY_BASE_URL", "https://env.test/registry") };
        let urls = get_registry_raw_base_urls(None);
        assert_eq!(urls, vec!["https://env.test/registry"]);
        match prev {
            Some(v) => unsafe { env::set_var("SKILLINDEX_REGISTRY_BASE_URL", v) },
            None => unsafe { env::remove_var("SKILLINDEX_REGISTRY_BASE_URL") },
        }
    }

    #[test]
    fn get_registry_dir_finds_fixture() {
        let dir = get_registry_dir();
        assert!(
            dir.join("index.json").exists(),
            "registry dir should contain index.json, got {}",
            dir.display()
        );
    }
}
