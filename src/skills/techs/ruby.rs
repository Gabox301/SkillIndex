use crate::skills::types::{DetectConfig, Technology};

pub const RUBY_TECH: Technology = Technology {
    id: "ruby",
    name: "Ruby",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["Gemfile", "Gemfile.lock", ".ruby-version", ".ruby-gemset"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["lucianghinda/superpowers-ruby/ruby"],
};
