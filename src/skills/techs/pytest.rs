use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const PYTEST_TECH: Technology = Technology {
    id: "pytest",
    name: "Pytest",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["pytest", "Pytest"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["wshobson/agents/python-testing-patterns"],
};
