use crate::skills::types::{DetectConfig, Technology};

pub const TASTE_SKILL: Technology = Technology {
    id: "taste-skill",
    name: "Taste Skill",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Leonxlnx/taste-skill/taste-skill"],
};
