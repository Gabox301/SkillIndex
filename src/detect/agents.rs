use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::detect::helpers::get_agent_folder_map_slice;

pub fn detect_agents(project_dir: &Path) -> Vec<String> {
    let mut agents: Vec<String> = vec!["universal".to_string()];
    let map: Vec<(String, String)> = get_agent_folder_map();
    for (folder, agent) in map {
        let folder_path: std::path::PathBuf = project_dir.join(&folder);
        let skills_path: std::path::PathBuf = folder_path.join("skills");
        if folder_path.exists() || skills_path.exists() {
            agents.push(agent);
        }
    }
    agents
}

pub fn get_all_possible_agents() -> Vec<String> {
    get_agent_folder_map()
        .into_iter()
        .map(|(_, agent)| agent)
        .collect()
}

fn get_agent_folder_map() -> Vec<(String, String)> {
    get_agent_folder_map_slice()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub fn get_installed_skill_names(project_dir: &Path) -> HashSet<String> {
    let lock_path: std::path::PathBuf = project_dir.join("skills-lock.json");
    if let Ok(data) = fs::read_to_string(&lock_path)
        && let Ok(v) = serde_json::from_str::<Value>(&data)
        && let Some(obj) = v.get("skills").and_then(|x| x.as_object())
    {
        return obj.keys().cloned().collect();
    }
    // Fallback when there is no lockfile: scan every known skills folder. Skills
    // now live under each mapped agent folder (e.g. `.kiro/skills`), and `.agents`
    // is only used for the universal destination.
    let mut folders: Vec<String> = get_agent_folder_map()
        .into_iter()
        .map(|(folder, _agent)| folder)
        .collect();
    folders.push(".agents".to_string());
    let mut set: HashSet<String> = HashSet::new();
    for folder in folders {
        let skills_dir: std::path::PathBuf = project_dir.join(&folder).join("skills");
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type()
                    && ft.is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    set.insert(name.to_string());
                }
            }
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn get_installed_from_lock() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        fs::write(
            dir.path().join("skills-lock.json"),
            r#"{"version":1,"skills":{"my-skill":{"source":"x/y","sourceType":"skillindex-registry","computedHash":"abc"}}}"#,
        )
        .unwrap();
        let set: HashSet<String> = get_installed_skill_names(dir.path());
        assert!(set.contains("my-skill"));
    }

    #[test]
    fn detect_agents_returns_universal() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let agents: Vec<String> = detect_agents(dir.path());
        assert!(agents.contains(&"universal".to_string()));
    }
}
