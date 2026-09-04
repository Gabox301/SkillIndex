export const terraformTech = {
  id: "terraform",
  name: "Terraform",
  detect: { "configFiles": [".terraform.lock.hcl", "terraform.tfvars", "main.tf", "variables.tf", "outputs.tf"] },
  skills: [
    "hashicorp/agent-skills/terraform-style-guide",
    "hashicorp/agent-skills/refactor-module",
    "hashicorp/agent-skills/terraform-stacks",
    "wshobson/agents/terraform-module-library",
  ],
};
