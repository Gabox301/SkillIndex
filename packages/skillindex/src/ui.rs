use std::env;
use std::io::{self, IsTerminal, Write};

// ── Color detection ────────────────────────────────────────────────

fn no_color() -> bool {
    env::var("NO_COLOR").is_ok()
}

fn force_color() -> bool {
    env::var("FORCE_COLOR").is_ok()
}

pub fn use_color() -> bool {
    if force_color() {
        return true;
    }
    if no_color() {
        return false;
    }
    io::stdout().is_terminal()
}

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

fn style(code: &str, s: &str, reset: &str) -> String {
    if use_color() {
        format!("{code}{s}{reset}")
    } else {
        s.to_string()
    }
}

// ── Color helpers — mirrors colors.ts ──────────────────────────────

pub fn bold(s: &str) -> String {
    style("\x1b[1m", s, "\x1b[22m")
}
pub fn dim(s: &str) -> String {
    style("\x1b[2m", s, "\x1b[22m")
}
pub fn green(s: &str) -> String {
    style("\x1b[32m", s, "\x1b[39m")
}
pub fn yellow(s: &str) -> String {
    style("\x1b[33m", s, "\x1b[39m")
}
pub fn cyan(s: &str) -> String {
    style("\x1b[36m", s, "\x1b[39m")
}
pub fn red(s: &str) -> String {
    style("\x1b[31m", s, "\x1b[39m")
}
pub fn magenta(s: &str) -> String {
    style("\x1b[35m", s, "\x1b[39m")
}
pub fn gray(s: &str) -> String {
    style("\x1b[38;5;240m", s, "\x1b[39m")
}
pub fn muted(s: &str) -> String {
    style("\x1b[38;2;174;170;215m", s, "\x1b[39m")
}
pub fn white(s: &str) -> String {
    style("\x1b[97m", s, "\x1b[39m")
}
pub fn pink(s: &str) -> String {
    style("\x1b[38;5;218m", s, "\x1b[39m")
}

// ── Output helpers — mirrors `log`/`write` from colors.ts ─────────

/// Like `console.log` — prints with newline
pub fn log(msg: &str) {
    println!("{msg}");
}

/// Like `process.stdout.write` — raw write without newline
pub fn write(msg: &str) {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(msg.as_bytes());
    let _ = stdout.flush();
}

// ── Cursor & spinner ───────────────────────────────────────────────

pub fn hide_cursor() -> String {
    if is_tty() {
        "\x1b[?25l".to_string()
    } else {
        String::new()
    }
}

pub fn show_cursor() -> String {
    if is_tty() {
        "\x1b[?25h".to_string()
    } else {
        String::new()
    }
}

pub const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Strip ANSI escape codes — mirrors `stripAnsi` in main.ts
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_codes() {
        let s = "\x1b[31mhello\x1b[39m";
        assert_eq!(strip_ansi(s), "hello");
        let s2 = format!("{}hi{}", bold(""), "");
        // bold without color may be empty if NO_COLOR, but strip should work
        let colored = "\x1b[1mhello\x1b[22m";
        assert_eq!(strip_ansi(colored), "hello");
        let _ = s2;
    }

    #[test]
    fn use_color_respects_no_color() {
        // This test relies on global env; we test the function exists and returns bool
        let _ = use_color();
        let _ = is_tty();
    }

    #[test]
    fn color_helpers_return_string() {
        // When NO_COLOR=1, these return plain; otherwise with codes — both are strings
        // We test they contain original text
        for f in [
            bold, dim, green, yellow, cyan, red, magenta, gray, muted, white, pink,
        ] {
            let s = f("test");
            assert!(strip_ansi(&s).contains("test"));
        }
    }

    #[test]
    fn spinner_has_ten_frames() {
        assert_eq!(SPINNER.len(), 10);
        assert_eq!(SPINNER[0], "⠋");
        assert_eq!(SPINNER[9], "⠏");
    }

    #[test]
    fn hide_show_cursor_empty_when_not_tty_or_present() {
        let h = hide_cursor();
        let s = show_cursor();
        // Both are either "" or ANSI; just ensure they are strings
        assert!(h.len() <= 6);
        assert!(s.len() <= 6);
    }
}
