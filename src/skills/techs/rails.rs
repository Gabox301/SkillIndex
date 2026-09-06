use crate::skills::types::{DetectConfig, Technology};

pub const RAILS_TECH: Technology = Technology {
    id: "rails",
    name: "Ruby on Rails",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["config/routes.rb", "config/application.rb", "bin/rails"],
        file_extensions: &[],
        gems: &["rails"],
        config_file_content: &[],
    },
    skills: &[
        "sergiodxa/agent-skills/ruby-on-rails-best-practices",
        "lucianghinda/superpowers-ruby/rails-guides",
        "igmarin/rails-agent-skills/rails-stack-conventions",
        "igmarin/rails-agent-skills/rails-code-review",
        "igmarin/rails-agent-skills/rails-migration-safety",
        "igmarin/rails-agent-skills/rails-security-review",
        "ombulabs/claude-code_rails-upgrade-skill/rails-upgrade",
    ],
};
