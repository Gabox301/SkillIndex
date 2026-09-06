use crate::skills::types::{DetectConfig, Technology};

pub const REACT_THREE_FIBER_TECH: Technology = Technology {
    id: "@react-three/fiber",
    name: "React Three Fiber",
    detect: DetectConfig {
        packages: &["@react-three/fiber"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["vercel-labs/json-render/react-three-fiber"],
};
