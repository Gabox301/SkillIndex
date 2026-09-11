use crate::skills::types::{DetectConfig, Technology};

pub const POSTGRES_RUBY_TECH: Technology = Technology {
    id: "postgres-ruby",
    name: "PostgreSQL",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &["pg"],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/rails-migration-safety"],
};
