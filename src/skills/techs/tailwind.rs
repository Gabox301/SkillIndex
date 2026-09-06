use crate::skills::types::{DetectConfig, Technology};

pub const TAILWIND_TECH: Technology = Technology {
    id: "tailwind",
    name: "Tailwind CSS",
    detect: DetectConfig {
        packages: &["tailwindcss", "@tailwindcss/vite"],
        package_patterns: &[],
        config_files: &[
            "tailwind.config.js",
            "tailwind.config.ts",
            "tailwind.config.cjs",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["giuseppe-trisciuoglio/developer-kit/tailwind-css-patterns"],
};
