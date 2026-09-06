use crate::skills::types::{DetectConfig, Technology};

pub const REMOTION_TECH: Technology = Technology {
    id: "remotion",
    name: "Remotion",
    detect: DetectConfig {
        packages: &["remotion", "@remotion/cli"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["remotion-dev/skills/remotion"],
};
