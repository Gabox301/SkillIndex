use crate::skills::types::{DetectConfig, Technology};

pub const EXPO_TECH: Technology = Technology {
    id: "expo",
    name: "Expo",
    detect: DetectConfig {
        packages: &["expo"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "Gabox301/SkillIndex/building-native-ui",
        "Gabox301/SkillIndex/native-data-fetching",
        "Gabox301/SkillIndex/upgrading-expo",
        "Gabox301/SkillIndex/expo-tailwind-setup",
        "expo/skills/expo-dev-client",
        "Gabox301/SkillIndex/expo-deployment",
        "Gabox301/SkillIndex/expo-cicd-workflows",
        "Gabox301/SkillIndex/expo-api-routes",
        "Gabox301/SkillIndex/use-dom",
    ],
};
