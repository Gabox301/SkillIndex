use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const JAVA_TECH: Technology = Technology {
    id: "java",
    name: "Java",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["pom.xml"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &[
                "sourceCompatibility",
                "targetCompatibility",
                "JavaVersion",
                "id(\"java\")",
                "id 'java'",
                "id(\"java-library\")",
                "id 'java-library'",
            ],
            scan_gradle_layout: true,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "github/awesome-copilot/java-docs",
        "affaan-m/everything-claude-code/java-coding-standards",
    ],
};
