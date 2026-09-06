use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const WORDPRESS_TECH: Technology = Technology {
    id: "wordpress",
    name: "WordPress",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &["^@wordpress\\/"],
        config_files: &["wp-config.php", "wp-login.php"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["composer.json", "style.css"],
            patterns: &["johnpbloch/wordpress", "wpackagist", "Theme Name:"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "wordpress/agent-skills/wp-plugin-development",
        "wordpress/agent-skills/wp-rest-api",
        "wordpress/agent-skills/wp-block-themes",
        "wordpress/agent-skills/wp-block-development",
        "wordpress/agent-skills/wp-performance",
        "wordpress/agent-skills/wordpress-router",
        "wordpress/agent-skills/wp-project-triage",
        "wordpress/agent-skills/wp-wpcli-and-ops",
    ],
};
