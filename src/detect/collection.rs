use std::collections::{HashMap, HashSet};

use crate::detect::constants::FRONTEND_BONUS_SKILLS;
use crate::detect::helpers::get_combos_slice;
use crate::display::{DisplayCombo, DisplayTechnology};
use crate::installer::SkillEntry;

pub fn collect_skills(
    detected: &[DisplayTechnology],
    is_frontend: bool,
    combos: &[DisplayCombo],
    installed_names: Option<&HashSet<String>>,
) -> Vec<SkillEntry> {
    let mut skill_map: HashMap<String, SkillEntry> = HashMap::new();
    let mut skills: Vec<SkillEntry> = Vec::new();

    let mut add_skill = |skill: String, source: String| {
        if let Some(existing) = skill_map.get_mut(&skill) {
            if !existing.sources.contains(&source) {
                existing.sources.push(source.clone());
                if let Some(entry) = skills
                    .iter_mut()
                    .find(|e: &&mut SkillEntry| e.skill == skill)
                    && !entry.sources.contains(&source)
                {
                    entry.sources.push(source.clone());
                }
            }
        } else {
            let installed: bool = if let Some(set) = installed_names {
                let parsed: crate::registry::ParsedSkillPath =
                    crate::registry::parse_skill_path(&skill);
                set.contains(&parsed.skill_name)
            } else {
                false
            };
            let entry: SkillEntry = SkillEntry {
                skill: skill.clone(),
                sources: vec![source.clone()],
                installed,
            };
            skill_map.insert(skill.clone(), entry.clone());
            skills.push(entry);
        }
    };

    for tech in detected {
        for skill in &tech.skills {
            add_skill(skill.clone(), tech.name.clone());
        }
    }

    for combo in combos {
        if let Some(c) = get_combos_slice()
            .iter()
            .find(|c: &&crate::skills::ComboSkill| c.name == combo.name)
        {
            for skill in c.skills {
                add_skill(skill.to_string(), combo.name.clone());
            }
        }
    }

    if is_frontend {
        for skill in FRONTEND_BONUS_SKILLS {
            add_skill(skill.to_string(), "Frontend".to_string());
        }
    }

    skills
}

/// Resuelve `--domain` a skills: trae el SET COMPLETO de cada tecnología
/// sin pasar por detección. Falla con el primer id desconocido (con
/// sugerencias) para no instalar un set a medias en silencio.
pub fn collect_domain_skills(
    ids: &[String],
    installed_names: Option<&HashSet<String>>,
) -> Result<Vec<SkillEntry>, String> {
    use crate::display::DisplayTechnology;
    use crate::skills::{find_technology, suggest_technologies};

    let mut techs: Vec<DisplayTechnology> = Vec::new();
    for id in ids {
        match find_technology(id) {
            Some(t) => techs.push(DisplayTechnology {
                id: t.id.to_string(),
                name: t.name.to_string(),
                skills: t.skills.iter().map(|s: &&str| s.to_string()).collect(),
            }),
            None => {
                let mut msg: String = format!("domain desconocido: {id}");
                let suggestions: Vec<String> = suggest_technologies(id);
                if !suggestions.is_empty() {
                    msg.push_str(&format!(". ¿Quisiste decir: {}?", suggestions.join(", ")));
                }
                return Err(msg);
            }
        }
    }
    Ok(collect_skills(&techs, false, &[], installed_names))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayTechnology;

    #[test]
    fn collect_skills_includes_tech_skills() {
        let tech: DisplayTechnology = DisplayTechnology {
            id: "react".into(),
            name: "React".into(),
            skills: vec!["vercel-labs/agent-skills/react-best-practices".into()],
        };
        let skills: Vec<SkillEntry> = collect_skills(&[tech], false, &[], None);
        assert_eq!(skills.len(), 1);
        assert_eq!(
            skills[0].skill,
            "vercel-labs/agent-skills/react-best-practices"
        );
    }

    #[test]
    fn collect_skills_frontend_bonus() {
        let skills: Vec<SkillEntry> = collect_skills(&[], true, &[], None);
        assert!(skills.iter().any(|s| s.skill.contains("frontend-design")));
    }

    #[test]
    fn collect_domain_skills_full_set() {
        let skills: Vec<SkillEntry> =
            collect_domain_skills(&["gentleman-programming".to_string()], None)
                .expect("known domain");
        assert_eq!(skills.len(), 24);
        assert!(
            skills
                .iter()
                .all(|s| s.sources == vec!["Gentleman Programming".to_string()])
        );
    }

    #[test]
    fn collect_domain_skills_case_insensitive_and_union() {
        let skills: Vec<SkillEntry> = collect_domain_skills(
            &["Gentleman-Programming".to_string(), "anydoc".to_string()],
            None,
        )
        .expect("known domains");
        assert_eq!(skills.len(), 25);
    }

    #[test]
    fn collect_domain_skills_unknown_suggests() {
        let err: String =
            collect_domain_skills(&["gentleman".to_string()], None).unwrap_err();
        assert!(err.contains("gentleman-programming"), "got: {err}");
    }

    #[test]
    fn collect_domain_skills_empty() {
        let skills: Vec<SkillEntry> = collect_domain_skills(&[], None).expect("empty ok");
        assert!(skills.is_empty());
    }
}
