use std::env;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use skillscout::args::Args;
use skillscout::banner::print_banner;
use skillscout::cache::clear_skillscout_cache;
use skillscout::claude::cleanup_claude_md;
use skillscout::detect::{
    collect_skills, detect_agents, detect_technologies, get_installed_skill_names,
};
use skillscout::display::{
    format_skill_label, print_detected, print_security_checks, print_skills_list, DisplayCombo,
    DisplayTechnology,
};
use skillscout::installer::{install_all, InstallOptions, SkillEntry};
use skillscout::prompt::{multi_select, MultiSelectOptions, Shortcut};
use skillscout::registry::{load_registry, security_check_for_entry};
use skillscout::ui::{bold, cyan, dim, green, is_tty, log, red, show_cursor, write, yellow};

const ISSUES_URL: &str = "https://github.com/GaboTech/skillscout/issues";

fn handle_sigint() {
    tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        write(&format!("{}\n", show_cursor()));
        std::process::exit(130);
    });
}

fn security_warning_for_skill(skill: &str) -> Option<String> {
    let registry = load_registry()?;
    let parsed = skillscout::registry::parse_skill_path(skill);
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
        Some("The sync review found issues that should be checked.".to_string())
    } else {
        Some(d)
    }
}

fn format_time(ms: u64) -> String {
    skillscout::display::format_time(ms)
}

fn brief_error_reason(stderr: &str, output: &str) -> String {
    let raw = if !stderr.trim().is_empty() {
        stderr
    } else {
        output
    };
    let stripped = skillscout::ui::strip_ansi(raw);
    let lines: Vec<String> = stripped
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("npm warn") && !l.starts_with("npm notice"))
        .collect();
    if lines.is_empty() {
        return "Unknown error".to_string();
    }
    let line = &lines[0];
    if line.len() > 80 {
        format!("{}...", &line[..77])
    } else {
        line.clone()
    }
}

fn strip_ansi(s: &str) -> String {
    skillscout::ui::strip_ansi(s)
}

fn print_summary(
    installed: usize,
    failed: usize,
    errors: &[skillscout::installer::InstallError],
    elapsed: u64,
    verbose: bool,
) {
    log("");
    if failed == 0 {
        log(&green(&bold(&format!(
            "   ✔ Done! {installed} skill{} installed in {}.",
            if installed != 1 { "s" } else { "" },
            format_time(elapsed)
        ))));
    } else {
        log(&yellow(&format!(
            "   Done: {}, {} in {}.",
            green(&format!("{installed} installed")),
            red(&format!("{failed} failed")),
            format_time(elapsed)
        )));
        if !errors.is_empty() {
            log("");
            log(&bold(&red("   Errors:")));
            for err in errors {
                log(&red(&format!("     ✘ {}", err.name)));
                if verbose {
                    if let Some(code) = err.exit_code {
                        log(&dim(&format!("       exit code {code}")));
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
                        log(&dim(&format!("       command: {}", err.command)));
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
                    "   Run again with --verbose to see the full error details.",
                ));
            }
            log(&dim(&format!(
                "   If it looks like an skillscout bug, please create an issue: {ISSUES_URL}"
            )));
        }
    }
    log("");
}

fn select_skills_sync(skills: Vec<SkillEntry>, auto_yes: bool) -> Vec<SkillEntry> {
    if auto_yes {
        print_skills_list(&skills);
        return skills;
    }

    const INSTALLED_TAG: &str = " (installed)";
    const SECURITY_TAG: &str = " (security check ⚠)";

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
            "{} found, {} already installed",
            skills.len(),
            installed_count
        )
    } else {
        format!("{} found", skills.len())
    };
    log(&format!(
        "{}{} {}",
        cyan("   ◆ "),
        bold("Select skills to install"),
        dim(&format!("({count_label})"))
    ));
    log("");

    // Build options
    #[allow(clippy::type_complexity)]
    let label_fn: Box<dyn Fn(&SkillEntry, usize) -> String> =
        Box::new(move |item: &SkillEntry, _idx: usize| {
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

    #[allow(clippy::type_complexity)]
    let hint_fn: Box<dyn Fn(&SkillEntry, usize) -> String> = Box::new(|item: &SkillEntry, _| {
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

    let group_fn: Box<dyn Fn(&SkillEntry) -> String> =
        Box::new(|item: &SkillEntry| item.sources.first().cloned().unwrap_or_default());

    let initial_selected: Vec<bool> = skills.iter().map(|s| !s.installed).collect();

    let mut shortcuts: Vec<Shortcut<SkillEntry>> = Vec::new();
    if installed_count > 0 {
        shortcuts.push(Shortcut {
            key: 'n',
            label: "new".to_string(),
            func: Box::new(|items: &[SkillEntry]| items.iter().map(|s| !s.installed).collect()),
        });
        shortcuts.push(Shortcut {
            key: 'i',
            label: "installed".to_string(),
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
        log(&dim("   Nothing selected."));
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
        let (cache_dir, removed) = clear_skillscout_cache();
        if removed {
            log(&green(&format!(
                "   ✔ Cleared skillscout cache: {}",
                cache_dir.display()
            )));
        } else {
            log(&dim(&format!(
                "   No skillscout cache found: {}",
                cache_dir.display()
            )));
        }
        log("");
        std::process::exit(0);
    }

    let version = env!("CARGO_PKG_VERSION");
    print_banner(version).await;

    let project_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    write(&dim("   Scanning project...\r"));
    let detect_result = detect_technologies(&project_dir);
    write("\x1b[K");

    if detect_result.detected.is_empty() && !detect_result.is_frontend {
        log(&yellow("   ⚠ No supported technologies detected."));
        log(&dim("   Make sure you run this in a project directory."));
        log("");
        std::process::exit(0);
    }

    // Convert to display types
    let detected_display: Vec<DisplayTechnology> = detect_result
        .detected
        .iter()
        .map(|t| DisplayTechnology {
            id: t.id.clone(),
            name: t.name.clone(),
            skills: t.skills.clone(),
        })
        .collect();
    let combos_display: Vec<DisplayCombo> = detect_result
        .combos
        .iter()
        .map(|c| DisplayCombo {
            id: c.id.clone(),
            name: c.name.clone(),
        })
        .collect();

    print_detected(
        &detected_display,
        &combos_display,
        detect_result.is_frontend,
    );

    let installed_names = get_installed_skill_names(&project_dir);
    let skills = collect_skills(
        &detect_result.detected,
        detect_result.is_frontend,
        &detect_result.combos,
        Some(&installed_names),
    );

    let resolved_agents = if args.agent.is_empty() {
        detect_agents()
    } else {
        args.agent.clone()
    };

    if skills.is_empty() {
        log(&yellow("   No skills available for your stack yet."));
        log(&dim("   Check https://skillscout.sh for the latest."));
        log("");
        std::process::exit(0);
    }

    if args.dry_run {
        print_skills_list(&skills);
        log(&dim(&format!("   Agents: {}", resolved_agents.join(", "))));
        log(&dim("   --dry-run: nothing was installed."));
        log("");
        std::process::exit(0);
    }

    // Warm registry in background
    let registry_handle = tokio::spawn(async {
        let _ = load_registry();
    });

    let selected_skills = select_skills_sync(skills.clone(), args.yes);

    log("");
    log(&(cyan("   ◆ ") + &bold("Installing skills...")));
    log(&dim(&format!("   Agents: {}", resolved_agents.join(", "))));
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
        pb.set_message("Installing skills...");

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

    if is_tty() && !args.verbose {
        // Move cursor up and rewrite Done! like TS
        let up = selected_skills.len() + 2;
        write(&format!("\x1b[{up}A\r\x1b[K"));
        log(&(green("   ◆ ") + &bold("Done!")));
        write(&format!("\x1b[{}B", selected_skills.len() + 1));
    }

    if claude_cleanup.cleaned {
        if claude_cleanup.deleted {
            log(&dim(
                "   Removed skillscout section from CLAUDE.md (file was empty, deleted).",
            ));
        } else {
            log(&dim("   Removed skillscout section from CLAUDE.md."));
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
