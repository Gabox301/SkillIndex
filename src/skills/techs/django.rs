use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const DJANGO_TECH: Technology = Technology {
    id: "django",
    name: "Django",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["django", "Django"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "vintasoftware/django-ai-plugins/django-expert",
        "affaan-m/everything-claude-code/django-patterns",
        "affaan-m/everything-claude-code/django-security",
    ],
};
