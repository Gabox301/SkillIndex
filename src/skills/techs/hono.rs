use crate::skills::types::{DetectConfig, Technology};

pub const HONO_TECH: Technology = Technology {
    id: "hono",
    name: "Hono",
    detect: DetectConfig {
        packages: &["hono"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/hono"],
};
