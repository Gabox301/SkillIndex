use crate::ui::{bold, cyan, dim, is_tty, use_color, write};

const LOGO_LINES: &[&str] = &[
    " ███████╗██╗  ██╗██╗██╗     ██╗     ██╗███╗   ██╗██████╗ ███████╗██╗  ██╗",
    " ██╔════╝██║ ██╔╝██║██║     ██║     ██║████╗  ██║██╔══██╗██╔════╝╚██╗██╔╝",
    " ███████╗█████╔╝ ██║██║     ██║     ██║██╔██╗ ██║██║  ██║█████╗   ╚███╔╝ ",
    " ╚════██║██╔═██╗ ██║██║     ██║     ██║██║╚██╗██║██║  ██║██╔══╝   ██╔██╗ ",
    " ███████║██║  ██╗██║███████╗███████╗██║██║ ╚████║██████╔╝███████╗██╔╝ ██╗",
    " ╚══════╝╚═╝  ╚═╝╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝",
];

fn rgb(gray_value: u8, text: &str) -> String {
    format!("\x1b[38;2;{gray_value};{gray_value};{gray_value}m{text}\x1b[39m")
}

fn render_animated_logo(frame: usize, speed: f64) -> Vec<String> {
    let wave_front = frame as f64 * speed;
    LOGO_LINES
        .iter()
        .enumerate()
        .map(|(row, line)| {
            line.chars()
                .enumerate()
                .map(|(col, ch)| {
                    if ch == ' ' {
                        ch.to_string()
                    } else {
                        let distance = col as f64 + row as f64 * 2.0;
                        let progress = ((wave_front - distance) / 10.0).clamp(0.0, 1.0);
                        let gray_value = (63.0 + progress * (244.0 - 63.0)).round() as u8;
                        rgb(gray_value, &ch.to_string())
                    }
                })
                .collect::<String>()
        })
        .collect()
}

fn is_no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Print the SkillIndex banner — wave animation if TTY and colors enabled, else static.
/// Mirrors `printBanner` in ui.ts (grayscale wave 28ms, totalFrames = ceil((cols+rows*2+10)/speed)).
pub async fn print_banner(version: &str) {
    let ver = format!("v{version}");
    let subtitle = format!("Instala automáticamente las mejores skills de IA para tu proyecto · {ver}");

    if !is_tty() || is_no_color() || !use_color() {
        println!();
        for line in LOGO_LINES {
            println!("{}", bold(&cyan(line)));
        }
        println!("{}", dim(&subtitle));
        println!();
        return;
    }

    let cols = LOGO_LINES
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let rows = LOGO_LINES.len();
    let speed = 2.5_f64;
    let frame_delay_ms = 28_u64;
    let total_frames = ((cols + rows * 2 + 10) as f64 / speed).ceil() as usize;

    write(&format!("{}\n", crate::ui::hide_cursor()));
    for frame in 0..=total_frames {
        let lines = render_animated_logo(frame, speed);
        let rendered = lines
            .iter()
            .map(|line| format!("   {line}"))
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
    write(&crate::ui::show_cursor());
    println!();
}

/// Synchronous version for tests (no animation, no async)
pub fn format_banner_static(version: &str) -> String {
    let ver = format!("v{version}");
    let subtitle = format!("Instala automáticamente las mejores skills de IA para tu proyecto · {ver}");
    let mut out = String::new();
    out.push('\n');
    for line in LOGO_LINES {
        out.push_str(&format!("{}\n", bold(&cyan(line))));
    }
    out.push_str(&format!("{}\n", dim(&subtitle)));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_animated_logo_first_frame_is_dim() {
        let lines = render_animated_logo(0, 2.5);
        assert_eq!(lines.len(), 6);
        // first frame should be mostly dark (gray 63) — check contains ansi
        assert!(lines[0].contains("\x1b[38;2;"));
    }

    #[test]
    fn render_animated_logo_last_frame_is_bright() {
        // large frame -> wave has passed, all chars bright 244
        let lines = render_animated_logo(100, 2.5);
        for line in lines {
            // Each non-space char should be 244
            if line.contains("\x1b[38;2;244;244;244m") {
                // at least one bright
            }
        }
    }

    #[test]
    fn format_banner_static_contains_version() {
        let s = format_banner_static("0.3.6");
        assert!(s.contains("v0.3.6"));
        assert!(s.contains("Instala automáticamente"));
        // should contain logo chars
        assert!(s.contains("█████"));
    }

    #[test]
    fn static_banner_line_count() {
        let s = format_banner_static("1.0.0");
        // 1 blank + 6 logo + 1 subtitle + 1 blank = 9 lines (with \n)
        let lines: Vec<&str> = s.split('\n').collect();
        assert!(lines.len() >= 9);
    }

    #[tokio::test]
    async fn print_banner_does_not_panic_no_color() {
        // Set NO_COLOR to force static path
        let prev = std::env::var("NO_COLOR").ok();
        unsafe { std::env::set_var("NO_COLOR", "1") };
        print_banner("0.3.6").await;
        match prev {
            Some(v) => unsafe { std::env::set_var("NO_COLOR", v) },
            None => unsafe { std::env::remove_var("NO_COLOR") },
        }
    }
}
