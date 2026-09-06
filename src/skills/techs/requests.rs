use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const REQUESTS_TECH: Technology = Technology {
    id: "requests",
    name: "Requests",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["requests", "Requests"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["affaan-m/everything-claude-code/python-patterns"],
};
