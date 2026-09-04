export const cloudflareAiTech = {
  id: 'cloudflare-ai',
  name: 'Cloudflare AI',
  detect: {
    packages: ['@cloudflare/ai'],
    configFileContent: { files: ['wrangler.json', 'wrangler.jsonc'], patterns: ['"ai"'] },
  },
  skills: ['cloudflare/skills/agents-sdk'],
};
