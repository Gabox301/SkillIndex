use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const LARAVEL_TECH: Technology = Technology {
    id: "laravel",
    name: "Laravel",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["artisan", "bootstrap/app.php"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["composer.json"],
            patterns: &["\"laravel/framework\"", "\"illuminate/"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "jeffallan/claude-skills/laravel-specialist",
        "affaan-m/everything-claude-code/laravel-patterns",
    ],
};
