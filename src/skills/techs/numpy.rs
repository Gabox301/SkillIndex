use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const NUMPY_TECH: Technology = Technology {
    id: "numpy",
    name: "NumPy",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["numpy", "NumPy", "numpy"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "pluginagentmarketplace/custom-plugin-python/machine-learning",
        "pluginagentmarketplace/custom-plugin-python/pandas-data-analysis",
    ],
};
