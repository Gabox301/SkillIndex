use crate::skills::types::{DetectConfig, Technology};

pub const THREEJS_TECH: Technology = Technology {
    id: "threejs",
    name: "Three.js",
    detect: DetectConfig {
        packages: &["three"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "cloudai-x/threejs-skills/threejs-animation",
        "cloudai-x/threejs-skills/threejs-fundamentals",
        "cloudai-x/threejs-skills/threejs-shaders",
        "cloudai-x/threejs-skills/threejs-geometry",
        "cloudai-x/threejs-skills/threejs-interaction",
        "cloudai-x/threejs-skills/threejs-materials",
        "cloudai-x/threejs-skills/threejs-postprocessing",
        "cloudai-x/threejs-skills/threejs-lighting",
        "cloudai-x/threejs-skills/threejs-textures",
        "cloudai-x/threejs-skills/threejs-loaders",
    ],
};
