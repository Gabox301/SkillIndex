export const aspnetMinimalApiTech = {
  id: 'aspnet-minimal-api',
  name: 'ASP.NET Minimal API',
  detect: {
    configFiles: ['appsettings.json'],
    configFileContent: { scanDotNetLayout: true, patterns: ['Microsoft.AspNetCore.OpenApi', 'Swashbuckle.AspNetCore'] },
  },
  skills: ['github/awesome-copilot/aspnet-minimal-api-openapi', 'dotnet/skills/minimal-api-file-upload'],
};
