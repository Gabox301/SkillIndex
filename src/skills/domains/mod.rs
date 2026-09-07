pub mod agent_workflows;
pub mod archify;
pub mod book_to_skill;
pub mod design_dna;
pub mod matt_pocock;
pub mod sdd_workflow;
pub mod taste_skill;

pub use agent_workflows::AGENT_WORKFLOWS;
pub use archify::ARCHIFY;
pub use book_to_skill::BOOK_TO_SKILL;
pub use design_dna::DESIGN_DNA;
pub use matt_pocock::MATTPOCOCK_SKILLS;
pub use sdd_workflow::SDD_WORKFLOW;
pub use taste_skill::TASTE_SKILL;

pub const DOMAINS: &[crate::skills::types::Technology] = &[
    AGENT_WORKFLOWS,
    ARCHIFY,
    BOOK_TO_SKILL,
    DESIGN_DNA,
    MATTPOCOCK_SKILLS,
    SDD_WORKFLOW,
    TASTE_SKILL,
];
