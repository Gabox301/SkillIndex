use crate::skills::types::{DetectConfig, Technology};

pub const TYPESCRIPT_TECH: Technology = Technology {
    id: "typescript",
    name: "TypeScript",
    detect: DetectConfig {
        packages: &["typescript"],
        package_patterns: &[],
        config_files: &["tsconfig.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["wshobson/agents/typescript-advanced-types"],
};
