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
        "Gabox301/SkillIndex/rails-stack-conventions",
        "Gabox301/SkillIndex/rails-code-review",
        "Gabox301/SkillIndex/rails-migration-safety",
        "Gabox301/SkillIndex/rails-security-review",
        "ombulabs/claude-code_rails-upgrade-skill/rails-upgrade",
    ],
};
