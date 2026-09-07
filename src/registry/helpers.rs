use std::env;

pub fn get_package_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn agent_folder_for(agent: &str) -> Option<&'static str> {
    for (folder, name) in crate::skills::AGENT_FOLDER_MAP {
        if *name == agent {
            return Some(folder);
        }
    }
    None
}

pub fn parse_skill_path(skill: &str) -> crate::registry::types::ParsedSkillPath {
    use crate::registry::types::ParsedSkillPath;
    if skill.starts_with("http://") || skill.starts_with("https://") {
        return ParsedSkillPath {
            repo: skill.to_string(),
            skill_name: String::new(),
            full: skill.to_string(),
        };
    }
    let parts: Vec<&str> = skill.split('/').collect();
    if parts.len() < 2 {
        return ParsedSkillPath {
            repo: skill.to_string(),
            skill_name: String::new(),
            full: skill.to_string(),
        };
    }
    let repo: String = format!("{}/{}", parts[0], parts[1]);
    let skill_name: String = parts[2..].join("/");
    ParsedSkillPath {
        repo,
        skill_name,
        full: skill.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_folder_for_known() {
        assert_eq!(agent_folder_for("claude-code"), Some(".claude"));
        assert_eq!(agent_folder_for("junie"), Some(".junie"));
        assert_eq!(agent_folder_for("codebuddy"), Some(".codebuddy"));
        assert_eq!(agent_folder_for("opencode"), Some(".opencode"));
        assert_eq!(agent_folder_for("antigravity"), Some(".antigravity"));
        assert_eq!(agent_folder_for("codex"), Some(".codex"));
        assert_eq!(agent_folder_for("crush"), Some(".crush"));
        assert_eq!(agent_folder_for("unknown"), None);
    }

    #[test]
    fn parse_skill_path_basic() {
        let p: crate::registry::ParsedSkillPath = parse_skill_path("owner/repo/hello-skill");
        assert_eq!(p.repo, "owner/repo");
        assert_eq!(p.skill_name, "hello-skill");
    }

    #[test]
    fn parse_skill_path_http() {
        let p: crate::registry::ParsedSkillPath = parse_skill_path("https://example.com/skill");
        assert_eq!(p.skill_name, "");
        assert_eq!(p.repo, "https://example.com/skill");
    }
}
