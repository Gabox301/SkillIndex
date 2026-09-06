use crate::skills::types::{DetectConfig, Technology};

pub const REACT_NATIVE_TECH: Technology = Technology {
    id: "react-native",
    name: "React Native",
    detect: DetectConfig {
        packages: &["react-native"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["sleekdotdesign/agent-skills/design-mobile-apps"],
};
