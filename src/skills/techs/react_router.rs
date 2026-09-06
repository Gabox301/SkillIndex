use crate::skills::types::{DetectConfig, Technology};

pub const REACT_ROUTER_TECH: Technology = Technology {
    id: "react-router",
    name: "React Router",
    detect: DetectConfig {
        packages: &[
            "react-router",
            "@react-router/node",
            "@react-router/dev",
            "@react-router/serve",
        ],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["clerk/skills/clerk-react-router-patterns"],
};
