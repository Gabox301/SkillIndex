export const aspnetcoreTech = {
  id: "aspnetcore",
  name: "ASP.NET Core",
  detect: { "configFiles": ["appsettings.json", "appsettings.Development.json"], "configFileContent": { "scanDotNetLayout": true, "patterns": ["Microsoft.NET.Sdk.Web"] } },
  skills: [
    "github/awesome-copilot/containerize-aspnetcore",
    "openai/skills/aspnet-core",
  ],
};
