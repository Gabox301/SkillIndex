use crate::skills::types::{DetectConfig, Technology};

pub const PHP_TECH: Technology = Technology {
    id: "php",
    name: "PHP",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["composer.json", "composer.lock"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["jeffallan/claude-skills/php-pro"],
};
