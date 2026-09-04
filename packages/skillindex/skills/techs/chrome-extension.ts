export const chromeExtensionTech = {
  id: "chrome-extension",
  name: "Chrome Extension",
  detect: { "configFileContent": { "files": ["manifest.json"], "patterns": ["manifest_version"] } },
  skills: [
    "mindrally/skills/chrome-extension-development",
  ],
};
