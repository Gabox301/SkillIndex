pub mod active_directory;
pub mod evidence_extraction;
pub mod exploitation;
pub mod incident_triage;
pub mod penetration_testing;
pub mod red_team_execution;
pub mod reverse_engineering;
pub mod security_analysis;
pub mod security_assessments;
pub mod security_auditing;
pub mod security_configuration;
pub mod security_deployment;
pub mod security_engineering;
pub mod security_hardening;
pub mod security_implementation;
pub mod security_testing;
pub mod security_tooling;
pub mod threat_detection;
pub mod threat_hunting;
pub mod vulnerability_scanning;

pub use active_directory::ACTIVE_DIRECTORY_TECH;
pub use evidence_extraction::EVIDENCE_EXTRACTION_TECH;
pub use exploitation::EXPLOITATION_TECH;
pub use incident_triage::INCIDENT_TRIAGE_TECH;
pub use penetration_testing::PENETRATION_TESTING_TECH;
pub use red_team_execution::RED_TEAM_EXECUTION_TECH;
pub use reverse_engineering::REVERSE_ENGINEERING_TECH;
pub use security_analysis::SECURITY_ANALYSIS_TECH;
pub use security_assessments::SECURITY_ASSESSMENTS_TECH;
pub use security_auditing::SECURITY_AUDITING_TECH;
pub use security_configuration::SECURITY_CONFIGURATION_TECH;
pub use security_deployment::SECURITY_DEPLOYMENT_TECH;
pub use security_engineering::SECURITY_ENGINEERING_TECH;
pub use security_hardening::SECURITY_HARDENING_TECH;
pub use security_implementation::SECURITY_IMPLEMENTATION_TECH;
pub use security_testing::SECURITY_TESTING_TECH;
pub use security_tooling::SECURITY_TOOLING_TECH;
pub use threat_detection::THREAT_DETECTION_TECH;
pub use threat_hunting::THREAT_HUNTING_TECH;
pub use vulnerability_scanning::VULNERABILITY_SCANNING_TECH;

pub const SECURITY_TECHS: &[crate::skills::types::Technology] = &[
    ACTIVE_DIRECTORY_TECH,
    EVIDENCE_EXTRACTION_TECH,
    EXPLOITATION_TECH,
    INCIDENT_TRIAGE_TECH,
    PENETRATION_TESTING_TECH,
    RED_TEAM_EXECUTION_TECH,
    REVERSE_ENGINEERING_TECH,
    SECURITY_ANALYSIS_TECH,
    SECURITY_ASSESSMENTS_TECH,
    SECURITY_AUDITING_TECH,
    SECURITY_CONFIGURATION_TECH,
    SECURITY_DEPLOYMENT_TECH,
    SECURITY_ENGINEERING_TECH,
    SECURITY_HARDENING_TECH,
    SECURITY_IMPLEMENTATION_TECH,
    SECURITY_TESTING_TECH,
    SECURITY_TOOLING_TECH,
    THREAT_DETECTION_TECH,
    THREAT_HUNTING_TECH,
    VULNERABILITY_SCANNING_TECH,
];
