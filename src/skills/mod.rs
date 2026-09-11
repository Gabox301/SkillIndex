pub mod agents;
pub mod combos;
pub mod domains;
pub mod frontend;
pub mod security;
pub mod techs;
pub mod types;

pub use agents::AGENT_FOLDER_MAP;
pub use frontend::{FRONTEND_BONUS_SKILLS, FRONTEND_PACKAGES, WEB_FRONTEND_EXTENSIONS};
pub use types::{ComboSkill, Technology};

use std::sync::LazyLock;

pub static SKILLS: LazyLock<Vec<Technology>> = LazyLock::new(|| {
    let mut v: Vec<Technology> = Vec::new();
    v.extend_from_slice(techs::TECHS);
    v.extend_from_slice(security::SECURITY_TECHS);
    v.extend_from_slice(domains::DOMAINS);
    v
});
pub static SKILLS_MAP: LazyLock<Vec<Technology>> = LazyLock::new(|| SKILLS.clone());

/// Busca una tecnología/dominio por id (insensible a mayúsculas).
/// Base del flag `--domain`: trae el set completo sin pasar por detección.
pub fn find_technology(id: &str) -> Option<Technology> {
    SKILLS_MAP
        .iter()
        .find(|t: &&Technology| t.id.eq_ignore_ascii_case(id))
        .cloned()
}

/// Sugerencias ante un id desconocido (subcadena en id o nombre, máx 5).
pub fn suggest_technologies(query: &str) -> Vec<String> {
    let q: String = query.to_lowercase();
    SKILLS_MAP
        .iter()
        .filter(|t: &&Technology| {
            t.id.to_lowercase().contains(&q) || t.name.to_lowercase().contains(&q)
        })
        .take(5)
        .map(|t: &Technology| t.id.to_string())
        .collect()
}

pub static COMBO_SKILLS_MAP: LazyLock<Vec<ComboSkill>> = LazyLock::new(|| {
    let mut v: Vec<ComboSkill> = Vec::new();
    v.extend_from_slice(combos::FRAMEWORK_COMBOS);
    v.push(combos::security_operations::SECURITY_OPERATIONS_COMBO);
    v.push(combos::red_team::RED_TEAM_COMBO);
    v.push(combos::cloud_security::CLOUD_SECURITY_COMBO);
    v.push(combos::forensics_ir::FORENSICS_IR_COMBO);
    v
});
