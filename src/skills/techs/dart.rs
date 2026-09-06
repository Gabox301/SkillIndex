use crate::skills::types::{DetectConfig, Technology};

pub const DART_TECH: Technology = Technology {
    id: "dart",
    name: "Dart",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["pubspec.yaml"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["kevmoo/dash_skills/dart-best-practices"],
};
