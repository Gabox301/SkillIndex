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
        "Gabox301/SkillIndex/rspec-best-practices",
        "Gabox301/SkillIndex/rspec-service-testing",
        "lucianghinda/superpowers-ruby/test-driven-development",
    ],
};
