use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const FLASK_TECH: Technology = Technology {
    id: "flask",
    name: "Flask",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[
                "pyproject.toml",
                "requirements.txt",
                "setup.py",
                "setup.cfg",
                "Pipfile",
            ],
            patterns: &["flask", "Flask"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["aj-geddes/useful-ai-prompts/flask-api-development"],
};
