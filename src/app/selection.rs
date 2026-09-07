use skillindex::display::format_skill_label;
use skillindex::installer::SkillEntry;
use skillindex::prompt::{MultiSelectOptions, Shortcut, multi_select};
use skillindex::ui::{bold, brand_cyan, dim, is_tty, log, yellow};

use super::security::security_warning_for_skill;

// Aliases para closures de multi_select — evitan `type_complexity` inline
// y hacen explícito el contrato de cada callback sin silenciar lints.
type AgentLabelFn = dyn Fn(&String, usize) -> String;
type SkillLabelFn = dyn Fn(&SkillEntry, usize) -> String;
type SkillHintFn = dyn Fn(&SkillEntry, usize) -> String;
type SkillGroupFn = dyn Fn(&SkillEntry) -> String;

pub fn select_agents_sync(agents: Vec<String>, auto_yes: bool) -> Vec<String> {
    let real_agents: Vec<String> = agents
        .iter()
        .filter(|a: &&String| a.as_str() != "universal")
        .cloned()
        .collect();
    if real_agents.is_empty() {
        if auto_yes || !is_tty() {
            return agents;
        }
        let all_possible: Vec<String> = skillindex::detect::get_all_possible_agents();
        log(&format!(
            "{}{} {}",
            brand_cyan("   ◆ "),
            bold("No se detectó ningún agente en el proyecto"),
            dim(&format!("({} disponibles)", all_possible.len()))
        ));
        log(&dim(
            "   Selecciona dónde instalar. Crea el/los directorios si aún no existen.",
        ));
        log("");

        let styled_label_fn: Box<AgentLabelFn> = Box::new(|item: &String, _| {
            let folder: &str = skillindex::registry::agent_folder_for(item).unwrap_or(".agents");
            format!("{} {}", bold(item), dim(&format!("({folder})")))
        });

        let opts: MultiSelectOptions<String> = MultiSelectOptions {
            label_fn: styled_label_fn,
            hint_fn: None,
            group_fn: None,
            initial_selected: Some(vec![false; all_possible.len()]),
            shortcuts: Vec::new(),
        };

        let selected: Vec<String> = multi_select(all_possible.clone(), opts).unwrap_or_default();

        if selected.is_empty() {
            log("");
            log(&dim(
                "   Ningún agente seleccionado — se usará .agents (universal).",
            ));
            log("");
            return vec!["universal".to_string()];
        }

        return selected;
    }
    if real_agents.len() <= 1 {
        return agents;
    }
    if auto_yes || !is_tty() {
        return agents;
    }

    log(&format!(
        "{}{} {}",
        brand_cyan("   ◆ "),
        bold("Selecciona dónde instalar"),
        dim(&format!("({} agentes detectados)", real_agents.len()))
    ));
    log(&dim(
        "   Desmarca los que no quieras. Por defecto instala en todos.",
    ));
    log("");

    let styled_label_fn: Box<AgentLabelFn> = Box::new(|item: &String, _| {
        let folder: &str = skillindex::registry::agent_folder_for(item).unwrap_or(".agents");
        format!("{} {}", bold(item), dim(&format!("({folder})")))
    });

    let opts: MultiSelectOptions<String> = MultiSelectOptions {
        label_fn: styled_label_fn,
        hint_fn: None,
        group_fn: None,
        initial_selected: Some(vec![true; real_agents.len()]),
        shortcuts: Vec::new(),
    };

    let selected_real: Vec<String> = multi_select(real_agents.clone(), opts).unwrap_or_default();

    if selected_real.is_empty() {
        log("");
        log(&dim(
            "   Ningún agente seleccionado — no se instalará nada.",
        ));
        log("");
        std::process::exit(0);
    }

    selected_real
}

pub fn select_skills_sync(skills: Vec<SkillEntry>, auto_yes: bool) -> Vec<SkillEntry> {
    if auto_yes {
        skillindex::display::print_skills_list(&skills);
        return skills;
    }

    const INSTALLED_TAG: &str = " (instalada)";
    const SECURITY_TAG: &str = " (revisión de seguridad ⚠)";

    let mut label_cache: std::collections::HashMap<String, (String, String, bool)> =
        std::collections::HashMap::new();
    for s in &skills {
        let label: String = format_skill_label(&s.skill, false);
        let styled: String = format_skill_label(&s.skill, true);
        let has_warn: bool = security_warning_for_skill(&s.skill).is_some();
        label_cache.insert(s.skill.clone(), (label, styled, has_warn));
    }
    let max_effective: usize = skills
        .iter()
        .map(|s: &SkillEntry| {
            let (label, _, has_warn) = label_cache.get(&s.skill).unwrap();
            label.len()
                + if s.installed { INSTALLED_TAG.len() } else { 0 }
                + if *has_warn { SECURITY_TAG.len() } else { 0 }
        })
        .max()
        .unwrap_or(0);

    let new_count: usize = skills.iter().filter(|s: &&SkillEntry| !s.installed).count();
    let installed_count: usize = skills.len() - new_count;
    let count_label: String = if installed_count > 0 {
        format!(
            "{} encontradas, {} ya instaladas",
            skills.len(),
            installed_count
        )
    } else {
        format!("{} encontradas", skills.len())
    };
    log(&format!(
        "{}{} {}",
        brand_cyan("   ◆ "),
        bold("Selecciona las skills a instalar"),
        dim(&format!("({count_label})"))
    ));
    log("");

    let label_fn: Box<SkillLabelFn> = Box::new(move |item: &SkillEntry, _idx: usize| {
        let (label, styled_label, has_warn) = label_cache.get(&item.skill).unwrap();
        let installed_tag: String = if item.installed {
            dim(INSTALLED_TAG)
        } else {
            String::new()
        };
        let security_tag: String = if *has_warn {
            yellow(SECURITY_TAG)
        } else {
            String::new()
        };
        let effective_len: usize = label.len()
            + if item.installed {
                INSTALLED_TAG.len()
            } else {
                0
            }
            + if *has_warn { SECURITY_TAG.len() } else { 0 };
        let pad: String = " ".repeat(max_effective.saturating_sub(effective_len));
        format!("{styled_label}{installed_tag}{security_tag}{pad}")
    });

    let hint_fn: Box<SkillHintFn> = Box::new(|item: &SkillEntry, _| {
        let tech_sources: Vec<&String> = item
            .sources
            .iter()
            .filter(|s: &&String| !s.contains(" + "))
            .collect();
        if tech_sources.len() > 1 {
            format!(
                "← {}",
                tech_sources
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        }
    });

    let group_fn: Box<SkillGroupFn> =
        Box::new(|item: &SkillEntry| item.sources.first().cloned().unwrap_or_default());

    let initial_selected: Vec<bool> = skills.iter().map(|s: &SkillEntry| !s.installed).collect();

    let mut shortcuts: Vec<Shortcut<SkillEntry>> = Vec::new();
    if installed_count > 0 {
        shortcuts.push(Shortcut {
            key: 'n',
            label: "nuevas".to_string(),
            func: Box::new(|items: &[SkillEntry]| {
                items.iter().map(|s: &SkillEntry| !s.installed).collect()
            }),
        });
        shortcuts.push(Shortcut {
            key: 'i',
            label: "instaladas".to_string(),
            func: Box::new(|items: &[SkillEntry]| {
                items.iter().map(|s: &SkillEntry| s.installed).collect()
            }),
        });
    }

    let opts: MultiSelectOptions<SkillEntry> = MultiSelectOptions {
        label_fn,
        hint_fn: Some(hint_fn),
        group_fn: Some(group_fn),
        initial_selected: Some(initial_selected),
        shortcuts,
    };

    let selected: Vec<SkillEntry> = multi_select(skills, opts).unwrap_or_default();

    if selected.is_empty() {
        log("");
        log(&dim("   Nada seleccionado."));
        log("");
        std::process::exit(0);
    }
    selected
}
