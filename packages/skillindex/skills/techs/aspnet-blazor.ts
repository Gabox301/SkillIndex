export const aspnetBlazorTech = {
  id: 'aspnet-blazor',
  name: 'Blazor',
  detect: {
    configFileContent: {
      scanDotNetLayout: true,
      patterns: ['Microsoft.NET.Sdk.BlazorWebAssembly', 'Microsoft.AspNetCore.Components'],
    },
  },
  skills: ['github/awesome-copilot/fluentui-blazor'],
};
