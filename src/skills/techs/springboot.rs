use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const SPRINGBOOT_TECH: Technology = Technology {
    id: "springboot",
    name: "Spring Boot",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[
            "src/main/resources/application.properties",
            "src/main/resources/application.yml",
            "src/main/resources/application.yaml",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pom.xml"],
            patterns: &["spring-boot-starter", "org.springframework.boot"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &["github/awesome-copilot/java-springboot"],
};
