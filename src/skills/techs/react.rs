use crate::skills::types::{DetectConfig, Technology};

pub const REACT_TECH: Technology = Technology {
    id: "react",
    name: "React",
    detect: DetectConfig {
        packages: &["react", "react-dom"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "vercel-labs/agent-skills/react-best-practices",
        "vercel-labs/agent-skills/composition-patterns",
    ],
};
