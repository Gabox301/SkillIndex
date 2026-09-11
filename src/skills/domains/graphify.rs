use crate::skills::types::{DetectConfig, Technology};

pub const GRAPHIFY: Technology = Technology {
    id: "graphify",
    name: "Graphify",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/graphify"],
};
