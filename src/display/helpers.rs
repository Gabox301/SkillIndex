use crate::ui::{bold, brand_cyan, strip_ansi};

pub fn format_skill_label(skill: &str, styled: bool) -> String {
    if skill.to_lowercase().starts_with("http://") || skill.to_lowercase().starts_with("https://") {
        if styled {
            return brand_cyan(skill);
        } else {
            return skill.to_string();
        }
    }
    let parts: Vec<&str> = skill.split('/').collect();
    if parts.len() != 3 {
        if styled {
            return brand_cyan(skill);
        } else {
            return skill.to_string();
        }
    }
    let author = parts[0];
    let skill_name = parts[2];
    if !styled {
        return format!("{author} › {skill_name}");
    }
    format!(
        "{} {} {}",
        crate::ui::muted(author),
        crate::ui::gray("›"),
        brand_cyan(&bold(skill_name))
    )
}

pub fn strip_ansi_owned(s: &str) -> String {
    strip_ansi(s)
}

pub fn visible_pad(value: &str, width: usize) -> String {
    let visible_len = strip_ansi(value).chars().count();
    if visible_len >= width {
        value.to_string()
    } else {
        format!("{}{}", value, " ".repeat(width - visible_len))
    }
}

pub fn truncate_visible(value: &str, width: usize) -> String {
    let plain = strip_ansi(value);
    let len = plain.chars().count();
    if len <= width {
        return value.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let truncated: String = plain.chars().take(width - 1).collect();
    format!("{truncated}…")
}

pub fn wrap_text(value: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![value.to_string()];
    }
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in words {
        if word.chars().count() > width {
            if !line.is_empty() {
                lines.push(line);
                line = String::new();
            }
            let chars: Vec<char> = word.chars().collect();
            for chunk in chars.chunks(width) {
                lines.push(chunk.iter().collect());
            }
            continue;
        }
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if next.chars().count() > width {
            lines.push(line);
            line = word.to_string();
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn format_time(ms: u64) -> String {
    if ms < 1000 {
        return format!("{ms}ms");
    }
    let s = ms as f64 / 1000.0;
    if s < 60.0 {
        return format!("{:.1}s", s);
    }
    let m = (s / 60.0).floor() as u64;
    let rem = (s % 60.0).round() as u64;
    format!("{m}m {rem}s")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::green;

    #[test]
    fn format_skill_label_plain_url() {
        let s = format_skill_label("https://example.com/skill", false);
        assert_eq!(s, "https://example.com/skill");
    }

    #[test]
    fn format_skill_label_plain_three_parts() {
        let s = format_skill_label("owner/repo/my-skill", false);
        assert_eq!(s, "owner › my-skill");
    }

    #[test]
    fn format_skill_label_plain_not_three() {
        let s = format_skill_label("owner/repo", false);
        assert_eq!(s, "owner/repo");
    }

    #[test]
    fn format_skill_label_styled_contains_parts() {
        let s = format_skill_label("owner/repo/my-skill", true);
        let plain = strip_ansi(&s);
        assert!(plain.contains("owner"));
        assert!(plain.contains("my-skill"));
        assert!(plain.contains("›"));
    }

    #[test]
    fn wrap_text_basic() {
        let lines = wrap_text("hello world foo bar", 10);
        assert!(lines.len() >= 2);
        for l in &lines {
            assert!(l.chars().count() <= 10);
        }
    }

    #[test]
    fn wrap_text_long_word_split() {
        let long = "a".repeat(50);
        let lines = wrap_text(&long, 10);
        assert!(lines.len() == 5);
        for l in &lines {
            assert!(l.chars().count() <= 10);
        }
    }

    #[test]
    fn visible_pad_pads_correctly() {
        let s = green("hi");
        let padded = visible_pad(&s, 5);
        assert_eq!(strip_ansi(&padded).chars().count(), 5);
    }

    #[test]
    fn truncate_visible_truncates() {
        let s = "hello world";
        let t = truncate_visible(s, 5);
        assert_eq!(t.chars().count(), 5);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn format_time_cases() {
        assert_eq!(format_time(500), "500ms");
        assert_eq!(format_time(1500), "1.5s");
        assert_eq!(format_time(61_000), "1m 1s");
    }
}
