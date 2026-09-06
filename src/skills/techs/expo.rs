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
        "expo/skills/building-native-ui",
        "expo/skills/native-data-fetching",
        "expo/skills/upgrading-expo",
        "expo/skills/expo-tailwind-setup",
        "expo/skills/expo-dev-client",
        "expo/skills/expo-deployment",
        "expo/skills/expo-cicd-workflows",
        "expo/skills/expo-api-routes",
        "expo/skills/use-dom",
    ],
};
