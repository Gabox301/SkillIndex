use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const FLUTTER_TECH: Technology = Technology {
    id: "flutter",
    name: "Flutter",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &["pubspec.yaml"],
            patterns: &["flutter:"],
            scan_gradle_layout: false,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "jeffallan/claude-skills/flutter-expert",
        "madteacher/mad-agents-skills/flutter-animations",
        "madteacher/mad-agents-skills/flutter-testing",
    ],
};
