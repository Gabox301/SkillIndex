export const cloudflareAgentsTech = {
  id: "cloudflare-agents",
  name: "Cloudflare Agents",
  detect: { "packages": ["agents"] },
  skills: [
    "cloudflare/skills/agents-sdk",
    "cloudflare/skills/sandbox-sdk",
  ],
};
