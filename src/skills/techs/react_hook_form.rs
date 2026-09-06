use crate::skills::types::{DetectConfig, Technology};

pub const REACT_HOOK_FORM_TECH: Technology = Technology {
    id: "react-hook-form",
    name: "React Hook Form",
    detect: DetectConfig {
        packages: &["react-hook-form"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["pproenca/dot-skills/react-hook-form"],
};
