use crate::skills::types::{DetectConfig, Technology};

pub const ANGULAR_TECH: Technology = Technology {
    id: "angular",
    name: "Angular",
    detect: DetectConfig {
        packages: &["@angular/core"],
        package_patterns: &[],
        config_files: &["angular.json"],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "angular/skills/angular-developer",
        "angular/angular/reference-core",
        "angular/angular/reference-signal-forms",
        "angular/angular/reference-compiler-cli",
        "angular/angular/adev-writing-guide",
        "angular/angular/pr_review",
    ],
};
