//! Integration tests for agent detection.
//! Mirrors detect-agents.test.ts — one test per `it(...)` block.
//!
//! Note: detect_agents() scans the HOME directory. All tests use
//! detect_agents_in_home(Some(&tmp)) to avoid mutating the real home directory.

use skillindex::detect::detect_agents_in_home;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// ── Helpers ────────────────────────────────────────────────────────

fn mkdir(base: &Path, rel: &str) {
    fs::create_dir_all(base.join(rel)).unwrap();
}

// ── detectAgents ──────────────────────────────────────────────────

#[test]
fn always_includes_universal_as_first_entry() {
    let dir = tempdir().unwrap();
    let agents = detect_agents_in_home(Some(dir.path()));
    assert_eq!(agents[0], "universal");
    assert_eq!(agents.len(), 1);
}

#[test]
fn detects_claude_code_from_dot_claude_skills() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".claude/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert!(agents.contains(&"universal".to_string()));
    assert!(agents.contains(&"claude-code".to_string()));
}

#[test]
fn detects_junie_from_dot_junie_skills() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".junie/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert!(agents.contains(&"junie".to_string()));
}

#[test]
fn detects_codebuddy_from_dot_codebuddy_skills() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".codebuddy/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert!(agents.contains(&"codebuddy".to_string()));
}

#[test]
fn detects_kiro_cli_from_dot_kiro_skills() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".kiro/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert!(agents.contains(&"kiro-cli".to_string()));
}

#[test]
fn detects_multiple_agents() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".claude/skills");
    mkdir(dir.path(), ".cline/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert_eq!(agents[0], "universal");
    assert!(agents.contains(&"claude-code".to_string()));
    assert!(agents.contains(&"cline".to_string()));
    assert_eq!(agents.len(), 3);
}

#[test]
fn detects_agent_folders_even_without_skills_subdirectory() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".claude")).unwrap();
    fs::create_dir_all(dir.path().join(".cursor")).unwrap();
    let agents = detect_agents_in_home(Some(dir.path()));
    assert!(agents.contains(&"universal".to_string()));
    assert!(agents.contains(&"claude-code".to_string()));
    assert!(agents.contains(&"cursor".to_string()));
}

#[test]
fn ignores_unknown_folders_with_skills_subdirectory() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".unknown-editor/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert_eq!(agents, vec!["universal"]);
}

#[test]
fn universal_is_always_first_regardless_of_other_agents() {
    let dir = tempdir().unwrap();
    mkdir(dir.path(), ".junie/skills");
    mkdir(dir.path(), ".claude/skills");
    let agents = detect_agents_in_home(Some(dir.path()));
    assert_eq!(agents[0], "universal");
}
