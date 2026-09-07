use skillindex::display::format_time;
use skillindex::installer::InstallError;
use skillindex::ui::{bold, dim, green, log, red, strip_ansi, yellow};

const ISSUES_URL: &str = "https://github.com/Gabox301/SkillIndex/issues";

pub fn brief_error_reason(stderr: &str, output: &str) -> String {
    let raw: &str = if !stderr.trim().is_empty() {
        stderr
    } else {
        output
    };
    let stripped: String = strip_ansi(raw);
    let lines: Vec<String> = stripped
        .lines()
        .map(|l: &str| l.trim().to_string())
        .filter(|l: &String| {
            !l.is_empty() && !l.starts_with("npm warn") && !l.starts_with("npm notice")
        })
        .collect();
    if lines.is_empty() {
        return "Error desconocido".to_string();
    }
    let line: &String = &lines[0];
    if line.len() > 80 {
        format!("{}...", &line[..77])
    } else {
        line.clone()
    }
}

pub fn print_summary(
    installed: usize,
    failed: usize,
    errors: &[InstallError],
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
                    let combined: String = format!("{}\n{}", err.stderr, err.output);
                    let stripped: String = strip_ansi(&combined);
                    let lines: Vec<String> = stripped
                        .lines()
                        .map(|l: &str| l.trim().to_string())
                        .filter(|l: &String| !l.is_empty())
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
                    let reason: String = brief_error_reason(&err.stderr, &err.output);
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
