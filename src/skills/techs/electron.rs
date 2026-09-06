use crate::skills::types::{DetectConfig, Technology};

pub const ELECTRON_TECH: Technology = Technology {
    id: "electron",
    name: "Electron",
    detect: DetectConfig {
        packages: &["electron"],
        package_patterns: &[],
        config_files: &[
            "electron-builder.yml",
            "electron-builder.json",
            "electron-builder.js",
            "forge.config.js",
            "forge.config.cjs",
            "forge.config.mjs",
            "forge.config.ts",
            "electron-vite.config.ts",
            "electron-vite.config.js",
            "electron-vite.config.mjs",
            "electron-vite.config.cjs",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "full-stack-skills/electron-skills/electron",
        "full-stack-skills/electron-skills/electron-egg",
        "full-stack-skills/electron-skills/upgradeLink",
    ],
};
