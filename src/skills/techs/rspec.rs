use crate::skills::types::{DetectConfig, Technology};

pub const RSPEC_TECH: Technology = Technology {
    id: "rspec",
    name: "RSpec",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[".rspec"],
        file_extensions: &[],
        gems: &["rspec", "rspec-rails"],
        config_file_content: &[],
    },
    skills: &[
        "igmarin/rails-agent-skills/rspec-best-practices",
        "igmarin/rails-agent-skills/rspec-service-testing",
        "lucianghinda/superpowers-ruby/test-driven-development",
    ],
};
