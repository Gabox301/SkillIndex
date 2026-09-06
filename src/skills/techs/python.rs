use crate::skills::types::{DetectConfig, Technology};

pub const PYTHON_TECH: Technology = Technology {
    id: "python",
    name: "Python",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "inferen-sh/skills/python-executor",
        "wshobson/agents/python-testing-patterns",
    ],
};
