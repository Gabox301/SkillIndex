use crate::skills::types::{DetectConfig, Technology};

pub const SUPABASE_TECH: Technology = Technology {
    id: "supabase",
    name: "Supabase",
    detect: DetectConfig {
        packages: &["@supabase/supabase-js", "@supabase/ssr"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "supabase/agent-skills/supabase-postgres-best-practices",
        "Gabox301/SkillIndex/supabase",
    ],
};
