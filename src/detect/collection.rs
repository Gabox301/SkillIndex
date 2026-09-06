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
                if let Some(entry) = skills.iter_mut().find(|e| e.skill == skill)
                    && !entry.sources.contains(&source)
                {
                    entry.sources.push(source.clone());
                }
            }
        } else {
            let installed = if let Some(set) = installed_names {
                let parsed = crate::registry::parse_skill_path(&skill);
                set.contains(&parsed.skill_name)
            } else {
                false
            };
            let entry = SkillEntry {
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
        if let Some(c) = get_combos_slice().iter().find(|c| c.name == combo.name) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayTechnology;

    #[test]
    fn collect_skills_includes_tech_skills() {
        let tech = DisplayTechnology {
            id: "react".into(),
            name: "React".into(),
            skills: vec!["vercel-labs/agent-skills/react-best-practices".into()],
        };
        let skills = collect_skills(&[tech], false, &[], None);
        assert_eq!(skills.len(), 1);
        assert_eq!(
            skills[0].skill,
            "vercel-labs/agent-skills/react-best-practices"
        );
    }

    #[test]
    fn collect_skills_frontend_bonus() {
        let skills = collect_skills(&[], true, &[], None);
        assert!(skills.iter().any(|s| s.skill.contains("frontend-design")));
    }
}
