use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const FASTAPI_TECH: Technology = Technology {
    id: "fastapi",
    name: "FastAPI",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["fastapi", "FastAPI"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "wshobson/agents/fastapi-templates",
        "mindrally/skills/fastapi-python",
    ],
};
