use crate::skills::types::{DetectConfig, Technology};

pub const ELYSIA_TECH: Technology = Technology {
    id: "elysia",
    name: "Elysia",
    detect: DetectConfig {
        packages: &["elysia"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["elysiajs/skills/elysiajs"],
};
