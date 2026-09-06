use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const SQLALCHEMY_TECH: Technology = Technology {
    id: "sqlalchemy",
    name: "SQLAlchemy",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["sqlalchemy", "SQLAlchemy"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "bobmatnyc/claude-mpm-skills/sqlalchemy",
        "wispbit-ai/skills/sqlalchemy-alembic-expert-best-practices-code-review",
    ],
};
