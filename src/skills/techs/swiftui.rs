use crate::skills::types::{DetectConfig, Technology};

pub const SWIFTUI_TECH: Technology = Technology {
    id: "swiftui",
    name: "SwiftUI",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &["Package.swift", "Podfile"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "avdlee/swiftui-agent-skill/swiftui-expert-skill",
        "avdlee/swift-concurrency-agent-skill/swift-concurrency",
        "avdlee/xcode-build-optimization-agent-skill/xcode-build-orchestrator",
        "avdlee/swift-testing-agent-skill/swift-testing-expert",
        "avdlee/core-data-agent-skill/core-data-expert",
    ],
};
