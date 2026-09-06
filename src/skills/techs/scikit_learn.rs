use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const SCIKIT_LEARN_TECH: Technology = Technology {
    id: "scikit-learn",
    name: "Scikit-Learn",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["scikit-learn", "scikit_learn", "sklearn"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "davila7/claude-code-templates/scikit-learn",
        "davila7/claude-code-templates/senior-data-scientist",
    ],
};
