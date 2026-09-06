use crate::skills::types::{DetectConfig, Technology};

pub const GSAP_TECH: Technology = Technology {
    id: "gsap",
    name: "GSAP",
    detect: DetectConfig {
        packages: &["gsap"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "greensock/gsap-skills/gsap-core",
        "greensock/gsap-skills/gsap-scrolltrigger",
        "greensock/gsap-skills/gsap-performance",
        "greensock/gsap-skills/gsap-plugins",
        "greensock/gsap-skills/gsap-timeline",
        "greensock/gsap-skills/gsap-utils",
        "greensock/gsap-skills/gsap-frameworks",
    ],
};
