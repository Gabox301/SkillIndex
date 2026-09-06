use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const ASPNET_BLAZOR_TECH: Technology = Technology {
    id: "aspnet-blazor",
    name: "Blazor",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &[
                "Microsoft.NET.Sdk.BlazorWebAssembly",
                "Microsoft.AspNetCore.Components",
            ],
            scan_gradle_layout: false,
            scan_dotnet_layout: true,
        }],
    },
    skills: &["github/awesome-copilot/fluentui-blazor"],
};
