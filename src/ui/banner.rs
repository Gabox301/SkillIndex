use super::{bold, brand_cyan, dim, is_tty, use_color, write};

const LOGO_LINES: &[&str] = &[
    " ███████╗██╗  ██╗██╗██╗     ██╗     ██╗███╗   ██╗██████╗ ███████╗██╗  ██╗",
    " ██╔════╝██║ ██╔╝██║██║     ██║     ██║████╗  ██║██╔══██╗██╔════╝╚██╗██╔╝",
    " ███████╗█████╔╝ ██║██║     ██║     ██║██╔██╗ ██║██║  ██║█████╗   ╚███╔╝ ",
    " ╚════██║██╔═██╗ ██║██║     ██║     ██║██║╚██╗██║██║  ██║██╔══╝   ██╔██╗ ",
    " ███████║██║  ██╗██║███████╗███████╗██║██║ ╚████║██████╔╝███████╗██╔╝ ██╗",
    " ╚══════╝╚═╝  ╚═╝╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝",
];

/// Cuántas "unidades de distancia" tarda un carácter en pasar de negro (sombra,
/// no revelado) a su color de marca pleno. Sin blanco de por medio.
const REVEAL_WIDTH: f64 = 5.0;

/// Color de marca final (asentado) para una posición dada, 0.0 = cian, 1.0 = naranja.
fn brand_rgb(progress: f64) -> (u8, u8, u8) {
    let progress: f64 = progress.clamp(0.0, 1.0);
    let r: u8 = (56.0 + progress * (251.0 - 56.0)).round() as u8;
    let g: u8 = (189.0 + progress * (146.0 - 189.0)).round() as u8;
    let b: u8 = (248.0 + progress * (60.0 - 248.0)).round() as u8;
    (r, g, b)
}

fn ansi_rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[39m")
}

fn max_wave_distance() -> f64 {
    let cols: f64 = LOGO_LINES
        .iter()
        .map(|l: &&str| l.chars().count())
        .max()
        .unwrap_or(0) as f64;
    let rows: f64 = LOGO_LINES.len() as f64;
    cols + (rows - 1.0) * 2.0
}

fn render_animated_logo(frame: usize, speed: f64) -> Vec<String> {
    let wave_front: f64 = frame as f64 * speed;
    let max_distance: f64 = max_wave_distance();

    LOGO_LINES
        .iter()
        .enumerate()
        .map(|(row, line)| {
            line.chars()
                .enumerate()
                .map(|(col, ch)| {
                    if ch == ' ' {
                        return ch.to_string();
                    }

                    let distance: f64 = col as f64 + row as f64 * 2.0;
                    let delta: f64 = wave_front - distance;

                    // Todavía no llegó la ola: sombra negra pura, sin color.
                    if delta <= 0.0 {
                        return ansi_rgb(0, 0, 0, &ch.to_string());
                    }

                    let hue_progress: f64 = distance / max_distance;
                    let (br, bg, bb) = brand_rgb(hue_progress);

                    // La ola ya asentó del todo: color de marca pleno.
                    if delta >= REVEAL_WIDTH {
                        return ansi_rgb(br, bg, bb, &ch.to_string());
                    }

                    // En tránsito: interpolación directa negro -> color de marca.
                    // Nunca pasa por blanco.
                    let t: f64 = delta / REVEAL_WIDTH;
                    let r: u8 = (t * br as f64).round() as u8;
                    let g: u8 = (t * bg as f64).round() as u8;
                    let b: u8 = (t * bb as f64).round() as u8;

                    ansi_rgb(r, g, b, &ch.to_string())
                })
                .collect::<String>()
        })
        .collect()
}

fn is_no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Print the SkillIndex banner — wave animation if TTY and colors enabled, else static.
pub async fn print_banner(version: &str) {
    let ver: String = format!("v{version}");
    let subtitle: String = format!(
        "Instala las mejores skills de IA para tu proyecto · {ver} · Desarrollado por Gabriel Ortega"
    );

    if !is_tty() || is_no_color() || !use_color() {
        println!();
        for line in LOGO_LINES {
            println!("{}", bold(&brand_cyan(line)));
        }
        println!("{}", dim(&subtitle));
        println!();
        return;
    }

    let speed: f64 = 2.5_f64;
    let frame_delay_ms: u64 = 28_u64;
    let rows: usize = LOGO_LINES.len();
    // Sumamos REVEAL_WIDTH para que el último carácter llegue a asentarse en color pleno.
    let total_frames: usize = ((max_wave_distance() + REVEAL_WIDTH) / speed).ceil() as usize;

    write(&format!("{}\n", super::hide_cursor()));
    for frame in 0..=total_frames {
        let lines: Vec<String> = render_animated_logo(frame, speed);
        let rendered: String = lines
            .iter()
            .map(|line: &String| format!("   {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        write(&rendered);
        write("\n");

        if frame < total_frames {
            write(&format!("\x1b[{rows}A\r"));
            tokio::time::sleep(tokio::time::Duration::from_millis(frame_delay_ms)).await;
        }
    }

    write(&format!("   {}\n", dim(&subtitle)));
    write(&super::show_cursor());
    println!();
}

/// Synchronous version for tests (no animation, no async)
pub fn format_banner_static(version: &str) -> String {
    let ver: String = format!("v{version}");
    let subtitle: String = format!(
        "Instala las mejores skills de IA para tu proyecto · {ver} · Desarrollado por Gabriel Ortega"
    );
    let mut out: String = String::new();
    out.push('\n');
    for line in LOGO_LINES {
        out.push_str(&format!("{}\n", bold(&brand_cyan(line))));
    }
    out.push_str(&format!("{}\n", dim(&subtitle)));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_animated_logo_first_frame_is_black_shadow() {
        // frame 0: la mayoría de las columnas todavía no fueron alcanzadas por la ola.
        let lines: Vec<String> = render_animated_logo(0, 2.5);
        assert_eq!(lines.len(), 6);
        assert!(
            lines[0].contains("\x1b[38;2;0;0;0m"),
            "el primer frame debería mostrar sombra negra, no blanco ni color"
        );
    }

    #[test]
    fn render_animated_logo_never_shows_white() {
        // Recorremos varios frames y nos aseguramos de que nunca aparezca blanco puro
        // ni nada cercano (255,255,255) — el problema que estábamos corrigiendo.
        for frame in 0..40 {
            let lines: Vec<String> = render_animated_logo(frame, 2.5);
            for line in &lines {
                assert!(
                    !line.contains("\x1b[38;2;255;255;255m"),
                    "no debería aparecer blanco puro en el frame {frame}"
                );
            }
        }
    }

    #[test]
    fn render_animated_logo_settles_to_brand_color() {
        // Frame muy alto: la ola ya pasó por todo el logo, sin blanco de por medio.
        let max_distance: f64 = max_wave_distance();
        let speed: f64 = 2.5;
        let settled_frame: usize = ((max_distance + REVEAL_WIDTH * 2.0) / speed).ceil() as usize;
        let lines: Vec<String> = render_animated_logo(settled_frame, speed);
        // Debe contener colores de marca (cian → naranja), no negro ni blanco
        assert!(lines[0].contains("\x1b[38;2;"));
        assert!(!lines[0].contains("\x1b[38;2;0;0;0m"));
        assert!(!lines[0].contains("\x1b[38;2;255;255;255m"));
        assert!(lines[5].contains("\x1b[38;2;"));
    }

    #[test]
    fn format_banner_static_contains_version() {
        let s: String = format_banner_static("0.3.6");
        assert!(s.contains("v0.3.6"));
        assert!(s.contains("Instala las mejores skills de IA para tu proyecto"));
        assert!(s.contains("Desarrollado por Gabriel Ortega"));
        assert!(s.contains("█████"));
    }

    #[test]
    fn static_banner_line_count() {
        let s: String = format_banner_static("1.0.0");
        let lines: Vec<&str> = s.split('\n').collect();
        assert!(lines.len() >= 9);
    }

    #[tokio::test]
    async fn print_banner_does_not_panic_no_color() {
        let prev: Option<String> = std::env::var("NO_COLOR").ok();
        unsafe { std::env::set_var("NO_COLOR", "1") };
        print_banner("0.3.6").await;
        match prev {
            Some(v) => unsafe { std::env::set_var("NO_COLOR", v) },
            None => unsafe { std::env::remove_var("NO_COLOR") },
        }
    }
}
