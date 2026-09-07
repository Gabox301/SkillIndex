pub mod security;
pub mod selection;
pub mod signals;
pub mod summary;

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use skillindex::args::Args;
use skillindex::detect::{
    collect_skills, detect_agents, detect_technologies, get_installed_skill_names, partition_combos,
};
use skillindex::display::{
    DisplayCombo, DisplayTechnology, print_detected, print_security_checks, print_skills_list,
};
use skillindex::infra::cache::clear_skillindex_cache;
use skillindex::infra::claude::cleanup_claude_md;
use skillindex::installer::{InstallError, InstallOptions, install_all, install_skill};
use skillindex::registry::load_registry;
use skillindex::ui::banner::print_banner;
use skillindex::ui::{bold, brand_cyan, dim, green, is_tty, log, write};

use security::ask_include_security_sync;
use selection::{select_agents_sync, select_skills_sync};
use signals::handle_sigint;
use summary::print_summary;

pub async fn run() {
    handle_sigint();

    let args: Args = Args::parse();

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

    // Instalación directa de una skill por path (ej. virgiliojr94/book-to-skill)
    if let Some(skill_path) = args.skill.clone() {
        let project_dir: PathBuf = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut resolved_agents: Vec<String> = if args.agent.is_empty() {
            detect_agents(&project_dir)
        } else {
            args.agent.clone()
        };
        if args.agent.is_empty() {
            resolved_agents = select_agents_sync(resolved_agents, args.yes);
        }
        let start: Instant = Instant::now();
        let opts: InstallOptions = InstallOptions {
            project_dir: Some(project_dir.clone()),
            ..Default::default()
        };
        let result: skillindex::installer::InstallResult =
            install_skill(&skill_path, &resolved_agents, opts).await;
        let elapsed: u64 = start.elapsed().as_millis() as u64;
        if result.success {
            print_security_checks(&result.security_check.into_iter().collect::<Vec<_>>());
            print_summary(1, 0, &[], elapsed, args.verbose);
        } else {
            print_summary(
                0,
                1,
                &[InstallError {
                    name: skill_path.clone(),
                    output: result.output.clone(),
                    stderr: result.stderr.clone(),
                    exit_code: result.exit_code,
                    command: result.command.clone(),
                }],
                elapsed,
                args.verbose,
            );
        }
        std::process::exit(if result.success { 0 } else { 1 });
    }

    let version: &str = env!("CARGO_PKG_VERSION");
    print_banner(version).await;

    let project_dir: PathBuf = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    write(&dim("   Analizando proyecto...\r"));
    let detect_result: skillindex::detect::DetectResult = detect_technologies(&project_dir);
    write("\x1b[K");

    let (regular_combos, security_combos) = partition_combos(detect_result.combos);

    // 1. Tecnologías detectadas (sin seguridad — se ofrece después)
    let detected_display: Vec<DisplayTechnology> = detect_result
        .detected
        .iter()
        .map(|t: &DisplayTechnology| DisplayTechnology {
            id: t.id.clone(),
            name: t.name.clone(),
            skills: t.skills.clone(),
        })
        .collect();
    let combos_display_regular: Vec<DisplayCombo> = regular_combos
        .iter()
        .map(|c: &DisplayCombo| DisplayCombo {
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
    let mut resolved_agents: Vec<String> = if args.agent.is_empty() {
        detect_agents(&project_dir)
    } else {
        args.agent.clone()
    };

    if args.agent.is_empty() {
        resolved_agents = select_agents_sync(resolved_agents, args.yes);
    }

    // 3. Seguridad (opcionales) — checkbox y/n
    let include_security: bool =
        ask_include_security_sync(&security_combos, args.security, args.yes);
    let final_combos: Vec<DisplayCombo> = if include_security {
        let mut v: Vec<DisplayCombo> = regular_combos.clone();
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
        log(&skillindex::ui::yellow(
            "   ⚠ No se detectaron tecnologías compatibles.",
        ));
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
    let installed_names: std::collections::HashSet<String> =
        get_installed_skill_names(&project_dir);
    let skills: Vec<skillindex::installer::SkillEntry> = collect_skills(
        &detect_result.detected,
        detect_result.is_frontend,
        &final_combos,
        Some(&installed_names),
    );

    if skills.is_empty() {
        log(&skillindex::ui::yellow(
            "   Aún no hay skills disponibles para tu stack.",
        ));
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
    let registry_handle: tokio::task::JoinHandle<()> = tokio::spawn(async {
        let _ = load_registry();
    });

    let selected_skills: Vec<skillindex::installer::SkillEntry> =
        select_skills_sync(skills.clone(), args.yes);

    log("");
    log(&(brand_cyan("   ◆ ") + &bold("Instalando skills...")));
    log(&dim(&format!("   Agentes: {}", resolved_agents.join(", "))));
    log("");

    let start: Instant = Instant::now();

    // Ensure registry loaded
    let _ = registry_handle.await;

    let install_result: skillindex::installer::InstallAllResult = if is_tty() && !args.verbose {
        let pb: ProgressBar = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb.set_message("Instalando skills...");

        let opts: InstallOptions = InstallOptions {
            project_dir: Some(project_dir.clone()),
            ..Default::default()
        };
        let res: skillindex::installer::InstallAllResult =
            install_all(selected_skills.clone(), resolved_agents.clone(), opts).await;
        pb.finish_and_clear();
        res
    } else {
        let opts: InstallOptions = InstallOptions {
            project_dir: Some(project_dir.clone()),
            ..Default::default()
        };
        install_all(selected_skills.clone(), resolved_agents.clone(), opts).await
    };

    let elapsed: u64 = start.elapsed().as_millis() as u64;
    let claude_cleanup: skillindex::claude::CleanupResult = cleanup_claude_md(&project_dir);

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
