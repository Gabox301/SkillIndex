use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const KOTLIN_MULTIPLATFORM_TECH: Technology = Technology {
    id: "kotlin-multiplatform",
    name: "Kotlin Multiplatform",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &[
                "kotlin(\"multiplatform\")",
                "org.jetbrains.kotlin.multiplatform",
                "id(\"org.jetbrains.kotlin.multiplatform\")",
                "kotlin-multiplatform",
            ],
            scan_gradle_layout: true,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "Kotlin/kotlin-agent-skills/kotlin-tooling-cocoapods-spm-migration",
        "Kotlin/kotlin-agent-skills/kotlin-tooling-agp9-migration",
    ],
};
