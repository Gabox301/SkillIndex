export const vercelDeployTech = {
  id: "vercel-deploy",
  name: "Vercel",
  detect: { "configFiles": ["vercel.json", ".vercel"], "packages": ["vercel", "@astrojs/vercel"] },
  skills: [
    "vercel-labs/agent-skills/deploy-to-vercel",
  ],
};
