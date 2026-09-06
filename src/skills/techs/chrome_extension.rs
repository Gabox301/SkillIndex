use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const CHROME_EXTENSION_TECH: Technology = Technology {
    id: "chrome-extension",
    name: "Chrome Extension",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["manifest.json"],
            patterns: &["manifest_version"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["mindrally/skills/chrome-extension-development"],
};
