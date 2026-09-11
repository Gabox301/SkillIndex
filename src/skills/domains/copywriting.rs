use crate::skills::types::{DetectConfig, Technology};

pub const COPYWRITING: Technology = Technology {
    id: "copywriting",
    name: "Copywriting",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/copywriting"],
};
