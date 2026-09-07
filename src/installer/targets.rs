use std::path::{Path, PathBuf};

use crate::registry::agent_folder_for;

/// Folder used when no mapped agent applies (explicit `universal` or fallback).
pub const UNIVERSAL_SKILLS_FOLDER: &str = ".agents";

/// A concrete destination where a skill must be installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    /// Agent folder relative to the project root, e.g. `.kiro` or `.agents`.
    pub folder: String,
    /// Absolute path to the folder that holds installed skill directories.
    pub skills_dir: PathBuf,
}

/// Resolve the concrete destination folders where a skill must be installed.
///
/// Each mapped agent (kiro, claude-code, …) resolves to its own folder, so a
/// skill is copied directly into every agent's `skills` directory. There is no
/// canonical `.agents` copy plus per-agent symlinks anymore — every target is an
/// independent copy, which removes the universal + agent double-path duplication.
///
/// `.agents` is only used when `universal` is the sole destination: either the
/// user chose it explicitly (`-a universal`) or nothing else resolved. When a
/// mapped agent is present, the auto-detector's `universal` entry is ignored so
/// we never recreate the universal + agent double path.
pub fn resolve_install_targets(project_dir: &Path, agents: &[String]) -> Vec<InstallTarget> {
    let mut folders: Vec<String> = Vec::new();
    for agent in agents {
        if agent == "universal" {
            continue;
        }
        if let Some(folder) = agent_folder_for(agent) {
            let folder: String = folder.to_string();
            if !folders.contains(&folder) {
                folders.push(folder);
            }
        }
    }
    if folders.is_empty() {
        folders.push(UNIVERSAL_SKILLS_FOLDER.to_string());
    }
    folders
        .into_iter()
        .map(|folder: String| {
            let skills_dir: PathBuf = project_dir.join(&folder).join("skills");
            InstallTarget { folder, skills_dir }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolve_install_targets_universal_fallback() {
        let targets: Vec<InstallTarget> = resolve_install_targets(Path::new("/proj"), &[]);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].folder, ".agents");
    }

    #[test]
    fn resolve_install_targets_skips_universal_when_mapped_present() {
        let targets: Vec<InstallTarget> = resolve_install_targets(
            Path::new("/proj"),
            &["universal".to_string(), "claude-code".to_string()],
        );
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].folder, ".claude");
    }

    #[test]
    fn resolve_install_targets_dedupes_folders() {
        let targets: Vec<InstallTarget> = resolve_install_targets(
            Path::new("/proj"),
            &["claude-code".to_string(), "claude-code".to_string()],
        );
        assert_eq!(targets.len(), 1);
    }
}
