use crate::skills::types::{DetectConfig, Technology};

pub const TURBOREPO_TECH: Technology = Technology {
    id: "turborepo",
    name: "Turborepo",
    detect: DetectConfig {
        packages: &["turbo"],
        package_patterns: &[],
        config_files: &["turbo.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["vercel/turborepo/turborepo"],
};
