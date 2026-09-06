use crate::skills::types::{ConfigFileContentBlock, DetectConfig, Technology};

pub const CLERK_TECH: Technology = Technology {
    id: "clerk",
    name: "Clerk",
    detect: DetectConfig {
        packages: &[
            "@clerk/nextjs",
            "@clerk/remix",
            "@clerk/astro",
            "@clerk/express",
            "@clerk/fastify",
            "@clerk/nuxt",
            "@clerk/vue",
            "@clerk/react",
            "@clerk/expo",
            "@clerk/tanstack-react-start",
            "@clerk/react-router",
            "@clerk/chrome-extension",
            "@clerk/backend",
        ],
        package_patterns: &["^@clerk\\/"],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[
            ConfigFileContentBlock {
                files: &["Package.swift"],
                patterns: &["clerk/clerk-ios", "ClerkSDK"],
                scan_gradle_layout: false,
                scan_dotnet_layout: false,
            },
            ConfigFileContentBlock {
                files: &[],
                patterns: &["com.clerk"],
                scan_gradle_layout: true,
                scan_dotnet_layout: false,
            },
        ],
    },
    skills: &[
        "clerk/skills/clerk",
        "clerk/skills/clerk-setup",
        "clerk/skills/clerk-custom-ui",
        "clerk/skills/clerk-backend-api",
        "clerk/skills/clerk-orgs",
        "clerk/skills/clerk-webhooks",
        "clerk/skills/clerk-testing",
    ],
};
