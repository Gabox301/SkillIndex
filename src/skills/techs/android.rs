use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const ANDROID_TECH: Technology = Technology {
    id: "android",
    name: "Android",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[ConfigFileContentBlock {
            files: &[],
            patterns: &[
                "com.android.application",
                "com.android.library",
                "id(\"com.android.application\")",
                "id(\"com.android.library\")",
                "com.android.kotlin.multiplatform.library",
            ],
            scan_gradle_layout: true,
            scan_dotnet_layout: false,
        }],
    },
    skills: &[
        "krutikJain/android-agent-skills/android-kotlin-core",
        "krutikJain/android-agent-skills/android-compose-foundations",
        "krutikJain/android-agent-skills/android-architecture-clean",
        "krutikJain/android-agent-skills/android-di-hilt",
        "krutikJain/android-agent-skills/android-gradle-build-logic",
        "krutikJain/android-agent-skills/android-coroutines-flow",
        "krutikJain/android-agent-skills/android-networking-retrofit-okhttp",
        "krutikJain/android-agent-skills/android-testing-unit",
    ],
};
