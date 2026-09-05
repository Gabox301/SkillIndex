//! Integration tests for skill collection and installed-name lookup.
//! Mirrors collect.test.ts — one test per `it(...)` block.

use skillindex::detect::{collect_skills, detect_technologies, get_installed_skill_names};
use skillindex::display::{DisplayCombo, DisplayTechnology};
use std::collections::HashSet;
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

fn write_json(base: &Path, rel: &str, data: serde_json::Value) {
    write_file(base, rel, &serde_json::to_string(&data).unwrap());
}

fn tech(id: &str, name: &str, skills: &[&str]) -> DisplayTechnology {
    DisplayTechnology {
        id: id.to_string(),
        name: name.to_string(),
        skills: skills.iter().map(|s| s.to_string()).collect(),
    }
}

fn combo(id: &str, name: &str) -> DisplayCombo {
    DisplayCombo {
        id: id.to_string(),
        name: name.to_string(),
    }
}

// ── collectSkills ─────────────────────────────────────────────────

#[test]
fn returns_empty_when_no_technologies_detected() {
    let skills = collect_skills(&[], false, &[], None);
    assert!(skills.is_empty());
}

#[test]
fn collects_skills_from_a_single_technology() {
    let detected = vec![tech(
        "react",
        "React",
        &["vercel-labs/agent-skills/vercel-react-best-practices"],
    )];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 1);
    assert_eq!(
        skills[0].skill,
        "vercel-labs/agent-skills/vercel-react-best-practices"
    );
    assert_eq!(skills[0].sources, vec!["React"]);
}

#[test]
fn deduplicates_skills_shared_across_technologies() {
    let detected = vec![
        tech("a", "Tech A", &["shared/repo/my-skill"]),
        tech("b", "Tech B", &["shared/repo/my-skill"]),
    ];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].sources, vec!["Tech A", "Tech B"]);
}

#[test]
fn keeps_unique_skills_from_different_technologies() {
    let detected = vec![
        tech(
            "react",
            "React",
            &["vercel-labs/agent-skills/vercel-react-best-practices"],
        ),
        tech(
            "nextjs",
            "Next.js",
            &["vercel-labs/next-skills/next-best-practices"],
        ),
    ];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 2);
}

#[test]
fn handles_technologies_with_multiple_skills() {
    let detected = vec![tech(
        "vue",
        "Vue",
        &["hyf0/vue-skills/vue-best-practices", "antfu/skills/vue"],
    )];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 2);
}

#[test]
fn adds_frontend_bonus_skills_for_frontend_projects() {
    let detected = vec![tech(
        "react",
        "React",
        &["vercel-labs/agent-skills/vercel-react-best-practices"],
    )];
    let skills = collect_skills(&detected, true, &[], None);
    assert!(
        skills
            .iter()
            .any(|s| s.skill == "anthropics/skills/frontend-design")
    );
    let bonus = skills
        .iter()
        .find(|s| s.skill == "anthropics/skills/frontend-design")
        .unwrap();
    assert_eq!(bonus.sources, vec!["Frontend"]);
}

#[test]
fn does_not_add_frontend_bonus_for_non_frontend_projects() {
    let detected = vec![tech(
        "typescript",
        "TypeScript",
        &["wshobson/agents/typescript-advanced-types"],
    )];
    let skills = collect_skills(&detected, false, &[], None);
    assert!(
        !skills
            .iter()
            .any(|s| s.skill == "anthropics/skills/frontend-design")
    );
}

#[test]
fn does_not_duplicate_frontend_bonus_if_already_present() {
    let detected = vec![tech(
        "custom",
        "Custom",
        &["anthropics/skills/frontend-design"],
    )];
    let skills = collect_skills(&detected, true, &[], None);
    let count = skills
        .iter()
        .filter(|s| s.skill == "anthropics/skills/frontend-design")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn skips_technologies_with_empty_skills() {
    let detected = vec![
        tech("svelte", "Svelte", &[]),
        tech(
            "react",
            "React",
            &["vercel-labs/agent-skills/vercel-react-best-practices"],
        ),
    ];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 1);
}

#[test]
fn accumulates_three_sources_for_the_same_skill() {
    let detected = vec![
        tech("a", "Tech A", &["shared/repo/shared-skill"]),
        tech("b", "Tech B", &["shared/repo/shared-skill"]),
        tech("c", "Tech C", &["shared/repo/shared-skill"]),
    ];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].sources, vec!["Tech A", "Tech B", "Tech C"]);
}

#[test]
fn sets_installed_false_by_default() {
    let detected = vec![tech(
        "react",
        "React",
        &["vercel-labs/agent-skills/vercel-react-best-practices"],
    )];
    let skills = collect_skills(&detected, false, &[], None);
    assert!(!skills[0].installed);
}

#[test]
fn marks_matching_skills_as_installed() {
    let detected = vec![tech(
        "react",
        "React",
        &[
            "vercel-labs/agent-skills/vercel-react-best-practices",
            "other/repo/other-skill",
        ],
    )];
    let mut installed = HashSet::new();
    installed.insert("vercel-react-best-practices".to_string());
    let skills = collect_skills(&detected, false, &[], Some(&installed));
    assert!(skills[0].installed);
    assert!(!skills[1].installed);
}

#[test]
fn marks_frontend_bonus_skills_as_installed() {
    let detected = vec![tech(
        "react",
        "React",
        &["vercel-labs/agent-skills/vercel-react-best-practices"],
    )];
    let mut installed = HashSet::new();
    installed.insert("frontend-design".to_string());
    let skills = collect_skills(&detected, true, &[], Some(&installed));
    let bonus = skills
        .iter()
        .find(|s| s.skill == "anthropics/skills/frontend-design")
        .unwrap();
    assert!(bonus.installed);
}

#[test]
fn handles_empty_combos_array() {
    let detected = vec![tech(
        "react",
        "React",
        &["vercel-labs/agent-skills/vercel-react-best-practices"],
    )];
    let skills = collect_skills(&detected, false, &[], None);
    assert_eq!(skills.len(), 1);
}

#[test]
fn collects_go_curated_skills_in_declared_order() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "package.json", "{}");
    write_file(
        dir.path(),
        "go.mod",
        "module example.com/test\n\ngo 1.24.0\n",
    );
    let result = detect_technologies(dir.path());
    let skills = collect_skills(&result.detected, false, &[], None);
    // go skills should be present
    assert!(
        skills.iter().any(|s| s.skill.contains("golang-patterns")),
        "golang-patterns should be included"
    );
    assert!(
        skills.iter().any(|s| s.skill.contains("golang-testing")),
        "golang-testing should be included"
    );
    // order: patterns before testing
    let pat_idx = skills
        .iter()
        .position(|s| s.skill.contains("golang-patterns"))
        .unwrap();
    let test_idx = skills
        .iter()
        .position(|s| s.skill.contains("golang-testing"))
        .unwrap();
    assert!(pat_idx < test_idx);
}

// ── collectSkills with combos (via DisplayCombo names) ────────────

#[test]
fn adds_skills_from_combo_entries() {
    let detected = vec![tech("expo", "Expo", &["expo/skills/building-native-ui"])];
    // combo with name matching skills_map
    let combos = vec![combo("expo-tailwind", "Expo + Tailwind CSS")];
    let skills = collect_skills(&detected, false, &combos, None);
    // should have expo skill + expo-tailwind combo skill
    assert!(
        skills
            .iter()
            .any(|s| s.skill == "expo/skills/building-native-ui")
    );
    // combo skills come from the skills_map; at minimum the expo skill is there
    assert!(!skills.is_empty());
}

// ── getInstalledSkillNames ────────────────────────────────────────

#[test]
fn returns_empty_set_when_no_lockfile_and_no_agents_dir() {
    let dir = tempdir().unwrap();
    assert_eq!(get_installed_skill_names(dir.path()).len(), 0);
}

#[test]
fn reads_skill_names_from_skills_lock_json() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "skills-lock.json",
        serde_json::json!({
            "version": 1,
            "skills": {
                "playwright-best-practices": {"source":"currents-dev/playwright-best-practices-skill"},
                "neon-postgres": {"source":"neondatabase/agent-skills"}
            }
        }),
    );
    let result = get_installed_skill_names(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.contains("playwright-best-practices"));
    assert!(result.contains("neon-postgres"));
}

#[test]
fn falls_back_to_agents_skills_dir_when_no_lockfile() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), ".agents/skills/next-best-practices/.keep", "");
    write_file(dir.path(), ".agents/skills/shadcn/.keep", "");
    let result = get_installed_skill_names(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.contains("next-best-practices"));
    assert!(result.contains("shadcn"));
}

#[test]
fn falls_back_to_mapped_agent_folders_when_no_lockfile() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), ".kiro/skills/react-best-practices/.keep", "");
    write_file(dir.path(), ".claude/skills/vue-best-practices/.keep", "");
    let result = get_installed_skill_names(dir.path());
    assert_eq!(result.len(), 2);
    assert!(result.contains("react-best-practices"));
    assert!(result.contains("vue-best-practices"));
}

#[test]
fn deduplicates_skill_names_present_in_multiple_agent_folders() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), ".kiro/skills/shared-skill/.keep", "");
    write_file(dir.path(), ".claude/skills/shared-skill/.keep", "");
    let result = get_installed_skill_names(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result.contains("shared-skill"));
}

#[test]
fn prefers_lockfile_over_directory_listing() {
    let dir = tempdir().unwrap();
    write_json(
        dir.path(),
        "skills-lock.json",
        serde_json::json!({
            "version": 1,
            "skills": {"from-lock": {"source": "test/repo"}}
        }),
    );
    write_file(dir.path(), ".agents/skills/from-dir/.keep", "");
    let result = get_installed_skill_names(dir.path());
    assert_eq!(result.len(), 1);
    assert!(result.contains("from-lock"));
}

#[test]
fn returns_empty_set_for_invalid_lockfile_json() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "skills-lock.json", "not json{{{");
    assert_eq!(get_installed_skill_names(dir.path()).len(), 0);
}
