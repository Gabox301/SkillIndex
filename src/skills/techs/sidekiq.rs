use crate::skills::types::{DetectConfig, Technology};

pub const SIDEKIQ_TECH: Technology = Technology {
    id: "sidekiq",
    name: "Sidekiq",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &["sidekiq"],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/rails-background-jobs"],
};
