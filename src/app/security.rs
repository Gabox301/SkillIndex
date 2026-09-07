use std::io::Write;

use skillindex::display::DisplayCombo;
use skillindex::registry::{load_registry, security_check_for_entry};
use skillindex::ui::{bold, brand_cyan, dim, is_tty, log, write};

pub fn security_warning_for_skill(skill: &str) -> Option<String> {
    let registry: skillindex::registry::Registry = load_registry()?;
    let parsed: skillindex::registry::ParsedSkillPath =
        skillindex::registry::parse_skill_path(skill);
    let entry: &skillindex::registry::RegistryEntry = registry.skills.get(&parsed.skill_name)?;
    let check: skillindex::registry::InstallSecurityCheck =
        security_check_for_entry(&parsed.skill_name, entry);
    if check.status != "warning" {
        return None;
    }
    let findings: Vec<String> = check
        .findings
        .iter()
        .map(|f: &String| f.trim().to_string())
        .filter(|s: &String| !s.is_empty())
        .collect();
    let mut detail: Vec<String> = Vec::new();
    if !check.summary.trim().is_empty() {
        detail.push(check.summary.trim().to_string());
    }
    if !findings.is_empty() {
        detail.push(findings.join("; "));
    }
    let d: String = detail.join(" ");
    if d.is_empty() {
        Some(
            "La revisión de sincronización encontró observaciones que deberías revisar."
                .to_string(),
        )
    } else {
        Some(d)
    }
}

pub fn ask_include_security_sync(
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
    log(&format!(
        "{}{} {}",
        brand_cyan("   ◆ "),
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
    if crossterm::terminal::enable_raw_mode().is_err() {
        log("");
        return false;
    }
    let mut include: bool = false;
    let mut decided: bool = false;
    while !decided {
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(key)) => {
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    crossterm::event::KeyCode::Char('y')
                    | crossterm::event::KeyCode::Char('Y')
                    | crossterm::event::KeyCode::Char('s')
                    | crossterm::event::KeyCode::Char('S')
                    | crossterm::event::KeyCode::Char(' ') => {
                        include = true;
                        decided = true;
                    }
                    crossterm::event::KeyCode::Char('n')
                    | crossterm::event::KeyCode::Char('N')
                    | crossterm::event::KeyCode::Enter
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Char('Q') => {
                        include = false;
                        decided = true;
                    }
                    _ => {}
                }
            }
            Ok(_) => continue,
            Err(_) => {
                include = false;
                break;
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
