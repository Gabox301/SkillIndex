use crate::skills::types::{DetectConfig, Technology};

pub const TERRAFORM_TECH: Technology = Technology {
    id: "terraform",
    name: "Terraform",
    detect: DetectConfig {
        packages: &[],
        package_patterns: &[],
        config_files: &[
            ".terraform.lock.hcl",
            "terraform.tfvars",
            "main.tf",
            "variables.tf",
            "outputs.tf",
        ],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "hashicorp/agent-skills/terraform-style-guide",
        "hashicorp/agent-skills/refactor-module",
        "hashicorp/agent-skills/terraform-stacks",
        "wshobson/agents/terraform-module-library",
    ],
};
