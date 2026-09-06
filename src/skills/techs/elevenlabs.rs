use crate::skills::types::{DetectConfig, Technology};

pub const ELEVENLABS_TECH: Technology = Technology {
    id: "elevenlabs",
    name: "ElevenLabs",
    detect: DetectConfig {
        packages: &["elevenlabs"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "inferen-sh/skills/elevenlabs-tts",
        "inferen-sh/skills/elevenlabs-music",
    ],
};
