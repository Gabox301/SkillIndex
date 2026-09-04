export interface ConfigFileContentBlock {
  files?: string[];
  patterns: string[];
  scanGradleLayout?: boolean;
  scanDotNetLayout?: boolean;
}
export interface DetectConfig {
  packages?: string[];
  packagePatterns?: RegExp[];
  configFiles?: string[];
  fileExtensions?: string[];
  gems?: string[];
  configFileContent?: ConfigFileContentBlock | ConfigFileContentBlock[];
}
export interface Technology {
  id: string;
  name: string;
  detect: DetectConfig;
  skills: string[];
}
export interface ComboSkill {
  id: string;
  name: string;
  requires: string[];
  skills: string[];
}
