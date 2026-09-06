use crate::skills::types::{DetectConfig, Technology};

pub const BUN_TECH: Technology = Technology {
    id: "bun",
    name: "Bun",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["bun.lockb", "bun.lock", "bunfig.toml"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/bun"],
};
