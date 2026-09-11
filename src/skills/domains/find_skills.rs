use crate::skills::types::{DetectConfig, Technology};

pub const FIND_SKILLS: Technology = Technology {
    id: "find-skills",
    name: "Find Skills",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/find-skills"],
};
