use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const ASPNETCORE_TECH: Technology = Technology {
    id: "aspnetcore",
    name: "ASP.NET Core",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["appsettings.json", "appsettings.Development.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &["Microsoft.NET.Sdk.Web"],
            scan_gradle_layout: false,
            scan_dotnet_layout: true,
        }],
    },
    skills: &[
        "github/awesome-copilot/containerize-aspnetcore",
        "openai/skills/aspnet-core",
    ],
};
