use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const PANDAS_TECH: Technology = Technology {
    id: "pandas",
    name: "Pandas",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["pandas", "Pandas"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "jeffallan/claude-skills/pandas-pro",
        "pluginagentmarketplace/custom-plugin-python/pandas-data-analysis",
    ],
};
