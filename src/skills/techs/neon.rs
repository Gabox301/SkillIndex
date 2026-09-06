use crate::skills::types::{DetectConfig, Technology};

pub const NEON_TECH: Technology = Technology {
    id: "neon",
    name: "Neon Postgres",
    detect: DetectConfig {
        packages: &["@neondatabase/serverless"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["neondatabase/agent-skills/neon-postgres"],
};
