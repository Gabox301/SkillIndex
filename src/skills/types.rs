#[derive(Debug, Clone)]
pub struct ConfigFileContentBlock {
    pub files: &'static [&'static str],
    pub patterns: &'static [&'static str],
    pub scan_gradle_layout: bool,
    pub scan_dotnet_layout: bool,
}

#[derive(Debug, Clone)]
pub struct DetectConfig {
    pub packages: &'static [&'static str],
    pub package_patterns: &'static [&'static str],
    pub config_files: &'static [&'static str],
    pub file_extensions: &'static [&'static str],
    pub gems: &'static [&'static str],
    pub config_file_content: &'static [ConfigFileContentBlock],
}

#[derive(Debug, Clone)]
pub struct Technology {
    pub id: &'static str,
    pub name: &'static str,
    pub detect: DetectConfig,
    pub skills: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct ComboSkill {
    pub id: &'static str,
    pub name: &'static str,
    pub requires: &'static [&'static str],
    pub skills: &'static [&'static str],
}
