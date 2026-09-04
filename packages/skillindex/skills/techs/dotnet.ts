export const dotnetTech = {
  id: "dotnet",
  name: ".NET",
  detect: { "configFiles": ["global.json", "NuGet.Config", "Directory.Build.props", "Directory.Packages.props"], "configFileContent": { "scanDotNetLayout": true, "patterns": ["<Project Sdk=\"Microsoft.NET.Sdk"] } },
  skills: [
    "github/awesome-copilot/dotnet-best-practices",
    "github/awesome-copilot/dotnet-design-pattern-review",
    "github/awesome-copilot/dotnet-upgrade",
  ],
};
