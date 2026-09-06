use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const CSHARP_TECH: Technology = Technology {
    id: "csharp",
    name: "C#",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &["<Project", "Microsoft.NET.Sdk"],
            scan_gradle_layout: false,
            scan_dotnet_layout: true,
        }],
    },
    skills: &[
        "github/awesome-copilot/csharp-xunit",
        "github/awesome-copilot/csharp-async",
        "github/awesome-copilot/csharp-docs",
        "github/awesome-copilot/csharp-nunit",
        "github/awesome-copilot/csharp-mstest",
        "github/awesome-copilot/csharp-tunit",
    ],
};
