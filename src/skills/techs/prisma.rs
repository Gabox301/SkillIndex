use crate::skills::types::{DetectConfig, Technology};

pub const PRISMA_TECH: Technology = Technology {
    id: "prisma",
    name: "Prisma",
    detect: DetectConfig {
        packages: &["prisma", "@prisma/client"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "prisma/skills/prisma-database-setup",
        "prisma/skills/prisma-client-api",
        "prisma/skills/prisma-cli",
        "prisma/skills/prisma-postgres",
    ],
};
