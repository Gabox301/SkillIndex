use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const FASTMCP_TECH: Technology = Technology {
    id: "fastmcp",
    name: "FastMCP",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
            patterns: &["fastmcp", "FastMCP"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["sharanharsoor/skills/fastmcp"],
};
