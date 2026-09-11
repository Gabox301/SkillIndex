pub mod anydoc;
pub mod archify;
pub mod book_to_skill;
pub mod copywriting;
pub mod design_dna;
pub mod find_skills;
pub mod gentleman_programming;
pub mod graphify;
pub mod matt_pocock;
pub mod taste_skill;

pub use anydoc::ANYDOC;
pub use archify::ARCHIFY;
pub use book_to_skill::BOOK_TO_SKILL;
pub use copywriting::COPYWRITING;
pub use design_dna::DESIGN_DNA;
pub use find_skills::FIND_SKILLS;
pub use gentleman_programming::GENTLEMAN_PROGRAMMING;
pub use graphify::GRAPHIFY;
pub use matt_pocock::MATTPOCOCK_SKILLS;
pub use taste_skill::TASTE_SKILL;

pub const DOMAINS: &[crate::skills::types::Technology] = &[
    ANYDOC,
    ARCHIFY,
    BOOK_TO_SKILL,
    COPYWRITING,
    DESIGN_DNA,
    FIND_SKILLS,
    GENTLEMAN_PROGRAMMING,
    GRAPHIFY,
    MATTPOCOCK_SKILLS,
    TASTE_SKILL,
];
