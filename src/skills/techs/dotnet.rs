use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const DOTNET_TECH: Technology = Technology {
    id: "dotnet",
    name: ".NET",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[
            "global.json",
            "NuGet.Config",
            "Directory.Build.props",
            "Directory.Packages.props",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &["<Project Sdk=\"Microsoft.NET.Sdk"],
            scan_gradle_layout: false,
            scan_dotnet_layout: true,
        }],
    },
    skills: &[
        "github/awesome-copilot/dotnet-best-practices",
        "github/awesome-copilot/dotnet-design-pattern-review",
        "github/awesome-copilot/dotnet-upgrade",
    ],
};
