use crate::skills::types::{DetectConfig, Technology};

pub const REDIS_RUBY_TECH: Technology = Technology {
    id: "redis-ruby",
    name: "Redis (Ruby)",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &["redis", "sidekiq", "resque", "redis-rails"],
        config_file_content: &[],
    },
    skills: &["Gabox301/SkillIndex/redis-development"],
};
