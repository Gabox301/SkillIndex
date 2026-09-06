use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const ASPNET_MINIMAL_API_TECH: Technology = Technology {
    id: "aspnet-minimal-api",
    name: "ASP.NET Minimal API",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["appsettings.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &["Microsoft.AspNetCore.OpenApi", "Swashbuckle.AspNetCore"],
            scan_gradle_layout: false,
            scan_dotnet_layout: true,
        }],
    },
    skills: &[
        "github/awesome-copilot/aspnet-minimal-api-openapi",
        "dotnet/skills/minimal-api-file-upload",
    ],
};
