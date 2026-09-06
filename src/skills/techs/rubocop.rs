use crate::skills::types::{DetectConfig, Technology};

pub const RUBOCOP_TECH: Technology = Technology {
    id: "rubocop",
    name: "RuboCop",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[".rubocop.yml"],
        file_extensions: &[],
        gems: &["rubocop", "rubocop-rails"],
        config_file_content: &[],
    },
    skills: &[
        "TheBushidoCollective/han/rubocop-configuration",
        "TheBushidoCollective/han/rubocop-cops",
        "TheBushidoCollective/han/rubocop-integration",
    ],
};
