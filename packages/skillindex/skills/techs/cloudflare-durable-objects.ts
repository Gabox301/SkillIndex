export const cloudflareDurableObjectsTech = {
  id: "cloudflare-durable-objects",
  name: "Durable Objects",
  detect: { "configFileContent": { "files": ["wrangler.json", "wrangler.jsonc", "wrangler.toml"], "patterns": ["durable_objects"] } },
  skills: [
    "cloudflare/skills/durable-objects",
  ],
};
