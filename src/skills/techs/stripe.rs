use crate::skills::types::{DetectConfig, Technology};

pub const STRIPE_TECH: Technology = Technology {
    id: "stripe",
    name: "Stripe",
    detect: DetectConfig {
        packages: &["stripe", "@stripe/stripe-js", "@stripe/react-stripe-js"],
        package_patterns: &[],
        config_files: &[],
        file_extensions: &[],
        gems: &[],
        config_file_content: &[],
    },
    skills: &[
        "stripe/ai/stripe-best-practices",
        "stripe/ai/upgrade-stripe",
    ],
};
