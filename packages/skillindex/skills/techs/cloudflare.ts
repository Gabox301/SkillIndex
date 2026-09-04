export const cloudflareTech = {
  id: "cloudflare",
  name: "Cloudflare",
  detect: { "packages": ["wrangler", "@cloudflare/workers-types", "@astrojs/cloudflare"], "configFiles": ["wrangler.toml", "wrangler.json", "wrangler.jsonc"] },
  skills: [
    "cloudflare/skills/cloudflare",
    "cloudflare/skills/wrangler",
    "cloudflare/skills/workers-best-practices",
    "cloudflare/skills/web-perf",
    "openai/skills/cloudflare-deploy",
  ],
};
