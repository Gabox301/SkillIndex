use std::env;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use skillindex::args::Args;
use skillindex::banner::print_banner;
use skillindex::cache::clear_skillindex_cache;
use skillindex::claude::cleanup_claude_md;
use skillindex::detect::{
    collect_skills, detect_agents, detect_technologies, get_installed_skill_names, partition_combos,
};
use skillindex::display::{
    DisplayCombo, DisplayTechnology, format_skill_label, print_detected, print_security_checks,
    print_skills_list,
};
use skillindex::installer::{InstallOptions, SkillEntry, install_all};
use skillindex::prompt::{MultiSelectOptions, Shortcut, multi_select};
use skillindex::registry::{load_registry, security_check_for_entry};
use skillindex::ui::{bold, cyan, dim, green, is_tty, log, red, show_cursor, write, yellow};

const ISSUES_URL: &str = "https://github.com/Gabox301/SkillIndex/issues";

// Aliases para closures de multi_select — evitan `type_complexity` inline
// y hacen explícito el contrato de cada callback sin silenciar lints.
type AgentLabelFn = dyn Fn(&String, usize) -> String;
type SkillLabelFn = dyn Fn(&SkillEntry, usize) -> String;
type SkillHintFn = dyn Fn(&SkillEntry, usize) -> String;
type SkillGroupFn = dyn Fn(&SkillEntry) -> String;

fn handle_sigint() {
    tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        write(&format!("{}\n", show_cursor()));
        std::process::exit(130);
    });
}

fn security_warning_for_skill(skill: &str) -> Option<String> {
    let registry = load_registry()?;
    let parsed = skillindex::registry::parse_skill_path(skill);
    let entry = registry.skills.get(&parsed.skill_name)?;
    let check = security_check_for_entry(&parsed.skill_name, entry);
    if check.status != "warning" {
        return None;
    }
    let findings: Vec<String> = check
        .findings
        .iter()
        .map(|f| f.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut detail = Vec::new();
    if !check.summary.trim().is_empty() {
        detail.push(check.summary.trim().to_string());
    }
    if !findings.is_empty() {
        detail.push(findings.join("; "));
    }
    let d = detail.join(" ");
    if d.is_empty() {
        Some(
            "La revisión de sincronización encontró observaciones que deberías revisar."
                .to_string(),
        )
    } else {
        Some(d)
    }
}

fn format_time(ms: u64) -> String {
    skillindex::display::format_time(ms)
}

fn brief_error_reason(stderr: &str, output: &str) -> String {
    let raw = if !stderr.trim().is_empty() {
        stderr
    } else {
        output
    };
    let stripped = skillindex::ui::strip_ansi(raw);
    let lines: Vec<String> = stripped
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("npm warn") && !l.starts_with("npm notice"))
        .collect();
    if lines.is_empty() {
        return "Error desconocido".to_string();
    }
    let line = &lines[0];
    if line.len() > 80 {
        format!("{}...", &line[..77])
    } else {
        line.clone()
    }
}

fn strip_ansi(s: &str) -> String {
    skillindex::ui::strip_ansi(s)
}

fn print_summary(
    installed: usize,
    failed: usize,
    errors: &[skillindex::installer::InstallError],
    elapsed: u64,
    verbose: bool,
) {
    log("");
    if failed == 0 {
        log(&green(&bold(&format!(
            "   ✔ ¡Listo! {installed} skill{} instalad{} en {}.",
            if installed != 1 { "s" } else { "" },
            if installed != 1 { "as" } else { "a" },
            format_time(elapsed)
        ))));
    } else {
        log(&yellow(&format!(
            "   Completado: {}, {} en {}.",
            green(&format!("{installed} instaladas")),
            red(&format!("{failed} con error")),
            format_time(elapsed)
        )));
        if !errors.is_empty() {
            log("");
            log(&bold(&red("   Errores:")));
            for err in errors {
                log(&red(&format!("     ✘ {}", err.name)));
                if verbose {
                    if let Some(code) = err.exit_code {
                        log(&dim(&format!("       código de salida {code}")));
                    }
                    let combined = format!("{}\n{}", err.stderr, err.output);
                    let stripped = strip_ansi(&combined);
                    let lines: Vec<String> = stripped
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .take(20)
                        .collect();
                    if !lines.is_empty() {
                        log("");
                        for line in &lines {
                            log(&dim(&format!("       {line}")));
                        }
                        if lines.len() == 20 {
                            log(&dim("       … (more lines)"));
                        }
                    }
                    if !err.command.is_empty() {
                        log("");
                        log(&dim(&format!("       comando: {}", err.command)));
                    }
                    log("");
                } else {
                    let reason = brief_error_reason(&err.stderr, &err.output);
                    log(&dim(&format!("       {reason}")));
                }
            }
            log("");
            if !verbose {
                log(&dim(
                    "   Ejecuta de nuevo con --verbose para ver los detalles completos del error.",
                ));
            }
            log(&dim(&format!(
                "   Si parece un error de skillindex, por favor crea un issue: {ISSUES_URL}"
            )));
        }
    }
    log("");
}

fn select_agents_sync(agents: Vec<String>, auto_yes: bool) -> Vec<String> {
    // Solo delega si hay ambigüedad real: más de un agente concreto detectado.
    // Si el usuario pasó -a/--agent o --yes, se respeta sin preguntar.
    let real_agents: Vec<String> = agents
        .iter()
        .filter(|a| a.as_str() != "universal")
        .cloned()
        .collect();
    if real_agents.len() <= 1 {
        return agents;
    }
    if auto_yes || !is_tty() {
        return agents;
    }

    log(&format!(
        "{}{} {}",
        cyan("   ◆ "),
        bold("Selecciona dónde instalar"),
        dim(&format!("({} agentes detectados)", real_agents.len()))
    ));
    log(&dim(
        "   Desmarca los que no quieras. Por defecto instala en todos.",
    ));
    log("");

    let styled_label_fn: Box<AgentLabelFn> = Box::new(|item: &String, _| {
        let folder = skillindex::registry::agent_folder_for(item).unwrap_or(".agents");
        format!("{} {}", bold(item), dim(&format!("({folder})")))
    });

    let opts = MultiSelectOptions {
        label_fn: styled_label_fn,
        hint_fn: None,
        group_fn: None,
        initial_selected: Some(vec![true; real_agents.len()]),
        shortcuts: Vec::new(),
    };

    let selected_real = multi_select(real_agents.clone(), opts).unwrap_or_default();

    if selected_real.is_empty() {
        log("");
        log(&dim(
            "   Ningún agente seleccionado — no se instalará nada.",
        ));
        log("");
        std::process::exit(0);
    }

    // Reconstruir lista final: siempre incluir universal si estaba originalmente?
    // No: universal solo es fallback cuando no hay agentes reales. Si el usuario
    // eligió agentes concretos, no incluimos universal para no crear .agents duplicado.
    // Si el usuario quiere universal explícito, debe pasarlo con -a universal.
    selected_real
}

fn ask_include_security_sync(
    security_combos: &[DisplayCombo],
    force_security: bool,
    auto_yes: bool,
) -> bool {
    if security_combos.is_empty() {
        return false;
    }
    if force_security {
        return true;
    }
    if auto_yes || !is_tty() {
        return false;
    }
    // puede no reportar is_terminal pero sí hay interacción real.
    log(&format!(
        "{}{} {}",
        cyan("   ◆ "),
        bold("Seguridad (opcionales)"),
        dim(&format!("— {} combos", security_combos.len()))
    ));
    log(&dim(&format!(
        "   {}",
        security_combos
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
            .join(" · ")
    )));
    log(&dim(
        "   ¿Incluir skills de seguridad? Por defecto no. [y/N]",
    ));
    log("");
    write(&dim("   ¿Incluir? [y/N]: "));
    let _ = std::io::stdout().flush();
    let _ = crossterm::terminal::enable_raw_mode();
    let mut include = false;
    loop {
        if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
            match key.code {
                crossterm::event::KeyCode::Char('y')
                | crossterm::event::KeyCode::Char('Y')
                | crossterm::event::KeyCode::Char('s')
                | crossterm::event::KeyCode::Char('S')
                | crossterm::event::KeyCode::Char(' ') => {
                    include = true;
                    break;
                }
                crossterm::event::KeyCode::Char('n')
                | crossterm::event::KeyCode::Char('N')
                | crossterm::event::KeyCode::Enter
                | crossterm::event::KeyCode::Esc => {
                    include = false;
                    break;
                }
                crossterm::event::KeyCode::Char('q')
                | crossterm::event::KeyCode::Char('Q') => {
                    include = false;
                    break;
                }
                _ => {}
            }
        }
    }
    let _ = crossterm::terminal::disable_raw_mode();
    if include {
        log(&dim("   → y"));
    } else {
        log(&dim("   → n"));
    }
    log("");
    include
}

fn select_skills_sync(skills: Vec<SkillEntry>, auto_yes: bool) -> Vec<SkillEntry> {
    if auto_yes {
        print_skills_list(&skills);
        return skills;
    }

    const INSTALLED_TAG: &str = " (instalada)";
    const SECURITY_TAG: &str = " (revisión de seguridad ⚠)";

    // label cache
    let mut label_cache: std::collections::HashMap<String, (String, String, bool)> =
        std::collections::HashMap::new();
    for s in &skills {
        let label = format_skill_label(&s.skill, false);
        let styled = format_skill_label(&s.skill, true);
        let has_warn = security_warning_for_skill(&s.skill).is_some();
        label_cache.insert(s.skill.clone(), (label, styled, has_warn));
    }
    let max_effective = skills
        .iter()
        .map(|s| {
            let (label, _, has_warn) = label_cache.get(&s.skill).unwrap();
            label.len()
                + if s.installed { INSTALLED_TAG.len() } else { 0 }
                + if *has_warn { SECURITY_TAG.len() } else { 0 }
        })
        .max()
        .unwrap_or(0);

    let new_count = skills.iter().filter(|s| !s.installed).count();
    let installed_count = skills.len() - new_count;
    let count_label = if installed_count > 0 {
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
        cyan("   ◆ "),
        bold("Selecciona las skills a instalar"),
        dim(&format!("({count_label})"))
    ));
    log("");

    // Build options
    let label_fn: Box<SkillLabelFn> = Box::new(move |item: &SkillEntry, _idx: usize| {
        let (label, styled_label, has_warn) = label_cache.get(&item.skill).unwrap();
        let installed_tag = if item.installed {
            dim(INSTALLED_TAG)
        } else {
            String::new()
        };
        let security_tag = if *has_warn {
            yellow(SECURITY_TAG)
        } else {
            String::new()
        };
        let effective_len = label.len()
            + if item.installed {
                INSTALLED_TAG.len()
            } else {
                0
            }
            + if *has_warn { SECURITY_TAG.len() } else { 0 };
        let pad = " ".repeat(max_effective.saturating_sub(effective_len));
        format!("{styled_label}{installed_tag}{security_tag}{pad}")
    });

    let hint_fn: Box<SkillHintFn> = Box::new(|item: &SkillEntry, _| {
        let tech_sources: Vec<&String> =
            item.sources.iter().filter(|s| !s.contains(" + ")).collect();
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

    let initial_selected: Vec<bool> = skills.iter().map(|s| !s.installed).collect();

    let mut shortcuts: Vec<Shortcut<SkillEntry>> = Vec::new();
    if installed_count > 0 {
        shortcuts.push(Shortcut {
            key: 'n',
            label: "nuevas".to_string(),
            func: Box::new(|items: &[SkillEntry]| items.iter().map(|s| !s.installed).collect()),
        });
        shortcuts.push(Shortcut {
            key: 'i',
            label: "instaladas".to_string(),
            func: Box::new(|items: &[SkillEntry]| items.iter().map(|s| s.installed).collect()),
        });
    }

    let opts = MultiSelectOptions {
        label_fn,
        hint_fn: Some(hint_fn),
        group_fn: Some(group_fn),
        initial_selected: Some(initial_selected),
        shortcuts,
    };

    let selected = multi_select(skills, opts).unwrap_or_default();

    if selected.is_empty() {
        log("");
        log(&dim("   Nada seleccionado."));
        log("");
        std::process::exit(0);
    }
    selected
}

#[tokio::main]
async fn main() {
    handle_sigint();

    let args = Args::parse();

    if args.clear_cache {
        let (cache_dir, removed) = clear_skillindex_cache();
        if removed {
            log(&green(&format!(
                "   ✔ Caché de skillindex limpiada: {}",
                cache_dir.display()
            )));
        } else {
            log(&dim(&format!(
                "   No se encontró caché de skillindex: {}",
                cache_dir.display()
            )));
        }
        log("");
        std::process::exit(0);
    }

    let version = env!("CARGO_PKG_VERSION");
    print_banner(version).await;

    let project_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    write(&dim("   Analizando proyecto...\r"));
    let detect_result = detect_technologies(&project_dir);
    write("\x1b[K");

    let (regular_combos, security_combos) = partition_combos(detect_result.combos);

    // 1. Tecnologías detectadas (sin seguridad — se ofrece después)
    let detected_display: Vec<DisplayTechnology> = detect_result
        .detected
        .iter()
        .map(|t| DisplayTechnology {
            id: t.id.clone(),
            name: t.name.clone(),
            skills: t.skills.clone(),
        })
        .collect();
    let combos_display_regular: Vec<DisplayCombo> = regular_combos
        .iter()
        .map(|c| DisplayCombo {
            id: c.id.clone(),
            name: c.name.clone(),
        })
        .collect();

    print_detected(
        &detected_display,
        &combos_display_regular,
        detect_result.is_frontend,
    );

    // 2. Agentes — decidir dónde instalar
    let mut resolved_agents = if args.agent.is_empty() {
        detect_agents(&project_dir)
    } else {
        args.agent.clone()
    };

    if args.agent.is_empty() {
        resolved_agents = select_agents_sync(resolved_agents, args.yes);
    }

    // 3. Seguridad (opcionales) — checkbox y/n
    let include_security = ask_include_security_sync(&security_combos, args.security, args.yes);
    let final_combos = if include_security {
        let mut v = regular_combos.clone();
        v.extend(security_combos.clone());
        v
    } else {
        regular_combos.clone()
    };
    if include_security && !security_combos.is_empty() {
        log(&dim(&format!(
            "   ↳ Seguridad incluida: {}",
            security_combos
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )));
        log("");
    }

    if detect_result.detected.is_empty() && !detect_result.is_frontend && final_combos.is_empty() {
        log(&yellow("   ⚠ No se detectaron tecnologías compatibles."));
        log(&dim(
            "   Asegúrate de ejecutar esto en el directorio de un proyecto.",
        ));
        log(&dim(
            "   Tip: activa Seguridad (opcionales) con --security si quieres skills de seguridad.",
        ));
        log("");
        std::process::exit(0);
    }

    // 4. Skills — con o sin seguridad según el check
    let installed_names = get_installed_skill_names(&project_dir);
    let skills = collect_skills(
        &detect_result.detected,
        detect_result.is_frontend,
        &final_combos,
        Some(&installed_names),
    );

    if skills.is_empty() {
        log(&yellow("   Aún no hay skills disponibles para tu stack."));
        log(&dim(
            "   Consulta https://skillindex.netlify.app para las últimas novedades.",
        ));
        log("");
        std::process::exit(0);
    }

    if args.dry_run {
        print_skills_list(&skills);
        log(&dim(&format!("   Agentes: {}", resolved_agents.join(", "))));
        log(&dim("   --dry-run: no se instaló nada."));
        log("");
        std::process::exit(0);
    }

    // Warm registry in background
    let registry_handle = tokio::spawn(async {
        let _ = load_registry();
    });

    let selected_skills = select_skills_sync(skills.clone(), args.yes);

    log("");
    log(&(cyan("   ◆ ") + &bold("Instalando skills...")));
    log(&dim(&format!("   Agentes: {}", resolved_agents.join(", "))));
    log("");

    let start = Instant::now();

    // Ensure registry loaded
    let _ = registry_handle.await;

    let install_result = if is_tty() && !args.verbose {
        // spinner mode with indicatif 80ms
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb.set_message("Instalando skills...");

        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            ..Default::default()
        };
        let res = install_all(selected_skills.clone(), resolved_agents.clone(), opts).await;
        pb.finish_and_clear();
        res
    } else {
        let opts = InstallOptions {
            project_dir: Some(project_dir.clone()),
            ..Default::default()
        };
        install_all(selected_skills.clone(), resolved_agents.clone(), opts).await
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let claude_cleanup = cleanup_claude_md(&project_dir);

    if claude_cleanup.cleaned {
        if claude_cleanup.deleted {
            log(&dim(
                "   Se eliminó la sección de skillindex de CLAUDE.md (archivo vacío, eliminado).",
            ));
        } else {
            log(&dim("   Se eliminó la sección de skillindex de CLAUDE.md."));
        }
        log("");
    }

    print_security_checks(&install_result.security_checks);
    print_summary(
        install_result.installed,
        install_result.failed,
        &install_result.errors,
        elapsed,
        args.verbose,
    );
}
