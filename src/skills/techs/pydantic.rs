use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const PYDANTIC_TECH: Technology = Technology {
    id: "pydantic",
    name: "Pydantic",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["pydantic", "Pydantic"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["bobmatnyc/claude-mpm-skills/pydantic"],
};
