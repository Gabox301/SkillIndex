use crate::skills::types::{DetectConfig, Technology};

pub const OXLINT_TECH: Technology = Technology {
    id: "oxlint",
    name: "oxlint",
    detect: DetectConfig {
        packages: &["oxlint"],
        package_patterns: &[],
        config_files: &[".oxlintrc.json", "oxlint.config.ts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["delexw/claude-code-misc/oxlint"],
};
