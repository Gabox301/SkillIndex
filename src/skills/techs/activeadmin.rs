use crate::skills::types::{DetectConfig, Technology};

pub const ACTIVEADMIN_TECH: Technology = Technology {
    id: "activeadmin",
    name: "ActiveAdmin",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &["activeadmin"],
        config_file_content: &[],
    },
    skills: &["igmarin/rails-agent-skills/rails-stack-conventions"],
};
