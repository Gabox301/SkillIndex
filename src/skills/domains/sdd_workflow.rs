use crate::skills::types::{DetectConfig, Technology};

pub const SDD_WORKFLOW: Technology = Technology {
    id: "sdd-workflow",
    name: "SDD Workflow",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "Gabox301/SkillIndex/sdd-apply",
        "Gabox301/SkillIndex/sdd-archive",
        "Gabox301/SkillIndex/sdd-design",
        "Gabox301/SkillIndex/sdd-explore",
        "Gabox301/SkillIndex/sdd-init",
        "Gabox301/SkillIndex/sdd-onboard",
        "Gabox301/SkillIndex/sdd-propose",
        "Gabox301/SkillIndex/sdd-spec",
        "Gabox301/SkillIndex/sdd-tasks",
        "Gabox301/SkillIndex/sdd-verify",
    ],
};
