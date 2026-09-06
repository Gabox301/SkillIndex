use crate::skills::types::{DetectConfig, Technology};

pub const INSTANTDB_TECH: Technology = Technology {
    id: "instantdb",
    name: "InstantDB",
    detect: DetectConfig {
        packages: &[
            "@instantdb/core",
            "@instantdb/react",
            "@instantdb/react-native",
            "@instantdb/react-native-mmkv",
            "@instantdb/admin",
        ],
        package_patterns: &[],
        config_files: &["instant.schema.ts", "instant.perms.ts"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &["instantdb/skills/instantdb"],
};
