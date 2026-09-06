use crate::skills::types::{DetectConfig, Technology};

pub const DEVISE_TECH: Technology = Technology {
    id: "devise",
    name: "Devise",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &["devise"],
        config_file_content: &[],
    },
    skills: &["igmarin/rails-agent-skills/rails-security-review"],
};
