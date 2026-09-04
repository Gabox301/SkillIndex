export const csharpTech = {
  id: "csharp",
  name: "C#",
  detect: { "configFileContent": { "scanDotNetLayout": true, "patterns": ["<Project", "Microsoft.NET.Sdk"] } },
  skills: [
    "github/awesome-copilot/csharp-xunit",
    "github/awesome-copilot/csharp-async",
    "github/awesome-copilot/csharp-docs",
    "github/awesome-copilot/csharp-nunit",
    "github/awesome-copilot/csharp-mstest",
    "github/awesome-copilot/csharp-tunit",
  ],
};
