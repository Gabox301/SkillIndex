pub mod anydoc;
pub mod archify;
pub mod book_to_skill;
pub mod copywriting;
pub mod design_dna;
pub mod find_skills;
pub mod gentleman_programming;
pub mod graphify;
pub mod landing_page_design;
pub mod matt_pocock;
pub mod scroll_world;
pub mod taste_skill;

pub use anydoc::ANYDOC;
pub use archify::ARCHIFY;
pub use book_to_skill::BOOK_TO_SKILL;
pub use copywriting::COPYWRITING;
pub use design_dna::DESIGN_DNA;
pub use find_skills::FIND_SKILLS;
pub use gentleman_programming::GENTLEMAN_PROGRAMMING;
pub use graphify::GRAPHIFY;
pub use landing_page_design::LANDING_PAGE_DESIGN;
pub use matt_pocock::MATTPOCOCK_SKILLS;
pub use scroll_world::SCROLL_WORLD;
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
    LANDING_PAGE_DESIGN,
    MATTPOCOCK_SKILLS,
    SCROLL_WORLD,
    TASTE_SKILL,
];

#[cfg(test)]
mod tests {
    use crate::registry::parse_skill_path;
    use crate::skills::find_technology;
    use std::path::PathBuf;

    /// The imported design skills must be reachable by domain id and their
    /// declared path must point at a real registry directory that ships
    /// SKILL.md, otherwise `--domain <id>` would offer a skill nobody can read.
    #[test]
    fn imported_design_domains_resolve_to_installable_skill_paths() {
        for id in ["landing-page-design", "scroll-world"] {
            let tech: crate::skills::types::Technology =
                find_technology(id).unwrap_or_else(|| panic!("domain {id} must resolve"));
            assert_eq!(tech.id, id);
            assert_eq!(tech.skills.len(), 1, "domain {id} declares one skill");

            let parsed: crate::registry::ParsedSkillPath = parse_skill_path(tech.skills[0]);
            assert_eq!(
                parsed.skill_name, id,
                "skill path must map to the domain id"
            );

            let dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("skills-registry")
                .join(&parsed.skill_name);
            assert!(
                dir.join("SKILL.md").is_file(),
                "missing registry SKILL.md at {}",
                dir.display()
            );
        }
    }
}
