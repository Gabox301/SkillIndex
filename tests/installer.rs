//! Integration tests for skill installation.
//! Mirrors installer.test.ts — one test per `it(...)` block.

use skillindex::hash::{bundle_hash, sha256_buffer};
use skillindex::installer::{
    InstallOptions, copy_dir, ensure_symlink_to, install_all, install_skill, rel_path_from_to,
    update_skills_lock,
};
use skillindex::registry::{Registry, RegistryEntry, Review, Reviewer, agent_folder_for};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// ── Helpers ────────────────────────────────────────────────────────

fn write_file(base: &Path, rel: &str, content: &str) {
    let p = base.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

fn make_entry(name: &str, source: &str, files: &[(&str, &str)]) -> RegistryEntry {
    let mut sha_map = HashMap::new();
    for (rel, content) in files {
        sha_map.insert(rel.to_string(), sha256_buffer(content.as_bytes()));
    }
    let mut sorted: Vec<(String, String)> = sha_map
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let bh = bundle_hash(&sorted);
    RegistryEntry {
        source: source.to_string(),
        skill_path: format!("{source}/{name}"),
        commit_sha: "deadbeef".to_string(),
        files: sha_map.keys().cloned().collect(),
        sha256: sha_map,
        bundle_hash: bh,
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

fn build_registry(reg_dir: &Path, entries: Vec<(String, RegistryEntry)>) {
    fs::create_dir_all(reg_dir).unwrap();
    let mut skills = HashMap::new();
    for (name, entry) in &entries {
        // Write actual skill files
        for rel_path in &entry.files {
            let content = entry.sha256.get(rel_path).map(|_| "").unwrap_or("");
            write_file(reg_dir, &format!("{name}/{rel_path}"), content);
        }
        skills.insert(name.clone(), entry.clone());
    }
    let reg = Registry {
        version: 1,
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        reviewer: Reviewer {
            model: "test-model".to_string(),
            prompt_version: "1.0.0".to_string(),
        },
        skills,
    };
    fs::write(
        reg_dir.join("index.json"),
        serde_json::to_string_pretty(&reg).unwrap(),
    )
    .unwrap();
}

fn registry_content(reg_dir: &Path, skill_name: &str, rel: &str, content: &str) {
    write_file(reg_dir, &format!("{skill_name}/{rel}"), content);
    // Update sha256 in index.json
    let index_path = reg_dir.join("index.json");
    let mut reg: Registry =
        serde_json::from_str(&fs::read_to_string(&index_path).unwrap()).unwrap();
    if let Some(entry) = reg.skills.get_mut(skill_name) {
        let hash = sha256_buffer(content.as_bytes());
        entry.sha256.insert(rel.to_string(), hash.clone());
        if !entry.files.contains(&rel.to_string()) {
            entry.files.push(rel.to_string());
        }
        let mut sorted: Vec<(String, String)> = entry
            .sha256
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        entry.bundle_hash = bundle_hash(&sorted);
    }
    fs::write(index_path, serde_json::to_string_pretty(&reg).unwrap()).unwrap();
}

// ── agentFolderFor ────────────────────────────────────────────────

#[test]
fn agent_folder_for_claude_code() {
    assert_eq!(agent_folder_for("claude-code"), Some(".claude"));
}

#[test]
fn agent_folder_for_junie() {
    assert_eq!(agent_folder_for("junie"), Some(".junie"));
}

#[test]
fn agent_folder_for_codebuddy() {
    assert_eq!(agent_folder_for("codebuddy"), Some(".codebuddy"));
}

#[test]
fn agent_folder_for_unknown_returns_none() {
    assert_eq!(agent_folder_for("nope"), None);
}

// ── copy_dir ──────────────────────────────────────────────────────

#[test]
fn copy_dir_copies_files_and_subdirs() {
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

// ── ensure_symlink_to ─────────────────────────────────────────────

#[test]
fn ensure_symlink_creates_link_or_copy_fallback() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("target");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("file.txt"), b"data").unwrap();
    let link = tmp.path().join("link/skill");
    ensure_symlink_to(&target, &link).unwrap();
    assert!(link.exists() || link.is_symlink());
    assert_eq!(fs::read_to_string(link.join("file.txt")).unwrap(), "data");
}

// ── rel_path_from_to ──────────────────────────────────────────────

#[test]
fn rel_path_sibling_dirs() {
    assert_eq!(
        rel_path_from_to(Path::new("/a/b/c"), Path::new("/a/b/d/e")),
        "../d/e"
    );
}

#[test]
fn rel_path_child() {
    assert_eq!(
        rel_path_from_to(Path::new("/a/b"), Path::new("/a/b/c/d")),
        "c/d"
    );
}

#[test]
fn rel_path_same_dir() {
    assert_eq!(
        rel_path_from_to(Path::new("/a/b/c"), Path::new("/a/b/c")),
        "."
    );
}

// ── update_skills_lock ────────────────────────────────────────────

#[test]
fn lock_preserves_existing_entries_and_sorts_keys() {
    let tmp = tempdir().unwrap();
    let project = tmp.path();
    fs::write(
        project.join("skills-lock.json"),
        serde_json::to_string(&serde_json::json!({
            "version": 1,
            "skills": {
                "zebra": {"source":"x/y","sourceType":"skillindex-registry","computedHash":"z"}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let entry = make_entry("alpha", "owner/repo", &[("SKILL.md", "# a")]);
    update_skills_lock(project, "alpha", &entry).unwrap();

    let content = fs::read_to_string(project.join("skills-lock.json")).unwrap();
    assert!(content.ends_with('\n'));
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    let keys: Vec<String> = v["skills"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, vec!["alpha", "zebra"]);
    assert_eq!(v["skills"]["zebra"]["source"], "x/y");
}

// ── install_skill ─────────────────────────────────────────────────

#[tokio::test]
async fn install_skill_copies_files_to_agents_and_updates_lock() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry(
        "hello-skill",
        "owner/repo",
        &[("SKILL.md", "# hello"), ("references/notes.md", "notes")],
    );
    build_registry(&reg_dir, vec![("hello-skill".into(), entry)]);
    registry_content(&reg_dir, "hello-skill", "SKILL.md", "# hello");
    registry_content(&reg_dir, "hello-skill", "references/notes.md", "notes");

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_skill("owner/repo/hello-skill", &[], opts).await;
    assert!(result.success, "install failed: {}", result.output);

    assert!(
        project_dir
            .join(".agents/skills/hello-skill/SKILL.md")
            .exists()
    );
    assert!(
        project_dir
            .join(".agents/skills/hello-skill/references/notes.md")
            .exists()
    );

    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(project_dir.join("skills-lock.json")).unwrap())
            .unwrap();
    assert_eq!(lock["skills"]["hello-skill"]["source"], "owner/repo");
    assert_eq!(
        lock["skills"]["hello-skill"]["sourceType"],
        "skillindex-registry"
    );
}

#[tokio::test]
async fn install_skill_rejects_when_skill_not_in_registry() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry("known", "owner/repo", &[("SKILL.md", "# known")]);
    build_registry(&reg_dir, vec![("known".into(), entry)]);

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_skill("owner/repo/unknown", &[], opts).await;
    assert!(!result.success);
    assert!(result.output.contains("no encontrada") || result.output.contains("not found"));
}

#[tokio::test]
async fn install_skill_rejects_disallowed_zip_files() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry(
        "archive-skill",
        "owner/repo",
        &[("downloads/tool.ZIP", "zip")],
    );
    build_registry(&reg_dir, vec![("archive-skill".into(), entry)]);
    // Remove the file so it must be downloaded — and .ZIP check fires before download
    fs::remove_dir_all(reg_dir.join("archive-skill")).ok();

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_skill("owner/repo/archive-skill", &[], opts).await;
    assert!(!result.success);
    assert!(
        result.output.contains("no permitido") || result.output.contains("not allowed"),
        "expected 'no permitido' in: {}",
        result.output
    );
}

#[tokio::test]
async fn install_skill_copies_directly_into_each_mapped_agent_folder() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry("s1", "owner/repo", &[("SKILL.md", "# s1")]);
    build_registry(&reg_dir, vec![("s1".into(), entry)]);
    registry_content(&reg_dir, "s1", "SKILL.md", "# s1");

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_skill(
        "owner/repo/s1",
        &["claude-code".to_string(), "junie".to_string()],
        opts,
    )
    .await;
    assert!(result.success, "install failed: {}", result.output);

    // Each mapped agent gets its own real copy — no symlinks, no canonical dir.
    let claude_skill = project_dir.join(".claude/skills/s1");
    let junie_skill = project_dir.join(".junie/skills/s1");
    assert!(claude_skill.join("SKILL.md").exists());
    assert!(junie_skill.join("SKILL.md").exists());
    assert_eq!(
        fs::read_to_string(claude_skill.join("SKILL.md")).unwrap(),
        "# s1"
    );
    assert_eq!(
        fs::read_to_string(junie_skill.join("SKILL.md")).unwrap(),
        "# s1"
    );
    // The copies are real directories, not symlinks.
    assert!(
        !claude_skill
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[tokio::test]
async fn install_skill_does_not_create_agents_when_mapped_agent_present() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry("s1", "owner/repo", &[("SKILL.md", "# s1")]);
    build_registry(&reg_dir, vec![("s1".into(), entry)]);
    registry_content(&reg_dir, "s1", "SKILL.md", "# s1");

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    // `universal` alongside a mapped agent must not spawn a second `.agents` path.
    let result = install_skill(
        "owner/repo/s1",
        &["universal".to_string(), "kiro-cli".to_string()],
        opts,
    )
    .await;
    assert!(result.success, "install failed: {}", result.output);
    assert!(project_dir.join(".kiro/skills/s1/SKILL.md").exists());
    assert!(!project_dir.join(".agents/skills/s1").exists());
}

#[tokio::test]
async fn install_skill_uses_agents_only_when_universal_is_explicit() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry("s1", "owner/repo", &[("SKILL.md", "# s1")]);
    build_registry(&reg_dir, vec![("s1".into(), entry)]);
    registry_content(&reg_dir, "s1", "SKILL.md", "# s1");

    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_skill("owner/repo/s1", &["universal".to_string()], opts).await;
    assert!(result.success, "install failed: {}", result.output);
    assert!(project_dir.join(".agents/skills/s1/SKILL.md").exists());
    assert!(!project_dir.join(".kiro/skills/s1").exists());
}

#[tokio::test]
async fn install_skill_reinstalls_only_the_missing_target() {
    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let entry = make_entry("s1", "owner/repo", &[("SKILL.md", "# s1")]);
    build_registry(&reg_dir, vec![("s1".into(), entry)]);
    registry_content(&reg_dir, "s1", "SKILL.md", "# s1");

    let agents = ["claude-code".to_string(), "junie".to_string()];
    let first = install_skill(
        "owner/repo/s1",
        &agents,
        InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            ..Default::default()
        },
    )
    .await;
    assert!(first.success, "first install failed: {}", first.output);

    // Remove one target; a second install must restore just that one.
    fs::remove_dir_all(project_dir.join(".junie/skills/s1")).unwrap();
    let second = install_skill(
        "owner/repo/s1",
        &agents,
        InstallOptions {
            project_dir: Some(project_dir.clone()),
            registry_dir: Some(reg_dir.clone()),
            ..Default::default()
        },
    )
    .await;
    assert!(second.success, "second install failed: {}", second.output);
    assert!(project_dir.join(".claude/skills/s1/SKILL.md").exists());
    assert!(project_dir.join(".junie/skills/s1/SKILL.md").exists());
}

// ── install_all ───────────────────────────────────────────────────

#[tokio::test]
async fn install_all_collects_security_checks() {
    use skillindex::installer::SkillEntry;

    let tmp = tempdir().unwrap();
    let reg_dir = tmp.path().join("registry");
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let e1 = make_entry("first-skill", "owner/repo", &[("SKILL.md", "# first")]);
    let mut e2 = make_entry("second-skill", "owner/repo", &[("SKILL.md", "# second")]);
    e2.security_check = Some(skillindex::registry::SecurityCheck {
        status: "warning".to_string(),
        findings: vec!["manual review".to_string()],
        summary: "Needs manual review.".to_string(),
        checked_at: "2026-01-01T00:00:00Z".to_string(),
    });

    build_registry(
        &reg_dir,
        vec![("first-skill".into(), e1), ("second-skill".into(), e2)],
    );
    registry_content(&reg_dir, "first-skill", "SKILL.md", "# first");
    registry_content(&reg_dir, "second-skill", "SKILL.md", "# second");

    let skill_entries = vec![
        SkillEntry {
            skill: "owner/repo/first-skill".to_string(),
            sources: vec![],
            installed: false,
        },
        SkillEntry {
            skill: "owner/repo/second-skill".to_string(),
            sources: vec![],
            installed: false,
        },
    ];
    let opts = InstallOptions {
        project_dir: Some(project_dir.clone()),
        registry_dir: Some(reg_dir.clone()),
        ..Default::default()
    };
    let result = install_all(skill_entries, vec![], opts).await;
    assert_eq!(result.installed, 2);
    let mut names: Vec<&str> = result
        .security_checks
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    names.sort();
    assert_eq!(names, vec!["first-skill", "second-skill"]);
    assert_eq!(
        result
            .security_checks
            .iter()
            .find(|c| c.name == "second-skill")
            .unwrap()
            .status,
        "warning"
    );
}
